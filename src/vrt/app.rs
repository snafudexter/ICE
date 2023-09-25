use crate::vrt::buffer::VRTBuffer;
use crate::vrt::descriptor_pool::{VRTDescriptorPoolBuilder, VRTDescriptorWriter};
use crate::vrt::layout::VRTDescriptorSetLayoutBuilder;
use crate::vrt::swapchain::MAX_FRAMES_IN_FLIGHT;
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

use super::device::VRTDevice;

use super::model::Model;
use super::renderer::VRTRenderer;
use super::result::VkResult;
use super::triangle_render_system::TriangleRenderSystem;

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
