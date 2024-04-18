use std::{collections::HashMap, time::Instant};
use crate::{lg_core::{lg_types::reference::Rfc, uuid::UUID}, StdError};
use super::{object::Object, vulkan::{vk_device::VkDevice, vk_instance::VkInstance, vk_memory_manager::VkMemoryManager, vk_object::VkObject, vk_physical_device::VkPhysicalDevice, vk_texture::VkTexture}};

pub struct ObjectData<T> {
    pub object: Rfc<VkObject<T>>,
    insertion_time: Instant,
}
pub struct ObjectStorage<T> {
    device: Rfc<VkDevice>,
    memory_manager: Rfc<VkMemoryManager>,
    objects: HashMap<UUID, ObjectData<T>>,
    textures: HashMap<UUID, Rfc<VkTexture>>,
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
            textures: HashMap::new(),
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
        let mut mem_manager = &mut self.memory_manager.borrow_mut();

        self.objects.entry(borrow.borrow().uuid())
            .and_modify(|od| od.insertion_time = Instant::now())
            .or_insert_with(|| {
                let vertex_buffer = Some(mem_manager.new_vertex_buffer(
                    &borrow.borrow().vertices, 
                    borrow.borrow().vertex_size(),
                ).unwrap());
        
                let index_buffer = Some(mem_manager.new_index_buffer(
                    &borrow.borrow().indices, 
                    borrow.borrow().index_size(),
                ).unwrap());
                
                let vk_texture = match self.textures.get(borrow.borrow().texture.borrow().uuid()) {
                    Some(vk_texture) => Some(vk_texture.clone()),
                    None => {
                        let texture = Rfc::new(VkTexture::new(
                            instance, 
                            &self.device.borrow(), 
                            physical_device, 
                            &mut mem_manager,
                            borrow.borrow().texture.clone()
                        ).unwrap());

                        self.textures.insert(borrow.borrow().texture.borrow().uuid().clone(), texture.clone());
                        
                        Some(texture.clone())
                    }
                };

                let vk_object = VkObject{
                    vertex_buffer,
                    index_buffer,
                    vk_texture,
                    object,                    
                };
                
                ObjectData {
                    object: Rfc::new(vk_object),
                    insertion_time: Instant::now()
                }
            });
    }
    pub fn get_objects(&self) -> &HashMap<UUID, ObjectData<T>> {
        &self.objects
    }
    pub unsafe fn destroy_inactive_objects(&mut self) -> Result<(), StdError>{
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
    unsafe fn destroy_resources(&self, uuid: &UUID) -> Result<(), StdError>{
        self.objects.get(uuid).unwrap().object.borrow_mut().destroy(&mut self.memory_manager.borrow_mut())?;

        Ok(())
    }
    unsafe fn remove(&mut self, uuid: &UUID) -> Result<(), StdError> {
        self.destroy_resources(uuid)?;
        self.objects.remove(uuid);
        
        Ok(())
    }
    pub unsafe fn destroy(&mut self) -> Result<(), StdError>{
        for (uuid, _) in &self.objects {
            self.destroy_resources(uuid)?;
        }
        self.objects.clear();
        self.textures
            .iter()
            .for_each(|(_, t)| t.borrow_mut().destroy(&self.device.borrow(), &mut self.memory_manager.borrow_mut()).unwrap());
        
        Ok(())
    }
}