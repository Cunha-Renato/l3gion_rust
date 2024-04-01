use vulkanalia:: {
    prelude::v1_0::*, 
    vk::{self, DescriptorSet},
};
use crate::MyError;

use super::{uniform_buffer::UniformBuffer, vk_device::VkDevice, vk_swapchain::VkSwapchain, vk_texture::VkTexture};

#[derive(Default)]
pub struct DescriptorData {
    pub layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub sets: Vec<DescriptorSet>,
}
impl DescriptorData {
    pub unsafe fn new_default(
        device: &Device, 
        swapchain: &VkSwapchain,
        uniform_buffer: &UniformBuffer,
    ) -> Result<Self, MyError> 
    {
        let layout = create_default_descriptor_set_layout(device)?;
        let pool = create_descriptor_pool(device, swapchain)?;
        let sets = create_default_descriptor_sets(
            device, 
            &layout, 
            &pool, 
            swapchain,
        )?;
        
        Ok(Self {
            layout,
            pool,
            sets
        })
    }
    pub unsafe fn update_default(
        &self,
        device: &Device,
        uniform_buffer: &UniformBuffer,
        texture: &VkTexture
    ) 
    {
        update_default_descriptor_sets(device, &self.sets, uniform_buffer, texture)
    }
    
    pub unsafe fn destroy_pool(&mut self, device: &VkDevice) {
        device.get_device().destroy_descriptor_pool(self.pool, None);
    }
}

pub unsafe fn create_default_descriptor_set_layout(
    device: &Device,
) -> Result<vk::DescriptorSetLayout, MyError>
{
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
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
    swapchain: &VkSwapchain,
) -> Result<vk::DescriptorPool, MyError>
{
    let sw_images_len = swapchain.images.len() as u32;
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(sw_images_len);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(sw_images_len);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(sw_images_len);

    Ok(device.create_descriptor_pool(&info, None)?)
}
unsafe fn create_default_descriptor_sets(
    device: &Device,
    layout: &vk::DescriptorSetLayout,
    pool: &vk::DescriptorPool,
    swapchain: &VkSwapchain,
) -> Result<Vec<vk::DescriptorSet>, MyError>
{
    // Allocate
    let layouts = vec![
        *layout; 
        swapchain.images.len()
    ];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(*pool)
        .set_layouts(&layouts);

    let descriptor_sets = device.allocate_descriptor_sets(&info)?;

    Ok(descriptor_sets)
}
pub unsafe fn update_default_descriptor_sets(
    device: &Device,
    sets: &[vk::DescriptorSet],
    uniform_buffer: &UniformBuffer,
    texture: &VkTexture,
) {
    for i in 0..sets.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffer.buffers[i])
            .offset(0)
            .range(uniform_buffer.ubo_size);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
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

        device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
    }
}