use crate::vrt::device::device::VRTDevice;
use crate::vrt::device::swapchain::Swapchain;
use crate::vrt::device::swapchain::MAX_FRAMES_IN_FLIGHT;
use crate::vrt::utils::result::{VkError::SwapChainExpired, VkResult};
use crate::VRTWindow;
use erupt::vk;
use erupt::vk::ClearColorValue;
use erupt::vk::ClearValue;
use erupt::vk::CommandBuffer;
use erupt::vk::Offset2DBuilder;
use erupt::vk::Rect2DBuilder;
use erupt::vk::RenderPassBeginInfoBuilder;
use erupt::vk::{
    CommandBufferAllocateInfoBuilder, CommandBufferBeginInfoBuilder, CommandBufferLevel,
};
use erupt::vk1_0::SubpassContents;
use erupt::SmallVec;

pub struct Renderer<'a> {
    window: &'a VRTWindow,
    swapchain: Swapchain,
    command_buffers: SmallVec<CommandBuffer>,
    device: &'a VRTDevice,
    current_frame_index: usize,
    is_frame_started: bool,
}

impl<'a> Renderer<'a> {
    pub fn new(device: &'a VRTDevice, window: &'a VRTWindow) -> VkResult<Self> {
        let swapchain = Swapchain::new(&device, window.get_extent())?;

        let command_buffers = Self::create_command_buffers(&device)?;
        Ok(Self {
            window,
            swapchain,
            command_buffers,
            device,
            current_frame_index: 0,
            is_frame_started: false,
        })
    }

    fn recreate_swapchain(&mut self) -> VkResult<()> {
        let mut extent = self.window.get_extent();
        while extent.width == 0 || extent.height == 0 {
            extent = self.window.get_extent();
        }
        self.swapchain = Swapchain::new(&self.device, extent)?;
        Ok(())
    }

    fn create_command_buffers(device: &VRTDevice) -> VkResult<SmallVec<CommandBuffer>> {
        let alloc_info = CommandBufferAllocateInfoBuilder::new()
            .command_pool(device.get_command_pool())
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);

        let command_buffers = unsafe {
            device
                .get_device_ptr()
                .allocate_command_buffers(&alloc_info)
        }
        .result()?;

        Ok(command_buffers)
    }

    fn free_command_buffers(&mut self) {
        unsafe {
            self.device.get_device_ptr().free_command_buffers(
                self.device.get_command_pool(),
                self.command_buffers.as_slice(),
            );
        }
    }

    fn begin_frame(&mut self) -> VkResult<CommandBuffer> {
        let image_index_result = self.swapchain.acquire_next_image(self.current_frame_index);

        let image_index = match image_index_result {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain()?;
                return Err(SwapChainExpired);
            }
            result => result,
        };

        self.is_frame_started = true;

        let begin_info = CommandBufferBeginInfoBuilder::new();

        let command_buffer = self.get_current_command_buffer();

        unsafe {
            self.device
                .get_device_ptr()
                .begin_command_buffer(command_buffer, &begin_info)
        }
        .result()?;

        Ok(command_buffer)
    }

    fn end_frame() {}

    fn get_current_command_buffer(&self) -> CommandBuffer {
        self.command_buffers[self.current_frame_index as usize]
    }

    fn begin_swapchain_render_pass(&self) {
        let clear_color = ClearValue {
            color: ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let render_pass_info = RenderPassBeginInfoBuilder::new()
            .render_pass(self.swapchain.get_render_pass())
            .framebuffer(self.swapchain.get_frame_buffer()[self.current_frame_index])
            .render_area(
                *Rect2DBuilder::new()
                    .offset(*Offset2DBuilder::new().x(0).y(0))
                    .extent(self.swapchain.get_extent()),
            )
            .clear_values(std::slice::from_ref(&clear_color));

        unsafe {
            self.device.get_device_ptr().cmd_begin_render_pass(
                self.get_current_command_buffer(),
                &render_pass_info,
                SubpassContents::INLINE,
            );
        }
    }

    fn end_swapchain_render_pass(&self) {
        unsafe {
            self.device
                .get_device_ptr()
                .cmd_end_render_pass(self.get_current_command_buffer());
        }
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        self.free_command_buffers();
    }
}
