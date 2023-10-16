use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, VirtualKeyCode},
};

pub struct FPSCamera {
    sensitivity: f32,
    yaw: f32,
    pitch: f32,
    target: glam::Vec3,
    track_mouse: bool,
    position: glam::Vec3,
    up: glam::Vec3,
    right: glam::Vec3,
    last_x: f64,
    last_y: f64,
}

impl FPSCamera {
    pub fn new(sensitivity: f32, yaw: f32, pitch: f32, up: glam::Vec3) -> Self {
        let mut n_self = Self {
            sensitivity,
            yaw,
            pitch,
            target: glam::Vec3::ONE,
            track_mouse: false,
            position: glam::vec3(0f32, 0f32, 3f32),
            up,
            right: glam::Vec3::X,
            last_x: 400f64,
            last_y: 300f64,
        };
        n_self.update_target();

        let direction = (n_self.position - n_self.target).normalize();
        n_self.right = n_self.up.cross(direction).normalize();
        n_self.up = direction.cross(n_self.right);

        return n_self;
    }

    pub fn update_target(&mut self) {
        let yaw_radians = self.yaw.to_radians();
        let pitch_radians = self.pitch.to_radians();
        let direction = glam::vec3(
            -yaw_radians.sin() * pitch_radians.cos(),
            pitch_radians.sin(),
            -yaw_radians.cos() * pitch_radians.cos(),
        );

        self.target = direction.normalize();
    }

    pub fn process_cursor_move_event(&mut self, position: PhysicalPosition<f64>) {
        if self.track_mouse {
            let x_offset = position.x - self.last_x;
            let y_offset = position.y - self.last_y;

            self.last_x = position.x;
            self.last_y = position.y;

            self.yaw += x_offset as f32 * self.sensitivity;
            self.pitch += y_offset as f32 * self.sensitivity;

            self.update_target();
        }
    }

    pub fn process_keyboard_event(
        &mut self,
        key: VirtualKeyCode,
        state: ElementState,
        frame_time: u32,
    ) {
        let camera_speed = self.sensitivity * frame_time as f32;

        if let (VirtualKeyCode::W, ElementState::Pressed) = (key, state) {
            self.position += camera_speed * self.target;
        }

        if let (VirtualKeyCode::S, ElementState::Pressed) = (key, state) {
            self.position -= camera_speed * self.target;
        }

        if let (VirtualKeyCode::A, ElementState::Pressed) = (key, state) {
            self.position -= self.target.cross(self.up).normalize() * camera_speed;
        }

        if let (VirtualKeyCode::D, ElementState::Pressed) = (key, state) {
            self.position += self.target.cross(self.up).normalize() * camera_speed;
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
