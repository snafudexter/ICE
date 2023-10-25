use crate::vrt::device::VRTDevice;
use crate::vrt::result::VkError;
use crate::vrt::result::{VkError::SwapChainExpired, VkResult};
use crate::vrt::swapchain::Swapchain;
use crate::vrt::swapchain::MAX_FRAMES_IN_FLIGHT;
use crate::VRTWindow;
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
use erupt::vk1_0::ViewportBuilder;
use erupt::vk1_0::{ClearDepthStencilValue, RenderPass};
use erupt::SmallVec;
use erupt::{vk, InstanceLoader};
use std::convert::TryFrom;
use std::sync::Arc;

pub struct VRTRenderer {
    swapchain: Swapchain,
    command_buffers: SmallVec<CommandBuffer>,
    device: Arc<VRTDevice>,
    current_frame_index: usize,
    is_frame_started: bool,
    image_index: u32,
}

impl VRTRenderer {
    pub unsafe fn new(device: Arc<VRTDevice>, window: &VRTWindow) -> VkResult<Self> {
        let swapchain = Swapchain::new(
            device.get_instance(),
            device.clone(),
            window.get_extent(),
            None,
        )?;

        let command_buffers = Self::create_command_buffers(&device)?;
        Ok(Self {
            swapchain,
            device,
            command_buffers,
            current_frame_index: 0,
            is_frame_started: false,
            image_index: 0,
        })
    }

    fn recreate_swapchain(&mut self, window: &VRTWindow) -> VkResult<()> {
        let mut extent = window.get_extent();
        while extent.width == 0 || extent.height == 0 {
            extent = window.get_extent();
        }

        unsafe {
            self.device.get_device_ptr().device_wait_idle().unwrap();
            self.swapchain = Swapchain::new(
                self.device.get_instance(),
                self.device.clone(),
                extent,
                Some(self.swapchain.get_swapchain_khr()),
            )
            .unwrap();
        }

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
            self.command_buffers.clear();
        }
    }

    pub fn begin_frame(&mut self, window: &VRTWindow) -> VkResult<CommandBuffer> {
        if self.is_frame_started == true {
            return Err(VkError::FrameAlreadyStarted);
        }

        let image_index_result = self.swapchain.acquire_next_image();

        let image_index = match image_index_result {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(window)?;
                return Err(SwapChainExpired);
            }
            result => result,
        }
        .unwrap();

        self.is_frame_started = true;
        self.image_index = image_index;

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

    pub fn end_frame(&mut self, window: &mut VRTWindow, command_buffer: CommandBuffer) {
        if !self.is_frame_started {
            println!("cannot end frame if it's not started");
            return;
        }
        unsafe {
            self.device
                .get_device_ptr()
                .end_command_buffer(command_buffer)
                .unwrap();
        }

        let present_result = self
            .swapchain
            .submit_command_buffer(
                &self.device,
                &command_buffer,
                &u32::try_from(self.image_index).unwrap(),
            )
            .unwrap();

        if present_result.raw == vk::Result::ERROR_OUT_OF_DATE_KHR
            || present_result.raw == vk::Result::SUBOPTIMAL_KHR
            || window.was_window_resized()
        {
            window.reset_resized_flag();
            self.recreate_swapchain(window).unwrap();
        } else {
            present_result.result().unwrap();
        }

        self.is_frame_started = false;
        self.current_frame_index = (self.current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn get_current_command_buffer(&self) -> CommandBuffer {
        self.command_buffers[self.current_frame_index as usize]
    }

    pub fn begin_swapchain_render_pass(&self, command_buffer: CommandBuffer) {
        if self.is_frame_started == false {
            println!("cannot begin swapchain render pass");
            return;
        }

        let color_clear_value = ClearValue {
            color: ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = ClearValue {
            depth_stencil: ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];

        let render_pass_info = RenderPassBeginInfoBuilder::new()
            .render_pass(self.swapchain.get_render_pass())
            .framebuffer(self.swapchain.get_frame_buffer()[self.image_index as usize])
            .render_area(
                *Rect2DBuilder::new()
                    .offset(*Offset2DBuilder::new().x(0).y(0))
                    .extent(self.swapchain.get_extent()),
            )
            .clear_values(clear_values);

        unsafe {
            self.device.get_device_ptr().cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                SubpassContents::INLINE,
            );

            let viewport = ViewportBuilder::new()
                .x(0.0)
                .y(self.swapchain.get_extent().height as f32)
                .width(self.swapchain.get_extent().width as f32)
                .height(-(self.swapchain.get_extent().height as f32))
                .min_depth(0.0)
                .max_depth(1.0);

            let scissor = Rect2DBuilder::new()
                .offset(*Offset2DBuilder::new().x(0).y(0))
                .extent(self.swapchain.get_extent());

            self.device.get_device_ptr().cmd_set_viewport(
                command_buffer,
                0,
                &std::slice::from_ref(&viewport),
            );

            self.device.get_device_ptr().cmd_set_scissor(
                command_buffer,
                0,
                &std::slice::from_ref(&scissor),
            );
        }
    }

    pub fn end_swapchain_render_pass(&self, command_buffer: CommandBuffer) {
        if self.is_frame_started == false {
            println!("cannot end swapchain render pass ");
            return;
        }
        unsafe {
            self.device
                .get_device_ptr()
                .cmd_end_render_pass(command_buffer);
        }
    }

    pub fn get_swapchain_render_pass(&self) -> RenderPass {
        self.swapchain.get_render_pass()
    }

    pub fn get_frame_index(&self) -> &usize {
        &self.current_frame_index
    }
}

impl Drop for VRTRenderer {
    fn drop(&mut self) {
        self.free_command_buffers();
    }
}
