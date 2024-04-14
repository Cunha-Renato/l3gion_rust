use nalgebra_glm as glm;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ViewProjUBO {
    pub proj: glm::Mat4,
    pub view: glm::Mat4,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelUBO {
    pub data: glm::Mat4,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelUBOId {
    pub data: glm::Mat4,
    pub id: glm::UVec4,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StorageBuffer {
    pub data: u32,
}