use std::mem::size_of;
use lg_renderer::renderer::lg_uniform::GlUniform;
use nalgebra_glm as glm;

#[repr(C)]
struct UBO {
    pub data: glm::Vec4,
}
impl GlUniform for UBO {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SSBO {
    pub data: glm::Vec4,
}
impl GlUniform for SSBO {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Data {
    pub mouse_position: glm::Vec2,    
    pub uuid: u32,
}
impl GlUniform for Data {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}