use crate::StdError;

use super::{
    application::ApplicationCore, 
    event::LgEvent, 
    lg_types::reference::Rfc, 
    renderer::{
        material::LgMaterial, mesh::LgMesh, shader::LgShader, texture::LgTexture, vertex::Vertex, Renderer
    }
};
use nalgebra_glm as glm;

struct TexStorage {
    grid: Rfc<LgTexture>,
    viking: Rfc<LgTexture>,
}
impl TexStorage {
    fn new() -> Result<Self, StdError> {
        Ok(Self {
            grid: Rfc::new(LgTexture::new("resources/textures/grid.png")?),
            viking: Rfc::new(LgTexture::new("resources/textures/viking.png")?),
        })
    }
}

pub struct TestScene {
    app_core: Rfc<ApplicationCore>,
    triangle: LgMesh<Vertex>,
    material: LgMaterial,

    _tex_storage: TexStorage,
}
impl TestScene {
    pub fn new(core: Rfc<ApplicationCore>) -> Self {
        let tex_storage = TexStorage::new().unwrap();

        let triangle = LgMesh::new(
            vec![
                Vertex { position: glm::vec2(-0.5, -0.5), tex_coord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec2( 0.5, -0.5), tex_coord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec2( 0.5,  0.5), tex_coord: glm::vec2(1.0, 0.0) },
                Vertex { position: glm::vec2(-0.5,  0.5), tex_coord: glm::vec2(0.0, 0.0) },
            ], 
            vec![
                0, 1, 2,
                2, 3, 0
            ]
        );
        let grid = LgMaterial::new(vec![
            Rfc::new(LgShader::builder()
                .stage(super::renderer::shader::ShaderStage::VERTEX)
                .src_code(std::path::Path::new("resources/shaders/src/std_v.vert")).unwrap()
                .build()
            ),
            Rfc::new(LgShader::builder()
                .stage(super::renderer::shader::ShaderStage::FRAGMENT)
                .src_code(std::path::Path::new("resources/shaders/src/std_f.frag")).unwrap()
                .build()
            )],
            tex_storage.grid.clone()
        );

        Self {
            triangle,
            material: grid,
            app_core: core,
            _tex_storage: tex_storage,
        } 
    }
    pub fn init(&mut self) {
        
    }
    pub fn on_update(&mut self) {
        unsafe {
            self.app_core.borrow_mut().renderer.borrow_mut().draw(
                &self.triangle,
                &self.material
            ).unwrap();
        }
    }
    pub fn on_event(&mut self, event: &LgEvent) {
        
    }
    pub fn destroy(&mut self) {

    }
}