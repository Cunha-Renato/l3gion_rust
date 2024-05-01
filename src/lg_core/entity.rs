use crate::StdError;

use super::{lg_types::reference::Rfc, renderer::{material::LgMaterial, mesh::LgMesh}, uuid::UUID};

pub struct LgEntity {
    uuid: UUID,
    mesh: Rfc<LgMesh>,
    material: Rfc<LgMaterial>,
}
impl LgEntity {
    pub fn new(mesh: Rfc<LgMesh>, material: Rfc<LgMaterial>) -> Result<Self, StdError> {
        Ok(Self {
            uuid: UUID::generate(),
            mesh,
            material,
        })
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid        
    }
    pub fn mesh(&self) -> &Rfc<LgMesh> {
        &self.mesh
    }
    pub fn material(&self) -> &Rfc<LgMaterial> {
        &self.material
    }
}