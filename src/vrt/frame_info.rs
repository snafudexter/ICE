use erupt::{
    vk1_0::{CommandBuffer, DescriptorSet},
    SmallVec,
};

use super::game_object::GameObject;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirectionalLight {
    direction: glam::Vec3,
    color: glam::Vec4,
}

impl DirectionalLight {
    pub fn new(direction: glam::Vec3, color: glam::Vec4) -> Self {
        Self { direction, color }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PointLight {
    position: glam::Vec4,
    color: glam::Vec4,
}

impl PointLight {
    pub fn new(position: glam::Vec4, color: glam::Vec4) -> Self {
        Self { position, color }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlobalUBO {
    model_matrix: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
    ambient_light_color: glam::Vec4,
    camera_pos_world: glam::Vec4,
    light: PointLight,
    // num_lights: i32,
}

impl GlobalUBO {
    pub fn new(
        model_matrix: glam::Mat4,
        view: glam::Mat4,
        projection: glam::Mat4,
        ambient_light_color: glam::Vec4,
        point_light: PointLight,
        camera_pos: glam::Vec4,
    ) -> Self {
        Self {
            model_matrix,
            view,
            projection,
            light: point_light,
            ambient_light_color,
            camera_pos_world: camera_pos, // num_lights,
        }
    }
}

pub struct FrameInfo<'a> {
    command_buffer: CommandBuffer,
    game_objects: &'a Vec<GameObject>,
    global_descriptor_set: &'a SmallVec<DescriptorSet>,
}

impl<'a> FrameInfo<'a> {
    pub fn new(
        command_buffer: CommandBuffer,
        game_objects: &'a Vec<GameObject>,
        global_descriptor_set: &'a SmallVec<DescriptorSet>,
    ) -> Self {
        Self {
            command_buffer,
            game_objects,
            global_descriptor_set,
        }
    }

    pub fn get_command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer
    }

    pub fn get_game_objects(&self) -> &Vec<GameObject> {
        &self.game_objects
    }

    pub fn get_global_descriptor_sets(&self) -> &'a SmallVec<DescriptorSet> {
        self.global_descriptor_set
    }
}
