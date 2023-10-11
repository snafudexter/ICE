use std::{convert::TryInto, mem, sync::Arc};

use erupt::{
    vk1_0::{
        Buffer, BufferCopyBuilder, BufferCreateInfoBuilder, BufferUsageFlags, CommandBuffer,
        CommandBufferAllocateInfoBuilder, CommandBufferBeginInfoBuilder, CommandBufferLevel,
        CommandBufferUsageFlags, CommandPool, DeviceMemory, DeviceSize, Fence, IndexType,
        MemoryAllocateInfoBuilder, MemoryPropertyFlags, PhysicalDevice, Queue, SharingMode,
        SubmitInfoBuilder,
    },
    DeviceLoader, InstanceLoader,
};

use super::{
    buffer::VRTBuffer,
    device::VRTDevice,
    result::{VkError, VkResult},
    vertex::Vertex,
};

pub struct Model {
    device: Arc<VRTDevice>,
    vertex_buffer: VRTBuffer,
    index_buffer: VRTBuffer,
}

impl Model {
    pub fn new(device: Arc<VRTDevice>) -> Self {
        let vertex_buffer = Self::create_vertex_buffer(device.clone()).unwrap();
        let index_buffer = Self::create_index_buffer(device.clone()).unwrap();

        Self {
            device,
            vertex_buffer,
            index_buffer,
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
            device.get_device_ptr().cmd_draw_indexed(
                command_buffer,
                Vertex::INDICES.len() as u32,
                1,
                0,
                0,
                0,
            );
            // device.get_device_ptr().cmd_draw(
            //     command_buffer,
            //     Vertex::VERTICES.len().try_into().unwrap(),
            //     1,
            //     0,
            //     0,
            // );
        }
    }

    fn create_index_buffer(device: Arc<VRTDevice>) -> VkResult<VRTBuffer> {
        let buffer_size = (mem::size_of::<u16>() * Vertex::INDICES.len()) as u64;
        let mut staging_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<u16>().try_into().unwrap(),
            Vertex::INDICES.len().try_into().unwrap(),
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            None,
        );

        staging_buffer.map(Some(buffer_size), Some(0));

        staging_buffer.write_to_buffer(
            Vertex::INDICES.as_ptr(),
            staging_buffer.get_mapped_memory().unwrap(),
            Vertex::INDICES.len() as DeviceSize,
            0,
        );
        staging_buffer.unmap();

        let index_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<u16>().try_into().unwrap(),
            Vertex::INDICES.len().try_into().unwrap(),
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

    fn create_vertex_buffer(device: Arc<VRTDevice>) -> VkResult<VRTBuffer> {
        let buffer_size = (mem::size_of::<Vertex>() * Vertex::VERTICES.len()) as DeviceSize;

        let mut staging_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<Vertex>().try_into().unwrap(),
            Vertex::VERTICES.len().try_into().unwrap(),
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            None,
        );

        staging_buffer.map(Some(buffer_size), Some(0));

        staging_buffer.write_to_buffer(
            Vertex::VERTICES.as_ptr(),
            staging_buffer.get_mapped_memory().unwrap(),
            Vertex::VERTICES.len() as DeviceSize,
            0,
        );
        staging_buffer.unmap();

        let vertex_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<Vertex>().try_into().unwrap(),
            Vertex::VERTICES.len().try_into().unwrap(),
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
