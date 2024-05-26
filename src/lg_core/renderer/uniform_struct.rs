use nalgebra_glm as glm;

#[repr(C)]
pub struct UBO {
    pub transform: glm::Mat4,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SSBO {
    pub data: glm::UVec4,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Data {
    pub mouse_position: glm::Vec2,    
    pub uuid: u32,
}