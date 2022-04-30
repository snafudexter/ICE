use std::process;

use erupt::vk::{
    AttachmentDescriptionBuilder, AttachmentLoadOp, AttachmentReferenceBuilder, AttachmentStoreOp,
    BlendFactor, BlendOp, ClearColorValue, ClearValue, ColorComponentFlags, CommandBuffer,
    CommandBufferAllocateInfoBuilder, CommandBufferBeginInfoBuilder, CommandBufferLevel,
    CommandPool, CommandPoolCreateInfoBuilder, CullModeFlags, Extent2D, FenceCreateFlags,
    FenceCreateInfoBuilder, Format, Framebuffer, FramebufferCreateInfoBuilder, FrontFace,
    GraphicsPipelineCreateInfoBuilder, Image, ImageLayout, ImageView, LogicOp, Offset2DBuilder,
    Pipeline, PipelineBindPoint, PipelineCache, PipelineColorBlendAttachmentStateBuilder,
    PipelineColorBlendStateCreateInfoBuilder, PipelineInputAssemblyStateCreateInfoBuilder,
    PipelineLayout, PipelineLayoutCreateInfoBuilder, PipelineMultisampleStateCreateInfoBuilder,
    PipelineRasterizationStateCreateInfoBuilder, PipelineShaderStageCreateInfoBuilder,
    PipelineStageFlags, PipelineVertexInputStateCreateInfoBuilder,
    PipelineViewportStateCreateInfoBuilder, PolygonMode, PresentInfoKHRBuilder, PrimitiveTopology,
    Rect2DBuilder, RenderPass, RenderPassBeginInfoBuilder, RenderPassCreateInfoBuilder,
    SampleCountFlagBits, SemaphoreCreateInfoBuilder, ShaderStageFlagBits, SubmitInfoBuilder,
    SubpassContents, SubpassDescriptionBuilder, ViewportBuilder,
};
use erupt::{DeviceLoader, SmallVec};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use super::device::device::VRTDevice;
use super::device::queue::CompleteQueueFamilyIndices;
use super::device::sync::SyncObjects;
use super::graphics::shader::base::Shader;
use super::graphics::utils::pipeline::RenderPipeline;
use super::utils::result::{VkError, VkResult};

// Compiled shaders
const VERTEX_SHADER_CODE: &[u8] = include_bytes!("./assets/shaders/vert.spirv");
const FRAGMENT_SHADER_CODE: &[u8] = include_bytes!("./assets/shaders/frag.spirv");

pub struct VRTApp {
    device: VRTDevice,
    pipeline: RenderPipeline,
}

impl VRTApp {
    pub fn new(window: &Window) -> VkResult<Self> {
        let device = VRTDevice::new(window).expect("Cannot create device");

        let (pipeline_layout, graphics_pipeline) = Self::create_graphics_pipeline(
            &device.get_device_ptr(),
            &device.get_swapchain_ptr().extent,
            render_pass,
        )?;

        let pipeline = RenderPipeline {
            pipeline: graphics_pipeline,
            layout: pipeline_layout,
            render_pass,
        };

        let command_buffers = Self::create_command_buffers(
            &device.get_device_ptr(),
            &device.get_swapchain_ptr().extent,
            render_pass,
            graphics_pipeline,
            &framebuffers,
            command_pool,
        )?;

        Ok(Self {
            sync,
            device,
            render_pass,
            pipeline,
            framebuffers,
            command_pool,
            command_buffers,
        })
    }

    fn create_graphics_pipeline(
        device: &DeviceLoader,
        extent: &Extent2D,
        render_pass: RenderPass,
    ) -> VkResult<(PipelineLayout, Pipeline)> {
        let vertex_shader_module = Shader::create_shader_module(device, VERTEX_SHADER_CODE)?;
        let fragment_shader_module = Shader::create_shader_module(device, FRAGMENT_SHADER_CODE)?;

        let name = std::ffi::CString::new("main").unwrap();

        let vertex_shader_stage_info = PipelineShaderStageCreateInfoBuilder::new()
            .stage(ShaderStageFlagBits::VERTEX)
            .module(vertex_shader_module)
            .name(&name);

        let fragment_shader_stage_info = PipelineShaderStageCreateInfoBuilder::new()
            .stage(ShaderStageFlagBits::FRAGMENT)
            .module(fragment_shader_module)
            .name(&name);

        let _shader_stages = [vertex_shader_stage_info, fragment_shader_stage_info];

        let _vertex_input_info = PipelineVertexInputStateCreateInfoBuilder::new();

        let _input_assembly = PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = ViewportBuilder::new()
            .x(0.0)
            .y(0.0)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = Rect2DBuilder::new()
            .offset(*Offset2DBuilder::new().x(0).y(0))
            .extent(*extent);

        let _viewport_state = PipelineViewportStateCreateInfoBuilder::new()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let _rasterizer = PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let _multisampling = PipelineMultisampleStateCreateInfoBuilder::new()
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

        let _color_blending = PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(false)
            .logic_op(LogicOp::COPY)
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let pipeline_layout_info = PipelineLayoutCreateInfoBuilder::new();

        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) };

        let pipeline_info = GraphicsPipelineCreateInfoBuilder::new()
            .stages(&_shader_stages)
            .vertex_input_state(&_vertex_input_info)
            .input_assembly_state(&_input_assembly)
            .viewport_state(&_viewport_state)
            .rasterization_state(&_rasterizer)
            .multisample_state(&_multisampling)
            .color_blend_state(&_color_blending)
            .layout(pipeline_layout.unwrap())
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_index(-1);

        let graphics_pipeline = unsafe {
            device.create_graphics_pipelines(
                PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            )
        }
        .result()?[0];

        unsafe { device.destroy_shader_module(fragment_shader_module, None) };
        unsafe { device.destroy_shader_module(vertex_shader_module, None) };

        Ok((pipeline_layout.result()?, graphics_pipeline))
    }

    fn process_event(
        &mut self,
        window: &Window,
        event: Event<()>,
        control_flow: &mut ControlFlow,
    ) -> VkResult<()> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let (Some(VirtualKeyCode::Escape), ElementState::Released) =
                        (input.virtual_keycode, input.state)
                    {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WindowEvent::Resized(new_inner_size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut new_inner_size,
                    ..
                } => {
                    if self.swapchain.extent.width != new_inner_size.width
                        || self.swapchain.extent.height != new_inner_size.height
                    {
                        self.recreate_swapchain(window)?;
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => self.draw_frame(window)?,
            Event::LoopDestroyed => unsafe { self.device.device_wait_idle() }.result()?,
            _ => (),
        }

        Ok(())
    }

    fn draw_frame(&mut self, window: &Window) -> VkResult<()> {
        let image_index = match image_index_result {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(window)?;
                return Ok(());
            }
            result => result,
        }?;

        if present_result.raw == vk::Result::ERROR_OUT_OF_DATE_KHR
            || present_result.raw == vk::Result::SUBOPTIMAL_KHR
        {
            self.recreate_swapchain(window)?;
        } else {
            present_result.result()?;
        }

        self.sync.current_frame = (self.sync.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    pub fn run(mut self, event_loop: EventLoop<()>, window: Window) -> ! {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            if let Err(err) = self.process_event(&window, event, control_flow) {
                eprintln!("Error: {:?}", color_eyre::Report::new(err));
                process::exit(1);
            }
        })
    }
}

impl Drop for VRTApp {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_command_pool(self.command_pool, None);

            self.device
                .get_device_ptr()
                .destroy_pipeline(self.pipeline.pipeline, None);

            self.device
                .get_device_ptr()
                .destroy_pipeline_layout(self.pipeline.layout, None);
        }
    }
}
