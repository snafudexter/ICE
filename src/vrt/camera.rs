use dolly::prelude::*;
use glam::{Vec3, Quat};

pub struct VRTCamera {
    rig: CameraRig,
    position: Vec3,
    yaw: f32,   // Horizontal angle
    pitch: f32, // Vertical angle
    sensitivity: f32,
    speed: f32,
}

impl VRTCamera {
    /// Creates a new FPS camera with the given parameters
    pub fn new(initial_position: Vec3, sensitivity: f32, speed: f32) -> Self {
        let rig = CameraRig::builder()
            .with(Position::new(initial_position))
            .with(Rotation::new(Quat::IDENTITY))
            .build();

        Self {
            rig,
            position: initial_position,
            yaw: 0.0,
            pitch: 0.0,
            sensitivity,
            speed,
        }
    }

    /// Updates the camera's position and rotation
    pub fn update(&mut self, delta_time: f32) {
        self.rig.update(delta_time);
        self.position = self.rig.driver_mut::<Position>().position.into();
    }

    /// Processes mouse movement to adjust the camera's yaw and pitch
    pub fn process_mouse_movement(&mut self, x_offset: f32, y_offset: f32) {
        self.yaw += x_offset * self.sensitivity;
        self.pitch -= y_offset * self.sensitivity; // Invert pitch for natural feel

        // Clamp the pitch to avoid gimbal lock
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        self.update_rotation();
    }

    /// Moves the camera in a given direction (WASD-style)
    pub fn process_keyboard_input(&mut self, direction: Vec3, delta_time: f32) {
        let velocity = self.speed * delta_time;

        // Calculate forward and right vectors
        let forward = Vec3::new(
            self.yaw.to_radians().cos(),
            0.0,
            self.yaw.to_radians().sin(),
        )
        .normalize();

        let right = forward.cross(Vec3::Y).normalize();

        // Apply movement based on input direction
        self.position += direction.x * right * velocity;
        self.position += direction.z * forward * velocity;

        self.rig.driver_mut::<Position>().translate(self.position);
    }

    /// Updates the camera's rotation based on the yaw and pitch
    fn update_rotation(&mut self) {
        let rotation = Quat::from_rotation_y(self.yaw.to_radians())
            * Quat::from_rotation_x(self.pitch.to_radians());
        self.rig.driver_mut::<Rotation>().rotation = rotation.into();
    }

    /// Gets the camera's position
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Gets the camera's orientation
    pub fn orientation(&self) -> Quat {
        self.rig.driver::<Rotation>().rotation.into()
    }
}



// use dolly::{prelude::*, transform::Transform};

// pub struct VRTCamera {
//     camera: CameraRig,
//     track_mouse: bool,
// }

// impl VRTCamera {
//     pub fn new() -> Self {
//         let mut camera = CameraRig::builder()
//             .with(Position::new(glam::vec3(0f32, 0f32, -5f32)))
//             .with(YawPitch::new())
//             .with(Smooth::new_position_rotation(1.0, 1.0))
//             .build();

//         Self {
//             camera,
//             track_mouse: false,
//         }
//     }

//     pub fn update(&mut self, frame_time: f32) -> Transform<RightHanded> {
//         self.camera.update(frame_time)
//     }

//     pub fn track_mouse(&mut self, track_mouse: bool) {
//         self.track_mouse = track_mouse;
//     }

//     pub fn process_cursor_move_event(&mut self, dx: f32, dy: f32) {
//         if self.track_mouse {
//             self.camera
//                 .driver_mut::<YawPitch>()
//                 .rotate_yaw_pitch(-0.1 * dx, 0.1 * dy);
//         }
//     }

//     pub fn translate_camera(&mut self, move_vec: glam::Vec3, frame_time: f32) {
//         let m_vec = self.camera.final_transform.rotation * move_vec.clamp_length_max(1.0);
//         self.camera
//             .driver_mut::<Position>()
//             .translate(m_vec * frame_time * 10.0);
//     }
// }
