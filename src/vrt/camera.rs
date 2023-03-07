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
}
