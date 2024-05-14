use std::mem::size_of;
use lg_renderer::renderer::lg_uniform::GlUniform;
use nalgebra_glm as glm;

macro_rules! impl_gluniform {
    ($struct_name:ident) => {
        impl GlUniform for $struct_name {
            fn size(&self) -> usize {
                std::mem::size_of::<Self>()
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

#[repr(C)]
pub struct UBO {
    pub transform: glm::Mat4,
}
impl_gluniform!(UBO);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SSBO {
    pub data: glm::UVec4,
}
impl_gluniform!(SSBO);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Data {
    pub mouse_position: glm::Vec2,    
    pub uuid: u32,
}
impl_gluniform!(Data);