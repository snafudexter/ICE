use erupt::vk::{
    AttachmentDescriptionBuilder, AttachmentLoadOp, AttachmentReferenceBuilder, AttachmentStoreOp,
    BlendFactor, BlendOp, ColorComponentFlags, CullModeFlags, Extent2D, Format, FrontFace,
    ImageLayout, LogicOp, Offset2DBuilder, PipelineBindPoint,
    PipelineColorBlendAttachmentStateBuilder, PipelineColorBlendStateCreateInfoBuilder,
    PipelineInputAssemblyStateCreateInfoBuilder, PipelineLayout, PipelineLayoutCreateInfoBuilder,
    PipelineMultisampleStateCreateInfoBuilder, PipelineRasterizationStateCreateInfoBuilder,
    PipelineShaderStageCreateInfoBuilder, PipelineVertexInputStateCreateInfoBuilder,
    PipelineViewportStateCreateInfoBuilder, PolygonMode, PrimitiveTopology, Rect2DBuilder,
    RenderPass, RenderPassCreateInfoBuilder, SampleCountFlagBits, ShaderStageFlagBits,
    SubpassDescriptionBuilder, ViewportBuilder,
};
use erupt::DeviceLoader;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;

use super::device::device::VRTDevice;
use super::graphics::shader::base::Shader;
use super::utils::result::VkResult;
use super::window::VRTWindow;

const APP_NAME: &str = "Vulkan Raytracer";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

// Compiled shaders
const VERTEX_SHADER_CODE: &[u8] = include_bytes!("./assets/shaders/vert.spirv");
const FRAGMENT_SHADER_CODE: &[u8] = include_bytes!("./assets/shaders/frag.spirv");

pub struct VRTApp {
    event_loop: EventLoop<()>,
    window: Window,
    device: VRTDevice,
    pipeline_layout: PipelineLayout,
    render_pass: RenderPass,
}

impl VRTApp {
    pub fn new() -> VkResult<Self> {
        let event_loop = EventLoop::new();
        let window = VRTWindow::build_window(&event_loop, APP_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
            .expect("Cannot create window.");

        let device = VRTDevice::new(&window).expect("Cannot create device");

        let render_pass = Self::create_render_pass(
            &device.get_device_ptr(),
            (*device.get_swapchain_ptr()).image_format,
        )?;
        let pipeline_layout = Self::create_graphics_pipeline(
            &device.get_device_ptr(),
            &device.get_swapchain_ptr().extent,
        )?;

        Ok(Self {
            event_loop,
            window,
            device,
            pipeline_layout,
            render_pass,
        })
    }

    fn create_render_pass(device: &DeviceLoader, image_format: Format) -> VkResult<RenderPass> {
        let color_attachment = AttachmentDescriptionBuilder::new()
            .format(image_format)
            .samples(SampleCountFlagBits::_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = AttachmentReferenceBuilder::new()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = SubpassDescriptionBuilder::new()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));

        let render_pass_info = RenderPassCreateInfoBuilder::new()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass));

        Ok(unsafe { device.create_render_pass(&render_pass_info, None) }.result()?)
    }

    fn create_graphics_pipeline(
        device: &DeviceLoader,
        extent: &Extent2D,
    ) -> VkResult<PipelineLayout> {
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

        unsafe { device.destroy_shader_module(fragment_shader_module, None) };
        unsafe { device.destroy_shader_module(vertex_shader_module, None) };

        Ok(pipeline_layout.result()?)
    }

    pub fn run(mut self) -> () {
        self.event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

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
                    _ => (),
                },
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => {}
                _ => (),
            }
        })
    }
}

impl Drop for VRTApp {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_pipeline_layout(self.pipeline_layout, None);

            self.device
                .get_device_ptr()
                .destroy_render_pass(self.render_pass, None);
        }
    }
}
