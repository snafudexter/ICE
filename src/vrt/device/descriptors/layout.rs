use std::sync::Arc;

use erupt::vk1_0::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutBindingBuilder,
    DescriptorSetLayoutCreateInfoBuilder, DescriptorType, ShaderStageFlags,
};

use crate::vrt::device::device::VRTDevice;

pub struct VRTDescriptorSetLayout {
    device: Arc<VRTDevice>,
    descriptor_set_layout: DescriptorSetLayout,
}

impl VRTDescriptorSetLayout {
    pub fn new(device: Arc<VRTDevice>, bindings: &Vec<DescriptorSetLayoutBindingBuilder>) -> Self {
        let layout_info = DescriptorSetLayoutCreateInfoBuilder::new().bindings(&bindings);
        let descriptor_set_layout = unsafe {
            device
                .clone()
                .get_device_ptr()
                .create_descriptor_set_layout(&layout_info, None)
                .unwrap()
        };
        Self {
            device,
            descriptor_set_layout,
        }
    }
}

pub struct VRTDescriptorSetLayoutBuilder<'a> {
    device: Arc<VRTDevice>,
    bindings: Vec<DescriptorSetLayoutBindingBuilder<'a>>,
}

impl VRTDescriptorSetLayoutBuilder<'_> {
    pub fn new(device: Arc<VRTDevice>) -> Self {
        Self {
            device,
            bindings: vec![],
        }
    }

    pub fn add_binding(
        &mut self,
        binding: u32,
        descriptor_type: DescriptorType,
        stage_flags: ShaderStageFlags,
        count: Option<u32>,
    ) -> &VRTDescriptorSetLayoutBuilder {
        let layout_binding = DescriptorSetLayoutBindingBuilder::new()
            .binding(binding)
            .descriptor_type(descriptor_type)
            .descriptor_count(count.unwrap_or(1))
            .stage_flags(stage_flags);
        self.bindings[binding as usize] = layout_binding;
        self
    }

    pub fn build(&self) -> VRTDescriptorSetLayout {
        VRTDescriptorSetLayout::new(self.device.clone(), &self.bindings)
    }
}
