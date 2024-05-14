use lg_renderer::renderer::lg_uniform::LgUniform;

use super::uuid::UUID;

pub struct LgEntity {
    uuid: UUID,
    pub uniforms: Vec<LgUniform>,
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