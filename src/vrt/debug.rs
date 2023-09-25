use crate::vrt::result::VkResult;
use erupt::vk::{
    Bool32, DebugUtilsMessageSeverityFlagBitsEXT, DebugUtilsMessageSeverityFlagsEXT,
    DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT,
    DebugUtilsMessengerCreateInfoEXTBuilder, DebugUtilsMessengerEXT,
    EXT_DEBUG_UTILS_EXTENSION_NAME, FALSE,
};
use erupt::{cstr, EntryLoader, InstanceLoader};
use std::array;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

pub const VALIDATION_LAYERS: &[*const c_char] = &[cstr!("VK_LAYER_KHRONOS_validation")];
pub const EXTENSIONS: &[*const c_char] = &[EXT_DEBUG_UTILS_EXTENSION_NAME];

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Messenger {
    inner: DebugUtilsMessengerEXT,
}

impl Messenger {
    pub fn new(instance: &InstanceLoader) -> VkResult<Self> {
        let create_info = messenger_create_info();

        Ok(Self {
            inner: unsafe { instance.create_debug_utils_messenger_ext(&create_info, None) }
                .result()?,
        })
    }

    pub unsafe fn destroy(self, instance: &InstanceLoader) {
        instance.destroy_debug_utils_messenger_ext(self.inner, None);
    }
}

pub fn check_validation_layer_support(entry: &EntryLoader) -> VkResult<bool> {
    let available_layers = unsafe { entry.enumerate_instance_layer_properties(None) }.result()?;

    println!("available_layers {:?}", &available_layers);
    Ok(VALIDATION_LAYERS
        .iter()
        .map(|layer_name| unsafe { CStr::from_ptr(*layer_name) })
        .all(|layer_name| {
            available_layers.iter().any(|layer_properties| {
                layer_name == unsafe { CStr::from_ptr(layer_properties.layer_name.as_ptr()) }
            })
        }))
}

pub fn messenger_create_info() -> DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
    DebugUtilsMessengerCreateInfoEXTBuilder::new()
        .message_severity(DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(DebugUtilsMessageTypeFlagsEXT::all())
        .pfn_user_callback(Some(debug_callback))
}

unsafe extern "system" fn debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagBitsEXT,
    message_types: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> Bool32 {
    use {DebugUtilsMessageSeverityFlagBitsEXT as Severity, DebugUtilsMessageTypeFlagsEXT as Type};

    let mut types = array::IntoIter::new([
        (Type::GENERAL_EXT, "General"),
        (Type::VALIDATION_EXT, "Validation"),
        (Type::PERFORMANCE_EXT, "Performance"),
    ])
    .filter_map(|(flag, flag_str)| message_types.contains(flag).then(|| flag_str))
    .collect::<Vec<_>>();

    if types.is_empty() {
        types.push("Unknown");
    }

    let types = types.join(" | ");

    let message = CStr::from_ptr((*p_callback_data).p_message);

    match message_severity {
        Severity::VERBOSE_EXT => log::debug!("[{}] {:?}", types, message),
        Severity::INFO_EXT => log::info!("[{}] {:?}", types, message),
        Severity::WARNING_EXT => log::warn!("[{}] {:?}", types, message),
        Severity::ERROR_EXT => log::error!("[{}] {:?}", types, message),
        _ => log::debug!("[{}] {:?}", types, message),
    }

    FALSE
}
