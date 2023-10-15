use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::{Extent2D, SwapchainKHR};
use ash::{extensions::ext::DebugUtils, vk, Entry};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::borrow::Cow;
use std::{default::Default, ffi::CStr};

#[cfg(any(target_os = "macos", target_os = "ios"))]
use ash::vk::{
    KhrGetPhysicalDeviceProperties2Fn, KhrPortabilityEnumerationFn, KhrPortabilitySubsetFn,
};

use crate::allocator::MemoryAllocator;
use crate::image::VulkanImage;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const MAX_FRAMES_IN_FLIGHT: u8 = 2;

#[cfg(debug_assertions)]
// const VALIDATION_LAYER: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\n") };
#[cfg(debug_assertions)]
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}

pub struct VulkanInstance {
    pub memory_allocator: MemoryAllocator,

    pub instance: ash::Instance,
    pub device: ash::Device,

    pub surface_loader: Surface,
    pub surface: vk::SurfaceKHR,

    pub swapchain_loader: Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub pipeline_layout: vk::PipelineLayout,

    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub frame_buffers: Vec<vk::Framebuffer>,
    pub depth_image: VulkanImage,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    pub render_pass: vk::RenderPass,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,

    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,

    frame: usize,

    #[cfg(debug_assertions)]
    debug_utils: DebugUtils,
    #[cfg(debug_assertions)]
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanInstance {
    pub fn new(application_name: &str, window: winit::window::Window) -> Self {
        let entry = Entry::linked();

        let app_info = unsafe {
            vk::ApplicationInfo::builder()
                .application_name(CStr::from_bytes_with_nul_unchecked(
                    application_name.as_bytes(),
                ))
                .application_version(0) // TODO
                .engine_name(CStr::from_bytes_with_nul_unchecked(b"Vent-Engine\0"))
                .engine_version(env!("CARGO_PKG_VERSION").parse().unwrap())
                .api_version(vk::API_VERSION_1_2)
                .build()
        };

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle())
                .expect("Unsupported Surface Extension")
                .to_vec();
        extension_names.push(DebugUtils::name().as_ptr());

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

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .flags(create_flags)
            .build();

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed Create Vulkan Instance")
        };
        #[cfg(debug_assertions)]
        let debug_utils_loader = DebugUtils::new(&entry, &instance);
        #[cfg(debug_assertions)]
        let debug_messenger = Self::create_debug_messenger(&debug_utils_loader);
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
        }
        .unwrap();
        let surface_loader = Surface::new(&entry, &instance);

        let (pdevice, graphics_queue_family_index, present_queue_family_index) =
            Self::create_physical_device(&instance, &surface_loader, surface);
        let surface_format =
            unsafe { surface_loader.get_physical_device_surface_formats(pdevice, surface) }
                .unwrap()[0];
        let device = Self::create_device(&instance, pdevice, graphics_queue_family_index);
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_family_index, 0) };

        let swapchain_loader = Swapchain::new(&instance, &device);

        let (swapchain, surface_resolution) = Self::create_swapchain(
            &swapchain_loader,
            surface_format,
            &surface_loader,
            pdevice,
            surface,
            window.inner_size(),
        );

        let (swapchain_image_views, swapchain_images) =
            Self::create_image_views(&device, &swapchain_loader, swapchain, surface_format);
        let pipeline_layout = Self::create_pipeline_layout(&device);
        let render_pass = Self::create_render_pass(&device, surface_format);

        let depth_image =
            VulkanImage::new_depth(&device, vk::Format::D16_UNORM, surface_resolution);
        let frame_buffers = Self::create_frame_buffers(
            &swapchain_image_views,
            render_pass,
            &device,
            depth_image.image_view,
            surface_resolution,
        );
        let command_pool = Self::create_command_pool(&device, graphics_queue_family_index);
        let command_buffers =
            Self::allocate_command_buffers(&device, command_pool, frame_buffers.len() as u32);

        let (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ) = Self::create_sync_objects(&device, &swapchain_images);

        let memory_allocator = MemoryAllocator::new(unsafe {
            instance.get_physical_device_memory_properties(pdevice)
        });

        let descriptor_pool = Self::create_descriptor_pool(&device);
        let descriptor_set_layout = Self::create_descriptor_set_layout(&device);
        let descriptor_sets = Self::allocate_descriptor_sets(
            &device,
            descriptor_pool,
            descriptor_set_layout,
            &swapchain_images,
        );

        Self {
            memory_allocator,
            instance,
            device,
            surface,
            surface_loader,
            swapchain_loader,
            swapchain,
            pipeline_layout,
            swapchain_images,
            swapchain_image_views,
            graphics_queue,
            present_queue,
            render_pass,
            debug_utils: debug_utils_loader,
            descriptor_pool,
            descriptor_set_layout,
            descriptor_sets,
            debug_messenger,
            depth_image,
            frame_buffers,
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            frame: 0,
        }
    }

    // Returns None if Surface should be resized else returns image index
    pub fn next_image(&self) -> Option<u32> {
        let in_flight_fence = self.in_flight_fences[self.frame];

        unsafe {
            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::max_value())
        }
        .unwrap();
        unsafe {
            match self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_semaphores[self.frame],
                vk::Fence::null(),
            ) {
                Ok((index, _)) => {
                    let image_in_flight = self.images_in_flight[index as usize];
                    self.device
                        .wait_for_fences(&[image_in_flight], true, u64::max_value())
                        .unwrap();
                    Some(index)
                }
                Err(_) => None,
            }
        }
    }

    pub fn submit(&self, image_index: u32, in_flight_fence: vk::Fence) {
        let wait_semaphores = &[self.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.command_buffers[image_index as usize]];
        let signal_semaphores = &[self.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores)
            .build();

        unsafe {
            self.device.reset_fences(&[in_flight_fence]).unwrap();

            self.device
                .queue_submit(self.graphics_queue, &[submit_info], in_flight_fence)
                .unwrap();
        }

        let swapchains = &[self.swapchain];
        let image_indices = &[image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices)
            .build();

        unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
        }
        .unwrap();
    }

    fn get_depth_format(instance: &ash::Instance, pdevice: vk::PhysicalDevice) -> vk::Format {
        let candidates = &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
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
        candidates.iter().cloned().find(|f| {
            let properties = unsafe { instance.get_physical_device_format_properties(pdevice, *f) };
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        })
    }

    fn create_debug_messenger(debug_utils_loader: &DebugUtils) -> vk::DebugUtilsMessengerEXT {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback))
            .build();

        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }.unwrap()
    }

    fn create_frame_buffers(
        swapchain_image_views: &Vec<vk::ImageView>,
        render_pass: vk::RenderPass,
        device: &ash::Device,
        depth_image_view: vk::ImageView,
        surface_resolution: Extent2D,
    ) -> Vec<vk::Framebuffer> {
        swapchain_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(surface_resolution.width)
                    .height(surface_resolution.height)
                    .layers(1)
                    .build();

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
        surface_loader: &Surface,
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
    ) -> ash::Device {
        let device_extension_names_raw = [
            Swapchain::name().as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            KhrPortabilitySubsetFn::name().as_ptr(),
        ];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)
            .build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features)
            .build();

        unsafe { instance.create_device(pdevice, &device_create_info, None) }.unwrap()
    }

    fn create_swapchain(
        swapchain_loader: &Swapchain,
        surface_format: vk::SurfaceFormatKHR,
        surface_loader: &Surface,
        pdevice: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        size: winit::dpi::PhysicalSize<u32>,
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
        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: size.width,
                height: size.height,
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
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
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
            .build();

        (
            unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }.unwrap(),
            surface_resolution,
        )
    }

    fn create_command_pool(device: &ash::Device, queue_family_index: u32) -> vk::CommandPool {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();
        unsafe { device.create_command_pool(&create_info, None) }.unwrap()
    }

    fn allocate_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        count: u32,
    ) -> Vec<vk::CommandBuffer> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count)
            .build();

        unsafe { device.allocate_command_buffers(&allocate_info) }.unwrap()
    }

    fn create_sync_objects(
        device: &ash::Device,
        swapchain_images: &Vec<vk::Image>,
    ) -> (
        Vec<vk::Semaphore>,
        Vec<vk::Semaphore>,
        Vec<vk::Fence>,
        Vec<vk::Fence>,
    ) {
        let (mut image_available_semaphores, mut render_finished_semaphores, mut in_flight_fences) =
            (Vec::new(), Vec::new(), Vec::new());

        let create_info = vk::SemaphoreCreateInfo::default();

        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

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
        swapchain_loader: &Swapchain,
        swapchain: SwapchainKHR,
        surface_format: vk::SurfaceFormatKHR,
    ) -> (Vec<vk::ImageView>, Vec<vk::Image>) {
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }.unwrap();
        (
            images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
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
                        .image(image)
                        .build();
                    unsafe { device.create_image_view(&create_view_info, None) }.unwrap()
                })
                .collect::<Vec<vk::ImageView>>(),
            images,
        )
    }

    fn create_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1000)
            .build()];

        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .build();

        unsafe { device.create_descriptor_pool(&create_info, None) }.unwrap()
    }

    fn allocate_descriptor_sets(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        swapchain_images: &Vec<vk::Image>,
    ) -> Vec<vk::DescriptorSet> {
        let layouts = vec![descriptor_set_layout; swapchain_images.len()];
        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts)
            .build();

        unsafe { device.allocate_descriptor_sets(&info) }.unwrap()
    }

    fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();

        let bindings = &[ubo_binding, sampler_binding];
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();

        unsafe { device.create_descriptor_set_layout(&info, None) }.unwrap()
    }

    fn create_pipeline_layout(device: &ash::Device) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo::default();

        unsafe { device.create_pipeline_layout(&create_info, None) }.unwrap()
    }

    fn create_render_pass(
        device: &ash::Device,
        surface_format: vk::SurfaceFormatKHR,
    ) -> vk::RenderPass {
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: surface_format.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build();

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies)
            .build();

        unsafe { device.create_render_pass(&create_info, None) }.unwrap()
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.in_flight_fences
                .iter()
                .for_each(|f| self.device.destroy_fence(*f, None));
            self.render_finished_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));
            self.image_available_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));

            self.device.destroy_render_pass(self.render_pass, None);

            self.swapchain_image_views
                .iter()
                .for_each(|v| self.device.destroy_image_view(*v, None));

            self.frame_buffers
                .iter()
                .for_each(|f| self.device.destroy_framebuffer(*f, None));
            self.depth_image.destroy(&self.device);

            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            self.device.destroy_command_pool(self.command_pool, None);

            // DEVICE DESTRUCTION
            self.device.destroy_device(None);

            self.surface_loader.destroy_surface(self.surface, None);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None)
        }
    }
}
