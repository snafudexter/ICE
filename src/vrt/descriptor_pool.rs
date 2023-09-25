use std::{rc::Rc, sync::Arc};

use erupt::{
    vk1_0::{
        DescriptorBufferInfoBuilder, DescriptorPool, DescriptorPoolCreateFlags,
        DescriptorPoolCreateInfoBuilder, DescriptorPoolSizeBuilder, DescriptorSet,
        DescriptorSetAllocateInfoBuilder, DescriptorSetLayout, DescriptorType,
        WriteDescriptorSetBuilder,
    },
    SmallVec,
};

use crate::vrt::device::VRTDevice;

use super::layout::VRTDescriptorSetLayout;

pub struct VRTDescriptorPool {
    device: Arc<VRTDevice>,
    descriptor_pool: DescriptorPool,
}

impl VRTDescriptorPool {
    pub fn new(
        device: Arc<VRTDevice>,
        max_sets: u32,
        pool_flags: DescriptorPoolCreateFlags,
        pool_sizes: &Vec<DescriptorPoolSizeBuilder>,
    ) -> Self {
        let descriptor_pool_builder = DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(pool_sizes)
            .flags(pool_flags)
            .max_sets(max_sets);

        let descriptor_pool = unsafe {
            device
                .get_device_ptr()
                .create_descriptor_pool(&descriptor_pool_builder, None)
                .unwrap()
        };

        Self {
            device,
            descriptor_pool,
        }
    }

    pub fn allocate_descriptor(
        &self,
        layout: &DescriptorSetLayout,
    ) -> Option<SmallVec<DescriptorSet>> {
        let alloc_info = DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(std::slice::from_ref(layout));

        Some(unsafe {
            self.device
                .get_device_ptr()
                .allocate_descriptor_sets(&alloc_info)
                .unwrap()
        })
    }
}

impl Drop for VRTDescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

pub struct VRTDescriptorPoolBuilder<'a> {
    device: Arc<VRTDevice>,
    pool_sizes: Vec<DescriptorPoolSizeBuilder<'a>>,
    pool_flags: DescriptorPoolCreateFlags,
    max_sets: u32,
}

impl VRTDescriptorPoolBuilder<'_> {
    pub fn new(device: Arc<VRTDevice>) -> Self {
        Self {
            device,
            pool_sizes: vec![],
            pool_flags: DescriptorPoolCreateFlags::empty(),
            max_sets: 1000,
        }
    }

    pub fn add_pool_size(mut self, descriptor_type: DescriptorType, count: u32) -> Self {
        self.pool_sizes.push(
            DescriptorPoolSizeBuilder::new()
                ._type(descriptor_type)
                .descriptor_count(count),
        );
        self
    }

    pub fn set_pool_flags(mut self, pool_flags: DescriptorPoolCreateFlags) -> Self {
        self.pool_flags = pool_flags;
        self
    }

    pub fn set_max_sets(mut self, max_sets: u32) -> Self {
        self.max_sets = max_sets;
        self
    }

    pub fn build(&self) -> VRTDescriptorPool {
        VRTDescriptorPool::new(
            self.device.clone(),
            self.max_sets,
            self.pool_flags,
            &self.pool_sizes,
        )
    }
}

pub struct VRTDescriptorWriter<'a> {
    layout: Rc<VRTDescriptorSetLayout<'a>>,
    pool: Rc<VRTDescriptorPool>,
    writes: Vec<WriteDescriptorSetBuilder<'a>>,
}

impl<'a> VRTDescriptorWriter<'a> {
    pub fn new(layout: Rc<VRTDescriptorSetLayout<'a>>, pool: Rc<VRTDescriptorPool>) -> Self {
        Self {
            layout: layout.clone(),
            pool: pool.clone(),
            writes: vec![],
        }
    }

    pub fn write_buffer(
        mut self,
        binding: u32,
        buffer_info: &'a DescriptorBufferInfoBuilder,
    ) -> Self {
        let binding_description = self.layout.bindings[binding as usize];
        let descriptor_write = WriteDescriptorSetBuilder::new()
            .descriptor_type(binding_description.descriptor_type)
            .dst_binding(binding)
            .buffer_info(std::slice::from_ref(buffer_info));
        self.writes.push(descriptor_write);
        self
    }

    pub fn build(mut self) -> Option<SmallVec<DescriptorSet>> {
        let success = self
            .pool
            .allocate_descriptor(self.layout.get_descriptor_set_layout());

        match success {
            Some(descriptor_sets) => {
                self.writes = self
                    .writes
                    .iter()
                    .zip(&descriptor_sets)
                    .map(|(write, descriptor_set)| write.dst_set(*descriptor_set))
                    .collect::<Vec<WriteDescriptorSetBuilder>>();

                // for (write, descriptor_set) in self.writes.iter() {
                //     write.dst_set(*descriptor_sets);
                // }
                self.overwrite();

                Some(descriptor_sets)
            }
            None => todo!(),
        }
    }

    fn overwrite(self) {
        unsafe {
            self.pool
                .device
                .get_device_ptr()
                .update_descriptor_sets(&self.writes, &[])
        };
    }
}
