use std::{convert::TryInto, mem, sync::Arc};

use erupt::{
    vk1_0::{
        Buffer, BufferCopyBuilder, BufferUsageFlags, CommandBuffer,
        CommandBufferAllocateInfoBuilder, CommandBufferBeginInfoBuilder, CommandBufferLevel,
        CommandBufferUsageFlags, CommandPool, DeviceSize, Fence, Format, IndexType,
        MemoryPropertyFlags, Queue, SubmitInfoBuilder, VertexInputAttributeDescriptionBuilder,
        VertexInputBindingDescriptionBuilder, VertexInputRate,
    },
    DeviceLoader,
};

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
#[derive(Copy, Clone, Debug)]
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

pub struct MeshObject {
    pub first_index: u32, // LOL name; first index in index buffer
    pub indices_count: u32,
}

pub struct Model {
    device: Arc<VRTDevice>,
    vertex_buffer: VRTBuffer,
    index_buffer: VRTBuffer,
    meshes: Vec<MeshObject>,
}

impl Model {
    pub fn new(device: Arc<VRTDevice>, path: &str) -> Self {
        let (models, materials) =
            tobj::load_obj(&path, &tobj::GPU_LOAD_OPTIONS).expect("Failed to OBJ load file");

        let mut all_vertices: Vec<ModelVertex> = vec![];
        let mut indices: Vec<u16> = vec![];
        let mut meshes: Vec<MeshObject> = vec![];
        let mut unique_vertices = std::collections::HashMap::new();

        for model in &models {
            let first_index = all_vertices.len() as u32;
            for index in &model.mesh.indices {
                let pos_offset = (3 * index) as usize;
                let tex_coord_offset = (2 * index) as usize;

                let vertex = ModelVertex {
                    position: glam::vec3(
                        model.mesh.positions[pos_offset],
                        model.mesh.positions[pos_offset + 1],
                        model.mesh.positions[pos_offset + 2],
                    ),
                    tex_coords: glam::vec2(
                        model.mesh.texcoords[tex_coord_offset],
                        model.mesh.texcoords[tex_coord_offset + 1],
                    ),
                    normal: glam::vec3(
                        model.mesh.normals[pos_offset],
                        model.mesh.normals[pos_offset + 1],
                        model.mesh.normals[pos_offset + 2],
                    ),
                };

                if let Some(index) = unique_vertices.get(&vertex) {
                    indices.push(*index as u16);
                } else {
                    let index = all_vertices.len();
                    unique_vertices.insert(vertex, index);
                    all_vertices.push(vertex);
                    indices.push(index as u16);
                }
            }
            meshes.push(MeshObject {
                first_index,
                indices_count: model.mesh.indices.len() as u32,
            });
        }

        let vertex_buffer = Self::create_vertex_buffer(device.clone(), all_vertices).unwrap();
        let index_buffer = Self::create_index_buffer(device.clone(), indices).unwrap();

        Self {
            device,
            vertex_buffer,
            index_buffer,
            meshes,
        }
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
                IndexType::UINT16,
            );
        }
    }

    pub fn draw(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer) {
        unsafe {
            for mesh in self.meshes.iter() {
                device.get_device_ptr().cmd_draw_indexed(
                    command_buffer,
                    mesh.indices_count as u32,
                    1,
                    mesh.first_index,
                    0,
                    0,
                );
            }
        }
    }

    fn create_index_buffer(device: Arc<VRTDevice>, indices: Vec<u16>) -> VkResult<VRTBuffer> {
        let buffer_size = (mem::size_of::<u16>() * indices.len()) as u64;
        let mut staging_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<u16>().try_into().unwrap(),
            indices.len().try_into().unwrap(),
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
            mem::size_of::<u16>().try_into().unwrap(),
            indices.len().try_into().unwrap(),
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
            mem::size_of::<ModelVertex>().try_into().unwrap(),
            vertices.len().try_into().unwrap(),
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
            mem::size_of::<ModelVertex>().try_into().unwrap(),
            vertices.len().try_into().unwrap(),
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

impl PartialEq for ModelVertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.normal == other.normal
            && self.tex_coords == other.tex_coords
    }
}

impl Eq for ModelVertex {}

impl std::hash::Hash for ModelVertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position[0].to_bits().hash(state);
        self.position[1].to_bits().hash(state);
        self.position[2].to_bits().hash(state);
        self.normal[0].to_bits().hash(state);
        self.normal[1].to_bits().hash(state);
        self.normal[2].to_bits().hash(state);
        self.tex_coords[0].to_bits().hash(state);
        self.tex_coords[1].to_bits().hash(state);
    }
}
