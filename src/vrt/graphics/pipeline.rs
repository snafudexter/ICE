use std::ffi::CString;
use std::fs::{metadata, File};
use std::io::Read;
use std::sync::Arc;

use erupt::vk1_0::{
    BlendFactor, BlendOp, ColorComponentFlags, CommandBuffer, CullModeFlags, DynamicState,
    FrontFace, GraphicsPipelineCreateInfoBuilder, LogicOp, Pipeline, PipelineBindPoint,
    PipelineCache, PipelineColorBlendAttachmentStateBuilder,
    PipelineColorBlendStateCreateInfoBuilder, PipelineDynamicStateCreateFlags,
    PipelineDynamicStateCreateInfoBuilder, PipelineInputAssemblyStateCreateInfoBuilder,
    PipelineLayoutCreateInfoBuilder, PipelineMultisampleStateCreateInfoBuilder,
    PipelineRasterizationStateCreateInfoBuilder, PipelineShaderStageCreateInfoBuilder,
    PipelineVertexInputStateCreateInfoBuilder, PipelineViewportStateCreateInfoBuilder, PolygonMode,
    PrimitiveTopology, Rect2DBuilder, RenderPass, SampleCountFlagBits, ShaderModule,
    ShaderModuleCreateInfoBuilder, ShaderStageFlagBits, ViewportBuilder,
};

use crate::vrt::device::device::VRTDevice;
use crate::vrt::utils::result::VkResult;

pub struct PipelineConfigInfo<'a> {
    input_assembly: PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    vertex_input_info: PipelineVertexInputStateCreateInfoBuilder<'a>,
    rasterizer: PipelineRasterizationStateCreateInfoBuilder<'a>,
    multisampling: PipelineMultisampleStateCreateInfoBuilder<'a>,
    color_blending: PipelineColorBlendStateCreateInfoBuilder<'a>,
    pipeline_layout_info: PipelineLayoutCreateInfoBuilder<'a>,
    dynamic_state_info: PipelineDynamicStateCreateInfoBuilder<'a>,
    color_blend_attachment: PipelineColorBlendAttachmentStateBuilder<'a>,
    viewport: ViewportBuilder<'a>,
    scissor: Rect2DBuilder<'a>,
}

pub struct VRTPipeline {
    graphics_pipeline: Pipeline,
    device: Arc<VRTDevice>,
}

impl VRTPipeline {
    pub fn new(
        device: Arc<VRTDevice>,
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        configInfo: &mut PipelineConfigInfo,
        render_pass: RenderPass,
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

        let pipeline_layout = unsafe {
            device
                .get_device_ptr()
                .create_pipeline_layout(&configInfo.pipeline_layout_info, None)
        };

        let color_blending = PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(false)
            .logic_op(LogicOp::COPY)
            .attachments(std::slice::from_ref(&configInfo.color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let viewport_state = PipelineViewportStateCreateInfoBuilder::new()
            // .viewports(std::slice::from_ref(&configInfo.viewport))
            // .scissors(std::slice::from_ref(&configInfo.scissor))
            .viewport_count(1)
            .scissor_count(1);

        let pipeline_info = GraphicsPipelineCreateInfoBuilder::new()
            .stages(&_shader_stages)
            .vertex_input_state(&configInfo.vertex_input_info)
            .input_assembly_state(&configInfo.input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&configInfo.rasterizer)
            .multisample_state(&configInfo.multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout.unwrap())
            .dynamic_state(&configInfo.dynamic_state_info)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_index(-1);

        let graphics_pipeline = unsafe {
            device.get_device_ptr().create_graphics_pipelines(
                PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            )
        }
        .result()
        .unwrap()[0];

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

        Self {
            graphics_pipeline,
            device,
        }
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
        let viewport = ViewportBuilder::new();
        let scissor = Rect2DBuilder::new();

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

        let dynamic_state_info = PipelineDynamicStateCreateInfoBuilder::new()
            .dynamic_states(&[DynamicState::VIEWPORT, DynamicState::SCISSOR])
            .flags(PipelineDynamicStateCreateFlags::empty());

        PipelineConfigInfo {
            input_assembly,
            vertex_input_info,
            rasterizer,
            multisampling,
            color_blending,
            pipeline_layout_info,
            color_blend_attachment,
            dynamic_state_info,
            viewport,
            scissor,
        }
    }

    pub fn bind(&self, command_buffer: CommandBuffer) {
        unsafe {
            self.device.get_device_ptr().cmd_bind_pipeline(
                command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );
        }
    }
}
