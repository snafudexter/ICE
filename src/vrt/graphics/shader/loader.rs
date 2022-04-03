use super::base::Shader;
use crate::vrt::utils::result::{VkError, VkResult};
use erupt::vk::{ShaderModule, ShaderModuleCreateInfoBuilder};
use erupt::DeviceLoader;

impl Shader {
    pub fn create_shader_module(device: &DeviceLoader, code: &[u8]) -> VkResult<ShaderModule> {
        let code = unsafe { std::slice::from_raw_parts::<u32>(code.as_ptr().cast(), code.len() / 4) };
        let create_info = ShaderModuleCreateInfoBuilder::new().code(code);

        Ok(unsafe { device.create_shader_module(&create_info, None) }.result()?)
    }
}
