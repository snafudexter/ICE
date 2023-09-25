use erupt::vk1_0::CommandBuffer;

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
pub struct GlobalUBO {
    model: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
    inverse_view: glam::Mat4,
    ambient_light_color: glam::Vec4,
    directional_lights: Vec<DirectionalLight>,
}

impl GlobalUBO {
    pub fn new(
        model: glam::Mat4,
        view: glam::Mat4,
        projection: glam::Mat4,
        ambient_light_color: glam::Vec4,
        directional_lights: Vec<DirectionalLight>,
    ) -> Self {
        Self {
            model,
            view,
            inverse_view: view.inverse(),
            projection,
            directional_lights,
            ambient_light_color,
        }
    }
}

pub struct FrameInfo {
    frame_index: usize,
    frame_time: u128,
    command_buffer: CommandBuffer,
    game_object: Vec<GameObject>,
}

impl FrameInfo {
    pub fn new(
        frame_index: usize,
        frame_time: u128,
        command_buffer: CommandBuffer,
        game_object: Vec<GameObject>,
    ) -> Self {
        Self {
            frame_index,
            frame_time,
            command_buffer,
            game_object,
        }
    }
}
