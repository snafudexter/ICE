use erupt::vk::{PhysicalDevice, QueueFlags};
use erupt::InstanceLoader;
use std::convert::TryInto;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct QueueFamilyIndices {
    graphics_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new(instance: &InstanceLoader, device: PhysicalDevice) -> Self {
        let mut indices = Self {
            graphics_family: None,
        };

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device, None) };

        for (i, queue_family) in queue_families.into_iter().enumerate() {
            log::info!(
                "Queue family indexed {} flags {:?}",
                &i,
                queue_family.queue_flags
            );
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i.try_into().unwrap());
            }

            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    pub fn complete(self) -> Option<CompleteQueueFamilyIndices> {
        Some(CompleteQueueFamilyIndices {
            graphics_family: self.graphics_family?,
        })
    }

    fn is_complete(self) -> bool {
        self.graphics_family.is_some()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CompleteQueueFamilyIndices {
    graphics_family: u32,
}
