use ash::extensions::ext::DebugUtils;
use ash::vk::Handle;
use ash::{vk, Entry, Instance};
use std::os::raw::c_void;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::instance::VulkanInstance;

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;

const REQUIRED_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

unsafe extern "system" fn vulkan_debug_callback(
    flag: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    use vk::DebugUtilsMessageSeverityFlagsEXT as Flag;

    let message = CStr::from_ptr((*p_callback_data).p_message);
    match flag {
        Flag::VERBOSE => log::debug!("{:?} - {:?}", typ, message),
        Flag::INFO => log::info!("{:?} - {:?}", typ, message),
        Flag::WARNING => log::warn!("{:?} - {:?}", typ, message),
        _ => log::error!("{:?} - {:?}", typ, message),
    }
    vk::FALSE
}

/// Get the pointers to the validation layers names.
/// Also return the corresponding `CString` to avoid dangling pointers.
pub fn get_layer_names_and_pointers() -> (Vec<CString>, Vec<*const c_char>) {
    let layer_names = REQUIRED_LAYERS
        .iter()
        .map(|name| CString::new(*name).unwrap())
        .collect::<Vec<_>>();
    let layer_names_ptrs = layer_names
        .iter()
        .map(|name| name.as_ptr())
        .collect::<Vec<_>>();
    (layer_names, layer_names_ptrs)
}

/// Check if the required validation set in `REQUIRED_LAYERS`
/// are supported by the Vulkan instance.
///
/// # Panics
///
/// Panic if at least one on the layer is not supported.
pub fn check_validation_layer_support(entry: &Entry) {
    for required in REQUIRED_LAYERS.iter() {
        let properties = entry.enumerate_instance_layer_properties().unwrap(); // TODO: Cache
        let layers = properties.iter().find(|layer| {
            let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
            let name = name.to_str().expect("Failed to get layer name pointer");
            required == &name
        });

        if layers.is_none() {
            panic!("Layer not supported: {}", required);
        }
    }
}
#[cfg(debug_assertions)]
pub fn set_object_name<H: Handle>(instance: &VulkanInstance, handle: H, name: &str) {
    let object_name = CString::new(name).expect("Failed to convert &str to CString");

    let debug_utils_object_name_info = vk::DebugUtilsObjectNameInfoEXT::builder()
        .object_type(H::TYPE)
        .object_handle(handle.as_raw())
        .object_name(&object_name);

    unsafe {
        instance
            .debug_utils
            .set_debug_utils_object_name(instance.device.handle(), &debug_utils_object_name_info)
            .expect("Failed to set debug object name")
    };
}

pub fn get_validation_features() -> vk::ValidationFeaturesEXT {
    if ENABLE_VALIDATION_LAYERS {
        let features = [
            vk::ValidationFeatureEnableEXT::BEST_PRACTICES,
            vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
        ];

        return vk::ValidationFeaturesEXT::builder()
            .enabled_validation_features(&features)
            .disabled_validation_features(&[]) // We need to give it an empty Array, If not we get an validation error
            .build();
    } else {
        vk::ValidationFeaturesEXT::default()
    }
}

/// Setup the debug message if validation layers are enabled.
#[must_use]
pub fn setup_debug_messenger(
    entry: &Entry,
    instance: &Instance,
) -> (DebugUtils, vk::DebugUtilsMessengerEXT) {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));
    let debug_utils = DebugUtils::new(entry, instance);
    let debug_utils_messenger = unsafe {
        debug_utils
            .create_debug_utils_messenger(&create_info, None)
            .unwrap()
    };

    (debug_utils, debug_utils_messenger)
}
