use std::{convert::TryInto, ffi::c_void, mem, ptr::copy_nonoverlapping, sync::Arc};

use color_eyre::owo_colors::OwoColorize;
use erupt::{
    vk1_0::{
        Buffer, BufferCopyBuilder, BufferCreateInfoBuilder, BufferUsageFlags, CommandBuffer,
        CommandBufferAllocateInfoBuilder, CommandBufferBeginInfoBuilder, CommandBufferLevel,
        CommandBufferUsageFlags, CommandPool, DeviceMemory, DeviceSize, Fence,
        MemoryAllocateInfoBuilder, MemoryMapFlags, MemoryPropertyFlags, PhysicalDevice, Queue,
        SharingMode, SubmitInfoBuilder,
    },
    DeviceLoader, InstanceLoader,
};

use crate::vrt::{
    device::device::VRTDevice,
    utils::result::{VkError, VkResult},
};

use super::vertex::Vertex;

pub struct Model {
    device: Arc<VRTDevice>,
    vertex_buffer: Buffer,
    vertex_buffer_memory: DeviceMemory,
}

impl Model {
    pub fn new(instance: &InstanceLoader, device: Arc<VRTDevice>) -> Self {
        let (vertex_buffer, vertex_buffer_memory) =
            Self::create_vertex_buffer(instance, device.clone()).unwrap();
        Self {
            device,
            vertex_buffer,
            vertex_buffer_memory,
        }
    }

    pub fn bind(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer) {
        unsafe {
            device.get_device_ptr().cmd_bind_vertex_buffers(
                command_buffer,
                0,
                std::slice::from_ref(&self.vertex_buffer),
                &[0],
            );
        }
    }

    pub fn draw(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer) {
        unsafe {
            device.get_device_ptr().cmd_draw(
                command_buffer,
                Vertex::VERTICES.len().try_into().unwrap(),
                1,
                0,
                0,
            );
        }
    }

    fn create_vertex_buffer(
        instance: &InstanceLoader,
        device: Arc<VRTDevice>,
    ) -> VkResult<(Buffer, DeviceMemory)> {
        let buffer_size = (mem::size_of::<Vertex>() * Vertex::VERTICES.len()) as DeviceSize;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            instance,
            device.get_physical_device(),
            &device.get_device_ptr(),
            buffer_size,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        )?;
        unsafe {
            let memory = device
                .get_device_ptr()
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    MemoryMapFlags::empty(),
                )
                .result()?;
            copy_nonoverlapping(
                Vertex::VERTICES.as_ptr(),
                memory.cast(),
                Vertex::VERTICES.len(),
            );
            device.get_device_ptr().unmap_memory(staging_buffer_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
            instance,
            device.get_physical_device(),
            &device.get_device_ptr(),
            buffer_size,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        Self::copy_buffer(
            &device.get_device_ptr(),
            device.get_queues().graphics,
            device.get_command_pool(),
            staging_buffer,
            vertex_buffer,
            buffer_size,
        )?;

        unsafe { device.get_device_ptr().destroy_buffer(staging_buffer, None) };
        unsafe {
            device
                .get_device_ptr()
                .free_memory(staging_buffer_memory, None)
        };

        Ok((vertex_buffer, vertex_buffer_memory))
    }

    fn find_memory_type(
        instance: &InstanceLoader,
        physical_device: PhysicalDevice,
        type_filter: u32,
        properties: MemoryPropertyFlags,
    ) -> VkResult<u32> {
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        (0..memory_properties.memory_type_count)
            .into_iter()
            .find(|&i| {
                (type_filter & (1 << i)) != 0
                    && memory_properties.memory_types[i as usize]
                        .property_flags
                        .contains(properties)
            })
            .ok_or(VkError::NoSuitableMemoryType)
    }

    fn create_buffer(
        instance: &InstanceLoader,
        physical_device: PhysicalDevice,
        device: &DeviceLoader,
        size: DeviceSize,
        usage: BufferUsageFlags,
        properties: MemoryPropertyFlags,
    ) -> VkResult<(Buffer, DeviceMemory)> {
        let buffer_info = BufferCreateInfoBuilder::new()
            .size(size)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }.result()?;
        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let alloc_info = MemoryAllocateInfoBuilder::new()
            .allocation_size(memory_requirements.size)
            .memory_type_index(Self::find_memory_type(
                instance,
                physical_device,
                memory_requirements.memory_type_bits,
                properties,
            )?);

        let buffer_memory = unsafe { device.allocate_memory(&alloc_info, None) }.result()?;
        unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0) }.result()?;

        Ok((buffer, buffer_memory))
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

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_buffer(self.vertex_buffer, None);
            self.device
                .get_device_ptr()
                .free_memory(self.vertex_buffer_memory, None);
        }
    }
}
