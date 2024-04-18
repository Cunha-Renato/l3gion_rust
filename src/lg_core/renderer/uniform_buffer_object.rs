use nalgebra_glm as glm;

pub trait Descriptor {}

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

impl Descriptor for ViewProjUBO {}
impl Descriptor for ModelUBO {}
impl Descriptor for ModelUBOId {}
impl Descriptor for StorageBuffer {}