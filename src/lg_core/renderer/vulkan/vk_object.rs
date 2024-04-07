use vulkanalia::vk::DeviceV1_0;

use crate::{lg_core::{lg_types::reference::Rfc, renderer::object::Object}, MyError};

use super::{index_buffer::VkIndexBuffer, vertex_buffer::VkVertexBuffer, vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice, vk_texture::VkTexture};

pub struct VkObject<T> {
    pub object: Rfc<Object<T>>,
    pub vk_texture: Option<VkTexture>,
    pub vertex_buffer: Option<VkVertexBuffer>,
    pub index_buffer: Option<VkIndexBuffer>,
}
impl<T: Clone> VkObject<T> {
    pub unsafe fn new(
        device: &VkDevice,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        object: Rfc<Object<T>>,
    ) -> Result<Self, MyError>
    {
        let borrow = object.clone();


        let vertex_buffer = Some(VkVertexBuffer::new(
            instance, 
            device, 
            physical_device, 
            &borrow.borrow().vertices, 
            borrow.borrow().vertex_size(),
        )?);
        
        let index_buffer = Some(VkIndexBuffer::new(
            instance, 
            device, 
            physical_device, 
            &borrow.borrow().indices, 
            borrow.borrow().index_size(),
        )?);
        
        let vk_texture = Some(VkTexture::new(
            instance, 
            device, 
            physical_device, 
            &borrow.borrow().texture.borrow()
        )?);
        
        Ok(Self {
            object,
            vk_texture,
            vertex_buffer,
            index_buffer,
        })
    }
    pub unsafe fn destroy(&mut self, device: &VkDevice) {
        // Free GPU resources
        // Clearing Vertices
        device.get_device().destroy_buffer(self.vertex_buffer.as_ref().unwrap().buffer, None);
        device.get_device().free_memory(self.vertex_buffer.as_ref().unwrap().memory, None);
        
        // Clearing Indices
        device.get_device().destroy_buffer(self.index_buffer.as_ref().unwrap().buffer, None);
        device.get_device().free_memory(self.index_buffer.as_ref().unwrap().memory, None);
        
        self.vk_texture.as_mut().unwrap().destroy(device);
    }
}