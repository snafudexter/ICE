use crate::vrt::buffer::VRTBuffer;
use crate::vrt::camera::VRTCamera;
use crate::vrt::descriptor_pool::{VRTDescriptorPoolBuilder, VRTDescriptorWriter};
use crate::vrt::frame_info::GlobalUBO;
use crate::vrt::layout::VRTDescriptorSetLayoutBuilder;
use crate::vrt::swapchain::MAX_FRAMES_IN_FLIGHT;
use crate::{VRTWindow, WINDOW_HEIGHT, WINDOW_WIDTH};
use std::process;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use erupt::SmallVec;

use erupt::vk1_0::{
    BufferUsageFlags, DescriptorSet, DescriptorType, DeviceSize, MemoryPropertyFlags,
    ShaderStageFlags, WHOLE_SIZE,
};
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use super::descriptor_pool::VRTDescriptorPool;
use super::device::VRTDevice;

use super::fps_camera::FPSCamera;
use super::frame_info::{FrameInfo, PointLight};
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
    simple_render_system: SimpleRenderSystem,
    ubo_buffers: Vec<VRTBuffer>,
    current_time: std::time::SystemTime,
    game_objects: Vec<GameObject>,
    descriptor_sets: Vec<SmallVec<DescriptorSet>>,
    camera: FPSCamera,
    global_pool: Rc<VRTDescriptorPool>,
    frame_time: u32, //global_descriptor_set_layout: VRTDescriptorSetLayout,
    start: Instant,
}

impl VRTApp {
    pub unsafe fn new(event_loop: &EventLoop<()>, app_name: &str, width: u32, height: u32) -> Self {
        println!("System OS {:?}", std::env::consts::OS);
        let window = VRTWindow::build_window(&event_loop, app_name, width, height)
            .expect("Cannot create window.");

        let device = Arc::new(VRTDevice::new(&window).expect("Cannot create device"));

        let renderer = VRTRenderer::new(device.clone(), &window).unwrap();

        // let model = Model::new(device.clone(), "./assets/models/smooth_vase.obj");
        // let ground = Model::new(device.clone(), "./assets/models/quad.obj");
        // let sponza = Model::new(device.clone(), "./assets/models/sponza/sponza.obj");
        let shapes = Model::new(device.clone(), "./assets/models/shapes.obj");
        let car = Model::new(device.clone(), "./assets/models/car.obj");

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

        let simple_render_system: SimpleRenderSystem = SimpleRenderSystem::new(
            device.clone(),
            renderer.get_swapchain_render_pass(),
            global_descriptor_set_layout.get_descriptor_set_layout(),
        );

        // let camera: CameraRig = CameraRig::builder()
        //     .with(Position::new(glam::vec3(0f32, 0f32, 10f32)))
        //     .with(YawPitch::new())
        //     .with(Smooth::new_position_rotation(1.0, 1.0))
        //     .build();

        let camera = FPSCamera::new(
            0.01,
            0.01,
            glam::vec3(0f32, 20f32, 20.0),
            glam::Vec3::ZERO,
            glam::Vec3::NEG_Y,
        );

        window.get_window_ptr().set_cursor_visible(false);

        Self {
            aspect_ratio: width as f32 / height as f32,
            device,
            window,
            renderer,
            game_objects: vec![
                // GameObject::new(Some(model)),
                // GameObject::new(Some(sponza)),
                //GameObject::new(Some(shapes)),
                GameObject::new(Some(car)),
            ],
            simple_render_system,
            current_time: std::time::SystemTime::now(), //global_descriptor_set_layout,
            ubo_buffers,
            descriptor_sets: global_descriptor_sets,
            global_pool,
            camera,
            frame_time: 0,
            start: Instant::now(),
        }
    }

    fn process_event(
        &mut self,
        event: Event<()>,
        control_flow: &mut ControlFlow,
        frame_time: u32,
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

                    self.camera.process_keyboard_event(
                        input.virtual_keycode.unwrap(),
                        input.state,
                        frame_time,
                    );
                }

                WindowEvent::CursorMoved { position, .. } => {
                    self.camera.rotate(
                        WINDOW_WIDTH as f32 / 2f32 - position.x as f32,
                        WINDOW_HEIGHT as f32 / 2f32 - position.y as f32,
                    );

                    self.window
                        .get_window_ptr()
                        .set_cursor_position(PhysicalPosition {
                            x: WINDOW_WIDTH as f32 / 2f32,
                            y: WINDOW_HEIGHT as f32 / 2f32,
                        })
                        .unwrap();
                }

                // Mouse input
                WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button,
                    ..
                } => {
                    if let MouseButton::Left = button {
                        self.camera.track_mouse(if state == ElementState::Pressed {
                            true
                        } else {
                            false
                        })
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
            Event::MainEventsCleared => self.window.get_window_ptr().request_redraw(),
            Event::RedrawRequested(_) => self.draw_frame()?,
            Event::LoopDestroyed => {
                unsafe { self.device.get_device_ptr().device_wait_idle() }.result()?
            }
            _ => (),
        }

        Ok(())
    }

    fn draw_frame(&mut self) -> VkResult<()> {
        let frame_time = self.current_time.elapsed().unwrap().as_millis();

        let rotation_angle = 90f32 * self.start.elapsed().as_secs_f32();

        self.current_time = std::time::SystemTime::now();

        self.frame_time = frame_time as u32;

        // println!("frame time: {:?}", frame_time);

        let command_buffer = self.renderer.begin_frame(&self.window).unwrap();

        let frame_index = self.renderer.get_frame_index();

        let frame_info = FrameInfo::new(
            command_buffer,
            &self.game_objects,
            &self.descriptor_sets[*frame_index],
        );

        let perspective =
            glam::Mat4::perspective_rh(45.0f32.to_radians(), self.aspect_ratio, 0.01f32, 100.0f32);

        // perspective.y_axis.y *= -1f32;

        let model_matrix = glam::Mat4::from_scale(glam::Vec3::ONE * 2f32)
            * glam::Mat4::from_axis_angle(
                glam::vec3(0f32, 1f32, 0f32),
                rotation_angle.to_radians(),
            )
            * glam::Mat4::from_translation(glam::vec3(0f32, 0f32, 0f32));

        let global_ubo = GlobalUBO::new(
            model_matrix,
            glam::Mat4::look_at_rh(
                *self.camera.get_position(),
                *self.camera.get_target(),
                glam::Vec3::Y,
            ),
            perspective,
            glam::vec4(1.0, 1.0, 1f32, 0.1),
            PointLight::new(
                glam::Vec4 {
                    x: 2.0f32,
                    y: 10f32,
                    z: 5.0f32,
                    w: 1.0f32,
                },
                glam::vec4(0.5, 1.0, 1.0, 1.0),
            ),
            glam::vec4(
                self.camera.get_position().x,
                self.camera.get_position().y,
                self.camera.get_position().z,
                1.0,
            ),
        );

        //println!("view position {:?}", self.camera.get_position());

        self.ubo_buffers[*frame_index as usize].write_to_buffer(
            &global_ubo,
            self.ubo_buffers[*frame_index as usize]
                .get_mapped_memory()
                .unwrap(),
            1 as DeviceSize,
            0,
        );

        // self.ubo_buffers[*frame_index as usize].flush(WHOLE_SIZE, 0);

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

            if let Err(err) = self.process_event(event, control_flow, self.frame_time) {
                eprintln!("Error: {:?}", color_eyre::Report::new(err));
                process::exit(1);
            }
        })
    }
}
