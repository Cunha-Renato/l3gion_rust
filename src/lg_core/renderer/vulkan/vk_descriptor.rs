#![allow(non_camel_case_types)]

use vulkanalia::{
    prelude::v1_2::*, 
    vk,
};
use crate::{lg_core::lg_types::reference::Rfc, MyError};
use super::{shader::{self, Shader, ShaderDescriptor}, vk_device::VkDevice, vk_memory_allocator::VkMemoryManager, vk_texture::VkTexture, vk_uniform_buffer::VkUniformBuffer};

pub(crate) const MAX_SETS: usize = 1000;
pub struct VkDescriptorData {
    device: Rfc<VkDevice>,
    descriptor_infos: Vec<ShaderDescriptor>,
    pub layouts: Vec<Vec<vk::DescriptorSetLayout>>,
    pool: vk::DescriptorPool,
    sets: Vec<Vec<vk::DescriptorSet>>,
    pub buffers: Vec<VkUniformBuffer>,
    memory_manager: Rfc<VkMemoryManager>,
}
impl VkDescriptorData {
    pub unsafe fn new (
        device: Rfc<VkDevice>,
        shaders: &[&Shader],
        memory_manager: Rfc<VkMemoryManager>,
        buffers: Vec<VkUniformBuffer>,
    ) -> Result<Self, MyError>
    {
        let mut layouts = Vec::new();
        
        let mut shader_infos = Vec::new();
        for shader in shaders {
            shader_infos.append(&mut shader.get_descriptors()?);
        }

        for _ in 0..MAX_SETS {
            layouts.push(get_layouts(&device.borrow(), &shader_infos)?);
        }
        
        let max_sets_pool = layouts.len() * layouts[0].len();
        
        let pool = create_pool(&device.borrow(), max_sets_pool as u32, &shader_infos)?;
        let sets = create_sets(&device.borrow(), &layouts, &pool)?;
        
        Ok(Self {
            device,
            descriptor_infos: shader_infos,
            layouts,
            pool,
            sets,
            buffers,
            memory_manager,
        })
    }
    
    pub unsafe fn update_buffer(
        &mut self, 
        buffer_index: usize,
        set_index: usize,
        binding: u32,
        obj_index: usize,
    ) {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(self.buffers[buffer_index].buffer)
            .offset(0)
            .range(self.buffers[buffer_index].range);
        
        let buffer_info = &[info];
        
        let ds_info = self.descriptor_infos
            .iter()
            .find(|ds| ds.binding == binding && ds.set == set_index as u32)
            .unwrap();
        
        let buffer_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.sets[obj_index][set_index])
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(ds_info.ds_type)
            .buffer_info(buffer_info);
        
        self.device.borrow().get_device().update_descriptor_sets(
            &[buffer_write], 
            &[] as &[vk::CopyDescriptorSet]
        )
    }
    pub unsafe fn update_sampled_image(
        &mut self,
        texture: &VkTexture,
        set_index: usize,
        binding: u32,
        obj_index: usize,
    ) {
        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture.image.view)
            .sampler(texture.sampler);

        let image_info = &[info];
        
        let ds_info = self.descriptor_infos
            .iter()
            .find(|ds| ds.binding == binding && ds.set == set_index as u32)
            .unwrap();

        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.sets[obj_index][set_index])
            .dst_binding(ds_info.binding)
            .dst_array_element(0)
            .descriptor_type(ds_info.ds_type)
            .image_info(image_info);

        self.device.borrow().get_device().update_descriptor_sets(
            &[sampler_write], 
            &[] as &[vk::CopyDescriptorSet]
        )
    }
    pub fn get_sets(&self, obj_index: usize) -> &Vec<vk::DescriptorSet> {
        &self.sets[obj_index]
    }
    pub unsafe fn destroy(&mut self) -> Result<(), MyError>{
        let dev = self.device.borrow();
        let device = dev.get_device();
        
        device.destroy_descriptor_pool(self.pool, None);
        self.layouts
            .iter()
            .for_each(|l| l
                    .iter()
                    .for_each(|l| device.destroy_descriptor_set_layout(*l, None)));

        for b in &self.buffers {
            self.memory_manager.borrow_mut().free_buffer_region(b.region.clone())?;
            device.destroy_buffer(b.buffer, None);
        }

        Ok(())
    }
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
unsafe fn create_pool(device: &VkDevice, max_set_pools: u32, shader_infos: &[shader::ShaderDescriptor]) -> Result<vk::DescriptorPool, MyError>
{
    let mut pool_sizes = Vec::new();
    
    for sds in shader_infos {
        pool_sizes.push(vk::DescriptorPoolSize::builder()
            .type_(sds.ds_type)
            .descriptor_count(max_set_pools)
            .build()
        )
    }
    
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(max_set_pools);

    Ok(device.get_device().create_descriptor_pool(&info, None)?)
}

// TODO: Currently it only works with descriptor per layout (which is bad). Fix!!!!
unsafe fn get_layouts(device: &VkDevice, shader_infos: &[shader::ShaderDescriptor]) -> Result<Vec<vk::DescriptorSetLayout>, MyError> {
    let mut result = Vec::new();

    let mut shader_infos = shader_infos.to_vec();
    shader_infos.sort_by(|a, b| a.set.partial_cmp(&b.set).unwrap());
    for sds in shader_infos {
        let binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(sds.binding)
            .descriptor_type(sds.ds_type)
            .descriptor_count(1)
            .stage_flags(sds.shader_stage)
            .build();
        let bindings = &[binding];
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings);
        
        result.push(device.get_device().create_descriptor_set_layout(&info, None)?);
    }

    Ok(result)
}