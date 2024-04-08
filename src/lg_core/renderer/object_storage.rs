use std::{collections::HashMap, time::Instant};
use crate::{lg_core::{lg_types::reference::Rfc, uuid::UUID}, MyError};
use super::{object::Object, vulkan::{vk_device::VkDevice, vk_instance::VkInstance, vk_memory_allocator::VkMemoryManager, vk_object::VkObject, vk_physical_device::VkPhysicalDevice}};

pub struct ObjectData<T> {
    pub object: Rfc<VkObject<T>>,
    insertion_time: Instant,
}
pub struct ObjectStorage<T> {
    device: Rfc<VkDevice>,
    memory_manager: Rfc<VkMemoryManager>,
    objects: HashMap<UUID, ObjectData<T>>,
    timer: Instant,
}
impl<T: Clone> ObjectStorage<T> {
    pub fn new(
        device: Rfc<VkDevice>,
        memory_manager: Rfc<VkMemoryManager>,
    ) -> Self {
        Self {
            device,
            memory_manager,
            objects: HashMap::new(),
            timer: Instant::now(),
        }
    }
    pub unsafe fn insert(
        &mut self, 
        object: Rfc<Object<T>>,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
    )
    {
        let borrow = object.clone();        

        self.objects.entry(borrow.borrow().uuid())
            .and_modify(|od| od.insertion_time = Instant::now())
            .or_insert_with(|| {
                let vk_object = VkObject::new(
                    &self.device.borrow(), 
                    instance, 
                    physical_device, 
                    &mut self.memory_manager.borrow_mut(),
                    object
                ).unwrap();
                
                ObjectData {
                    object: Rfc::new(vk_object),
                    insertion_time: Instant::now()
                }
            });
    }
    pub fn get_objects(&self) -> &HashMap<UUID, ObjectData<T>> {
        &self.objects
    }
    pub unsafe fn destroy_inactive_objects(&mut self) -> Result<(), MyError>{
        let elapsed_time = self.timer.elapsed().as_secs();

        if elapsed_time >= 5 {
            let mut objects_to_remove = Vec::new();

            for (uuid, obj_data) in &self.objects {
                if obj_data.insertion_time.elapsed().as_secs() >= 300 {
                    objects_to_remove.push(uuid.clone());
                }
            }
            for uuid in objects_to_remove {
                self.remove(&uuid)?;
            }
            
            self.timer = Instant::now();
        }
        
        Ok(())
    }
    unsafe fn destroy_resources(&self, uuid: &UUID) -> Result<(), MyError>{
        self.objects.get(uuid).unwrap().object.borrow_mut().destroy(&self.device.borrow(), &mut self.memory_manager.borrow_mut())?;
        Ok(())
    }
    unsafe fn remove(&mut self, uuid: &UUID) -> Result<(), MyError> {
        self.destroy_resources(uuid)?;
        self.objects.remove(uuid);
        
        Ok(())
    }
    pub unsafe fn destroy(&mut self) -> Result<(), MyError>{
        for (uuid, _) in &self.objects {
            self.destroy_resources(uuid)?;
        }
        self.objects.clear();
        
        Ok(())
    }
}