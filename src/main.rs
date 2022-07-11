mod vrt;
use vrt::app::VRTApp;
use vrt::window::VRTWindow;
use winit::event_loop::EventLoop;

const APP_NAME: &str = "VulkSim";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let event_loop = EventLoop::new();
    let _logger = flexi_logger::Logger::try_with_env_or_str("info")?.start()?;
    let mut window = VRTWindow::build_window(&event_loop, APP_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
        .expect("Cannot create window.");

    let app = VRTApp::new(window.get_window_ptr())?;
    app.run(event_loop, &mut window);
    Ok(())
}
