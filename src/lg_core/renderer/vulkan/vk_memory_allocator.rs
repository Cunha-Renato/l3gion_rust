use std::{cell::RefCell, collections::HashMap, rc::Rc};
use vulkanalia::{
    prelude::v1_2::*, vk
};
use crate::MyError;
use super::{vk_device::VkDevice, vk_image::VkImage, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice};

#[derive(Default)]
pub struct VkMemoryRegion {
    pub index: u32,
    pub size: u64,
    pub begin: u64,
    pub end: u64,
    allocated: bool,
}

#[derive(Default)]
struct VkMemory {
    regions: Vec<Rc<RefCell<VkMemoryRegion>>>,
    memory: vk::DeviceMemory,
}

pub struct VkMemManager {
    memories: HashMap<u32, VkMemory>,
    images_to_bind: Vec<&'static VkImage>,
}
impl VkMemManager {
    pub fn new() -> Self {
        Self {
            memories: HashMap::default(),
            images_to_bind: Vec::default(),
        }
    }
    pub unsafe fn stage_alloc(
        &mut self,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        requirements: vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Rc<RefCell<VkMemoryRegion>>, MyError>
    {
        let index = get_memory_type_index(
            instance,
            physical_device,
            properties,
            requirements,
        )?;

        let region = {
            let memory = self.memories.entry(index).or_insert_with(|| VkMemory::default());
            let previous_end = memory.regions.last().map_or(0, |r| r.borrow().end);

            let mut region = VkMemoryRegion::default();
            region.index = index;
            region.allocated = false;
            region.size = requirements.size;
            region.begin = previous_end;
            region.end = region.begin + region.size;

            let result = Rc::new(RefCell::new(region));
            memory.regions.push(result.clone());

            result
        };

        Ok(region)
    }
    pub unsafe fn allocate(
        &mut self,
        device: &VkDevice,
    ) -> Result<(), MyError>
    {
        for (_, memory) in &mut self.memories {
            let size = memory.regions.iter().map(|region| {
                region.borrow_mut().allocated = true;
                region.borrow().size
            }).sum::<u64>();

            let info = vk::MemoryAllocateInfo::builder()
                .allocation_size(size)
                .memory_type_index(memory.regions[0].borrow().index);

            memory.memory = device.get_device().allocate_memory(&info, None)?;
        }
        
        Ok(())
    }
    pub fn stage_image_binding(&mut self, image: &VkImage) {
        // self.images_to_bind.push(image);
        todo!()
    }
    pub unsafe fn bind_images(
        &mut self,
        device: &VkDevice,
    ) -> Result<(), MyError>
    {
        /* for image in self.images_to_bind {
            let region = image.memory_region.borrow();
            let memory = self.memories.get(&region.index).unwrap();

            device.get_device().bind_image_memory(image.image, memory.memory, region.begin)?;
        } */
        
        Ok(())
    }

    pub unsafe fn free_regions(&mut self, device: &VkDevice, regions: &[Rc<RefCell<VkMemoryRegion>>]) -> Result<(), MyError> {
        for region in regions {
            let index = region.borrow().index;

            if let Some(memory) = self.memories.get_mut(&index) {
                let mut borrow = region.borrow_mut();
                borrow.allocated = false;

                if let Some(index) = memory.regions.iter().position(|r| Rc::ptr_eq(r, region)) {
                    memory.regions.remove(index);

                    // Optionally deallocate the memory if it's no longer used by any regions
                    if memory.regions.is_empty() {
                        device.get_device().free_memory(memory.memory, None);
                    }
                }
            }
        }

        Ok(())
    }

    pub unsafe fn defragment(&mut self, device: &VkDevice) -> Result<(), MyError> {
        // TODO
        
        Ok(())
    }

    pub unsafe fn rearrange_memory(
        &mut self,
        device: &VkDevice,
    ) -> Result<(), MyError>
    {
        // TODO
        Ok(())
    }
}

pub unsafe fn get_memory_type_index(
    instance: &VkInstance,
    physical_device: &VkPhysicalDevice,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32, MyError>
{
    let memory = instance
        .get_instance()
        .get_physical_device_memory_properties(*physical_device.get_device());
    
    if let Some(result) = (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            
            suitable && memory_type.property_flags.contains(properties)
        })
    {
        return Ok(result);
    }

    Err("Failed to find suitable memory type!".into())
}