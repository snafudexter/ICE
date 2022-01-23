use crate::vrt::device::queue::{CompleteQueueFamilyIndices, QueueFamilyIndices};
use erupt::utils::surface;
use erupt::vk::{
    make_api_version, ApplicationInfoBuilder, InstanceCreateInfoBuilder, PhysicalDevice,
    API_VERSION_1_1,
};
use erupt::{EntryLoader, InstanceLoader};
use std::os::raw::c_char;
use std::sync::Arc;
use winit::window::Window;

#[cfg(debug_assertions)]
use crate::vrt::utils::debug;

use crate::vrt::utils::result::{VkError, VkResult};

pub struct VRTDevice {
    _physical_device: PhysicalDevice,
    #[cfg(debug_assertions)]
    debug_messenger: debug::Messenger,
    instance: Arc<InstanceLoader>,
    _entry: EntryLoader,
}

impl VRTDevice {
    pub fn new(window: &Window) -> VkResult<Self> {
        let entry = EntryLoader::new()?;
        let instance = Self::create_instance(window, &entry)?;

        #[cfg(debug_assertions)]
        let debug_messenger = debug::Messenger::new(&instance)?;

        let (physical_device, _queue_family_indices) = Self::pick_physical_device(&instance)?;

        Ok(Self {
            _physical_device: physical_device,
            #[cfg(debug_assertions)]
            debug_messenger,
            instance,
            _entry: entry,
        })
    }

    fn create_instance(window: &Window, entry: &EntryLoader) -> VkResult<Arc<InstanceLoader>> {
        #[cfg(debug_assertions)]
        use erupt::ExtendableFrom;

        #[cfg(debug_assertions)]
        if !debug::check_validation_layer_support(entry)? {
            return Err(VkError::ValidationLayerUnavailable);
        }

        let app_info = ApplicationInfoBuilder::new()
            .application_version(make_api_version(0, 1, 0, 0))
            .engine_version(make_api_version(0, 1, 0, 0))
            .api_version(API_VERSION_1_1);

        let extensions = Self::required_extensions(window)?;

        let create_info = InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_extension_names(&extensions);

        #[cfg(debug_assertions)]
        let mut debug_create_info = debug::messenger_create_info();
        #[cfg(debug_assertions)]
        let create_info = create_info
            .enabled_layer_names(debug::VALIDATION_LAYERS)
            .extend_from(&mut debug_create_info);

        Ok(Arc::new(unsafe {
            InstanceLoader::new(entry, &create_info)
        }?))
    }

    fn required_extensions(window: &Window) -> VkResult<Vec<*const c_char>> {
        let extensions = surface::enumerate_required_extensions(window).result()?;

        #[cfg(debug_assertions)]
        let mut extensions = extensions;
        #[cfg(debug_assertions)]
        extensions.extend(debug::EXTENSIONS);

        Ok(extensions)
    }

    fn pick_physical_device(
        instance: &InstanceLoader,
    ) -> VkResult<(PhysicalDevice, CompleteQueueFamilyIndices)> {
        let devices = unsafe { instance.enumerate_physical_devices(None) }.result()?;

        if devices.is_empty() {
            return Err(VkError::NoVulkanGpu);
        }

        devices
            .into_iter()
            .find_map(|device| Some((device, Self::is_device_suitable(instance, device)?)))
            .ok_or(VkError::NoSuitableGpu)
    }

    fn is_device_suitable(
        instance: &InstanceLoader,
        device: PhysicalDevice,
    ) -> Option<CompleteQueueFamilyIndices> {
        let indices = QueueFamilyIndices::new(instance, device);

        indices.complete()
    }
}

impl Drop for VRTDevice {
    fn drop(&mut self) {
        unsafe {
            #[cfg(debug_assertions)]
            self.debug_messenger.destroy(&self.instance);
            self.instance.destroy_instance(None)
        }
    }
}
