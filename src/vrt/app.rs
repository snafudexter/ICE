use crate::vrt::device::buffer::VRTBuffer;
use crate::vrt::device::descriptors::descriptor_pool::{
    VRTDescriptorPoolBuilder, VRTDescriptorWriter,
};
use crate::vrt::device::descriptors::layout::VRTDescriptorSetLayoutBuilder;
use crate::vrt::device::swapchain::{Swapchain, MAX_FRAMES_IN_FLIGHT};
use crate::VRTWindow;
use std::process;
use std::rc::Rc;
use std::sync::Arc;

use erupt::SmallVec;
use glam::Mat4;

use erupt::vk1_0::{
    BufferUsageFlags, DescriptorSet, DescriptorType, DeviceSize, MemoryPropertyFlags,
    ShaderStageFlags,
};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use super::device::descriptors::descriptor_pool::VRTDescriptorPool;
use super::device::descriptors::layout::VRTDescriptorSetLayout;
use super::device::device::VRTDevice;

use super::graphics::model::Model;
use super::graphics::renderer::VRTRenderer;
use super::graphics::triangle_render_system::TriangleRenderSystem;
use super::utils::result::VkResult;

pub struct VRTApp {
    device: Arc<VRTDevice>,
    window: VRTWindow,
    renderer: VRTRenderer,
    triangle_render_system: TriangleRenderSystem,
    model: Model,
    //global_pool: VRTDescriptorPool,
    //global_descriptor_set_layout: VRTDescriptorSetLayout,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct GlobalUBO {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

impl VRTApp {
    pub fn new(event_loop: &EventLoop<()>, app_name: &str, width: u32, height: u32) -> Self {
        println!("System OS {:?}", std::env::consts::OS);
        let window = VRTWindow::build_window(&event_loop, app_name, width, height)
            .expect("Cannot create window.");

        let device = Arc::new(VRTDevice::new(&window).expect("Cannot create device"));

        let renderer = VRTRenderer::new(device.clone(), &window).unwrap();

        let model = Model::new(device.get_instance(), device.clone());

        let mut ubo_buffers: Vec<VRTBuffer> = vec![];
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let ubo_buffer = VRTBuffer::new(
                device.clone(),
                std::mem::size_of::<GlobalUBO>() as DeviceSize,
                1,
                BufferUsageFlags::UNIFORM_BUFFER,
                MemoryPropertyFlags::HOST_VISIBLE,
                None,
            );
            ubo_buffer.map(None, None);
            ubo_buffers.push(ubo_buffer);
        }

        let global_pool = std::rc::Rc::new(
            VRTDescriptorPoolBuilder::new(device.clone())
                .set_max_sets(u32::try_from(MAX_FRAMES_IN_FLIGHT).unwrap())
                .add_pool_size(
                    DescriptorType::UNIFORM_BUFFER,
                    u32::try_from(MAX_FRAMES_IN_FLIGHT).unwrap(),
                )
                .build(),
        );

        let global_descriptor_set_layout = VRTDescriptorSetLayoutBuilder::new(device.clone())
            .add_binding(
                0,
                DescriptorType::UNIFORM_BUFFER,
                ShaderStageFlags::ALL_GRAPHICS,
                None,
            )
            .build();

        let global_descriptor_set_layout = Rc::new(global_descriptor_set_layout);

        let mut global_descriptor_sets: SmallVec<DescriptorSet>;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info =
                ubo_buffers[i].get_buffer_info(std::mem::size_of::<GlobalUBO>() as DeviceSize);
            global_descriptor_sets =
                VRTDescriptorWriter::new(global_descriptor_set_layout.clone(), global_pool.clone())
                    .write_buffer(0, &buffer_info)
                    .build()
                    .unwrap();
        }

        let triangle_render_system = TriangleRenderSystem::new(
            device.clone(),
            renderer.get_swapchain_render_pass(),
            global_descriptor_set_layout.get_descriptor_set_layout(),
        );

        Self {
            device,
            window,
            renderer,
            triangle_render_system,
            model,
            //global_descriptor_set_layout,
        }
    }

    // fn create_graphics_pipeline(
    //     device: &DeviceLoader,
    //     extent: &Extent2D,
    //     render_pass: RenderPass,
    // ) -> VkResult<(PipelineLayout, Pipeline)> {
    //     let vertex_shader_module = Shader::create_shader_module(device, VERTEX_SHADER_CODE)?;
    //     let fragment_shader_module = Shader::create_shader_module(device, FRAGMENT_SHADER_CODE)?;

    //     let name = std::ffi::CString::new("main").unwrap();

    //     let vertex_shader_stage_info = PipelineShaderStageCreateInfoBuilder::new()
    //         .stage(ShaderStageFlagBits::VERTEX)
    //         .module(vertex_shader_module)
    //         .name(&name);

    //     let fragment_shader_stage_info = PipelineShaderStageCreateInfoBuilder::new()
    //         .stage(ShaderStageFlagBits::FRAGMENT)
    //         .module(fragment_shader_module)
    //         .name(&name);

    //     let _shader_stages = [vertex_shader_stage_info, fragment_shader_stage_info];

    //     let _vertex_input_info = PipelineVertexInputStateCreateInfoBuilder::new();

    //     let _input_assembly = PipelineInputAssemblyStateCreateInfoBuilder::new()
    //         .topology(PrimitiveTopology::TRIANGLE_LIST)
    //         .primitive_restart_enable(false);

    //     let viewport = ViewportBuilder::new()
    //         .x(0.0)
    //         .y(0.0)
    //         .width(extent.width as f32)
    //         .height(extent.height as f32)
    //         .min_depth(0.0)
    //         .max_depth(1.0);

    //     let scissor = Rect2DBuilder::new()
    //         .offset(*Offset2DBuilder::new().x(0).y(0))
    //         .extent(*extent);

    //     let _viewport_state = PipelineViewportStateCreateInfoBuilder::new()
    //         .viewports(std::slice::from_ref(&viewport))
    //         .scissors(std::slice::from_ref(&scissor));

    //     let _rasterizer = PipelineRasterizationStateCreateInfoBuilder::new()
    //         .depth_clamp_enable(false)
    //         .rasterizer_discard_enable(false)
    //         .polygon_mode(PolygonMode::FILL)
    //         .line_width(1.0)
    //         .cull_mode(CullModeFlags::BACK)
    //         .front_face(FrontFace::CLOCKWISE)
    //         .depth_bias_enable(false);

    //     let _multisampling = PipelineMultisampleStateCreateInfoBuilder::new()
    //         .sample_shading_enable(false)
    //         .rasterization_samples(SampleCountFlagBits::_1)
    //         .min_sample_shading(1.0)
    //         .alpha_to_coverage_enable(false)
    //         .alpha_to_one_enable(false);

    //     let color_blend_attachment = PipelineColorBlendAttachmentStateBuilder::new()
    //         .color_write_mask(
    //             ColorComponentFlags::R
    //                 | ColorComponentFlags::G
    //                 | ColorComponentFlags::B
    //                 | ColorComponentFlags::A,
    //         )
    //         .blend_enable(false)
    //         .src_color_blend_factor(BlendFactor::ONE)
    //         .dst_color_blend_factor(BlendFactor::ZERO)
    //         .color_blend_op(BlendOp::ADD)
    //         .src_alpha_blend_factor(BlendFactor::ONE)
    //         .dst_alpha_blend_factor(BlendFactor::ZERO)
    //         .alpha_blend_op(BlendOp::ADD);

    //     let _color_blending = PipelineColorBlendStateCreateInfoBuilder::new()
    //         .logic_op_enable(false)
    //         .logic_op(LogicOp::COPY)
    //         .attachments(std::slice::from_ref(&color_blend_attachment))
    //         .blend_constants([0.0, 0.0, 0.0, 0.0]);

    //     let pipeline_layout_info = PipelineLayoutCreateInfoBuilder::new();

    //     let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) };

    //     let pipeline_info = GraphicsPipelineCreateInfoBuilder::new()
    //         .stages(&_shader_stages)
    //         .vertex_input_state(&_vertex_input_info)
    //         .input_assembly_state(&_input_assembly)
    //         .viewport_state(&_viewport_state)
    //         .rasterization_state(&_rasterizer)
    //         .multisample_state(&_multisampling)
    //         .color_blend_state(&_color_blending)
    //         .layout(pipeline_layout.unwrap())
    //         .render_pass(render_pass)
    //         .subpass(0)
    //         .base_pipeline_index(-1);

    //     let graphics_pipeline = unsafe {
    //         device.create_graphics_pipelines(
    //             PipelineCache::null(),
    //             std::slice::from_ref(&pipeline_info),
    //             None,
    //         )
    //     }
    //     .result()?[0];

    //     unsafe { device.destroy_shader_module(fragment_shader_module, None) };
    //     unsafe { device.destroy_shader_module(vertex_shader_module, None) };

    //     Ok((pipeline_layout.result()?, graphics_pipeline))
    // }

    fn process_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) -> VkResult<()> {
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
                    self.window.resize_callback(new_inner_size);
                }
                _ => (),
            },
            Event::MainEventsCleared => self.draw_frame()?,
            Event::RedrawRequested(_) => self.draw_frame()?,
            Event::LoopDestroyed => {
                unsafe { self.device.get_device_ptr().device_wait_idle() }.result()?
            }
            _ => (),
        }

        Ok(())
    }

    fn draw_frame(&mut self) -> VkResult<()> {
        let command_buffer = self.renderer.begin_frame(&self.window).unwrap();

        self.renderer.begin_swapchain_render_pass(command_buffer);

        self.triangle_render_system
            .render(self.device.clone(), command_buffer, &self.model);

        self.renderer.end_swapchain_render_pass(command_buffer);
        self.renderer.end_frame(&mut self.window, command_buffer);
        Ok(())
    }

    pub fn run(&'static mut self, event_loop: EventLoop<()>) -> ! {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            if let Err(err) = self.process_event(event, control_flow) {
                eprintln!("Error: {:?}", color_eyre::Report::new(err));
                process::exit(1);
            }
        })
    }
}

// impl Drop for VRTApp<'_> {
//     fn drop(&mut self) {
//         unsafe {
//             self.device
//                 .get_device_ptr()
//                 .destroy_command_pool(self.command_pool, None);

//             self.device
//                 .get_device_ptr()
//                 .destroy_pipeline(self.pipeline.pipeline, None);

//             self.device
//                 .get_device_ptr()
//                 .destroy_pipeline_layout(self.pipeline.layout, None);
//         }
//     }
// }
