use std::ffi::CString;
use std::fs::{metadata, File};
use std::io::Read;
use std::sync::Arc;

use erupt::vk1_0::{
    BlendFactor, BlendOp, ColorComponentFlags, CullModeFlags, DynamicState, FrontFace, LogicOp,
    PipelineColorBlendAttachmentStateBuilder, PipelineColorBlendStateCreateInfoBuilder,
    PipelineDynamicStateCreateInfo, PipelineDynamicStateCreateInfoBuilder,
    PipelineInputAssemblyStateCreateInfoBuilder, PipelineLayoutCreateInfoBuilder,
    PipelineMultisampleStateCreateInfoBuilder, PipelineRasterizationStateCreateInfoBuilder,
    PipelineShaderStageCreateInfoBuilder, PipelineVertexInputStateCreateInfoBuilder, PolygonMode,
    PrimitiveTopology, SampleCountFlagBits, ShaderModule, ShaderModuleCreateInfoBuilder,
    ShaderStageFlagBits,
};

use crate::vrt::device::device::VRTDevice;
use crate::vrt::utils::result::VkResult;

pub struct PipelineConfigInfo<'a> {
    input_assembly: PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    vertex_input_info: PipelineVertexInputStateCreateInfoBuilder<'a>,
    rasterizer: PipelineRasterizationStateCreateInfoBuilder<'a>,
    color_blend_attachment: PipelineColorBlendAttachmentStateBuilder<'a>,
    multisampling: PipelineMultisampleStateCreateInfoBuilder<'a>,
    color_blending: PipelineColorBlendStateCreateInfoBuilder<'a>,
    pipeline_layout_info: PipelineLayoutCreateInfoBuilder<'a>,
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

        let rasterizer = PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = PipelineMultisampleStateCreateInfoBuilder::new()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlagBits::_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let color_blend_attachment = PipelineColorBlendAttachmentStateBuilder::new()
            .color_write_mask(
                ColorComponentFlags::R
                    | ColorComponentFlags::G
                    | ColorComponentFlags::B
                    | ColorComponentFlags::A,
            )
            .blend_enable(false)
            .src_color_blend_factor(BlendFactor::ONE)
            .dst_color_blend_factor(BlendFactor::ZERO)
            .color_blend_op(BlendOp::ADD)
            .src_alpha_blend_factor(BlendFactor::ONE)
            .dst_alpha_blend_factor(BlendFactor::ZERO)
            .alpha_blend_op(BlendOp::ADD);

        let color_blending = PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(false)
            .logic_op(LogicOp::COPY)
            //.attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let pipeline_layout_info = PipelineLayoutCreateInfoBuilder::new();

        let mut dynamic_state_info = PipelineDynamicStateCreateInfoBuilder::new()
            .dynamic_states(&[DynamicState::VIEWPORT, DynamicState::SCISSOR]);
        dynamic_state_info.dynamic_state_count = 2;

        PipelineConfigInfo {
            input_assembly,
            vertex_input_info,
            rasterizer,
            multisampling,
            color_blend_attachment,
            color_blending,
            pipeline_layout_info,
        }
    }
}
