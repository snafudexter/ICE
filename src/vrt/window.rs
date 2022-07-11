use erupt::vk::Extent2D;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::error::OsError;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct VRTWindow {
    resized: bool,
    width: u32,
    height: u32,
    window: Window,
}

impl VRTWindow {
    pub fn build_window(
        event_loop: &EventLoop<()>,
        app_name: &str,
        width: u32,
        height: u32,
    ) -> Result<Self, OsError> {
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

        Ok(Self {
            window,
            width,
            height,
            resized: false,
        })
    }

    pub fn get_extent(&self) -> Extent2D {
        Extent2D {
            width: self.width,
            height: self.height,
        }
    }

    pub fn get_window_ptr(&self) -> &Window {
        &self.window
    }

    pub fn resize_callback(&mut self, new_inner_size: winit::dpi::PhysicalSize<u32>) {
        self.width = new_inner_size.width;
        self.height = new_inner_size.height;
        self.resized = true;
    }

    fn reset_resized_flag(&mut self) {
        self.resized = false;
    }
}
