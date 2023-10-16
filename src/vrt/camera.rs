use dolly::{prelude::*, transform::Transform};

pub struct VRTCamera {
    camera: CameraRig,
    track_mouse: bool,
}

impl VRTCamera {
    pub fn new() -> Self {
        let mut camera = CameraRig::builder()
            .with(Position::new(glam::vec3(0f32, 0f32, -5f32)))
            .with(YawPitch::new())
            .with(Smooth::new_position_rotation(1.0, 1.0))
            .build();

        Self {
            camera,
            track_mouse: false,
        }
    }

    pub fn update(&mut self, frame_time: f32) -> Transform<RightHanded> {
        self.camera.update(frame_time)
    }

    pub fn track_mouse(&mut self, track_mouse: bool) {
        self.track_mouse = track_mouse;
    }

    pub fn process_cursor_move_event(&mut self, dx: f32, dy: f32) {
        if self.track_mouse {
            self.camera
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-0.1 * dx, 0.1 * dy);
        }
    }

    pub fn translate_camera(&mut self, move_vec: glam::Vec3, frame_time: f32) {
        let m_vec = self.camera.final_transform.rotation * move_vec.clamp_length_max(1.0);
        self.camera
            .driver_mut::<Position>()
            .translate(m_vec * frame_time * 10.0);
    }
}
