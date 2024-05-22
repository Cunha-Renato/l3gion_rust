use super::{renderer::uniform::Uniform, uuid::UUID};

pub struct LgEntity {
    uuid: UUID,
    pub uniforms: Vec<Uniform>,
    pub mesh: String,
    pub material: String,
}
impl LgEntity {
    pub fn new(mesh: &str, material: &str) -> Self {
        Self {
            uuid: UUID::generate(),
            uniforms: Vec::new(),
            mesh: String::from(mesh),
            material: String::from(material),
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
}