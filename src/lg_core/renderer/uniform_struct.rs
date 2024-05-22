use lg_renderer::impl_lg_buffer_data;
use nalgebra_glm as glm;

#[repr(C)]
pub struct UBO {
    pub transform: glm::Mat4,
}
impl_lg_buffer_data!(UBO);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SSBO {
    pub data: glm::UVec4,
}
impl_lg_buffer_data!(SSBO);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Data {
    pub mouse_position: glm::Vec2,    
    pub uuid: u32,
}
impl_lg_buffer_data!(Data);