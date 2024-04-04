use std::{borrow::Borrow, collections::HashMap, time::Instant};
use vulkanalia::vk::DeviceV1_0;

use crate::lg_core::{lg_types::reference::Ref, uuid::UUID};
use super::{object::Object, vulkan::{vk_device::VkDevice, vk_instance::VkInstance, vk_object::VkObject, vk_physical_device::VkPhysicalDevice}};

pub struct ObjectData<T> {
    pub object: Ref<VkObject<T>>,
    insertion_time: Instant,
}
pub struct ObjectStorage<T> {
    objects: HashMap<UUID, ObjectData<T>>,
    timer: Instant,
}
impl<T> ObjectStorage<T> {
    pub fn init() -> Self {
        Self {
            objects: HashMap::new(),
            timer: Instant::now(),
        }
    }
    pub unsafe fn insert(
        &mut self, 
        object: Ref<Object<T>>,
        device: &VkDevice,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
    )
    {
        let borrow = object.clone();        

        self.objects.entry(borrow.borrow().uuid())
            .and_modify(|od| od.insertion_time = Instant::now())
            .or_insert_with(|| {
                let vk_object = VkObject::new(
                    device, 
                    instance, 
                    physical_device, 
                    object
                ).unwrap();
                
                ObjectData {
                    object: Ref::new(vk_object),
                    insertion_time: Instant::now()
                }
            });
    }
    pub fn get_objects(&self) -> &HashMap<UUID, ObjectData<T>> {
        &self.objects
    }
    pub unsafe fn destroy_inactive_objects(&mut self, device: &VkDevice) {
        let elapsed_time = self.timer.elapsed().as_secs();

        if elapsed_time >= 5 {
            let mut objects_to_remove = Vec::new();

            for (uuid, obj_data) in &self.objects {
                if obj_data.insertion_time.elapsed().as_secs() >= 300 {
                    objects_to_remove.push(uuid.clone());
                }
            }
            for uuid in objects_to_remove {
                self.remove(&uuid, device);
            }
            
            self.timer = Instant::now();
        }
    }
    unsafe fn destroy_buffers(&self, uuid: &UUID, device: &VkDevice) {
        let object = self.objects.get(uuid).unwrap().object.borrow();
        
        // Free GPU resources
        // Clearing Vertices
        device.get_device().destroy_buffer(object.vertex_buffer.as_ref().unwrap().buffer, None);
        device.get_device().free_memory(object.vertex_buffer.as_ref().unwrap().memory, None);
        
        // Clearing Indices
        device.get_device().destroy_buffer(object.index_buffer.as_ref().unwrap().buffer, None);
        device.get_device().free_memory(object.index_buffer.as_ref().unwrap().memory, None);

    }
    unsafe fn remove(&mut self, uuid: &UUID, device: &VkDevice) {
        self.destroy_buffers(uuid, device);
        self.objects.remove(uuid);
    }
    pub unsafe fn destroy(&mut self, device: &VkDevice) {
        for (uuid, _) in &self.objects {
            self.destroy_buffers(uuid, device);
        }
        self.objects.clear();
    }
}