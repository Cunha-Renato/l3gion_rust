use super::{renderer::uniform::Uniform, uuid::UUID};
use nalgebra_glm as glm;

/// Angle in Radians
#[derive(Clone, Default)]
pub struct LgEntity {
    pub uniforms: Vec<Uniform>,
    pub mesh: UUID,
    pub material: UUID,

    uuid: UUID,
    position: glm::Vec3,
    scale: glm::Vec3,
    rotation_axis: glm::Vec3,
    rotation_angle: f32,
    
    model_matrix: glm::Mat4,
}
impl LgEntity {
    pub fn new(mesh: UUID, material: UUID, position: glm::Vec3) -> Self {
        let mut result = Self {
            uuid: UUID::generate(),
            uniforms: Vec::new(),
            mesh,
            material,
            position,
            scale: glm::vec3(1.0, 1.0, 1.0),
            rotation_axis: glm::vec3(0.0, 0.0, 0.0),
            rotation_angle: 0.0,
            model_matrix: glm::Mat4::identity(),
        };
        result.set_model();
        
        result
    }

    pub fn set_position(&mut self, new_pos: glm::Vec3) {
        self.position = new_pos;
        self.set_model();
    }
    pub fn set_scale(&mut self, new_scale: glm::Vec3) {
        self.scale = new_scale;
        self.set_model();
    }
    pub fn set_rotation_axis(&mut self, new_axis: glm::Vec3) {
        self.rotation_axis = new_axis;
        self.set_model();
    }
    pub fn set_rotation_angle(&mut self, new_angle: f32) {
        self.rotation_angle = new_angle;
        self.set_model();
    }

    pub fn position(&self) -> glm::Vec3 {
        self.position
    }
    pub fn scale(&self) -> glm::Vec3 {
        self.scale
    }
    pub fn rotation_axis(&self) -> glm::Vec3 {
        self.rotation_axis
    }
    pub fn rotation_angle(&self) -> f32 {
        self.rotation_angle
    }

    pub fn model(&self) -> glm::Mat4 {
        self.model_matrix
    }

    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
}
// Private
impl LgEntity {
    fn set_model(&mut self) {
        let identity = glm::Mat4::identity();
        let translation = glm::translate(&identity, &self.position);
        let rotation = glm::rotate(&identity, self.rotation_angle, &self.rotation_axis);
        let scale = glm::scale(&identity, &self.scale);

        self.model_matrix = translation * rotation * scale;
    }
}