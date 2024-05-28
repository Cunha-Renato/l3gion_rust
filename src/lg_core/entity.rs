use super::{renderer::uniform::Uniform, uuid::UUID};

pub struct LgEntity {
    uuid: UUID,
    pub uniforms: Vec<Uniform>,
    pub mesh: UUID,
    pub material: UUID,
}
impl LgEntity {
    pub fn new(mesh: UUID, material: UUID) -> Self {
        Self {
            uuid: UUID::generate(),
            uniforms: Vec::new(),
            mesh,
            material,
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
}