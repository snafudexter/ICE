use std::{ops::Deref, sync::Arc};

use erupt::vk1_0::{
    CommandBuffer, DescriptorSetLayout, PipelineLayout, PipelineLayoutCreateInfoBuilder, RenderPass,
};

use super::{model::Model, pipeline::VRTPipeline};
use crate::vrt::device::device::VRTDevice;

const VERTEX_SHADER: &str = "./assets/shaders/vert.spirv";
const FRAGMENT_SHADER: &str = "./assets/shaders/frag.spirv";

pub struct TriangleRenderSystem {
    pipeline: VRTPipeline,
    device: Arc<VRTDevice>,
    pipeline_layout: PipelineLayout,
}

impl TriangleRenderSystem {
    pub fn new(
        device: Arc<VRTDevice>,
        render_pass: RenderPass,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> Self {
        let mut config_info = VRTPipeline::default_pipeline_config_info();

        let pipeline_layout = Self::create_pipeline_layout(device.clone(), descriptor_set_layout);

        config_info.pipeline_layout = pipeline_layout;

        let pipeline = VRTPipeline::new(
            device.clone(),
            VERTEX_SHADER,
            FRAGMENT_SHADER,
            &mut config_info,
            render_pass,
        );

        Self {
            pipeline,
            device,
            pipeline_layout,
        }
    }

    fn create_pipeline_layout(
        device: Arc<VRTDevice>,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> PipelineLayout {
        let pipeline_layout_info = PipelineLayoutCreateInfoBuilder::new()
            .set_layouts(std::slice::from_ref(descriptor_set_layout));

        unsafe {
            device
                .get_device_ptr()
                .create_pipeline_layout(&pipeline_layout_info, None)
        }
        .unwrap()
    }

    pub fn render(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer, model: &Model) {
        self.pipeline.bind(command_buffer);
        model.bind(device.clone(), command_buffer);
        model.draw(device, command_buffer)
    }
}

impl Drop for TriangleRenderSystem {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
