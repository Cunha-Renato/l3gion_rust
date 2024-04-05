use nalgebra_glm as glm;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UniformBufferObject {
    pub model: glm::Mat4,
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
}
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