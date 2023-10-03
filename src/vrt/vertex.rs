use erupt::vk::{
    Format, VertexInputAttributeDescriptionBuilder, VertexInputBindingDescriptionBuilder,
    VertexInputRate,
};
use glam::{Vec2, Vec3};

macro_rules! size_of {
    ($ty:ty) => {
        std::mem::size_of::<$ty>() as u32
    };
}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {{
        let base = std::mem::MaybeUninit::<$ty>::uninit();
        let base_ptr = base.as_ptr();
        let field_ptr = unsafe { std::ptr::addr_of!((*base_ptr).$field) };
        unsafe { field_ptr.cast::<u8>().offset_from(base_ptr.cast::<u8>()) as u32 }
    }};
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Vertex {
    position: Vec3,
    color: Vec3,
    normal: Vec3,
}

impl Vertex {
    pub const VERTICES: [Vertex; 3] = [
        Vertex::new(
            Vec3::from_array([0.0, 1.0, 0.0]),
            Vec3::from_array([1.0, 0.0, 0.0]),
            Vec3::from_array([0.0, 0.0, -1.0]),
        ),
        Vertex::new(
            Vec3::from_array([-1.0, 1.0, 0.0]),
            Vec3::from_array([0.0, 1.0, 0.0]),
            Vec3::from_array([0.0, 0.0, -1.0]),
        ),
        Vertex::new(
            Vec3::from_array([1.0, 0.0, 0.0]),
            Vec3::from_array([0.0, 1.0, 1.0]),
            Vec3::from_array([0.0, 0.0, -1.0]),
        ),
    ];

    pub const fn new(position: Vec3, color: Vec3, normal: Vec3) -> Self {
        Self {
            position,
            color,
            normal,
        }
    }

    pub fn binding_description() -> VertexInputBindingDescriptionBuilder<'static> {
        VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(size_of!(Self))
            .input_rate(VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [VertexInputAttributeDescriptionBuilder<'static>; 3] {
        [
            VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(0)
                .format(Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, position)),
            VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(1)
                .format(Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, color)),
            VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(2)
                .format(Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, normal)),
        ]
    }
}
