use crate::vrt::buffer::VRTBuffer;
use crate::vrt::descriptor_pool::{VRTDescriptorPoolBuilder, VRTDescriptorWriter};
use crate::vrt::frame_info::GlobalUBO;
use crate::vrt::game_object;
use crate::vrt::layout::VRTDescriptorSetLayoutBuilder;
use crate::vrt::render_systems::simple_render_system;
use crate::vrt::swapchain::MAX_FRAMES_IN_FLIGHT;
use crate::VRTWindow;
use std::process;
use std::rc::Rc;
use std::sync::Arc;

use erupt::SmallVec;
use glam::Mat4;

use erupt::vk1_0::{
    BufferUsageFlags, DescriptorSet, DescriptorType, DeviceSize, MemoryPropertyFlags,
    ShaderStageFlags, WHOLE_SIZE,
};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use super::device::VRTDevice;

use super::frame_info::{DirectionalLight, FrameInfo, PointLight};
use super::game_object::GameObject;
use super::model::Model;
use super::render_systems::simple_render_system::SimpleRenderSystem;
use super::render_systems::triangle_render_system::TriangleRenderSystem;
use super::renderer::VRTRenderer;
use super::result::VkResult;
use dolly::prelude::*;

pub struct VRTApp {
    aspect_ratio: f32,
    device: Arc<VRTDevice>,
    window: VRTWindow,
    renderer: VRTRenderer,
    triangle_render_system: TriangleRenderSystem,
    simple_render_system: SimpleRenderSystem,
    ubo_buffers: Vec<VRTBuffer>,
    current_time: std::time::SystemTime,
    game_objects: Vec<GameObject>,
    descriptor_sets: Vec<SmallVec<DescriptorSet>>, //global_pool: VRTDescriptorPool,
                                                   //global_descriptor_set_layout: VRTDescriptorSetLayout,
}

impl VRTApp {
    pub fn new(event_loop: &EventLoop<()>, app_name: &str, width: u32, height: u32) -> Self {
        println!("System OS {:?}", std::env::consts::OS);
        let window = VRTWindow::build_window(&event_loop, app_name, width, height)
            .expect("Cannot create window.");

        let device = Arc::new(VRTDevice::new(&window).expect("Cannot create device"));

        let renderer = VRTRenderer::new(device.clone(), &window).unwrap();

        let model = Model::new(device.get_instance(), device.clone());

        let global_pool = std::rc::Rc::new(
            VRTDescriptorPoolBuilder::new(device.clone())
                .set_max_sets(u32::try_from(MAX_FRAMES_IN_FLIGHT).unwrap())
                .add_pool_size(
                    DescriptorType::UNIFORM_BUFFER,
                    u32::try_from(MAX_FRAMES_IN_FLIGHT).unwrap(),
                )
                .build(),
        );

        let mut ubo_buffers: Vec<VRTBuffer> = vec![];
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let mut ubo_buffer = VRTBuffer::new(
                device.clone(),
                std::mem::size_of::<GlobalUBO>() as DeviceSize,
                1,
                BufferUsageFlags::UNIFORM_BUFFER,
                MemoryPropertyFlags::HOST_VISIBLE,
                None,
            );
            ubo_buffer.map(Some((std::mem::size_of::<GlobalUBO>()) as DeviceSize), None);
            ubo_buffers.push(ubo_buffer);
        }

        let global_descriptor_set_layout = VRTDescriptorSetLayoutBuilder::new(device.clone())
            .add_binding(
                0,
                DescriptorType::UNIFORM_BUFFER,
                ShaderStageFlags::ALL_GRAPHICS,
                Some(1),
            )
            .build();

        let global_descriptor_set_layout = Rc::new(global_descriptor_set_layout);

        let mut global_descriptor_sets: Vec<SmallVec<DescriptorSet>> = vec![].into();

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info =
                ubo_buffers[i].get_buffer_info(std::mem::size_of::<GlobalUBO>() as DeviceSize);
            global_descriptor_sets.push(
                VRTDescriptorWriter::new(global_descriptor_set_layout.clone(), global_pool.clone())
                    .write_buffer(0, &buffer_info)
                    .build(i)
                    .unwrap(),
            )
        }

        let triangle_render_system = TriangleRenderSystem::new(
            device.clone(),
            renderer.get_swapchain_render_pass(),
            global_descriptor_set_layout.get_descriptor_set_layout(),
        );

        let simple_render_system: SimpleRenderSystem = SimpleRenderSystem::new(
            device.clone(),
            renderer.get_swapchain_render_pass(),
            global_descriptor_set_layout.get_descriptor_set_layout(),
        );

        let game_object = GameObject::new(Some(model));

        Self {
            aspect_ratio: width as f32 / height as f32,
            device,
            window,
            renderer,
            triangle_render_system,
            game_objects: vec![game_object],
            simple_render_system,
            current_time: std::time::SystemTime::now(), //global_descriptor_set_layout,
            ubo_buffers,
            descriptor_sets: global_descriptor_sets,
        }
    }

    fn process_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) -> VkResult<()> {
        self.current_time = std::time::SystemTime::now();
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
        let frame_time: u128 = self.current_time.elapsed().unwrap().as_micros();
        self.current_time = std::time::SystemTime::now();

        let command_buffer = self.renderer.begin_frame(&self.window).unwrap();

        let frame_index = self.renderer.get_frame_index();

        println!("************draw_frame************");
        println!("descriptor_sets count {:?}", self.descriptor_sets.len());

        let frame_info = FrameInfo::new(
            *frame_index,
            frame_time,
            command_buffer,
            &self.game_objects,
            &self.descriptor_sets[*frame_index],
        );

        let mut camera: CameraRig = CameraRig::builder()
            .with(Position::new(glam::vec3(0f32, 0f32, -2f32)))
            .with(YawPitch::new())
            .with(Smooth::new_position_rotation(1.0, 1.0))
            .build();

        let camera_xform = camera.update(frame_time as f32);

        let global_ubo = GlobalUBO::new(
            glam::Mat4::IDENTITY,
            glam::Mat4::look_at_rh(
                camera_xform.position,
                <[f32; 3]>::from(camera_xform.position + camera_xform.forward()).into(),
                <[f32; 3]>::from(camera_xform.up()).into(),
            ),
            glam::Mat4::perspective_rh(45.0f32.to_radians(), self.aspect_ratio, 0.01f32, 100.0f32),
            glam::vec4(1.0, 1.0, 0f32, 1.0),
            vec![PointLight::new(
                glam::Vec3 {
                    x: 1.0f32,
                    y: -1f32,
                    z: 0.0f32,
                },
                glam::vec4(1.0, 1.0, 1.0, 1.0),
            )],
        );

        self.ubo_buffers[*frame_index as usize].write_to_buffer(
            &global_ubo,
            self.ubo_buffers[*frame_index as usize]
                .get_mapped_memory()
                .unwrap(),
            1 as DeviceSize,
            0,
        );

        self.ubo_buffers[*frame_index as usize].flush(WHOLE_SIZE, 0);

        self.renderer.begin_swapchain_render_pass(command_buffer);

        self.simple_render_system
            .render(self.device.clone(), frame_info);

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
