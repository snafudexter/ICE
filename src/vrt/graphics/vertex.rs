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
    position: Vec2,
    color: Vec3,
}

impl Vertex {
    // pub const VERTICES: [Vertex; 3] = [
    //     Vertex::new(const_vec2!([0.0, -0.5]), const_vec3!([1.0, 0.0, 0.0])),
    //     Vertex::new(const_vec2!([0.5, 0.5]), const_vec3!([0.0, 1.0, 0.0])),
    //     Vertex::new(const_vec2!([-0.5, 0.5]), const_vec3!([0.0, 0.0, 1.0])),
    // ];

    // pub const fn new(position: Vec2, color: Vec3) -> Self {
    //     Self { position, color }
    // }

    pub fn binding_description() -> VertexInputBindingDescriptionBuilder<'static> {
        VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(size_of!(Self))
            .input_rate(VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [VertexInputAttributeDescriptionBuilder<'static>; 2] {
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
        ]
    }
}
