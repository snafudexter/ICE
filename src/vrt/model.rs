use std::{convert::TryInto, fs::File, io::BufReader, mem, sync::Arc};

use erupt::vk1_0::{
    Buffer, BufferCopyBuilder, BufferUsageFlags, CommandBuffer, CommandBufferAllocateInfoBuilder,
    CommandBufferBeginInfoBuilder, CommandBufferLevel, CommandBufferUsageFlags, CommandPool,
    DeviceSize, Fence, Format, IndexType, MemoryPropertyFlags, Queue, SubmitInfoBuilder,
    VertexInputAttributeDescriptionBuilder, VertexInputBindingDescriptionBuilder, VertexInputRate,
};
use erupt::{vk, DeviceLoader};

use super::texture::VRTTexture;
use super::{buffer::VRTBuffer, device::VRTDevice, result::VkResult};

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

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ModelVertex {
    pub position: glam::Vec3,
    pub tex_coords: glam::Vec2,
    pub normal: glam::Vec3,
}

impl ModelVertex {
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
                .format(Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, position)),
            VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(1)
                .format(Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, tex_coords)),
            VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(2)
                .format(Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, normal)),
        ]
    }
}

#[derive(Debug)]
struct MeshData {
    first_index: u32, // Starting index for drawing
    index_count: u32,
    vertex_offset: i32,
    base_color_texture: Option<VRTTexture>
}


pub struct Model {
    vertex_buffer: VRTBuffer,
    index_buffer: VRTBuffer,
    meshes: Vec<MeshData>,
}

impl Model {
    pub fn new(device: Arc<VRTDevice>, path: &str) -> VkResult<Self> {
        let scenes = easy_gltf::load(path).expect("Failed to load glTF");
        let mut meshes: Vec<MeshData> = Vec::new();
        let mut vertices: Vec<ModelVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for scene in scenes {
            for model in scene.models {

                let first_index = indices.len() as u32;
                let vertex_offset = vertices.len() as i32;

                for vertex in model.vertices() {
                    vertices.push(ModelVertex {
                        position: glam::Vec3::new(
                            vertex.position.x,
                            vertex.position.y,
                            vertex.position.z,
                        ),
                        normal: glam::Vec3::new(vertex.normal.x, vertex.normal.y, vertex.normal.z),
                        tex_coords: glam::Vec2::new(vertex.tex_coords.x, vertex.tex_coords.y),
                    });
                }

                indices.extend_from_slice(&model.indices().unwrap());

                meshes.push(MeshData {
                    first_index,
                    index_count: model.indices().unwrap().len() as u32,
                    vertex_offset,
                    base_color_texture: match model.material().pbr.base_color_texture.clone() {
                        Some(base_color_texture) => {Some(VRTTexture::new(device.clone(), base_color_texture.as_raw(), base_color_texture.width(), base_color_texture.height(), vk::Format::R8G8B8A8_SRGB).unwrap())},
                        _ =>  None
                    } 
                });
            }
        }

        // Create buffers for vertex and index data
        let vertex_buffer = Self::create_vertex_buffer(
            device.clone(),
            vertices.clone()
        )?;
        let index_buffer = Self::create_index_buffer(
            device.clone(),
            indices.clone()
        )?;

        Ok(Self {
            vertex_buffer,
            index_buffer,
            meshes,
        })
    }

    pub fn bind(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer) {
        unsafe {
            device.get_device_ptr().cmd_bind_vertex_buffers(
                command_buffer,
                0,
                std::slice::from_ref(&self.vertex_buffer.get_buffer()),
                &[0],
            );

            device.get_device_ptr().cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer.get_buffer(),
                0,
                IndexType::UINT32,
            );
        }
    }

    pub fn draw(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer) {
        unsafe {
            for mesh in &self.meshes {
                device.get_device_ptr().cmd_draw_indexed(
                    command_buffer,
                    mesh.index_count, // Number of indices to draw
                    1,                         // Instance count
                    mesh.first_index,          // Starting index in the index buffer
                    mesh.vertex_offset,                         // Vertex offset
                    0,                         // First instance
                );
            }
        }
    }

    fn create_index_buffer(device: Arc<VRTDevice>, indices: Vec<u32>) -> VkResult<VRTBuffer> {
        let buffer_size = (mem::size_of::<u32>() * indices.len()) as u64;

        let mut staging_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<u32>().try_into()?,
            indices.len().try_into()?,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            None,
        );

        staging_buffer.map(Some(buffer_size), Some(0));
        staging_buffer.write_to_buffer(
            indices.as_ptr(),
            staging_buffer.get_mapped_memory().unwrap(),
            indices.len() as DeviceSize,
            0,
        );
        staging_buffer.unmap();

        let index_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<u32>().try_into()?,
            indices.len().try_into()?,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
            None,
        );

        Self::copy_buffer(
            &device.get_device_ptr(),
            device.get_queues().graphics,
            device.get_command_pool(),
            staging_buffer.get_buffer(),
            index_buffer.get_buffer(),
            buffer_size,
        )?;

        Ok(index_buffer)
    }

    fn create_vertex_buffer(
        device: Arc<VRTDevice>,
        vertices: Vec<ModelVertex>,
    ) -> VkResult<VRTBuffer> {
        let buffer_size = (mem::size_of::<ModelVertex>() * vertices.len()) as DeviceSize;

        let mut staging_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<ModelVertex>().try_into()?,
            vertices.len().try_into()?,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            None,
        );

        staging_buffer.map(Some(buffer_size), Some(0));
        staging_buffer.write_to_buffer(
            vertices.as_ptr(),
            staging_buffer.get_mapped_memory().unwrap(),
            vertices.len() as DeviceSize,
            0,
        );
        staging_buffer.unmap();

        let vertex_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<ModelVertex>().try_into()?,
            vertices.len().try_into()?,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
            None,
        );

        Self::copy_buffer(
            &device.get_device_ptr(),
            device.get_queues().graphics,
            device.get_command_pool(),
            staging_buffer.get_buffer(),
            vertex_buffer.get_buffer(),
            buffer_size,
        )?;

        Ok(vertex_buffer)
    }

    fn copy_buffer(
        device: &DeviceLoader,
        graphics_queue: Queue,
        command_pool: CommandPool,
        src: Buffer,
        dst: Buffer,
        size: DeviceSize,
    ) -> VkResult<()> {
        let alloc_info = CommandBufferAllocateInfoBuilder::new()
            .level(CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info) }.result()?[0];

        let begin_info =
            CommandBufferBeginInfoBuilder::new().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.result()?;

        let copy_region = BufferCopyBuilder::new().size(size);
        unsafe {
            device.cmd_copy_buffer(command_buffer, src, dst, std::slice::from_ref(&copy_region))
        };

        unsafe { device.end_command_buffer(command_buffer) }.result()?;

        let submit_info =
            SubmitInfoBuilder::new().command_buffers(std::slice::from_ref(&command_buffer));

        unsafe {
            device.queue_submit(
                graphics_queue,
                std::slice::from_ref(&submit_info),
                Fence::null(),
            )
        }
        .result()?;
        unsafe { device.queue_wait_idle(graphics_queue) }.result()?;

        unsafe { device.free_command_buffers(command_pool, std::slice::from_ref(&command_buffer)) };

        Ok(())
    }
}
