use std::{any::Any, rc::Rc};

use crate::StdError;

#[derive(Clone, Debug)]
pub enum LgUniformType {
    STRUCT,
    STORAGE_BUFFER,
    COMBINED_IMAGE_SAMPLER
}

#[derive(Clone)]
pub struct LgUniform {
    name: String,
    u_type: LgUniformType,
    binding: usize,
    set: usize,
    pub data: Rc<dyn GlUniform>,
}
impl LgUniform {
    pub fn new<T: 'static + GlUniform>(
        name: &str,
        u_type: LgUniformType, 
        binding: usize,
        set: usize,
        data: T
    ) -> Self 
    {
        // let data = as_dyn!(data, dyn GlUniform);
        let data = Rc::new(data) as Rc<dyn GlUniform>;
        Self {
            name: String::from(name),
            u_type,
            binding,
            set,
            data,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn u_type(&self) -> LgUniformType {
        self.u_type.clone()
    }
    pub fn binding(&self) -> usize {
        self.binding
    }
    pub fn set(&self) -> usize {
        self.set
    }
    pub fn data(&self) -> *const std::ffi::c_void {
        self.data.as_c_void()
    }
}
pub trait GlUniform: 'static
{
    fn size(&self) -> usize;
    fn as_c_void(&self) -> *const std::ffi::c_void {
        let ptr = self as *const Self;
        
        ptr as *const std::ffi::c_void
    }
    fn as_any(&self) -> &dyn Any;
}