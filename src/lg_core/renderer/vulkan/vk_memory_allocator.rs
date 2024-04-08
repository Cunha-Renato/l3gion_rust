#![allow(non_camel_case_types)]

use std::ffi::c_void;
use vulkanalia::{
    prelude::v1_2::*, vk
};
use crate::{lg_core::lg_types::reference::Rfc, MyError};
use super::{vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice};

const SIZE_OF_MEGABYTES: u64 = 1_000_000;
const MEMORY_BLOCK_SIZE: u64 = 256 * SIZE_OF_MEGABYTES;

#[derive(Debug, Clone, Copy)]
pub enum VkMemoryUsageFlags {
    GPU,
    CPU,
    GPU_CPU,
    CPU_GPU, 
}
impl VkMemoryUsageFlags {
    fn map_vulkan(&self) -> vk::MemoryPropertyFlags {
        match self {
            VkMemoryUsageFlags::GPU => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            VkMemoryUsageFlags::CPU => vk::MemoryPropertyFlags::HOST_VISIBLE,
            VkMemoryUsageFlags::GPU_CPU => {
                vk::MemoryPropertyFlags::DEVICE_LOCAL
                    | vk::MemoryPropertyFlags::HOST_VISIBLE
            },
            VkMemoryUsageFlags::CPU_GPU => {
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_CACHED
            },
        }
    }
}

#[derive(Default, Debug)]
pub struct VkMemoryRegion {
    begin: u64,
    size: u64,
    memory_index: usize,
}

struct VkMemory {
    memory: vk::DeviceMemory,
    size: u64,
}

#[derive(Default)]
struct VkMemoryHeap {
    memory_index: u32,
    image_regions: Vec<Rfc<VkMemoryRegion>>,
    buffer_regions: Vec<Rfc<VkMemoryRegion>>,
    image_memory: Option<VkMemory>,
    buffer_memory: Option<VkMemory>,
}

pub struct VkMemoryManager {
    device: Rfc<VkDevice>,
    heaps: Vec<VkMemoryHeap>,
}
impl VkMemoryManager {
    pub unsafe fn new(
        device: Rfc<VkDevice>,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
    ) -> Result<Self, MyError>
    {
        let heap_indices = vec![
            get_memory_type_index(instance, physical_device, VkMemoryUsageFlags::GPU)?,
            get_memory_type_index(instance, physical_device, VkMemoryUsageFlags::CPU)?,
            get_memory_type_index(instance, physical_device, VkMemoryUsageFlags::GPU_CPU)?,
            get_memory_type_index(instance, physical_device, VkMemoryUsageFlags::CPU_GPU)?,
        ];
        
        let heaps = heap_indices
            .iter()
            .map(|i| VkMemoryHeap {
                memory_index: *i,
                image_regions: Vec::new(),
                buffer_regions: Vec::new(),
                image_memory: None,
                buffer_memory: None,
            })
            .collect();
            
        Ok(Self {
            device,
            heaps,
        })
    }

    // Image
    pub unsafe fn alloc_image(
        &mut self,
        image: &vk::Image,
        memory_usage: VkMemoryUsageFlags,
    ) -> Result<Rfc<VkMemoryRegion>, MyError>
    {
        let heap = &mut self.heaps[memory_usage as usize];
        let requirements = self.device.borrow().get_device().get_image_memory_requirements(*image);
        
        let begin = match &heap.image_memory {
            Some(mem) => Self::find_best_fit(mem.size, requirements.size, &heap.image_regions)?,
            None => {
                let info = vk::MemoryAllocateInfo::builder()
                    .allocation_size(MEMORY_BLOCK_SIZE)
                    .memory_type_index(heap.memory_index);
                
                let img_memory = VkMemory {
                    memory: self.device.borrow().get_device().allocate_memory(&info, None)?,
                    size: MEMORY_BLOCK_SIZE,
                };
                
                heap.image_memory = Some(img_memory);
                
                0
            },
        };
        
        let size = Self::with_alignment(requirements.size, requirements.alignment);

        let region = Rfc::new(VkMemoryRegion {
            begin,
            size,
            memory_index: memory_usage as usize,
        });
        
        heap.image_regions.push(region.clone());
        heap.image_regions.sort_by(|r1, r2| r1
            .borrow().begin
            .partial_cmp(&r2.borrow().begin).unwrap()
        );

        Ok(region.clone())
    }
    pub unsafe fn alloc_buffer(
        &mut self,
        buffer: &vk::Buffer,
        memory_usage: VkMemoryUsageFlags        
    ) -> Result<Rfc<VkMemoryRegion>, MyError> {
        let heap = &mut self.heaps[memory_usage as usize];
        let requirements = self.device.borrow().get_device().get_buffer_memory_requirements(*buffer);
        
        let begin = match &heap.buffer_memory {
            Some(mem) => Self::find_best_fit(mem.size, requirements.size, &heap.buffer_regions)?,
            None => {
                let info = vk::MemoryAllocateInfo::builder()
                    .allocation_size(MEMORY_BLOCK_SIZE)
                    .memory_type_index(heap.memory_index);
                
                let buff_memory = VkMemory {
                    memory: self.device.borrow().get_device().allocate_memory(&info, None)?,
                    size: MEMORY_BLOCK_SIZE,
                };
                
                heap.buffer_memory = Some(buff_memory);
                
                0
            },
        };
        
        let size = Self::with_alignment(requirements.size, requirements.alignment);

        let region = Rfc::new(VkMemoryRegion {
            begin,
            size,
            memory_index: memory_usage as usize,
        });
        
        heap.buffer_regions.push(region.clone());
        heap.buffer_regions.sort_by(|r1, r2| r1
            .borrow().begin
            .partial_cmp(&r2.borrow().begin).unwrap()
        );

        Ok(region.clone())
    }
    pub unsafe fn bind_image(
        &self,
        image: &vk::Image,
        region: Rfc<VkMemoryRegion>
    ) -> Result<(), MyError>
    {
        let memory = self.heaps[region.borrow().memory_index].image_memory.as_ref().unwrap();
        self.device.borrow().get_device().bind_image_memory(*image, memory.memory, region.borrow().begin)?;
        
        Ok(())
    }
    pub unsafe fn bind_buffer(
        &self,
        buffer: &vk::Buffer,
        region: Rfc<VkMemoryRegion>
    ) -> Result<(), MyError>
    {
        let memory = self.heaps[region.borrow().memory_index].buffer_memory.as_ref().unwrap();
        self.device.borrow().get_device().bind_buffer_memory(*buffer, memory.memory, region.borrow().begin)?;
        
        Ok(())
    }
    
    pub unsafe fn map_image(&mut self, region: Rfc<VkMemoryRegion>, flags: vk::MemoryMapFlags) -> Result<(), MyError> {
        if let Some(memory) = &self.heaps[region.borrow().memory_index].image_memory {
            self.device.borrow().get_device().map_memory(
                memory.memory, 
                region.borrow().begin, 
                region.borrow().size, 
                flags
            )?;
        }
        else { 
            return Err("Trying to map an image that wasn't created (VkMemoryManager)".into()); 
        }

        Ok(())
    }
    pub unsafe fn map_buffer(&mut self, region: Rfc<VkMemoryRegion>, offset: u64, size: u64, flags: vk::MemoryMapFlags) -> Result<*mut c_void,  MyError> {
        if let Some(memory) = &self.heaps[region.borrow().memory_index].buffer_memory {
            return Ok(self.device.borrow().get_device().map_memory(
                memory.memory, 
                region.borrow().begin + offset, 
                size,
                flags
            )?);
        }
        else { 
            return Err("Trying to map a buffer that wasn't created (VkMemoryManager)".into()); 
        }
    }

    pub unsafe fn unmap_image(&mut self, region: Rfc<VkMemoryRegion>) -> Result<(), MyError> {
        if let Some(memory) = &self.heaps[region.borrow().memory_index].image_memory {
            self.device.borrow().get_device().unmap_memory(memory.memory);
        }
        else {
            return Err(("Trying to unmap image memory that doesn't exists (VkMemoryManager)").into());
        }
        
        Ok(())
    }
    pub unsafe fn unmap_buffer(&mut self, region: Rfc<VkMemoryRegion>) -> Result<(), MyError> {
        if let Some(memory) = &self.heaps[region.borrow().memory_index].buffer_memory {
            self.device.borrow().get_device().unmap_memory(memory.memory);
        }
        else {
            return Err(("Trying to unmap buffer memory that doesn't exists (VkMemoryManager)").into());
        }
        
        Ok(())
    }

    pub unsafe fn free_image_region(&mut self, region: Rfc<VkMemoryRegion>) -> Result<(), MyError>{
        let regions = &mut self.heaps[region.borrow().memory_index].image_regions;
        
        if let Some((index, _)) = regions
            .iter()
            .enumerate()
            .find(|(_, rg)| rg.borrow().begin == region.borrow().begin)
        {
            regions.remove(index);
            *region.borrow_mut() = VkMemoryRegion::default();

            return Ok(());
        }
        
        Err("Trying to free memory that doesn't exists (VkMemoryManager)".into())
    }
    pub unsafe fn free_buffer_region(&mut self, region: Rfc<VkMemoryRegion>) -> Result<(), MyError> {
        let regions = &mut self.heaps[region.borrow().memory_index].buffer_regions;
        
        if let Some((index, _)) = regions
            .iter()
            .enumerate()
            .find(|(_, rg)| rg.borrow().begin == region.borrow().begin)
        {
            regions.remove(index);
            *region.borrow_mut() = VkMemoryRegion::default();

            return Ok(());
        }
        
        Err("Trying to free memory that doesn't exists (VkMemoryManager)".into())
    }
    pub unsafe fn destroy(&mut self, device: &VkDevice) {
        let device = device.get_device();
        
        self.heaps
            .iter()
            .for_each(|h| {
                if let Some(img_mem) = &h.image_memory { device.free_memory(img_mem.memory, None); }
                if let Some(buff_mem) = &h.buffer_memory { device.free_memory(buff_mem.memory, None); }
            })
    }
    
    // Return the begin for the new region, based on best fit
    fn find_best_fit(heap_size: u64, required_size: u64, regions: &Vec<Rfc<VkMemoryRegion>>) -> Result<u64, MyError> {
        let mut best = 0;
        let mut found = false;
        
        if !regions.is_empty() {
            for i in 0..regions.len() {
                if i + 1 < regions.len() {
                    let r1 = regions[i].borrow();
                    let r2 = regions[i+1].borrow();

                    // If the available region in memory is big enough
                    let end = r1.begin + r1.size;
                    if r2.begin - end > required_size {
                        if r2.begin - end < best {
                            best = end;
                            found = true;
                        } 
                        else if !found {
                            best = end;
                            found = true;
                        }
                    }
                }
            }
            
            if let Some(region) = regions.last() {
                let end = region.borrow().begin + region.borrow().size;

                if heap_size - end > required_size && (!found || heap_size - end < best) {
                    best = end;
                    found = true;
                }
            }
            if let Some(region) = regions.first() {
                let begin = region.borrow().begin;
                if begin > 0 && begin > required_size && !found {
                    found = true;
                }
            }
        }
        else {
            return Ok(best);
        }
            
        match found {
            true => Ok(best),
            false => Err("Memory out of space (VkMemoryManager)!".into())
        }
    }
    
    fn with_alignment(original: u64, alignment: u64) -> u64 {
        let mut result = 0;
        
        while result < original {
            result += alignment;
        }
        
        result
    }
}

unsafe fn get_memory_type_index(
    instance: &VkInstance,
    physical_device: &VkPhysicalDevice,
    properties: VkMemoryUsageFlags,
) -> Result<u32, MyError>
{
    let properties = properties.map_vulkan();
    let memory = instance
        .get_instance()
        .get_physical_device_memory_properties(*physical_device.get_device());
    
    if let Some(result) = memory.memory_types
        .iter()
        .enumerate()
        .find(|(_, mt)| {
            mt.property_flags.contains(properties)
        })
        .map(|(i, _)| i as u32)
    {
        return Ok(result);
    }

    Err("Failed to find suitable memory type!".into())
}