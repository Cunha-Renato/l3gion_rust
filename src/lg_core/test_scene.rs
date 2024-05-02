use crate::StdError;

use super::{
    application::ApplicationCore, entity::LgEntity, event::LgEvent, lg_types::reference::Rfc, renderer::{
        material::LgMaterial, mesh::LgMesh, shader::LgShader, texture::LgTexture, vertex::Vertex
    }
};
use crate::lg_core::renderer::Renderer;
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

struct ShaderStorage {
    std_v: Rfc<LgShader>,
    std_f: Rfc<LgShader>,
}
impl ShaderStorage {
    fn new() -> Self {
        Self {
            std_v: Rfc::new(LgShader::builder()
                .stage(super::renderer::shader::ShaderStage::VERTEX)
                .src_code(std::path::Path::new("resources/shaders/src/std_v.vert")).unwrap()
                .build()
            ),
            std_f: Rfc::new(LgShader::builder()
                .stage(super::renderer::shader::ShaderStage::FRAGMENT)
                .src_code(std::path::Path::new("resources/shaders/src/std_f.frag")).unwrap()
                .build()
            )
        }
    }
}

struct MaterialStorage {
    grid: Rfc<LgMaterial>,
    viking: Rfc<LgMaterial>,
    red: Rfc<LgMaterial>,
}
impl MaterialStorage {
    fn new(shader_storage: &ShaderStorage, tex_storage: &TexStorage) -> Self {
        let grid = Rfc::new(LgMaterial::new(vec![
                shader_storage.std_v.clone(),
                shader_storage.std_f.clone(),
            ],
            tex_storage.grid.clone()
        ));
        let viking = Rfc::new(LgMaterial::new(vec![
                shader_storage.std_v.clone(),
                shader_storage.std_f.clone(),
            ],
            tex_storage.viking.clone()
        ));
        let red = Rfc::new(LgMaterial::new(vec![
                shader_storage.std_v.clone(),
                shader_storage.std_f.clone(),
            ],
            tex_storage.viking.clone()
        ));
        
        Self {
            grid,
            viking,
            red,
        }
    }
}

struct MeshStorage {
    big_quad: Rfc<LgMesh>,
    med_quad: Rfc<LgMesh>,
    smol_quad: Rfc<LgMesh>,
}
impl MeshStorage {
    fn new() -> Self {
        let big_quad = Rfc::new(LgMesh::new(
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
        ));
        let med_quad = Rfc::new(LgMesh::new(
            vec![
                Vertex { position: glm::vec2(-0.3, -0.3), tex_coord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec2( 0.3, -0.3), tex_coord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec2( 0.3,  0.3), tex_coord: glm::vec2(1.0, 0.0) },
                Vertex { position: glm::vec2(-0.3,  0.3), tex_coord: glm::vec2(0.0, 0.0) },
            ], 
            vec![
                0, 1, 2,
                2, 3, 0
            ]
        ));
        let smol_quad = Rfc::new(LgMesh::new(
            vec![
                Vertex { position: glm::vec2(-0.15, -0.15), tex_coord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec2( 0.15, -0.15), tex_coord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec2( 0.15,  0.15), tex_coord: glm::vec2(1.0, 0.0) },
                Vertex { position: glm::vec2(-0.15,  0.15), tex_coord: glm::vec2(0.0, 0.0) },
            ], 
            vec![
                0, 1, 2,
                2, 3, 0
            ]
        ));
        
        Self {
            big_quad,
            med_quad,
            smol_quad,
        }
    }
}

pub struct TestScene {
    app_core: Rfc<ApplicationCore>,
    meshes: MeshStorage,
    materials: MaterialStorage,
    textures: TexStorage,
    shaders: ShaderStorage,
    
    big: LgEntity,
    smol: LgEntity,
}
impl TestScene {
    pub fn new(core: Rfc<ApplicationCore>) -> Self {
        let textures = TexStorage::new().unwrap();
        let shaders = ShaderStorage::new();
        let materials = MaterialStorage::new(&shaders, &textures);
        let meshes = MeshStorage::new();

        let big = LgEntity::new(meshes.big_quad.clone(), materials.grid.clone()).unwrap();
        let smol = LgEntity::new(meshes.med_quad.clone(), materials.grid.clone()).unwrap();

        Self {
            app_core: core,
            meshes,
            materials,
            textures,
            shaders,
            
            big,
            smol,
        }
    }
    pub fn init(&mut self) {
    }
    pub fn on_update(&mut self) {
        unsafe {
            self.app_core.borrow_mut().renderer.borrow_mut().draw(
                &self.big
            ).unwrap();
            self.app_core.borrow_mut().renderer.borrow_mut().draw(
                &self.smol
            ).unwrap()
        }
    }
    pub fn on_event(&mut self, event: &LgEvent) {
        
    }
    pub fn destroy(&mut self) {

    }
}