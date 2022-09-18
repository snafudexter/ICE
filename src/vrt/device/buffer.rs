use std::{ffi::c_void, os::raw::c_int, ptr::copy_nonoverlapping, sync::Arc};

use erupt::vk1_0::{
    Buffer, BufferUsageFlags, DeviceMemory, DeviceSize, MemoryMapFlags, MemoryPropertyFlags,
    WHOLE_SIZE,
};

use crate::vrt::graphics::vertex::Vertex;

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
        }
    }

    pub fn map(&self, size: DeviceSize, offset: DeviceSize) -> *mut c_void {
        unsafe {
            self.device
                .get_device_ptr()
                .map_memory(self.memory, offset, size, MemoryMapFlags::empty())
                .result()
                .unwrap()
        }
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
}

impl Drop for VRTBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_buffer(self.buffer, None);
            self.device.get_device_ptr().free_memory(self.memory, None);
        }
    }
}
