use super::{renderer::uniform::Uniform, uuid::UUID};
use nalgebra_glm as glm;

pub struct LgEntity {
    uuid: UUID,
    pub uniforms: Vec<Uniform>,
    pub mesh: UUID,
    pub material: UUID,
    pub position: glm::Vec3,
    pub scale: glm::Vec3,
    pub rotation_axis: glm::Vec3,
    /// Radians
    pub rotation_angle: f32,
}
impl LgEntity {
    pub fn new(mesh: UUID, material: UUID, position: glm::Vec3) -> Self {
        Self {
            uuid: UUID::generate(),
            uniforms: Vec::new(),
            mesh,
            material,
            position,
            scale: glm::vec3(1.0, 1.0, 1.0),
            rotation_axis: glm::vec3(0.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
}