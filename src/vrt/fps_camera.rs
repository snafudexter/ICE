use std::f32::consts::PI;

const DEF_FOV: f32 = 45.0;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, VirtualKeyCode},
};

pub struct FPSCamera {
    sensitivity: f32,
    mouse_sensitivity: f32,
    yaw: f32,
    pitch: f32,
    target: glam::Vec3,
    look: glam::Vec3,
    track_mouse: bool,
    position: glam::Vec3,
    up: glam::Vec3,
    WORLD_UP: glam::Vec3,
    right: glam::Vec3,
}

impl FPSCamera {
    pub fn new(
        sensitivity: f32,
        mouse_sensitivity: f32,
        position: glam::Vec3,
        target: glam::Vec3,
        up: glam::Vec3,
    ) -> Self {
        let look_dir = position - target;

        let pitch = -look_dir
            .y
            .atan2((look_dir.x * look_dir.x + look_dir.z * look_dir.z).sqrt());

        let yaw = look_dir.x.atan2(look_dir.z) + PI;

        let mut _self = Self {
            mouse_sensitivity,
            sensitivity,
            yaw,
            pitch,
            target: glam::Vec3::ZERO,
            track_mouse: false,
            position,
            up,
            right: glam::Vec3::X,
            look: glam::Vec3::NEG_Z,
            WORLD_UP: glam::Vec3::Y,
        };
        _self.update_camera_vectors();

        _self
    }

    fn move_camera(&mut self, offset: glam::Vec3) {
        self.position += offset;
        self.update_camera_vectors();
    }

    pub fn update_camera_vectors(&mut self) {
        let mut look: glam::Vec3 = glam::Vec3::ZERO;
        look.x = self.pitch.cos() * self.yaw.sin();
        look.y = self.pitch.sin();
        look.z = self.pitch.cos() * self.yaw.cos();

        self.look = look.normalize();
        self.right = self.look.cross(self.WORLD_UP).normalize();
        self.up = self.right.cross(self.look).normalize();
        self.target = self.position + self.look;
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        let two_pi = PI * 2f32;
        self.yaw += (yaw * self.mouse_sensitivity).to_radians();
        self.pitch += (pitch * self.mouse_sensitivity).to_radians();

        self.pitch = self
            .pitch
            .clamp(-PI / 2.0f32 + 0.1f32, PI / 2.0f32 - 0.1f32);

        if self.yaw > two_pi {
            self.yaw -= two_pi;
        } else if self.yaw < 0f32 {
            self.yaw += two_pi;
        }

        self.update_camera_vectors();
    }

    pub fn get_up(&self) -> glam::Vec3 {
        self.up
    }

    pub fn get_look(&self) -> glam::Vec3 {
        self.look
    }

    pub fn get_right(&self) -> glam::Vec3 {
        self.right
    }

    pub fn process_keyboard_event(
        &mut self,
        key: VirtualKeyCode,
        state: ElementState,
        frame_time: u32,
    ) {
        let camera_speed = self.sensitivity * frame_time as f32;

        if let (VirtualKeyCode::W, ElementState::Pressed) = (key, state) {
            self.move_camera(camera_speed * self.get_look());
        }

        if let (VirtualKeyCode::S, ElementState::Pressed) = (key, state) {
            self.move_camera(camera_speed * -self.get_look());
        }

        if let (VirtualKeyCode::A, ElementState::Pressed) = (key, state) {
            self.move_camera(camera_speed * -self.get_right());
        }

        if let (VirtualKeyCode::D, ElementState::Pressed) = (key, state) {
            self.move_camera(camera_speed * self.get_right());
        }

        if let (VirtualKeyCode::Q, ElementState::Pressed) = (key, state) {
            self.move_camera(camera_speed * -self.get_up());
        }

        if let (VirtualKeyCode::E, ElementState::Pressed) = (key, state) {
            self.move_camera(camera_speed * self.get_up());
        }
    }

    pub fn track_mouse(&mut self, track_mouse: bool) {
        self.track_mouse = track_mouse;
    }

    pub fn get_position(&self) -> &glam::Vec3 {
        &self.position
    }

    pub fn get_target(&self) -> &glam::Vec3 {
        &self.target
    }
}
