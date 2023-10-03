use std::sync::Arc;

use erupt::vk1_0::{
    CommandBuffer, DescriptorSetLayout, PipelineBindPoint, PipelineLayout,
    PipelineLayoutCreateInfoBuilder, RenderPass,
};

use crate::vrt::{
    device::VRTDevice,
    frame_info::{self, FrameInfo},
    model::Model,
    pipeline::VRTPipeline,
};

const VERTEX_SHADER: &str = "./assets/shaders/vert.spirv";
const FRAGMENT_SHADER: &str = "./assets/shaders/frag.spirv";

pub struct SimpleRenderSystem {
    pipeline: VRTPipeline,
    device: Arc<VRTDevice>,
    pipeline_layout: PipelineLayout,
}

impl SimpleRenderSystem {
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

    pub fn render(&self, device: Arc<VRTDevice>, frame_info: FrameInfo) {
        self.pipeline.bind(*frame_info.get_command_buffer());

        unsafe {
            device.get_device_ptr().cmd_bind_descriptor_sets(
                *frame_info.get_command_buffer(),
                PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                frame_info.get_global_descriptor_sets(),
                &[],
            );
        }

        for game_object in frame_info.get_game_objects().iter() {
            let command_buffer = *frame_info.get_command_buffer();
            let model = game_object.get_model();
            model.bind(device.clone(), command_buffer);
            model.draw(device.clone(), command_buffer);
        }
    }
}

impl Drop for SimpleRenderSystem {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
