use crate::{lg_core::{lg_types::reference::Rfc, renderer::object::Object}, StdError};

use super::{vk_buffer::VkBuffer, vk_memory_manager::VkMemoryManager, vk_texture::VkTexture};

pub struct VkObject<T> {
    pub object: Rfc<Object<T>>,
    pub vk_texture: Option<Rfc<VkTexture>>,
    pub vertex_buffer: Option<Rfc<VkBuffer>>,
    pub index_buffer: Option<Rfc<VkBuffer>>,
}
impl<T: Clone> VkObject<T> {
    pub unsafe fn destroy(&mut self, memory_manager: &mut VkMemoryManager) -> Result<(), StdError>{
        // Free GPU resources
        // Clearing Vertices
        memory_manager.destroy_buffer(self.vertex_buffer.as_ref().unwrap().clone())?;
        
        // Clearing Indices
        memory_manager.destroy_buffer(self.index_buffer.as_ref().unwrap().clone())?;
        
        Ok(())
    }
}