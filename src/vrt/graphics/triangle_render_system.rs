use std::{ops::Deref, sync::Arc};

use erupt::vk1_0::{CommandBuffer, RenderPass};

use super::{model::Model, pipeline::VRTPipeline};
use crate::vrt::device::device::VRTDevice;

const VERTEX_SHADER: &str = "./assets/shaders/vert.spirv";
const FRAGMENT_SHADER: &str = "./assets/shaders/frag.spirv";

pub struct TriangleRenderSystem {
    pipeline: VRTPipeline,
    device: Arc<VRTDevice>,
}

impl TriangleRenderSystem {
    pub fn new(device: Arc<VRTDevice>, render_pass: RenderPass) -> Self {
        let mut config_info = VRTPipeline::default_pipeline_config_info();

        let pipeline = VRTPipeline::new(
            device.clone(),
            VERTEX_SHADER,
            FRAGMENT_SHADER,
            &mut config_info,
            render_pass,
        );

        Self { pipeline, device }
    }

    pub fn render(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer, model: &Model) {
        self.pipeline.bind(command_buffer);
        model.bind(device.clone(), command_buffer);
        model.draw(device, command_buffer)
    }
}
