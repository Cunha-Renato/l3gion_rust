use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};
use crate::MyError;
use super::{command_buffer::VkCommandPool, memory::get_memory_type_index};

pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory), MyError>
{
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    
    let buffer = device.create_buffer(&buffer_info, None)?;
    
    let requirements = device.get_buffer_memory_requirements(buffer);
    
    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance, 
            physical_device, 
            properties, 
            requirements
        )?);
    
    let buffer_memory = device.allocate_memory(&memory_info, None)?;
    
    device.bind_buffer_memory(buffer, buffer_memory, 0)?;
    
    Ok((buffer, buffer_memory))
}

pub unsafe fn copy_buffer_to_image(
    device: &Device,
    command_pool: &VkCommandPool,
    queue: &vk::Queue,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) -> Result<(), MyError>
{
    let command_buffer = command_pool.begin_single_time_commands(device)?;

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

    device.cmd_copy_buffer_to_image(
        command_buffer,
        buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region],
    );

    command_pool.end_single_time_commands(device, queue, command_buffer)?;

    Ok(())
}
pub unsafe fn copy_buffer(
    device: &Device,
    command_pool: &VkCommandPool,
    queue: &vk::Queue,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<(), MyError>
{
    let command_buffer = command_pool.begin_single_time_commands(device)?;
    
    let regions = vk::BufferCopy::builder().size(size);
    
    device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);
    
    command_pool.end_single_time_commands(device, queue, command_buffer)?;
    
    Ok(())
}