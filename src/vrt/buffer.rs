use std::{ffi::c_void, os::raw::c_int, ptr::copy_nonoverlapping, sync::Arc};

use erupt::{
    utils::VulkanResult,
    vk1_0::{
        Buffer, BufferUsageFlags, DescriptorBufferInfoBuilder, DeviceMemory, DeviceSize,
        MappedMemoryRange, MappedMemoryRangeBuilder, MemoryMapFlags, MemoryPropertyFlags,
        WHOLE_SIZE,
    },
};

use crate::vrt::vertex::Vertex;

use super::device::VRTDevice;

pub struct VRTBuffer {
    device: Arc<VRTDevice>,
    buffer: Buffer,
    buffer_size: DeviceSize,
    memory: DeviceMemory,
    instance_count: u32,
    instance_size: DeviceSize,
    alignment_size: DeviceSize,
    usage_flags: BufferUsageFlags,
    memory_property_flags: MemoryPropertyFlags,
    mapped_memory: Option<*mut c_void>,
}

impl VRTBuffer {
    pub fn new(
        device: Arc<VRTDevice>,
        instance_size: DeviceSize,
        instance_count: u32,
        usage_flags: BufferUsageFlags,
        memory_property_flags: MemoryPropertyFlags,
        min_offset_alignment: Option<DeviceSize>,
    ) -> Self {
        let alignment_size = Self::get_alignment(instance_size, min_offset_alignment.unwrap_or(1));
        let buffer_size = alignment_size * instance_count as u64;
        let (buffer, memory) = device
            .clone()
            .create_buffer(buffer_size, usage_flags, memory_property_flags)
            .unwrap();

        Self {
            buffer,
            memory,
            device,
            instance_count,
            instance_size,
            alignment_size,
            usage_flags,
            memory_property_flags,
            buffer_size,
            mapped_memory: None,
        }
    }

    pub fn map(&mut self, size: Option<DeviceSize>, offset: Option<DeviceSize>) {
        unsafe {
            let mapped_memory = Some(
                self.device
                    .get_device_ptr()
                    .map_memory(
                        self.memory,
                        offset.unwrap_or(0),
                        size.unwrap_or(WHOLE_SIZE),
                        MemoryMapFlags::empty(),
                    )
                    .result()
                    .unwrap() as *mut c_void,
            );
            self.mapped_memory = mapped_memory;
        }
    }

    pub fn get_mapped_memory(&self) -> Option<*mut c_void> {
        self.mapped_memory
    }

    pub fn write_to_buffer<T>(
        &self,
        data: *const T,
        mapped: *mut c_void,
        size: DeviceSize,
        offset: DeviceSize,
    ) {
        unsafe {
            if (size == WHOLE_SIZE) {
                copy_nonoverlapping(data, mapped.cast(), self.buffer_size as usize);
            } else {
                let m = mapped as u64;
                let memory_offset = (mapped as u64 + offset) as *mut c_void;
                copy_nonoverlapping(data, memory_offset.cast(), size as usize);
            }
        }
    }

    pub fn flush(&self, size: DeviceSize, offset: DeviceSize) {
        unsafe {
            let mapped_memory_range = MappedMemoryRangeBuilder::new()
                .memory(self.memory)
                .size(size)
                .offset(offset);

            self.device
                .get_device_ptr()
                .flush_mapped_memory_ranges(&[mapped_memory_range]);
        }
    }

    pub fn unmap(&self) {
        unsafe {
            self.device.get_device_ptr().unmap_memory(self.memory);
        }
    }

    fn get_alignment(instance_size: DeviceSize, min_offset_alignment: DeviceSize) -> DeviceSize {
        if min_offset_alignment > 0 {
            return (instance_size + min_offset_alignment - 1) & !(min_offset_alignment - 1);
        }
        return instance_size;
    }

    pub fn get_buffer(&self) -> Buffer {
        self.buffer
    }

    pub fn get_buffer_info(&self, range: DeviceSize) -> DescriptorBufferInfoBuilder {
        DescriptorBufferInfoBuilder::new()
            .buffer(self.buffer)
            .offset(0)
            .range(range)
    }
}

impl Drop for VRTBuffer {
    fn drop(&mut self) {
        //self.unmap();
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_buffer(self.buffer, None);
            self.device.get_device_ptr().free_memory(self.memory, None);
        }
    }
}
