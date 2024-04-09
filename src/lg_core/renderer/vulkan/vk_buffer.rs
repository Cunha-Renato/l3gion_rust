use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};
use crate::{lg_core::lg_types::reference::Rfc, MyError};
use super::{vk_device::VkDevice, vk_memory_allocator::{VkMemoryManager, VkMemoryRegion, VkMemoryUsageFlags}};

pub unsafe fn create_buffer(
    device: &VkDevice,
    memory_manager: &mut VkMemoryManager,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: VkMemoryUsageFlags,
) -> Result<(vk::Buffer, Rfc<VkMemoryRegion>), MyError>
{
    let device = device.get_device();
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    
    let buffer = device.create_buffer(&buffer_info, None)?;
    
    let buffer_region = memory_manager.alloc_buffer(&buffer, properties)?;
    memory_manager.bind_buffer(&buffer, buffer_region.clone())?;
    
    Ok((buffer, buffer_region))
}

pub unsafe fn copy_buffer_to_image(
    device: &VkDevice,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) -> Result<(), MyError>
{
    let command_buffer = device.get_transfer_queue().begin_single_time_commands(device)?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });

    device.get_device().cmd_copy_buffer_to_image(
        command_buffer,
        buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region],
    );

    device.get_transfer_queue().end_single_time_commands(device, command_buffer)?;

    Ok(())
}
pub unsafe fn copy_buffer(
    device: &VkDevice,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<(), MyError>
{
    let command_buffer = device.get_transfer_queue().begin_single_time_commands(device)?;
    
    let regions = vk::BufferCopy::builder().size(size);
    
    device.get_device().cmd_copy_buffer(command_buffer, source, destination, &[regions]);
    
    device.get_transfer_queue().end_single_time_commands(device, command_buffer)?;
    
    Ok(())
}