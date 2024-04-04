use crate::{lg_core::{lg_types::reference::Ref, renderer::object::Object}, MyError};

use super::{index_buffer::VkIndexBuffer, vertex_buffer::VkVertexBuffer, vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice};

pub struct VkObject<T> {
    pub object: Ref<Object<T>>,
    pub vertex_buffer: Option<VkVertexBuffer>,
    pub index_buffer: Option<VkIndexBuffer>,
}
impl<T> VkObject<T> {
    pub unsafe fn new(
        device: &VkDevice,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        object: Ref<Object<T>>,
    ) -> Result<Self, MyError>
    {
        let borrow = object.clone();


        let vertex_buffer = Some(VkVertexBuffer::new(
            instance, 
            device, 
            physical_device, 
            borrow.borrow().vertices(), 
            borrow.borrow().vertex_size(),
        )?);
        
        let index_buffer = Some(VkIndexBuffer::new(
            instance, 
            device, 
            physical_device, 
            borrow.borrow().indices(), 
            borrow.borrow().index_size(),
        )?);
        
        Ok(Self {
            object,
            vertex_buffer,
            index_buffer,
        })
    }
}