use nalgebra_glm as glm;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UniformBufferObject {
    model: glm::Mat4,
    view: glm::Mat4,
    proj: glm::Mat4,
}