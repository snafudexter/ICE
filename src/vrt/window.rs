use winit::dpi::{LogicalPosition, LogicalSize};
use winit::error::OsError;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct VRTWindow {}

impl VRTWindow {
    pub fn build_window(
        event_loop: &EventLoop<()>,
        app_name: &str,
        width: u32,
        height: u32,
    ) -> Result<Window, OsError> {
        let window = WindowBuilder::new()
            .with_title(app_name)
            .with_inner_size(LogicalSize::new(width, height))
            .with_resizable(false)
            .build(event_loop)?;

        if let Some(monitor) = window.current_monitor() {
            let monitor_size = monitor.size().to_logical::<f64>(monitor.scale_factor());

            window.set_outer_position(LogicalPosition::new(
                (monitor_size.width - f64::from(width)) / 2.0,
                (monitor_size.height - f64::from(height)) / 2.0,
            ));
        }

        Ok(window)
    }
}
