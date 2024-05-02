use crate::{as_dyn, lg_core::lg_types::reference::Rfc};

#[derive(Clone)]
pub enum LgUniformType {
    STRUCT,
    COMBINED_IMAGE_SAMPLER
}

#[derive(Clone)]
pub struct LgUniform {
    name: String,
    u_type: LgUniformType,
    binding: u32,
    set: u32,
    pub data: Rfc<dyn GlUniform>,
}
impl LgUniform {
    pub fn new<T: 'static + GlUniform>(
        name: String,
        u_type: LgUniformType, 
        binding: u32,
        set: u32,
        data: T
    ) -> Self 
    {
        let data = as_dyn!(data, dyn GlUniform);
        Self {
            name,
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
        self.u_type
    }
    pub fn binding(&self) -> u32 {
        self.binding
    }
    pub fn set(&self) -> u32 {
        self.set
    }
    pub fn data(&self) -> *const std::ffi::c_void {
        self.data.borrow().as_c_void()
    }
}
pub trait GlUniform {
    fn as_c_void(&self) -> *const std::ffi::c_void {
        let ptr = self as *const Self;
        
        ptr as *const std::ffi::c_void
    }
}