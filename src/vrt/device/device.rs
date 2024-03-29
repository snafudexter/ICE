use crate::vrt::device::queue::{CompleteQueueFamilyIndices, QueueFamilyIndices, Queues};
use crate::vrt::utils::result::{VkError, VkResult};
use crate::vrt::window::VRTWindow;
use erupt::utils::surface;
use erupt::vk::CommandPool;
use erupt::vk::CommandPoolCreateInfoBuilder;
use erupt::vk::{
    make_api_version, ApplicationInfoBuilder, DeviceCreateInfoBuilder,
    DeviceQueueCreateInfoBuilder, Extent2D, Framebuffer, FramebufferCreateInfoBuilder, ImageView,
    InstanceCreateInfoBuilder, PhysicalDevice, PhysicalDeviceFeaturesBuilder, PresentModeKHR,
    RenderPass, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR, API_VERSION_1_1,
    KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME, KHR_PORTABILITY_SUBSET_EXTENSION_NAME,
    KHR_SWAPCHAIN_EXTENSION_NAME,
};
use erupt::vk1_0::{
    Buffer, BufferCreateInfoBuilder, BufferUsageFlags, CommandPoolCreateFlags, DeviceMemory,
    DeviceSize, MemoryAllocateInfoBuilder, MemoryPropertyFlags, PhysicalDeviceProperties,
    SharingMode,
};
use erupt::SmallVec;
use erupt::{DeviceLoader, EntryLoader, InstanceLoader};
use std::collections::BTreeSet;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use std::sync::Arc;
use winit::window::Window;

#[cfg(debug_assertions)]
use crate::vrt::utils::debug;

const DEVICE_EXTENSIONS: *const c_char = KHR_SWAPCHAIN_EXTENSION_NAME;

pub const INSTANCE_EXTENSIONS: &[*const c_char] =
    &[KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME];

#[derive(Debug, Clone)]
pub struct SwapchainSupportDetails {
    capabilities: SurfaceCapabilitiesKHR,
    formats: SmallVec<SurfaceFormatKHR>,
    present_modes: SmallVec<PresentModeKHR>,
}

pub struct VRTDevice {
    _queues: Queues,
    queue_family_indices: CompleteQueueFamilyIndices,
    device: Arc<DeviceLoader>,
    surface: SurfaceKHR,
    _physical_device: PhysicalDevice,
    #[cfg(debug_assertions)]
    debug_messenger: debug::Messenger,
    instance: Arc<InstanceLoader>,
    _entry: EntryLoader,
    swapchain_support: SwapchainSupportDetails,
    command_pool: CommandPool,
}

impl VRTDevice {
    pub fn new(window: &VRTWindow) -> VkResult<Self> {
        let entry = EntryLoader::new()?;
        let instance = Self::create_instance(window.get_window_ptr(), &entry)?;

        #[cfg(debug_assertions)]
        let debug_messenger = debug::Messenger::new(&instance)?;

        let surface = Self::create_surface(window.get_window_ptr(), &instance)?;

        let (physical_device, queue_family_indices, swapchain_support) =
            Self::pick_physical_device(&instance, surface)?;
        println!("physical_device {:?}", &physical_device);

        let (device, queues) =
            Self::create_logical_device(&instance, physical_device, queue_family_indices)?;

        let command_pool = Self::create_command_pool(&queue_family_indices, &device)?;

        let properties: PhysicalDeviceProperties;
        unsafe {
            properties = instance.get_physical_device_properties(physical_device);
        }

        Ok(Self {
            _queues: queues,
            device,
            _physical_device: physical_device,
            surface,
            #[cfg(debug_assertions)]
            debug_messenger,
            instance,
            _entry: entry,
            queue_family_indices,
            swapchain_support,
            command_pool,
        })
    }

    pub fn create_buffer(
        &self,
        size: DeviceSize,
        usage: BufferUsageFlags,
        properties: MemoryPropertyFlags,
    ) -> VkResult<(Buffer, DeviceMemory)> {
        let buffer_info = BufferCreateInfoBuilder::new()
            .size(size)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let buffer = unsafe { self.device.create_buffer(&buffer_info, None) }.result()?;
        let memory_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        let alloc_info = MemoryAllocateInfoBuilder::new()
            .allocation_size(memory_requirements.size)
            .memory_type_index(Self::find_memory_type(
                &self.instance.clone(),
                self._physical_device,
                memory_requirements.memory_type_bits,
                properties,
            )?);

        let buffer_memory = unsafe { self.device.allocate_memory(&alloc_info, None) }.result()?;
        unsafe { self.device.bind_buffer_memory(buffer, buffer_memory, 0) }.result()?;

        Ok((buffer, buffer_memory))
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

    pub fn get_swapchain_support(&self) -> VkResult<SwapchainSupportDetails> {
        SwapchainSupportDetails::new(&self.instance, self.surface, self._physical_device)
    }

    pub fn get_physical_device(&self) -> PhysicalDevice {
        self._physical_device
    }

    pub fn get_device_ptr(&self) -> Arc<DeviceLoader> {
        self.device.clone()
    }

    pub fn get_command_pool(&self) -> CommandPool {
        self.command_pool
    }

    pub fn get_queue_family_indices(&self) -> CompleteQueueFamilyIndices {
        self.queue_family_indices
    }

    pub fn get_queues(&self) -> &Queues {
        &self._queues
    }

    pub fn get_surface(&self) -> SurfaceKHR {
        self.surface
    }

    pub fn get_instance(&self) -> &InstanceLoader {
        &self.instance
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
        let mut extensions = surface::enumerate_required_extensions(window).result()?;

        #[cfg(debug_assertions)]
        let mut extensions = extensions;
        #[cfg(debug_assertions)]
        extensions.extend(debug::EXTENSIONS);

        if std::env::consts::OS.contains("macos") {
            extensions.extend(INSTANCE_EXTENSIONS);
        }

        Ok(extensions)
    }

    fn pick_physical_device(
        instance: &InstanceLoader,
        surface: SurfaceKHR,
    ) -> VkResult<(
        PhysicalDevice,
        CompleteQueueFamilyIndices,
        SwapchainSupportDetails,
    )> {
        let devices = unsafe { instance.enumerate_physical_devices(None) }.result()?;

        if devices.is_empty() {
            return Err(VkError::NoVulkanGpu);
        }

        for device in devices {
            if let Some((indices, swapchain_support)) =
                Self::is_device_suitable(instance, surface, device)?
            {
                return Ok((device, indices, swapchain_support));
            }
        }

        Err(VkError::NoSuitableGpu)
    }

    fn is_device_suitable(
        instance: &InstanceLoader,
        surface: SurfaceKHR,
        device: PhysicalDevice,
    ) -> VkResult<Option<(CompleteQueueFamilyIndices, SwapchainSupportDetails)>> {
        let indices = QueueFamilyIndices::new(instance, surface, device)?;

        if let Some(indices) = indices.complete() {
            if Self::check_device_extension_support(instance, device)? {
                let swapchain_support = SwapchainSupportDetails::new(instance, surface, device)?;

                Ok(swapchain_support
                    .is_adequate()
                    .then(|| (indices, swapchain_support)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn check_device_extension_support(
        instance: &InstanceLoader,
        device: PhysicalDevice,
    ) -> VkResult<bool> {
        let available_extensions =
            unsafe { instance.enumerate_device_extension_properties(device, None, None) }
                .result()?;

        let device_extensions: &[*const c_char];
        if std::env::consts::OS.contains("macos") {
            device_extensions = &[DEVICE_EXTENSIONS, KHR_PORTABILITY_SUBSET_EXTENSION_NAME];
        } else {
            device_extensions = &[DEVICE_EXTENSIONS];
        }

        let required_extensions = device_extensions
            .iter()
            .map(|ptr| unsafe { CStr::from_ptr(*ptr) });

        Ok(check_support(
            available_extensions
                .iter()
                .map(|extension| unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) }),
            required_extensions,
        ))
    }

    fn create_logical_device(
        instance: &InstanceLoader,
        physical_device: PhysicalDevice,
        indices: CompleteQueueFamilyIndices,
    ) -> VkResult<(Arc<DeviceLoader>, Queues)> {
        let unique_queue_families =
            BTreeSet::from([indices.graphics_family(), indices.present_family()]);

        let queue_priority = 1.0;
        let queue_create_infos = unique_queue_families
            .into_iter()
            .map(|queue_family| {
                DeviceQueueCreateInfoBuilder::new()
                    .queue_family_index(queue_family)
                    .queue_priorities(slice::from_ref(&queue_priority))
            })
            .collect::<Vec<_>>();

        let device_features = PhysicalDeviceFeaturesBuilder::new();

        let device_extensions: &[*const c_char];
        if std::env::consts::OS.contains("macos") {
            device_extensions = &[DEVICE_EXTENSIONS, KHR_PORTABILITY_SUBSET_EXTENSION_NAME];
        } else {
            device_extensions = &[DEVICE_EXTENSIONS];
        }

        let create_info = DeviceCreateInfoBuilder::new()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(device_extensions);

        #[cfg(debug_assertions)]
        let create_info = create_info.enabled_layer_names(debug::VALIDATION_LAYERS);

        let device =
            Arc::new(unsafe { DeviceLoader::new(instance, physical_device, &create_info) }?);

        let queues = Queues {
            graphics: unsafe { device.get_device_queue(indices.graphics_family(), 0) },
            present: unsafe { device.get_device_queue(indices.present_family(), 0) },
        };

        Ok((device, queues))
    }

    fn create_surface(window: &Window, instance: &InstanceLoader) -> VkResult<SurfaceKHR> {
        Ok(unsafe { surface::create_surface(instance, window, None) }.result()?)
    }

    fn create_command_pool(
        indices: &CompleteQueueFamilyIndices,
        device: &DeviceLoader,
    ) -> VkResult<CommandPool> {
        let pool_info = CommandPoolCreateInfoBuilder::new()
            .queue_family_index(indices.graphics_family())
            .flags(
                CommandPoolCreateFlags::TRANSIENT | CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            );

        Ok(unsafe { device.create_command_pool(&pool_info, None) }.result()?)
    }
}

impl Drop for VRTDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);

            self.instance.destroy_surface_khr(self.surface, None);

            #[cfg(debug_assertions)]
            self.debug_messenger.destroy(&self.instance);

            self.instance.destroy_instance(None)
        }
    }
}

fn check_support<'a>(
    available: impl IntoIterator<Item = &'a CStr>,
    required: impl IntoIterator<Item = &'a CStr>,
) -> bool {
    let mut required = required.into_iter().collect::<BTreeSet<_>>();

    for available in available {
        required.remove(available);
    }

    required.is_empty()
}

impl SwapchainSupportDetails {
    pub fn new(
        instance: &InstanceLoader,
        surface: SurfaceKHR,
        device: PhysicalDevice,
    ) -> VkResult<Self> {
        let capabilities =
            unsafe { instance.get_physical_device_surface_capabilities_khr(device, surface) }
                .result()?;

        let formats =
            unsafe { instance.get_physical_device_surface_formats_khr(device, surface, None) }
                .result()?;

        let present_modes = unsafe {
            instance.get_physical_device_surface_present_modes_khr(device, surface, None)
        }
        .result()?;

        Ok(Self {
            capabilities,
            formats,
            present_modes,
        })
    }

    pub fn is_adequate(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }

    pub fn capabilities(&self) -> &SurfaceCapabilitiesKHR {
        &self.capabilities
    }

    pub fn formats(&self) -> &[SurfaceFormatKHR] {
        &self.formats
    }

    pub fn present_modes(&self) -> &[PresentModeKHR] {
        &self.present_modes
    }
}
