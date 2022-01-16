mod vrt;
use vrt::app::VRTApp;

fn main() {
    let app = VRTApp::new();
    app.run();
}
