
use crate::vrt::frame_info::GlobalUBO;
use crate::{VRTWindow, WINDOW_HEIGHT, WINDOW_WIDTH};
use std::process;
use std::sync::Arc;
use std::time::Instant;


use erupt::vk1_0::DeviceSize;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use super::device::VRTDevice;

use super::fps_camera::FPSCamera;
use super::frame_info::{FrameInfo, PointLight};
use super::game_object::GameObject;
use super::model::Model;
use super::render_systems::simple_render_system::SimpleRenderSystem;
use super::renderer::VRTRenderer;
use super::result::VkResult;

pub struct VRTApp {
    device: Arc<VRTDevice>,
    window: VRTWindow,
    renderer: VRTRenderer,
    simple_render_system: SimpleRenderSystem,
    current_time: std::time::SystemTime,
    game_objects: Vec<GameObject>,
    camera: FPSCamera,
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

        let sponza = Model::new(device.clone(), "./assets/models/sponza-gltf-pbr/sponza.glb").expect("problem loading model");

        

        let simple_render_system: SimpleRenderSystem = SimpleRenderSystem::new(
            device.clone(),
            renderer.get_swapchain_render_pass(),
            width,
            height
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

        println!("All Loaded");

        Self {
            
            device,
            window,
            renderer,
            game_objects: vec![
                // GameObject::new(Some(model)),
                // GameObject::new(Some(sponza)),
                //GameObject::new(Some(shapes)),
                GameObject::new(Some(sponza)),
            ],
            simple_render_system,
            current_time: std::time::SystemTime::now(), //global_descriptor_set_layout,
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

        

        self.current_time = std::time::SystemTime::now();

        self.frame_time = frame_time as u32;

        // println!("frame time: {:?}", frame_time);

        let command_buffer = self.renderer.begin_frame(&self.window).unwrap();

        let frame_index = self.renderer.get_frame_index();

        self.simple_render_system.update(&self.camera, frame_index);
        

        // self.ubo_buffers[*frame_index as usize].flush(WHOLE_SIZE, 0);

        self.renderer.begin_swapchain_render_pass(command_buffer);

        self.simple_render_system
            .render(self.device.clone(), frame_index, command_buffer, &self.game_objects);

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
