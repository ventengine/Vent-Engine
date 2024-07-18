use ash::ext::{debug_utils, validation_features};
use ash::khr::swapchain;
use ash::prelude::VkResult;
use ash::vk::{Extent2D, PushConstantRange, SwapchainKHR};
use ash::{khr, vk, Entry};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use std::{default::Default, ffi::CStr};

#[cfg(any(target_os = "macos", target_os = "ios"))]
use ash::vk::{
    KhrGetPhysicalDeviceProperties2Fn, KhrPortabilityEnumerationFn, KhrPortabilitySubsetFn,
};

use crate::allocator::MemoryAllocator;
use crate::debug::{self, check_validation_layer_support, setup_debug_messenger};
use crate::image::{DepthImage, VulkanImage};
use crate::surface;

pub const MAX_FRAMES_IN_FLIGHT: u8 = 2;

pub struct VulkanInstance {
    // Important: Do not drop ash::Entry, As it drops all functions pointers
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device_features: vk::PhysicalDeviceFeatures,
    pub device: ash::Device,

    pub surface_loader: khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    pub swapchain_loader: khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,

    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub frame_buffers: Vec<vk::Framebuffer>,
    pub depth_format: vk::Format,
    pub depth_image: DepthImage,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub render_pass: vk::RenderPass,
    pub global_command_pool: vk::CommandPool,
    pub command_pools: Vec<vk::CommandPool>,
    pub command_buffers: Vec<vk::CommandBuffer>,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,

    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,

    pub memory_allocator: MemoryAllocator,
    pub vulkan_version: u32,

    pub vsync: bool,

    frame: usize,

    // debugging
    pub validation: bool,
    pub debug_utils: Option<debug_utils::Instance>,
    pub debug_utils_device: Option<debug_utils::Device>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanInstance {
    pub fn new(
        application_name: &String,
        application_version: u32,
        vsync: bool,
        window: &vent_window::Window,
    ) -> Self {
        let entry = unsafe { Entry::load().expect("Failed to load Vulkan Library") };

        let engine_version = vk::make_api_version(
            0,
            env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        );
        let vulkan_version = match unsafe { entry.try_enumerate_instance_version().unwrap() } {
            Some(version) => version,
            None => {
                log::warn!("Failed to get vulkan instance version, Probably using Vulkan 1.0");
                vk::API_VERSION_1_0
            }
        };

        // We now set validation at runtime, This will be usefull later, We should make this an startup flag so release builds for example could be debugged
        let validation = cfg!(debug_assertions);

        let app_info = unsafe {
            vk::ApplicationInfo::default()
                .application_name(CStr::from_bytes_with_nul_unchecked(
                    application_name.as_bytes(),
                ))
                .application_version(application_version)
                .engine_name(CStr::from_bytes_with_nul_unchecked(b"Vent-Engine\0"))
                .engine_version(engine_version)
                .api_version(vk::API_VERSION_1_3)
        };

        let display_handle = window.display_handle().expect("No Display Handle");
        let window_handle = window.window_handle().expect("No Window Handle");

        let mut extension_names = surface::enumerate_required_extensions(display_handle.as_raw())
            .expect("Unsupported Surface Extension")
            .to_vec();
        if validation {
            extension_names.push(validation_features::NAME.as_ptr());
            extension_names.push(debug_utils::NAME.as_ptr());
        }

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(KhrPortabilityEnumerationFn::name().as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extension_names.push(KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
        }

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let (layer_names_ptrs, mut validation_features) = if validation {
            check_validation_layer_support(&entry);
            (
                debug::get_layer_names_and_pointers(),
                debug::get_validation_features(),
            )
        } else {
            (
                (Vec::new(), Vec::new()),
                vk::ValidationFeaturesEXT::default(),
            )
        };

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layer_names_ptrs.1)
            .flags(create_flags)
            .push_next(&mut validation_features);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed Create Vulkan Instance")
        };

        let surface = unsafe {
            surface::create_surface(&entry, &instance, display_handle, window_handle, None)
        }
        .unwrap();
        let surface_loader = khr::surface::Instance::new(&entry, &instance);

        let (pdevice, graphics_queue_family_index, present_queue_family_index) =
            Self::create_physical_device(&instance, &surface_loader, surface);

        let info = unsafe { instance.get_physical_device_properties(pdevice) };
        unsafe {
            log::debug!(
                "Vulkan {}, Selected graphics device (`{}`)",
                info.api_version,
                CStr::from_ptr(info.device_name.as_ptr()).to_string_lossy()
            );
        }

        let surface_format =
            unsafe { surface_loader.get_physical_device_surface_formats(pdevice, surface) }
                .unwrap()[0];

        let device_features = unsafe { instance.get_physical_device_features(pdevice) };

        let device = Self::create_device(
            &instance,
            pdevice,
            graphics_queue_family_index,
            device_features,
        );

        let (debug_utils, debug_utils_device, debug_messenger) = if validation {
            let (utils, device_utils, messenger) =
                setup_debug_messenger(&entry, &instance, &device);
            (Some(utils), Some(device_utils), Some(messenger))
        } else {
            (None, None, None)
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_family_index, 0) };

        let swapchain_loader = khr::swapchain::Device::new(&instance, &device);

        let (swapchain, surface_resolution) = Self::create_swapchain(
            &swapchain_loader,
            surface_format,
            &surface_loader,
            pdevice,
            vsync,
            surface,
            (window.width(), window.height()),
            None,
        );

        let (swapchain_image_views, swapchain_images) =
            Self::create_image_views(&device, &swapchain_loader, swapchain, surface_format);

        let depth_format = Self::get_depth_format(&instance, pdevice);

        let render_pass = Self::create_render_pass(&device, surface_format, depth_format);

        let memory_allocator = MemoryAllocator::new(unsafe {
            instance.get_physical_device_memory_properties(pdevice)
        });

        let depth_image =
            VulkanImage::new_depth(&device, &memory_allocator, depth_format, surface_resolution);
        let frame_buffers = Self::create_frame_buffers(
            &swapchain_image_views,
            render_pass,
            &device,
            depth_image.image_view,
            surface_resolution,
        );
        let (global_command_pool, command_pools) = Self::create_command_pools(
            &device,
            graphics_queue_family_index,
            swapchain_images.len(),
        );
        let command_buffers =
            Self::allocate_command_buffers(&device, &command_pools, swapchain_images.len());

        let (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ) = Self::create_sync_objects(&device, &swapchain_images);

        Self {
            entry,
            instance,
            physical_device: pdevice,
            device_features,
            device,
            surface_loader,
            surface,
            surface_format,
            surface_resolution,
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            frame_buffers,
            depth_format,
            depth_image,
            graphics_queue,
            present_queue,
            render_pass,
            global_command_pool,
            command_pools,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            memory_allocator,
            vulkan_version,
            vsync,
            frame: 0,
            validation,
            debug_utils,
            debug_utils_device,
            debug_messenger,
        }
    }

    // returns the next image's index and whether the swapchain is suboptimal for the surface.
    pub fn next_image(&self) -> VkResult<(u32, bool)> {
        let in_flight_fence = self.in_flight_fences[self.frame];

        unsafe {
            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)
                .unwrap();
            self.device.reset_fences(&[in_flight_fence]).unwrap();
        }
        unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_semaphores[self.frame],
                vk::Fence::null(),
            )
        }
    }

    pub fn recreate_swap_chain(&mut self, new_size: Option<(u32, u32)>) {
        let new_size = new_size.unwrap_or((
            self.surface_resolution.width,
            self.surface_resolution.height,
        ));
        unsafe {
            self.device.device_wait_idle().unwrap();

            let (swapchain, surface_resolution) = Self::create_swapchain(
                &self.swapchain_loader,
                self.surface_format,
                &self.surface_loader,
                self.physical_device,
                self.vsync,
                self.surface,
                new_size,
                Some(self.swapchain),
            );
            // We reuse the old Swapchain and then deleting it
            self.clean_swapchain();

            self.swapchain = swapchain;
            self.surface_resolution = surface_resolution;

            (self.swapchain_image_views, self.swapchain_images) = Self::create_image_views(
                &self.device,
                &self.swapchain_loader,
                self.swapchain,
                self.surface_format,
            );

            self.depth_image.destroy(&self.device);
            self.depth_image = VulkanImage::new_depth(
                &self.device,
                &self.memory_allocator,
                self.depth_format,
                surface_resolution,
            );

            self.frame_buffers = Self::create_frame_buffers(
                &self.swapchain_image_views,
                self.render_pass,
                &self.device,
                self.depth_image.image_view,
                self.surface_resolution,
            );
        }
    }

    /**
     * Returns if should resize
     */
    pub fn submit(&mut self, image_index: u32) -> VkResult<bool> {
        let in_flight_fence = self.in_flight_fences[self.frame];

        let wait_semaphores = vk::SemaphoreSubmitInfo::default()
            .semaphore(self.image_available_semaphores[self.frame])
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT);

        let command_buffers = vk::CommandBufferSubmitInfo::default()
            .command_buffer(self.command_buffers[image_index as usize]);
        let signal_semaphores = vk::SemaphoreSubmitInfo::default()
            .semaphore(self.render_finished_semaphores[self.frame])
            .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS);

        let signal_infos = [signal_semaphores];
        let command_infos = [command_buffers];
        let wait_infos = [wait_semaphores];
        let submit_info = vk::SubmitInfo2::default()
            .wait_semaphore_infos(&wait_infos)
            .command_buffer_infos(&command_infos)
            .signal_semaphore_infos(&signal_infos);

        unsafe {
            self.device
                .queue_submit2(self.graphics_queue, &[submit_info], in_flight_fence)
                .unwrap();
        }

        let swapchains = &[self.swapchain];
        let image_indices = &[image_index];
        let binding = [self.render_finished_semaphores[self.frame]];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&binding)
            .swapchains(swapchains)
            .image_indices(image_indices);

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT as usize;

        unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
        }
    }

    unsafe fn clean_swapchain(&mut self) {
        self.frame_buffers
            .drain(..)
            .for_each(|f| self.device.destroy_framebuffer(f, None));

        self.swapchain_image_views
            .drain(..)
            .for_each(|v| self.device.destroy_image_view(v, None));

        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);
    }

    fn get_depth_format(instance: &ash::Instance, pdevice: vk::PhysicalDevice) -> vk::Format {
        let candidates = &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
            vk::Format::D16_UNORM,
        ];

        Self::get_supported_format(
            instance,
            pdevice,
            candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
        .expect("No Depth Format found")
    }

    pub fn get_supported_format(
        instance: &ash::Instance,
        pdevice: vk::PhysicalDevice,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Option<vk::Format> {
        candidates
            .iter()
            .find(|f| {
                let properties =
                    unsafe { instance.get_physical_device_format_properties(pdevice, **f) };
                match tiling {
                    vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                    vk::ImageTiling::OPTIMAL => {
                        properties.optimal_tiling_features.contains(features)
                    }
                    _ => false,
                }
            })
            .copied()
    }

    fn create_frame_buffers(
        swapchain_image_views: &[vk::ImageView],
        render_pass: vk::RenderPass,
        device: &ash::Device,
        depth_image_view: vk::ImageView,
        surface_resolution: Extent2D,
    ) -> Vec<vk::Framebuffer> {
        swapchain_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(surface_resolution.width)
                    .height(surface_resolution.height)
                    .layers(1);

                unsafe {
                    device
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap()
                }
            })
            .collect::<Vec<vk::Framebuffer>>()
    }

    fn create_physical_device(
        instance: &ash::Instance,
        surface_loader: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> (vk::PhysicalDevice, u32, u32) {
        let pdevices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Physical device error")
        };
        for pdevice in pdevices {
            let properties =
                unsafe { instance.get_physical_device_queue_family_properties(pdevice) };

            let graphics = properties
                .iter()
                .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .map(|i| i as u32);

            let mut present = None;
            for (index, _properties) in properties.iter().enumerate() {
                if unsafe {
                    surface_loader.get_physical_device_surface_support(
                        pdevice,
                        index as u32,
                        surface,
                    )
                }
                .unwrap()
                {
                    present = Some(index as u32);
                    break;
                }
            }

            if let (Some(graphics), Some(present)) = (graphics, present) {
                return (pdevice, graphics, present);
            }
        }
        panic!("Missing required queue families")
    }

    fn create_device(
        instance: &ash::Instance,
        pdevice: vk::PhysicalDevice,
        queue_family_index: u32,
        available_features: vk::PhysicalDeviceFeatures,
    ) -> ash::Device {
        let device_extension_names_raw = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            portability_subset::NAME.as_ptr(),
        ];

        let mut features_1_3 = vk::PhysicalDeviceVulkan13Features::default()
            .synchronization2(true)
            .maintenance4(true);

        let features = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: available_features.sampler_anisotropy,
            ..Default::default()
        };
        //        features.shader_clip_distance = available_features.shader_clip_distance;

        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features)
            .push_next(&mut features_1_3);

        unsafe { instance.create_device(pdevice, &device_create_info, None) }.unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    fn create_swapchain(
        swapchain_loader: &khr::swapchain::Device,
        surface_format: vk::SurfaceFormatKHR,
        surface_loader: &khr::surface::Instance,
        pdevice: vk::PhysicalDevice,
        vsync: bool,
        surface: vk::SurfaceKHR,
        size: (u32, u32),
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> (vk::SwapchainKHR, Extent2D) {
        let surface_capabilities =
            unsafe { surface_loader.get_physical_device_surface_capabilities(pdevice, surface) }
                .unwrap();
        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }

        let width = size.0;
        let height = size.1;

        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: width.clamp(
                    surface_capabilities.min_image_extent.width,
                    surface_capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    surface_capabilities.min_image_extent.height,
                    surface_capabilities.max_image_extent.height,
                ),
            },
            _ => surface_capabilities.current_extent,
        };
        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };
        let present_modes =
            unsafe { surface_loader.get_physical_device_surface_present_modes(pdevice, surface) }
                .unwrap();

        let wanted_mode = if vsync {
            vk::PresentModeKHR::FIFO
        } else {
            vk::PresentModeKHR::MAILBOX
        };
        let mut present_mode = vk::PresentModeKHR::FIFO; // Vsync, Always supported
        if present_modes.contains(&wanted_mode) {
            present_mode = wanted_mode;
        } else {
            log::warn!("Swapchain: wanted mode is not supported, Using FIFO");
        }

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1)
            .old_swapchain(old_swapchain.unwrap_or_default());

        (
            unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }.unwrap(),
            surface_resolution,
        )
    }

    fn create_command_pools(
        device: &ash::Device,
        queue_family_index: u32,
        count: usize,
    ) -> (vk::CommandPool, Vec<vk::CommandPool>) {
        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);
        let global = unsafe { device.create_command_pool(&create_info, None) }.unwrap();

        let mut pools: Vec<_> = Vec::with_capacity(count);
        for _ in 0..count {
            let create_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(queue_family_index)
                .flags(vk::CommandPoolCreateFlags::TRANSIENT);
            pools.push(unsafe { device.create_command_pool(&create_info, None) }.unwrap());
        }
        (global, pools)
    }

    fn allocate_command_buffers(
        device: &ash::Device,
        command_pools: &[vk::CommandPool],
        count: usize,
    ) -> Vec<vk::CommandBuffer> {
        let mut buffers = Vec::with_capacity(count);
        for pool in command_pools.iter().take(count) {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(*pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            buffers.push(unsafe { device.allocate_command_buffers(&allocate_info) }.unwrap()[0]);
        }
        buffers
    }

    fn create_sync_objects(
        device: &ash::Device,
        swapchain_images: &[vk::Image],
    ) -> (
        Vec<vk::Semaphore>,
        Vec<vk::Semaphore>,
        Vec<vk::Fence>,
        Vec<vk::Fence>,
    ) {
        let (mut image_available_semaphores, mut render_finished_semaphores, mut in_flight_fences) =
            (Vec::new(), Vec::new(), Vec::new());

        let create_info = vk::SemaphoreCreateInfo::default();

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores
                .push(unsafe { device.create_semaphore(&create_info, None) }.unwrap());
            render_finished_semaphores
                .push(unsafe { device.create_semaphore(&create_info, None) }.unwrap());

            in_flight_fences.push(unsafe { device.create_fence(&fence_info, None) }.unwrap());
        }

        let images_in_flight = swapchain_images.iter().map(|_| vk::Fence::null()).collect();

        (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        )
    }

    fn create_image_views(
        device: &ash::Device,
        swapchain_loader: &khr::swapchain::Device,
        swapchain: SwapchainKHR,
        surface_format: vk::SurfaceFormatKHR,
    ) -> (Vec<vk::ImageView>, Vec<vk::Image>) {
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }.unwrap();
        (
            images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::default()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);
                    unsafe { device.create_image_view(&create_view_info, None) }.unwrap()
                })
                .collect::<Vec<vk::ImageView>>(),
            images,
        )
    }

    pub fn allocate_descriptor_sets(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        size: usize,
    ) -> Vec<vk::DescriptorSet> {
        let layouts = (0..size).map(|_| descriptor_set_layout).collect::<Vec<_>>();
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        unsafe { device.allocate_descriptor_sets(&info) }.unwrap()
    }

    pub fn create_pipeline_layout(
        &self,
        push_constant_ranges: &[PushConstantRange],
        descriptor_set_layout: &[vk::DescriptorSetLayout],
    ) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(push_constant_ranges)
            .set_layouts(descriptor_set_layout);

        unsafe { self.device.create_pipeline_layout(&create_info, None) }.unwrap()
    }

    fn create_render_pass(
        device: &ash::Device,
        surface_format: vk::SurfaceFormatKHR,
        depth_format: vk::Format,
    ) -> vk::RenderPass {
        let renderpass_attachments = [
            vk::AttachmentDescription2 {
                format: surface_format.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            vk::AttachmentDescription2 {
                format: depth_format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];
        let color_attachment_refs = [vk::AttachmentReference2 {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ..Default::default()
        }];
        let depth_attachment_ref = vk::AttachmentReference2::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let dependencies = [
            vk::SubpassDependency2 {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                dst_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                src_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                ..Default::default()
            },
            vk::SubpassDependency2 {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                ..Default::default()
            },
        ];

        let subpass = vk::SubpassDescription2::default()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let create_info = vk::RenderPassCreateInfo2::default()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        unsafe { device.create_render_pass2(&create_info, None) }.unwrap()
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.clean_swapchain();

            self.in_flight_fences
                .drain(..)
                .for_each(|f| self.device.destroy_fence(f, None));
            self.render_finished_semaphores
                .drain(..)
                .for_each(|s| self.device.destroy_semaphore(s, None));
            self.image_available_semaphores
                .drain(..)
                .for_each(|s| self.device.destroy_semaphore(s, None));

            self.device.destroy_render_pass(self.render_pass, None);

            self.depth_image.destroy(&self.device);

            self.command_pools
                .iter()
                .for_each(|p| self.device.destroy_command_pool(*p, None));
            self.device
                .destroy_command_pool(self.global_command_pool, None);

            // DEVICE DESTRUCTION
            self.device.destroy_device(None);

            self.surface_loader.destroy_surface(self.surface, None);

            if let Some(debug_messenger) = self.debug_messenger {
                if let Some(debug_utils) = &self.debug_utils {
                    debug_utils.destroy_debug_utils_messenger(debug_messenger, None);
                } else {
                    unreachable!()
                }
            }
            self.instance.destroy_instance(None);
        }
    }
}
