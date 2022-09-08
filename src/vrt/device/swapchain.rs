use crate::vrt::device::device::VRTDevice;
use crate::vrt::device::sync::SyncObjects;
use crate::vrt::utils::result::{VkError, VkResult};
use erupt::vk::CommandBuffer;
use erupt::vk::Framebuffer;
use erupt::vk::PresentInfoKHRBuilder;
use erupt::vk::RenderPass;
use erupt::vk::SwapchainCreateInfoKHRBuilder;
use erupt::vk::{
    AttachmentDescriptionBuilder, AttachmentLoadOp, AttachmentReferenceBuilder, AttachmentStoreOp,
    ColorSpaceKHR, ComponentMappingBuilder, ComponentSwizzle, CompositeAlphaFlagBitsKHR, Extent2D,
    Extent2DBuilder, FenceCreateFlags, FenceCreateInfoBuilder, Format,
    FramebufferCreateInfoBuilder, Image, ImageAspectFlags, ImageLayout,
    ImageSubresourceRangeBuilder, ImageUsageFlags, ImageView, ImageViewCreateInfoBuilder,
    ImageViewType, PhysicalDevice, PipelineBindPoint, PipelineStageFlags, PresentModeKHR,
    RenderPassCreateInfoBuilder, SampleCountFlagBits, SemaphoreCreateInfoBuilder, SharingMode,
    SubmitInfoBuilder, SubpassDescriptionBuilder, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
    SurfaceKHR, SwapchainKHR,
};
use erupt::DeviceLoader;
use erupt::{InstanceLoader, SmallVec};
use std::sync::Arc;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[derive(Clone)]
pub struct Swapchain {
    image_views: Vec<ImageView>,
    extent: Extent2D,
    image_format: Format,
    images: SmallVec<Image>,
    swapchain: SwapchainKHR,
    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,
    sync: SyncObjects<MAX_FRAMES_IN_FLIGHT>,
    device: Arc<DeviceLoader>,
}

#[derive(Debug, Clone)]
pub struct SwapchainSupportDetails {
    capabilities: SurfaceCapabilitiesKHR,
    formats: SmallVec<SurfaceFormatKHR>,
    present_modes: SmallVec<PresentModeKHR>,
}

impl Swapchain {
    pub fn new(device: &VRTDevice, extent: Extent2D) -> VkResult<Self> {
        let (extent, image_format, images, swapchain) = Self::create_swapchain(extent, device)?;

        let image_views = Self::create_image_views(device, &images, image_format)?;

        let render_pass = Self::create_render_pass(&device.get_device_ptr(), image_format)?;

        let framebuffers = Self::create_framebuffers(device, &image_views, &extent, render_pass)?;

        let sync = Self::create_sync_objects(device, &images)?;

        Ok(Self {
            swapchain,
            images,
            extent,
            image_format,
            sync,
            framebuffers,
            render_pass,
            image_views,
            device: device.get_device_ptr(),
        })
    }

    pub fn acquire_next_image(&self, index: usize) -> Result<u32, erupt::vk1_0::Result> {
        unsafe {
            self.device.wait_for_fences(
                std::slice::from_ref(&self.sync.in_flight_fences[self.sync.current_frame]),
                true,
                u64::MAX,
            )
        }
        .result()?;

        unsafe {
            self.device.acquire_next_image_khr(
                self.swapchain,
                u64::MAX,
                self.sync.image_available_semaphores[self.sync.current_frame],
                erupt::vk::Fence::null(),
            )
        }
        .result()
    }

    fn submit_command_buffer(
        &self,
        device: &VRTDevice,
        command_buffers: &SmallVec<CommandBuffer>,
        image_index: &u32,
    ) -> VkResult<erupt::utils::VulkanResult<()>> {
        let submit_info = SubmitInfoBuilder::new()
            .wait_semaphores(std::slice::from_ref(
                &self.sync.image_available_semaphores[self.sync.current_frame],
            ))
            .wait_dst_stage_mask(std::slice::from_ref(
                &PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ))
            .command_buffers(std::slice::from_ref(
                &command_buffers[*image_index as usize],
            ))
            .signal_semaphores(std::slice::from_ref(
                &self.sync.render_finished_semaphores[self.sync.current_frame],
            ));

        unsafe {
            self.device.reset_fences(std::slice::from_ref(
                &self.sync.in_flight_fences[self.sync.current_frame],
            ))
        }
        .result()?;

        unsafe {
            self.device.queue_submit(
                device.get_queues().graphics,
                std::slice::from_ref(&submit_info),
                self.sync.in_flight_fences[self.sync.current_frame],
            )
        }
        .result()?;

        let present_info = PresentInfoKHRBuilder::new()
            .wait_semaphores(std::slice::from_ref(
                &self.sync.render_finished_semaphores[self.sync.current_frame],
            ))
            .swapchains(std::slice::from_ref(&self.swapchain))
            .image_indices(std::slice::from_ref(&image_index));

        Ok(unsafe {
            self.device
                .queue_present_khr(device.get_queues().present, &present_info)
        })
    }

    fn create_swapchain(
        extent: Extent2D,
        device: &VRTDevice,
    ) -> VkResult<(Extent2D, Format, SmallVec<Image>, SwapchainKHR)> {
        let swapchain_support = device.get_swapchain_support()?;

        let surface_format = Self::choose_swap_surface_format(swapchain_support.formats());
        let present_mode = Self::choose_swap_present_mode(swapchain_support.present_modes());
        let extent = Self::choose_swap_extent(extent, swapchain_support.capabilities());

        let image_count = if swapchain_support.capabilities().max_image_count > 0 {
            swapchain_support
                .capabilities()
                .max_image_count
                .min(swapchain_support.capabilities().min_image_count + 1)
        } else {
            swapchain_support.capabilities().min_image_count + 1
        };

        let indices = [
            device.get_queue_family_indices().graphics_family(),
            device.get_queue_family_indices().present_family(),
        ];
        let (sharing_mode, indices) = if indices[0] == indices[1] {
            (SharingMode::EXCLUSIVE, &[][..])
        } else {
            (SharingMode::CONCURRENT, &indices[..])
        };

        let create_info = SwapchainCreateInfoKHRBuilder::new()
            .surface(device.get_surface())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(indices)
            .pre_transform(swapchain_support.capabilities().current_transform)
            .composite_alpha(CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe {
            device
                .get_device_ptr()
                .create_swapchain_khr(&create_info, None)
        }
        .result()?;

        let images = unsafe {
            device
                .get_device_ptr()
                .get_swapchain_images_khr(swapchain, None)
        }
        .result()?;
        let image_format = surface_format.format;

        Ok((extent, image_format, images, swapchain))
    }

    fn create_image_views(
        device: &VRTDevice,
        images: &[Image],
        image_format: Format,
    ) -> VkResult<Vec<ImageView>> {
        images
            .iter()
            .map(|image| {
                let create_info = ImageViewCreateInfoBuilder::new()
                    .image(*image)
                    .view_type(ImageViewType::_2D)
                    .format(image_format)
                    .components(
                        *ComponentMappingBuilder::new()
                            .r(ComponentSwizzle::IDENTITY)
                            .g(ComponentSwizzle::IDENTITY)
                            .b(ComponentSwizzle::IDENTITY)
                            .a(ComponentSwizzle::IDENTITY),
                    )
                    .subresource_range(
                        *ImageSubresourceRangeBuilder::new()
                            .aspect_mask(ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );
                unsafe {
                    device
                        .get_device_ptr()
                        .create_image_view(&create_info, None)
                }
                .map_err(VkError::Vk)
            })
            .collect()
    }

    fn create_render_pass(device: &DeviceLoader, image_format: Format) -> VkResult<RenderPass> {
        let color_attachment = AttachmentDescriptionBuilder::new()
            .format(image_format)
            .samples(SampleCountFlagBits::_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = AttachmentReferenceBuilder::new()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = SubpassDescriptionBuilder::new()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));

        let render_pass_info = RenderPassCreateInfoBuilder::new()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass));

        Ok(unsafe { device.create_render_pass(&render_pass_info, None) }.result()?)
    }

    fn create_framebuffers(
        device: &VRTDevice,
        image_views: &Vec<ImageView>,
        extent: &Extent2D,
        render_pass: RenderPass,
    ) -> VkResult<Vec<Framebuffer>> {
        image_views
            .iter()
            .map(|image_view| {
                let framebuffer_info = FramebufferCreateInfoBuilder::new()
                    .render_pass(render_pass)
                    .attachments(std::slice::from_ref(image_view))
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                unsafe {
                    device
                        .get_device_ptr()
                        .create_framebuffer(&framebuffer_info, None)
                }
                .map_err(VkError::Vk)
            })
            .collect()
    }

    fn create_sync_objects(
        device: &VRTDevice,
        images: &[Image],
    ) -> VkResult<SyncObjects<MAX_FRAMES_IN_FLIGHT>> {
        fn create_objects<T>(
            create_fn: impl Fn() -> VkResult<T>,
        ) -> VkResult<[T; MAX_FRAMES_IN_FLIGHT]>
        where
            T: Default + Copy,
        {
            let mut objects = [T::default(); MAX_FRAMES_IN_FLIGHT];
            for object in &mut objects {
                *object = create_fn()?;
            }
            Ok(objects)
        }

        let semaphore_info = SemaphoreCreateInfoBuilder::new();
        let create_semaphore = || {
            unsafe {
                device
                    .get_device_ptr()
                    .create_semaphore(&semaphore_info, None)
            }
            .map_err(VkError::Vk)
        };

        let fence_info = FenceCreateInfoBuilder::new().flags(FenceCreateFlags::SIGNALED);
        let create_fence = || {
            unsafe { device.get_device_ptr().create_fence(&fence_info, None) }.map_err(VkError::Vk)
        };

        Ok(SyncObjects {
            image_available_semaphores: create_objects(create_semaphore)?,
            render_finished_semaphores: create_objects(create_semaphore)?,
            in_flight_fences: create_objects(create_fence)?,
            images_in_flight: vec![None; images.len()],
            current_frame: 0,
        })
    }

    fn choose_swap_surface_format(available_formats: &[SurfaceFormatKHR]) -> SurfaceFormatKHR {
        *available_formats
            .iter()
            .find(|format| {
                format.format == Format::B8G8R8A8_SRGB
                    && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR_KHR
            })
            .unwrap_or(&available_formats[0])
    }

    fn choose_swap_present_mode(available_present_modes: &[PresentModeKHR]) -> PresentModeKHR {
        *available_present_modes
            .iter()
            .find(|present_mode| **present_mode == PresentModeKHR::MAILBOX_KHR)
            .unwrap_or(&PresentModeKHR::FIFO_KHR)
    }

    fn choose_swap_extent(extent: Extent2D, capabilities: &SurfaceCapabilitiesKHR) -> Extent2D {
        if capabilities.current_extent.width == u32::MAX {
            *Extent2DBuilder::new()
                .width(extent.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ))
                .height(extent.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ))
        } else {
            capabilities.current_extent
        }
    }

    pub fn get_render_pass(&self) -> RenderPass {
        self.render_pass
    }

    pub fn get_frame_buffer(&self) -> Vec<Framebuffer> {
        self.framebuffers
    }

    pub fn get_extent(&self) -> Extent2D {
        self.extent
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }

            self.device.destroy_swapchain_khr(self.swapchain, None);

            for framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }

            self.device.destroy_render_pass(self.render_pass, None);

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_fence(self.sync.in_flight_fences[i], None);
                self.device
                    .destroy_semaphore(self.sync.render_finished_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.sync.image_available_semaphores[i], None);
            }
        }
    }
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
