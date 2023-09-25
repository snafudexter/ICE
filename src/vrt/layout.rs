use std::sync::Arc;

use erupt::vk1_0::{
    DescriptorSetLayout, DescriptorSetLayoutBindingBuilder, DescriptorSetLayoutCreateInfoBuilder,
    DescriptorType, ShaderStageFlags,
};

use crate::vrt::device::VRTDevice;

pub struct VRTDescriptorSetLayout<'a> {
    device: Arc<VRTDevice>,
    descriptor_set_layout: DescriptorSetLayout,
    pub bindings: Vec<DescriptorSetLayoutBindingBuilder<'a>>,
}

impl VRTDescriptorSetLayout<'_> {
    // pub fn new(device: Arc<VRTDevice>, bindings: Vec<DescriptorSetLayoutBindingBuilder>) -> Self {
    //     let layout_info = DescriptorSetLayoutCreateInfoBuilder::new().bindings(&bindings);
    //     let descriptor_set_layout = unsafe {
    //         device
    //             .clone()
    //             .get_device_ptr()
    //             .create_descriptor_set_layout(&layout_info, None)
    //             .unwrap()
    //     };
    //     Self {
    //         device,
    //         descriptor_set_layout,
    //         bindings,
    //     }
    // }

    pub fn get_descriptor_set_layout(&self) -> &DescriptorSetLayout {
        &self.descriptor_set_layout
    }
}

pub struct VRTDescriptorSetLayoutBuilder<'a> {
    device: Arc<VRTDevice>,
    bindings: Vec<DescriptorSetLayoutBindingBuilder<'a>>,
}

impl<'a> VRTDescriptorSetLayoutBuilder<'a> {
    pub fn new(device: Arc<VRTDevice>) -> Self {
        Self {
            device,
            bindings: vec![],
        }
    }

    pub fn add_binding(
        mut self,
        binding: u32,
        descriptor_type: DescriptorType,
        stage_flags: ShaderStageFlags,
        count: Option<u32>,
    ) -> VRTDescriptorSetLayoutBuilder<'a> {
        let layout_binding = DescriptorSetLayoutBindingBuilder::new()
            .binding(binding)
            .descriptor_type(descriptor_type)
            .descriptor_count(count.unwrap_or(1))
            .stage_flags(stage_flags);
        self.bindings.push(layout_binding);
        self
    }

    pub fn build(self) -> VRTDescriptorSetLayout<'a> {
        //VRTDescriptorSetLayout::new(self.device.clone(), self.bindings)
        let layout_info = DescriptorSetLayoutCreateInfoBuilder::new().bindings(&self.bindings);
        let descriptor_set_layout = unsafe {
            self.device
                .get_device_ptr()
                .create_descriptor_set_layout(&layout_info, None)
                .unwrap()
        };

        VRTDescriptorSetLayout {
            device: self.device.clone(),
            descriptor_set_layout,
            bindings: self.bindings,
        }
    }
}

impl Drop for VRTDescriptorSetLayout<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}
