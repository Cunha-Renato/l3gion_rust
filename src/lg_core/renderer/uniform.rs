use crate::{as_dyn, lg_core::lg_types::reference::Rfc};

pub enum LgUniformType {
    STRUCT,
}

pub struct LgUniform {
    u_type: LgUniformType, 
    pub data: Rfc<dyn GlUniform>,
}
impl LgUniform {
    pub fn new<T: 'static + GlUniform>(u_type: LgUniformType, data: T) -> Self {
        let data = as_dyn!(data, dyn GlUniform);
        Self {
            u_type,
            data,
        }
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