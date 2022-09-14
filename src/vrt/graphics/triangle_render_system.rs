use std::sync::Arc;

use super::pipeline::VRTPipeline;
use crate::vrt::device::device::VRTDevice;

const VERTEX_SHADER: &str = "./assets/shaders/vert.spirv";
const FRAGMENT_SHADER: &str = "./assets/shaders/vert.spirv";

pub struct TriangleRenderSystem {}

impl TriangleRenderSystem {
    pub fn new(device: Arc<VRTDevice>) -> Self {
        let configInfo = VRTPipeline::default_pipeline_config_info();

        let pipeline = VRTPipeline::new(device, VERTEX_SHADER, FRAGMENT_SHADER, &configInfo);

        Self {}
    }
}
