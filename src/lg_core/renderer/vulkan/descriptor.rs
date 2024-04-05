use std::mem::size_of;

use vulkanalia::{
    prelude::v1_2::*, 
    vk,
};
use crate::{lg_core::renderer::uniform_buffer_object::UniformBufferObject, MyError};

use super::{uniform_buffer::UniformBuffer, vk_device::VkDevice, vk_texture::VkTexture};

#[derive(Default)]
pub struct DescriptorData {
    pub layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub sets: Vec<vk::DescriptorSet>,
}
impl DescriptorData {
    pub unsafe fn new_default(
        device: &Device, 
        lenght: u32,
    ) -> Result<Self, MyError> 
    {
        let layout = create_default_descriptor_set_layout(device)?;
        let pool = create_descriptor_pool(device, lenght)?;
        let sets = create_default_descriptor_sets(
            device, 
            &layout, 
            &pool, 
            lenght,
        )?;
        
        Ok(Self {
            layout,
            pool,
            sets
        })
    }
    pub unsafe fn update_default(
        &self,
        index: usize,
        ubo_offset: u64,
        device: &Device,
        uniform_buffer: &UniformBuffer,
        texture: &VkTexture,
    ) 
    {
        update_default_descriptor_sets(device, index, ubo_offset, &self.sets, uniform_buffer, texture)
    }
    
    pub unsafe fn destroy_pool(&mut self, device: &VkDevice) {
        device.get_device().destroy_descriptor_pool(self.pool, None);
    }
    pub unsafe fn destroy_layout(&mut self, device: &VkDevice) {
        device.get_device().destroy_descriptor_set_layout(self.layout, None);
    }
}

pub unsafe fn create_default_descriptor_set_layout(
    device: &Device,
) -> Result<vk::DescriptorSetLayout, MyError>
{
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT); 

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    Ok(device.create_descriptor_set_layout(&info, None)?)
}
unsafe fn create_descriptor_pool(
    device: &Device,
    lenght: u32
) -> Result<vk::DescriptorPool, MyError>
{
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .descriptor_count(1);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(lenght);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(lenght);

    Ok(device.create_descriptor_pool(&info, None)?)
}
unsafe fn create_default_descriptor_sets(
    device: &Device,
    layout: &vk::DescriptorSetLayout,
    pool: &vk::DescriptorPool,
    lenght: u32,
) -> Result<Vec<vk::DescriptorSet>, MyError>
{
    // Allocate
    let layouts = vec![
        *layout; 
        lenght as usize
    ];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(*pool)
        .set_layouts(&layouts);

    let descriptor_sets = device.allocate_descriptor_sets(&info)?;

    Ok(descriptor_sets)
}
pub unsafe fn update_default_descriptor_sets(
    device: &Device,
    i: usize,
    ubo_offset: u64,
    sets: &[vk::DescriptorSet],
    uniform_buffer: &UniformBuffer,
    texture: &VkTexture,
) {
    let info = vk::DescriptorBufferInfo::builder()
        .buffer(uniform_buffer.buffers[0])
        .offset(0)
        .range(size_of::<UniformBufferObject>() as u64);

    let buffer_info = &[info];
    let ds_write = vk::WriteDescriptorSet::builder()
        .dst_set(sets[i])
        .dst_binding(ubo_offset as u32)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .buffer_info(buffer_info);

    let info = vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(texture.image.view)
        .sampler(texture.sampler);

    let image_info = &[info];
    let sampler_write = vk::WriteDescriptorSet::builder()
        .dst_set(sets[i])
        .dst_binding(1)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(image_info);

    device.update_descriptor_sets(&[ds_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
}