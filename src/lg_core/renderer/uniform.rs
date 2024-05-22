use std::ops::Deref;

use lg_renderer::renderer::lg_buffer::*;
use crate::lg_core::lg_types::reference::Rfc;
use super::buffer::Buffer;

#[derive(Clone)]
pub struct Uniform {
    name: String,
    u_type: lg_renderer::renderer::lg_uniform::LgUniformType,
    binding: usize,
    set: usize,
    update_data: bool,
    pub buffer: Rfc<Buffer>,
}
impl Uniform {
    pub fn new(
        name: &str,
        u_type: lg_renderer::renderer::lg_uniform::LgUniformType,
        binding: usize,
        set: usize,
        update_data: bool,
        buffer: Rfc<Buffer>
    ) -> Self 
    {
        Self {
            name: name.to_string(),
            u_type,
            binding,
            set,
            update_data,
            buffer
        }
    }
    pub fn new_with_data(
        name: &str,
        u_type: lg_renderer::renderer::lg_uniform::LgUniformType,
        binding: usize,
        set: usize,
        update_data: bool,
        data: impl LgBufferData
    ) -> Self 
    {
        Self {
            name: name.to_string(),
            u_type,
            binding,
            set, 
            update_data,
            buffer: Rfc::new(Buffer::new(data))
        }
    }
}
impl lg_renderer::renderer::lg_uniform::LgUniform for Uniform {
    fn name(&self) -> &str {
        &self.name
    }

    fn u_type(&self) -> lg_renderer::renderer::lg_uniform::LgUniformType {
        self.u_type
    }

    fn binding(&self) -> usize {
        self.binding
    }

    fn set(&self) -> usize {
        self.set
    }

    fn update_data(&self) -> bool {
        self.update_data
    }
    
    fn data_size(&self) -> usize {
        self.buffer.borrow().data_size()
    }
    
    fn get_raw_data(&self) -> *const std::ffi::c_void {
        self.buffer.borrow().get_raw_data()
    }
    
    fn set_data(&mut self, data: impl LgBufferData) {
        self.buffer.borrow_mut().set_data(data)
    }
}