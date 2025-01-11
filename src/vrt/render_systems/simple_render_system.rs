use std::{rc::Rc, sync::Arc};

use erupt::{vk::{BufferUsageFlags, DescriptorSet, DescriptorType, DeviceSize, MemoryPropertyFlags, ShaderStageFlags}, vk1_0::{
    CommandBuffer, DescriptorSetLayout, PipelineBindPoint, PipelineLayout,
    PipelineLayoutCreateInfoBuilder, RenderPass,
}, SmallVec};

use crate::vrt::{
    buffer::VRTBuffer, descriptor_pool::{VRTDescriptorPool, VRTDescriptorPoolBuilder, VRTDescriptorWriter}, device::VRTDevice, fps_camera::FPSCamera, frame_info::{self, FrameInfo, GlobalUBO, PointLight}, game_object::GameObject, layout::VRTDescriptorSetLayoutBuilder, model::Model, pipeline::VRTPipeline, swapchain::MAX_FRAMES_IN_FLIGHT
};

const VERTEX_SHADER: &str = "./assets/shaders/vert.spirv";
const FRAGMENT_SHADER: &str = "./assets/shaders/frag.spirv";

pub struct SimpleRenderSystem {
    aspect_ratio: f32,
    pipeline: VRTPipeline,
    device: Arc<VRTDevice>,
    ubo_buffers: Vec<VRTBuffer>,
    descriptor_sets: Vec<SmallVec<DescriptorSet>>,
    global_pool: Rc<VRTDescriptorPool>,
    pipeline_layout: PipelineLayout,
}

impl SimpleRenderSystem {
    pub fn new(
        device: Arc<VRTDevice>,
        render_pass: RenderPass,
        width: u32,
        height: u32
    ) -> Self {
        let mut config_info = VRTPipeline::default_pipeline_config_info();

        let global_pool = std::rc::Rc::new(
            VRTDescriptorPoolBuilder::new(device.clone())
                .set_max_sets(u32::try_from(MAX_FRAMES_IN_FLIGHT).unwrap())
                .add_pool_size(
                    DescriptorType::UNIFORM_BUFFER,
                    MAX_FRAMES_IN_FLIGHT as u32,
                )
                .add_pool_size(DescriptorType::COMBINED_IMAGE_SAMPLER, MAX_FRAMES_IN_FLIGHT as u32)
                .build(),
        );

        let mut ubo_buffers: Vec<VRTBuffer> = vec![];
        for _i in 0..MAX_FRAMES_IN_FLIGHT {
            let mut ubo_buffer = VRTBuffer::new(
                device.clone(),
                std::mem::size_of::<GlobalUBO>() as DeviceSize,
                1,
                BufferUsageFlags::UNIFORM_BUFFER,
                MemoryPropertyFlags::HOST_VISIBLE,
                None,
            );
            ubo_buffer.map(Some((std::mem::size_of::<GlobalUBO>()) as DeviceSize), None);
            ubo_buffers.push(ubo_buffer);
        }

        let global_descriptor_set_layout = VRTDescriptorSetLayoutBuilder::new(device.clone())
            .add_binding(
                0,
                DescriptorType::UNIFORM_BUFFER,
                ShaderStageFlags::ALL_GRAPHICS,
                Some(1),
            )
            .add_binding(1, DescriptorType::COMBINED_IMAGE_SAMPLER, ShaderStageFlags::FRAGMENT, Some(1))
            .build();

        let global_descriptor_set_layout = Rc::new(global_descriptor_set_layout);



        let mut global_descriptor_sets: Vec<SmallVec<DescriptorSet>> = vec![].into();

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info =
                ubo_buffers[i].get_buffer_info(std::mem::size_of::<GlobalUBO>() as DeviceSize);
            global_descriptor_sets.push(
                VRTDescriptorWriter::new(global_descriptor_set_layout.clone(), global_pool.clone())
                    .write_buffer(0, &buffer_info)
                    .build(i)
                    .unwrap(),
            )
        }

        let pipeline_layout = Self::create_pipeline_layout(device.clone(), global_descriptor_set_layout.get_descriptor_set_layout());

        config_info.pipeline_layout = pipeline_layout;

        let pipeline = VRTPipeline::new(
            device.clone(),
            VERTEX_SHADER,
            FRAGMENT_SHADER,
            &mut config_info,
            render_pass,
        );

        Self {
            aspect_ratio: width as f32 / height as f32,
            pipeline,
            device,
            pipeline_layout,
            ubo_buffers,
            descriptor_sets: global_descriptor_sets,
            global_pool,
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

    pub fn update(&self, camera: &FPSCamera, frame_index: &usize) {
        let perspective =
            glam::Mat4::perspective_rh(45.0f32.to_radians(), self.aspect_ratio, 0.01f32, 100.0f32);

        // perspective.y_axis.y *= -1f32;

        let model_matrix = glam::Mat4::from_scale(glam::Vec3::ONE * 2f32)
            * glam::Mat4::from_translation(glam::vec3(0f32, 0f32, 0f32));

        let global_ubo = GlobalUBO::new(
            model_matrix,
            glam::Mat4::look_at_rh(
                *camera.get_position(),
                *camera.get_target(),
                glam::Vec3::Y,
            ),
            perspective,
            glam::vec4(1.0, 1.0, 1f32, 0.1),
            PointLight::new(
                glam::vec4(2.0f32, 10f32, 5.0f32, 1.0f32),
                glam::vec4(0.5, 1.0, 1.0, 1.0),
            ),
            glam::vec4(
                camera.get_position().x,
                camera.get_position().y,
                camera.get_position().z,
                1.0,
            ),
        );

        //println!("view position {:?}", self.camera.get_position());

        self.ubo_buffers[*frame_index as usize].write_to_buffer(
            &global_ubo,
            self.ubo_buffers[*frame_index as usize]
                .get_mapped_memory()
                .unwrap(),
            1 as DeviceSize,
            0,
        );
    }

    pub fn render(&self, device: Arc<VRTDevice>, frame_index: &usize, command_buffer: CommandBuffer, game_objects: &Vec<GameObject>) {

        let frame_info = FrameInfo::new(
            command_buffer,
            game_objects,
            &self.descriptor_sets[*frame_index],
        );


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
