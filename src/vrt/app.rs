use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::vrt::{device::VRTDevice, window::VRTWindow};

const APP_NAME: &str = "Vulkan Raytracer";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct VRTApp {
    event_loop: EventLoop<()>,
    window: Window,
    device: VRTDevice,
}

impl VRTApp {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = VRTWindow::build_window(&event_loop, APP_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
            .expect("Cannot create window.");

        let device = VRTDevice::new(&window).expect("Cannot create device");
        Self {
            event_loop,
            window,
            device,
        }
    }

    pub fn run(self) -> ! {
        self.event_loop.run(move |event, _, control_flow| {
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
