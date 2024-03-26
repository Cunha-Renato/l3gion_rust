use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};
use crate::{lg_core::renderer::texture::Texture, MyError};
use super::{buffer, command_buffer::VkCommandPool, image::ImageData};

pub struct VkTexture {
    pub texture: Texture,
    pub image_data: ImageData,
    pub sampler: vk::Sampler,
}
impl VkTexture {
    pub unsafe fn new(
        instance: &Instance,
        device: &Device,
        physical_device: &vk::PhysicalDevice,
        command_pool: &VkCommandPool,
        queue: &vk::Queue,
        texture: Texture
    ) -> Result<Self, MyError> 
    {
        let (staging_buffer, staging_buffer_memory) = buffer::create_buffer(
            instance, 
            device, 
            physical_device, 
            texture.size(), 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_COHERENT 
            | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        // Copy
        let memory = device.map_memory(staging_buffer_memory, 0, texture.size(), vk::MemoryMapFlags::empty())?;
        memcpy(texture.pixels().as_ptr(), memory.cast(), texture.pixels().len());
        device.unmap_memory(staging_buffer_memory);

        // Creating Image        
        let mut tex_image = ImageData::new_with_memory(
            instance, 
            physical_device, 
            device, 
            vk::Format::R8G8B8A8_SRGB, 
            vk::ImageType::_2D, 
            vk::ImageViewType::_2D, 
            vk::ImageAspectFlags::COLOR, 
            texture.width(), 
            texture.height(), 
            texture.mip_level(), 
            vk::SampleCountFlags::_1, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST 
                | vk::ImageUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL
        )?;
        
        tex_image.transition_layout(
            device, 
            command_pool, 
            queue, 
            vk::Format::R8G8B8A8_SRGB, 
            vk::ImageLayout::UNDEFINED, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
            texture.mip_level()
        )?;
        
        buffer::copy_buffer_to_image(
            device, 
            command_pool, 
            queue, 
            staging_buffer, 
            tex_image.images[0], 
            texture.width(), 
            texture.height()
        )?;

        tex_image.transition_layout(
            device, 
            command_pool, 
            queue, 
            vk::Format::R8G8B8A8_SRGB, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 
            texture.mip_level()
        )?;
        
        // Cleanup
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        generate_mipmaps(
            instance, 
            device, 
            physical_device, 
            command_pool, 
            queue, 
            tex_image.images[0], 
            vk::Format::R8G8B8A8_SRGB, 
            texture.width(), 
            texture.height(), 
            texture.mip_level()
        )?;
        
        let info = vk::SamplerCreateInfo::builder()
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
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .min_lod(0.0)
            .max_lod(texture.mip_level() as f32)
            .mip_lod_bias(0.0);

        let sampler = device.create_sampler(&info, None)?;
        
        Ok(Self {
            texture,
            image_data: tex_image,
            sampler,
        })
    }
}
unsafe fn generate_mipmaps(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    command_pool: &VkCommandPool,
    queue: &vk::Queue,
    image: vk::Image,
    format: vk::Format,
    width: u32,
    height: u32,
    mip_levels: u32
) -> Result<(), MyError>
{
    // TODO: Aparently it's not common to generate mipmaps in the runtime, so you must find a way to do that in the texture itself!

    if !instance
        .get_physical_device_format_properties(*physical_device, format)
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
    {
        return Err("Texture image format does not support linear blitting!".into());
    }

    let command_buffer = command_pool.begin_single_time_commands(device)?;
    
    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_array_layer(0)
        .layer_count(1)
        .level_count(1);
    
    let mut barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(subresource);

    let mut mip_width = width;
    let mut mip_height = height;
    
    for i in 1..mip_levels {
        barrier.subresource_range.base_mip_level = i - 1;
        barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;
        
        device.cmd_pipeline_barrier(
            command_buffer, 
            vk::PipelineStageFlags::TRANSFER, 
            vk::PipelineStageFlags::TRANSFER, 
            vk::DependencyFlags::empty(), 
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );
        
        let src_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i - 1)
            .base_array_layer(0)
            .layer_count(1);
        
        let dst_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i)
            .base_array_layer(0)
            .layer_count(1);
        
        let blit = vk::ImageBlit::builder()
            .src_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: mip_width as i32,
                    y: mip_height as i32,
                    z: 1,
                },
            ])
            .src_subresource(src_subresource)
            .dst_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: (if mip_width > 1 { mip_width / 2} else { 1 } as i32),
                    y: (if mip_height > 1 { mip_height / 2} else { 1 } as i32),
                    z: 1,
                }
            ])
            .dst_subresource(dst_subresource);
        
        device.cmd_blit_image(
            command_buffer, 
            image, 
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL, 
            image, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
            &[blit], 
            vk::Filter::LINEAR,
        );
        
        barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
        
        device.cmd_pipeline_barrier(
            command_buffer, 
            vk::PipelineStageFlags::TRANSFER, 
            vk::PipelineStageFlags::FRAGMENT_SHADER, 
            vk::DependencyFlags::empty(), 
            &[] as &[vk::MemoryBarrier], 
            &[] as &[vk::BufferMemoryBarrier], 
            &[barrier],
        );
        
        if mip_width > 1 { mip_width /= 2 }
        if mip_height > 1 { mip_height /= 2 }
    }
    
    barrier.subresource_range.base_mip_level = mip_levels - 1;
    barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    device.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[barrier],
    );

    command_pool.end_single_time_commands(device, queue, command_buffer)?;

    Ok(())
}