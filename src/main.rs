mod vrt;
use vrt::app::VRTApp;
use vrt::window::VRTWindow;
use winit::event_loop::EventLoop;
use std::env;


const APP_NAME: &str = "VulkSim";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

#[cfg(target_os = "linux")]
fn set_x11_backend() {
    env::set_var("WINIT_UNIX_BACKEND", "x11");
}

fn main() -> color_eyre::Result<()> {

    #[cfg(target_os = "linux")]
    set_x11_backend();

    color_eyre::install()?;
    let event_loop = EventLoop::new();

    unsafe {
        let app = Box::leak(Box::new(VRTApp::new(
            &event_loop,
            APP_NAME,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )));
        app.run(event_loop);
    }
}
