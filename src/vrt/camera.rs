use dolly::prelude::*;

pub struct VRTCamera {
    camera: CameraRig,
}

impl VRTCamera {
    pub fn new(yaw: f32, pitch: f32) -> Self {
        let camera: CameraRig = CameraRig::builder()
            .with(YawPitch::new().yaw_degrees(yaw).pitch_degrees(pitch))
            .with(Smooth::new_rotation(1.5))
            .with(Arm::new(dolly::glam::Vec3::Z * 8.0))
            .build();

        Self { camera }
    }

    pub fn get_view_matrix(
        position: glam::Vec3,
        direction: glam::Vec3,
        up: glam::Vec3,
    ) -> glam::Mat4 {
        let w = glam::Vec3::normalize(direction);
        let u = glam::Vec3::normalize(glam::Vec3::cross(w, up));
        let v = glam::Vec3::cross(w, u);

        let mut view_matrix = glam::Mat4::IDENTITY;

        view_matrix
    }
}
