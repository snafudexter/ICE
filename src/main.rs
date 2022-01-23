mod vrt;
use vrt::app::VRTApp;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _logger = flexi_logger::Logger::try_with_env_or_str("info")?.start()?;
    let app = VRTApp::new();
    app.run();
}
