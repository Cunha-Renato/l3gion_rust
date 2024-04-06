use std::mem::size_of;

use vulkanalia::{
    prelude::v1_2::*, 
    vk,
};
use crate::{lg_core::renderer::uniform_buffer_object::{ModelUBO, ViewProjUBO}, MyError};
use super::{vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice, vk_texture::VkTexture, vk_uniform_buffer::VkUniformBuffer};

pub(crate) const MAX_SETS: usize = 1000;

pub enum BufferCategory {
    VIEW_PROJ = 0,
    MODEL = 1,
}
pub enum Layout {
    VIEW_PROJ = 0,
    IMAGE = 1,
    MODEL = 2,
}

pub struct VkPipelineDescriptorData {
    pub layouts: Vec<Vec<vk::DescriptorSetLayout>>,
    pool: vk::DescriptorPool,
    sets: Vec<Vec<vk::DescriptorSet>>,
    pub buffers: Vec<VkUniformBuffer>,
    offset: usize,
}
impl VkPipelineDescriptorData {
    pub unsafe fn new(
        device: &VkDevice,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice
    ) -> Result<Self, MyError>
    {
        let layouts = get_layouts(device)?;
        let max_sets_pool = layouts.len() * layouts[0].len();

        let pool = create_pool(device, max_sets_pool as u32)?;
        let sets = create_sets(device, &layouts, &pool)?;
        let buffers = vec![
            VkUniformBuffer::new::<ViewProjUBO>(
                instance, 
                device, 
                physical_device
            )?,
            VkUniformBuffer::new::<ModelUBO>(
                instance, 
                device, 
                physical_device
            )?,
        ];

        Ok(Self {
            layouts,
            pool,
            sets,
            buffers,
            offset: 0,
        })
    }
    pub unsafe fn update_model(
        &mut self,
        device: &VkDevice,
        obj_index: usize,
    ) {
        let buffer_index = BufferCategory::MODEL as usize;
        let set_index = Layout::MODEL as usize;

        let info = vk::DescriptorBufferInfo::builder()
            .buffer(self.buffers[buffer_index].buffer)
            .offset(0)
            .range(self.buffers[buffer_index].range);
        
        let buffer_info = &[info];
        let model_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.sets[obj_index][set_index])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .buffer_info(buffer_info);
        
        device.get_device().update_descriptor_sets(
            &[model_write], 
            &[] as &[vk::CopyDescriptorSet]
        );
    }
    pub unsafe fn update_vp(
        &mut self,
        device: &VkDevice,
        obj_index: usize,
    ) {
        let buffer_index = BufferCategory::VIEW_PROJ as usize;        
        let set_index = Layout::VIEW_PROJ as usize;

        let info = vk::DescriptorBufferInfo::builder()
            .buffer(self.buffers[buffer_index].buffer)
            .offset(0)
            .range(self.buffers[buffer_index].range);
        
        let buffer_info = &[info];
        let vp_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.sets[obj_index][set_index])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);
        
        device.get_device().update_descriptor_sets(
            &[vp_write], 
            &[] as &[vk::CopyDescriptorSet]
        );
    }
    pub unsafe fn update_image(
        &mut self,
        device: &VkDevice,
        texture: &VkTexture,
        obj_index: usize,
    ) {
        let set_index = Layout::IMAGE as usize;
        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture.image.view)
            .sampler(texture.sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.sets[obj_index][set_index])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_info);

        device.get_device().update_descriptor_sets(
            &[sampler_write], 
            &[] as &[vk::CopyDescriptorSet]
        );
    }
    pub fn get_sets(&self, obj_index: usize) -> &Vec<vk::DescriptorSet> {
        &self.sets[obj_index]
    }
    pub unsafe fn destroy(&mut self, device: &VkDevice) {
        let device = device.get_device();
        
        device.destroy_descriptor_pool(self.pool, None);
        self.layouts
            .iter()
            .for_each(|l| l
                    .iter()
                    .for_each(|l| device.destroy_descriptor_set_layout(*l, None)));
        self.buffers
            .iter()
            .for_each(|b| {
                device.free_memory(b.memory, None);
                device.destroy_buffer(b.buffer, None);
            })
    }
}
unsafe fn get_layouts(device: &VkDevice) -> Result<Vec<Vec<vk::DescriptorSetLayout>>, MyError>
{
    let mut result = Vec::new();
    for _ in 0..MAX_SETS {
        let mut layouts = Vec::new();

        // View
        let view_proj = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let binding = &[view_proj];
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(binding);

        layouts.push(device.get_device().create_descriptor_set_layout(&info, None)?);

        // Image
        let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(); 

        let binding = &[sampler_binding];
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(binding);   

        layouts.push(device.get_device().create_descriptor_set_layout(&info, None)?);
        
        // Model
        let model = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let binding = &[model];
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(binding);

        layouts.push(device.get_device().create_descriptor_set_layout(&info, None)?);

        result.push(layouts);
    }

    Ok(result)
}
unsafe fn create_pool(
    device: &VkDevice,
    max_sets_pool: u32,
) -> Result<vk::DescriptorPool, MyError>
{
    let view_proj = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(max_sets_pool)
        .build();

    let model = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .descriptor_count(max_sets_pool)
        .build();

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(max_sets_pool)
        .build();

    let pool_sizes = &[view_proj, sampler_size, model];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(max_sets_pool);

    Ok(device.get_device().create_descriptor_pool(&info, None)?)
}
unsafe fn create_sets(
    device: &VkDevice,
    layouts: &Vec<Vec<vk::DescriptorSetLayout>>,
    pool: &vk::DescriptorPool,
) -> Result<Vec<Vec<vk::DescriptorSet>>, MyError>
{
    let mut result = Vec::new();
    for layouts_vec in layouts {
        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(*pool)
            .set_layouts(layouts_vec);

        let descriptor_sets = device.get_device().allocate_descriptor_sets(&info)?;
        result.push(descriptor_sets);
    }

    Ok(result)
}