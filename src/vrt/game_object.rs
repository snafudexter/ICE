use super::model::Model;

struct TransformComponent {
    translation: glam::Vec3,
    scale: glam::Vec3,
    rotation: glam::Vec3,
}

pub struct GameObject {
    transform_component: TransformComponent,
    model: Option<Model>,
}

impl GameObject {
    pub fn new() -> Self {
        Self {
            transform_component: TransformComponent {
                translation: glam::Vec3::ZERO,
                scale: glam::Vec3::ONE,
                rotation: glam::Vec3::ZERO,
            },
            model: None,
        }
    }
}
