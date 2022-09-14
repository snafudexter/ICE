use std::ffi::CString;
use std::fs::{metadata, File};
use std::io::Read;
use std::sync::Arc;

use erupt::vk1_0::{
    PipelineInputAssemblyStateCreateInfoBuilder, PipelineShaderStageCreateInfoBuilder,
    PipelineVertexInputStateCreateInfoBuilder, PrimitiveTopology, ShaderModule,
    ShaderModuleCreateInfoBuilder, ShaderStageFlagBits,
};

use crate::vrt::device::device::VRTDevice;
use crate::vrt::utils::result::VkResult;

pub struct PipelineConfigInfo<'a> {
    input_assembly: PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    vertex_input_info: PipelineVertexInputStateCreateInfoBuilder<'a>,
}

pub struct VRTPipeline {}

impl VRTPipeline {
    pub fn new(
        device: Arc<VRTDevice>,
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        configInfo: &PipelineConfigInfo,
    ) -> Self {
        let vertex_shader_module =
            Self::create_shader_module(device.clone(), &Self::read_file(&vertex_shader_path))
                .unwrap();
        let fragment_shader_module =
            Self::create_shader_module(device.clone(), &Self::read_file(&fragment_shader_path))
                .unwrap();
        let name = CString::new("main").unwrap();

        let vertex_shader_stage_info = PipelineShaderStageCreateInfoBuilder::new()
            .stage(ShaderStageFlagBits::VERTEX)
            .module(vertex_shader_module)
            .name(&name);

        let fragment_shader_stage_info = PipelineShaderStageCreateInfoBuilder::new()
            .stage(ShaderStageFlagBits::FRAGMENT)
            .module(fragment_shader_module)
            .name(&name);

        let _shader_stages = [vertex_shader_stage_info, fragment_shader_stage_info];

        unsafe {
            device
                .get_device_ptr()
                .destroy_shader_module(fragment_shader_module, None)
        };
        unsafe {
            device
                .get_device_ptr()
                .destroy_shader_module(vertex_shader_module, None)
        };

        Self {}
    }

    pub fn create_shader_module(device: Arc<VRTDevice>, code: &[u8]) -> VkResult<ShaderModule> {
        let code =
            unsafe { std::slice::from_raw_parts::<u32>(code.as_ptr().cast(), code.len() / 4) };
        let create_info = ShaderModuleCreateInfoBuilder::new().code(code);

        Ok(unsafe {
            device
                .get_device_ptr()
                .create_shader_module(&create_info, None)
        }
        .result()?)
    }

    fn read_file(path: &str) -> Vec<u8> {
        let mut file = File::open(path).unwrap();
        let meta = metadata(path).unwrap();
        let mut buffer = vec![0; meta.len() as usize];
        file.read(&mut buffer).unwrap();
        buffer
    }

    pub fn default_pipeline_config_info() -> PipelineConfigInfo<'static> {
        let input_assembly = PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let vertex_input_info = PipelineVertexInputStateCreateInfoBuilder::new();

        PipelineConfigInfo {
            input_assembly,
            vertex_input_info,
        }
    }
}
