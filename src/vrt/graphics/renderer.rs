use crate::vrt::device::device::VRTDevice;
use crate::vrt::device::swapchain::Swapchain;
use crate::vrt::utils::result::VkResult;
use erupt::vk::CommandBuffer;
use erupt::vk::CommandBufferBeginInfoBuilder;
use erupt::vk::Extent2D;
use erupt::vk::RenderPass;
use erupt::vk::{CommandBufferAllocateInfoBuilder, CommandBufferLevel, PipelineBindPoint};
use erupt::DeviceLoader;
use erupt::SmallVec;
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    window: Window,
    swapchain: Arc<Swapchain>,
    command_buffers: SmallVec<CommandBuffer>,
    device: Arc<DeviceLoader>,
}

impl Renderer {
    pub fn new(device: Arc<DeviceLoader>, window: &Window) -> VkResult<Self> {}

    fn recreate_swapchain(&self, device: &VRTDevice) {
        unsafe { device.get_device_ptr().device_wait_idle() }.result()?;

        unsafe { self.cleanup_swapchain() };

        let queue_family_indices =
            QueueFamilyIndices::new(&self.instance, self.surface, self.physical_device)?
                .complete()
                .ok_or(VkError::NoSuitableGpu)?;
        let swapchain_support =
            SwapchainSupportDetails::new(&self.instance, self.surface, self.physical_device)?;
    }

    fn create_command_buffers(
        device: &VRTDevice,
        command_buffer_count: u32,
    ) -> VkResult<SmallVec<CommandBuffer>> {
        let alloc_info = CommandBufferAllocateInfoBuilder::new()
            .command_pool(device.get_command_pool())
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(command_buffer_count);

        let command_buffers = unsafe {
            device
                .get_device_ptr()
                .allocate_command_buffers(&alloc_info)
        }
        .result()?;

        Ok(command_buffers)
    }

    fn begin_swapchain_render_pass() {
        let clear_color = ClearValue {
            color: ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let render_pass_info = RenderPassBeginInfoBuilder::new()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(
                *Rect2DBuilder::new()
                    .offset(*Offset2DBuilder::new().x(0).y(0))
                    .extent(*extent),
            )
            .clear_values(std::slice::from_ref(&clear_color));

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                SubpassContents::INLINE,
            );
        }
    }

    fn end_swapchain_render_pass() {
        unsafe {
            device.cmd_end_render_pass(command_buffer);
        }
    }
}
