use super::{
    buffer::VRTBuffer,
    device::VRTDevice,
    result::{VkError, VkResult},
};
use erupt::{
    vk,
    vk1_0::{
        DeviceMemory, DeviceSize, Format, Image, ImageAspectFlags, ImageLayout, ImageUsageFlags,
        ImageView, ImageViewCreateInfoBuilder, MemoryPropertyFlags, Sampler,
        SamplerCreateInfoBuilder,
    },
};
use std::{ffi::c_void, fs::File, path::Path, sync::Arc};

#[derive(Debug)]
pub struct VRTTexture {
    device: Arc<VRTDevice>,
    image: Image,
    image_memory: DeviceMemory,
    image_view: ImageView,
    sampler: Sampler,
    width: u32,
    height: u32,
    // mip_levels: u32,
    format: Format,
}

impl VRTTexture {
    pub fn new(
        device: Arc<VRTDevice>,
        image_data: &Vec<u8>,
        width: u32,
        height: u32,
        format: Format
    ) -> VkResult<Self> {

        // Create (staging)

        let mut staging_buffer = VRTBuffer::new(
            device.clone(),
            std::mem::size_of::<u8>().try_into()?,
            image_data.len().try_into().unwrap(),
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            None,
        );

        // Copy (staging)
        let buffer_size = (std::mem::size_of::<u8>() * image_data.len()) as u64;
        staging_buffer.map(Some(buffer_size), Some(0));

        staging_buffer.write_to_buffer(
            image_data.as_ptr(),
            staging_buffer.get_mapped_memory().unwrap(),
            image_data.len() as DeviceSize,
            0,
        );

        staging_buffer.unmap();

        let (texture_image, texture_image_memory) = Self::create_image(
            device.clone(),
            width,
            height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        Self::transition_image_layout(
            device.clone(),
            texture_image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;
    
        unsafe { Self::copy_buffer_to_image(device.clone(), staging_buffer.get_buffer(), texture_image, width, height) }?;
    
        Self::transition_image_layout(
            device.clone(),
            texture_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;
    

        let image_view =
            Self::create_image_view(device.clone(), texture_image, vk::Format::R8G8B8A8_SRGB)
                .unwrap();

        let sampler: Sampler = Self::create_texture_sampler(device.clone()).unwrap();

        Ok(Self {
            device,
            image: texture_image,
            image_memory: texture_image_memory,
            image_view,
            sampler,
            width,
            height,
            // mip_levels,
            format,
        })
    }

    fn create_texture_sampler(device: Arc<VRTDevice>) -> VkResult<Sampler> {
        let info = vk::SamplerCreateInfoBuilder::new()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

        Ok(unsafe{device.get_device_ptr().create_sampler(&info, None).unwrap()})
    }

    fn create_image_view(
        device: Arc<VRTDevice>,
        image: vk::Image,
        format: vk::Format,
    ) -> VkResult<vk::ImageView> {
        let subresource_range = vk::ImageSubresourceRangeBuilder::new()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let info = vk::ImageViewCreateInfoBuilder::new()
            .image(image)
            .view_type(vk::ImageViewType::_2D)
            .format(format)
            .subresource_range(*subresource_range);

        unsafe {
            Ok(device
                .get_device_ptr()
                .create_image_view(&info, None)
                .unwrap())
        }
    }

    pub fn get_image_view(&self) -> ImageView {
        self.image_view
    }

    pub fn get_sampler(&self) -> Sampler {
        self.sampler
    }

    fn load_image(path: &Path) -> VkResult<(u32, u32, Vec<u8>)> {
        let image = File::open(path)?;
        // Placeholder for image loading logic (use image-rs or similar library)
        let decoder = png::Decoder::new(image);
        let mut reader = decoder.read_info()?;

        let mut pixels = vec![0; reader.info().raw_bytes()];
        reader.next_frame(&mut pixels)?;

        let size = reader.info().raw_bytes() as u64;
        let (width, height) = reader.info().size();
        Ok((width, height, pixels))
    }

    fn create_image(
        device: Arc<VRTDevice>,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> VkResult<(vk::Image, vk::DeviceMemory)> {
        let info = vk::ImageCreateInfoBuilder::new()
            .image_type(vk::ImageType::_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .samples(vk::SampleCountFlagBits::_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe { device.get_device_ptr().create_image(&info, None).unwrap() };

        let requirements = unsafe { device.get_device_ptr().get_image_memory_requirements(image) };

        let info = vk::MemoryAllocateInfoBuilder::new()
            .allocation_size(requirements.size)
            .memory_type_index(VRTDevice::find_memory_type(
                device.get_instance(),
                device.get_physical_device(),
                requirements.memory_type_bits,
                properties,
            )?);

        let image_memory = unsafe {
            device
                .get_device_ptr()
                .allocate_memory(&info, None)
                .unwrap()
        };

        unsafe {
            device
                .get_device_ptr()
                .bind_image_memory(image, image_memory, 0)
                .unwrap()
        };

        Ok((image, image_memory))
    }

    fn transition_image_layout(
        device: Arc<VRTDevice>,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> VkResult<()> {
        unsafe {
            let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
                match (old_layout, new_layout) {
                    (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                    ),
                    (
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    ) => (
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::AccessFlags::SHADER_READ,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                    ),
                    _ => return Err(VkError::UnsupportedLayoutTransition),
                };

            let command_buffer = Self::begin_single_time_commands(device.clone())?;

            let subresource = vk::ImageSubresourceRangeBuilder::new()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let barrier = vk::ImageMemoryBarrierBuilder::new()
                .old_layout(old_layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(*subresource)
                .src_access_mask(src_access_mask)
                .dst_access_mask(dst_access_mask);

            device.get_device_ptr().cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrierBuilder],
                &[] as &[vk::BufferMemoryBarrierBuilder],
                &[barrier],
            );

            Self::end_single_time_commands(device, command_buffer)?;

            Ok(())
        }
    }

    unsafe fn copy_buffer_to_image(
        device: Arc<VRTDevice>,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) -> VkResult<()> {
        let command_buffer = Self::begin_single_time_commands(device.clone())?;
    
        let subresource = vk::ImageSubresourceLayersBuilder::new()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);
    
        let region = vk::BufferImageCopyBuilder::new()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(*subresource)
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            });
    
        device.get_device_ptr().cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    
        Self::end_single_time_commands(device.clone(), command_buffer)?;
    
        Ok(())
    }
    

    fn begin_single_time_commands(device: Arc<VRTDevice>) -> VkResult<vk::CommandBuffer> {
        unsafe {
            let info = vk::CommandBufferAllocateInfoBuilder::new()
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_pool(device.get_command_pool())
                .command_buffer_count(1);

            let command_buffer = device
                .get_device_ptr()
                .allocate_command_buffers(&info)
                .unwrap()[0];

            // Begin

            let info = vk::CommandBufferBeginInfoBuilder::new()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            device
                .get_device_ptr()
                .begin_command_buffer(command_buffer, &info);

            Ok(command_buffer)
        }
    }

    fn end_single_time_commands(
        device: Arc<VRTDevice>,
        command_buffer: vk::CommandBuffer,
    ) -> VkResult<()> {
        // End
        unsafe {
            device.get_device_ptr().end_command_buffer(command_buffer);

            // Submit

            let command_buffers = &[command_buffer];
            let info = vk::SubmitInfoBuilder::new().command_buffers(command_buffers);

            device.get_device_ptr().queue_submit(
                device.get_queues().graphics,
                &[info],
                vk::Fence::null(),
            );
            device
                .get_device_ptr()
                .queue_wait_idle(device.get_queues().graphics);

            // Cleanup

            device
                .get_device_ptr()
                .free_command_buffers(device.get_command_pool(), &[command_buffer]);
        }

        Ok(())
    }
}

impl Drop for VRTTexture {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_ptr()
                .destroy_sampler(self.sampler, None);
            self.device
                .get_device_ptr()
                .destroy_image_view(self.image_view, None);
            self.device.get_device_ptr().destroy_image(self.image, None);
            self.device
                .get_device_ptr()
                .free_memory(self.image_memory, None);
        }
    }
}
