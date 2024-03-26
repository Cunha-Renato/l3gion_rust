use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};

use crate::MyError;
use super::{command_buffer::*, memory::get_memory_type_index};

#[derive(Default)]
pub struct ImageData {
    pub images: Vec<vk::Image>,
    pub memories: Option<Vec<vk::DeviceMemory>>,
    pub views: Vec<vk::ImageView>,
}
impl ImageData {
    // Public
    pub unsafe fn new(
        images: Vec<vk::Image>,
        device: &Device,
        format: vk::Format,
        image_type: vk::ImageViewType,
        aspect_mask: vk::ImageAspectFlags,
        mip_levels: u32,
    ) -> Result<Self, MyError> 
    {
        Ok(Self {
            images,
            memories: None,
            views: Self::get_image_views(
                device, 
                images, 
                format, 
                image_type, 
                aspect_mask, 
                mip_levels
            )?
        }) 
    }
    pub unsafe fn new_with_memory(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        device: &Device,
        format: vk::Format,
        image_type: vk::ImageType,
        image_view_type: vk::ImageViewType,
        aspect_mask: vk::ImageAspectFlags,
        width: u32,
        height: u32,
        mip_levels: u32,
        samples: vk::SampleCountFlags,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self, MyError>
    {
        let (image, memory) = create_image(
            instance, 
            physical_device, 
            device, 
            width, 
            height, 
            mip_levels, 
            samples, 
            image_type, 
            format, 
            tiling, 
            usage, 
            properties
        )?;
        
        let images = vec![image];
        let memories = Some(vec![memory]);
        Ok(Self {
            images,
            memories,
            views: Self::get_image_views(
                device, 
                images, 
                format, 
                image_view_type, 
                aspect_mask, 
                mip_levels
            )?
        })
    }
    pub unsafe fn transition_layout(
        &mut self,
        device: &Device,
        command_pool: &VkCommandPool,
        queue: &vk::Queue,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        mip_levels: u32
    ) -> Result<(), MyError>
    {
        let (
        src_access_mask,
        dst_access_mask,
        src_stage_mask,
        dst_stage_mask
        ) = match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER
            ),
            _ => return Err("Unsuported image layout transition!".into()),
        };

        let command_buffer = command_pool.begin_single_time_commands(device)?;

        let subresource = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1);

        let barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout) 
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.images[0])
            .subresource_range(subresource)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);

        device.cmd_pipeline_barrier(
            command_buffer, 
            src_stage_mask, 
            dst_stage_mask,
            vk::DependencyFlags::empty(), 
            &[] as &[vk::MemoryBarrier], 
            &[] as &[vk::BufferMemoryBarrier], 
            &[barrier]
        );

        command_pool.end_single_time_commands(device, queue, command_buffer)?;

        Ok(())
    }
    
    // Private
    unsafe fn get_image_views(
        device: &Device,
        images: Vec<vk::Image>,
        format: vk::Format,
        image_type: vk::ImageViewType,
        aspect_mask: vk::ImageAspectFlags,
        mip_levels: u32
    ) -> Result<Vec<vk::ImageView>, MyError>
    {
        images.iter()
            .map(|i| Self::create_image_view(
                device, 
                *i, 
                format, 
                image_type, 
                aspect_mask, 
                mip_levels
            ))
            .collect::<Result<Vec<_>, _>>()
    }
    unsafe fn create_image_view(
        device: &Device,
        image: vk::Image,
        format: vk::Format,
        image_type: vk::ImageViewType,
        aspect_mask: vk::ImageAspectFlags,
        mip_levels: u32,
    ) -> Result<vk::ImageView, MyError>
    {
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1);
        
        let info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(image_type)
            .format(format)
            .subresource_range(subresource_range);
        
        Ok(device.create_image_view(&info, None)?)
    }
}

pub unsafe fn create_image(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    device: &Device,
    width: u32,
    height: u32,
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    image_type: vk::ImageType,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags
) -> Result<(vk::Image, vk::DeviceMemory), MyError>
{
    let info = vk::ImageCreateInfo::builder()
        .image_type(image_type)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(mip_levels)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples);

    let image = device.create_image(&info, None)?;

    // Memory

    let requirements = device.get_image_memory_requirements(image);

    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(instance, physical_device, properties, requirements)?);

    let image_memory = device.allocate_memory(&info, None)?;

    device.bind_image_memory(image, image_memory, 0)?;

    Ok((image, image_memory))
}