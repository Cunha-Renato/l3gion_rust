#![allow(non_camel_case_types)]

use std::path::Path;
use lg_renderer::renderer::{lg_shader::ShaderStage, lg_uniform::{GlUniform, LgUniform, LgUniformType}};
use crate::StdError;
use self::{material::LgMaterial, mesh::LgMesh, shader::LgShader, texture::LgTexture, uniform::{SSBO, UBO}, vertex::Vertex};
use super::{entity::LgEntity, uuid::UUID};
use nalgebra_glm as glm;

pub mod vertex;
pub mod mesh;
pub mod material;
pub mod texture;
pub mod shader;
pub mod uniform;

struct ObjectStorage {
    meshes: Vec<LgMesh>,
    shaders: Vec<LgShader>,
    textures: Vec<LgTexture>,
    materials: Vec<LgMaterial>
}
impl ObjectStorage {
    fn new() -> Result<Self, StdError> {
        // Meshes
        let meshes = vec![
            LgMesh::new(
                "big_quad", 
                vec![
                    Vertex { position: glm::vec3(-0.5, -0.5, 0.0), tex_coord: glm::vec2(0.0, 1.0) },
                    Vertex { position: glm::vec3( 0.5, -0.5, 0.0), tex_coord: glm::vec2(1.0, 1.0) },
                    Vertex { position: glm::vec3( 0.5,  0.5, 0.0), tex_coord: glm::vec2(1.0, 0.0) },
                    Vertex { position: glm::vec3(-0.5,  0.5, 0.0), tex_coord: glm::vec2(0.0, 0.0) },
                ], 
                vec![
                    0, 1, 2,
                    2, 3, 0
                ]
            ),
            LgMesh::new(
                "med_quad",
                vec![
                    Vertex { position: glm::vec3(-0.3, -0.3, 0.0), tex_coord: glm::vec2(0.0, 1.0) },
                    Vertex { position: glm::vec3( 0.3, -0.3, 0.0), tex_coord: glm::vec2(1.0, 1.0) },
                    Vertex { position: glm::vec3( 0.3,  0.3, 0.0), tex_coord: glm::vec2(1.0, 0.0) },
                    Vertex { position: glm::vec3(-0.3,  0.3, 0.0), tex_coord: glm::vec2(0.0, 0.0) },
                ], 
                vec![
                    0, 1, 2,
                    2, 3, 0
                ]
            ),
            LgMesh::new(
                "smol_quad",
                vec![
                    Vertex { position: glm::vec3(-0.15, -0.15, 0.0), tex_coord: glm::vec2(0.0, 1.0) },
                    Vertex { position: glm::vec3( 0.15, -0.15, 0.0), tex_coord: glm::vec2(1.0, 1.0) },
                    Vertex { position: glm::vec3( 0.15,  0.15, 0.0), tex_coord: glm::vec2(1.0, 0.0) },
                    Vertex { position: glm::vec3(-0.15,  0.15, 0.0), tex_coord: glm::vec2(0.0, 0.0) },
                ], 
                vec![
                    0, 1, 2,
                    2, 3, 0
                ]
            ),
        ];
        
        // Shaders
        let shaders = vec![
            LgShader::builder("std_v")
                .stage(ShaderStage::VERTEX)
                .src_code(Path::new("resources/shaders/src/std_v.vert")).unwrap()
                .build(),
            LgShader::builder("std_f")
                .stage(ShaderStage::FRAGMENT)
                .src_code(Path::new("resources/shaders/src/std_f.frag")).unwrap()
                .build(),
            LgShader::builder("uniform_v")
                .stage(ShaderStage::VERTEX)
                .src_code(Path::new("resources/shaders/src/uniform_v.vert")).unwrap()
                .build(),
            LgShader::builder("uniform_f")
                .stage(ShaderStage::FRAGMENT)
                .src_code(Path::new("resources/shaders/src/uniform_f.frag")).unwrap()
                .build(),
            LgShader::builder("obj_picker_v")
                .stage(ShaderStage::VERTEX)
                .src_code(Path::new("resources/shaders/src/obj_picker_v.vert")).unwrap()
                .build(),
            LgShader::builder("obj_picker_f")
                .stage(ShaderStage::FRAGMENT)
                .src_code(Path::new("resources/shaders/src/obj_picker_f.frag")).unwrap()
                .build(),
        ];

        // Textures
        let textures = vec![
            LgTexture::new("grid", "resources/textures/grid.png")?,
            LgTexture::new("viking", "resources/textures/viking.png")?,
        ];

        let ssbo = SSBO {
            data: glm::UVec4::new(0, 0, 0, 0),
        };
        let materials = vec![
            LgMaterial::new(
                "grid", 
                vec!["std_v".to_string(), "std_f".to_string()], 
                Some("grid".to_string()), 
                vec![]
            ),
            LgMaterial::new(
                "viking", 
                vec!["std_v".to_string(), "std_f".to_string()], 
                Some("viking".to_string()), 
                vec![]
            ),
            LgMaterial::new(
                "uniform",
                vec!["uniform_v".to_string(), "uniform_f".to_string()],
                None,
                vec![],
            ),
            LgMaterial::new(
                "obj_picker", 
                vec!["obj_picker_v".to_string(), "obj_picker_f".to_string()], 
                None,
                vec![LgUniform::new(
                    "ssbo", 
                    LgUniformType::STORAGE_BUFFER, 
                    2, 
                    0, 
                    false,
                    ssbo
                )]
            ),
        ];

        Ok(Self {
            meshes,
            shaders,
            textures,
            materials,
        })
    }
}

pub struct LgRenderer {
    renderer: lg_renderer::renderer::LgRenderer<UUID>,
    obj_storage: ObjectStorage,
}
impl LgRenderer {
    pub fn new(renderer: lg_renderer::renderer::LgRenderer<UUID>) -> Result<Self, StdError> {
        Ok(Self {
            renderer,
            obj_storage: ObjectStorage::new()?
        })
    }

    pub unsafe fn draw_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        let mesh = self.obj_storage.meshes
            .iter()
            .find(|m| m.name() == &entity.mesh).unwrap();
        let material = self.obj_storage.materials
            .iter()
            .find(|m| m.name() == &entity.material).unwrap();

        let texture = if let Some(texture) = material.texture() {
            let tex = self.obj_storage.textures
                .iter()
                .find(|t| t.name() == texture).unwrap();
            
            Some((tex.uuid().clone(), tex))
        } else { None };

        let mut shaders = Vec::new();
        for shader in material.shaders() {
            shaders.push(self.obj_storage.shaders
                .iter()
                .find(|s| s.name() == shader)
                .map(|s| (s.uuid().clone(), s)).unwrap()
            );
        }
        let entity_ubos = &entity.uniforms;
        let material_ubos = &material.uniforms;

        self.renderer.draw(
            (mesh.uuid().clone(), mesh.vertices(), mesh.indices()), 
            texture, 
            (material.uuid().clone(), &shaders), 
            vec![(entity.uuid().clone(), entity_ubos), (material.uuid().clone(), material_ubos)]
        )?;
        
        Ok(())
    }
    pub unsafe fn begin(&self) {
        self.renderer.begin()
    }
    pub unsafe fn end(&self) -> Result<(), StdError> {
        self.renderer.end()
    }
    pub unsafe fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError> {
        self.renderer.resize(new_size)?;

        Ok(())
    }
    pub unsafe fn read_material_ubo<T: Clone>(&self, material_name: &str, uniform_name: &str) -> Result<T, StdError> {
        let material = self.get_material(material_name).unwrap();
        let (index, _) = material.uniforms
            .iter()
            .enumerate()
            .find(|(_, u)| {
                u.name() == uniform_name
            }).unwrap();
        
        self.renderer.read_uniform_buffer::<T>(material.uuid().clone(), index)
    }
    pub unsafe fn reset_material_ubo(&self, material_name: &str, uniform_name: &str) -> Result<(), StdError> {
        let material = self.get_material(material_name).unwrap();
        let (index, uniform) = material.uniforms
            .iter()
            .enumerate()
            .find(|(_, u)| {
                u.name() == uniform_name
            }).unwrap();

        self.renderer.set_uniform_buffer(material.uuid().clone(), index, uniform)
    }
}
impl LgRenderer {
    pub fn get_mesh(&self, name: &str) -> Option<&LgMesh> {
        self.obj_storage.meshes
            .iter()
            .find(|m| m.name() == name)
    }
    pub fn get_mut_mesh(&mut self, name: &str) -> Option<&mut LgMesh> {
        self.obj_storage.meshes
            .iter_mut()
            .find(|m| m.name() == name)
    }
    pub fn get_shader(&self, name: &str) -> Option<&LgShader> {
        self.obj_storage.shaders
            .iter()
            .find(|s| s.name() == name)
    }
    pub fn get_mut_shader(&mut self, name: &str) -> Option<&mut LgShader> {
        self.obj_storage.shaders
            .iter_mut()
            .find(|s| s.name() == name)
    }
    pub fn get_texture(&self, name: &str) -> Option<&LgTexture> {
        self.obj_storage.textures
            .iter()
            .find(|t| t.name() == name)
    }
    pub fn get_mut_texture(&mut self, name: &str) -> Option<&mut LgTexture> {
        self.obj_storage.textures
            .iter_mut()
            .find(|t| t.name() == name)
    }
    pub fn get_material(&self, name: &str) -> Option<&LgMaterial> {
        self.obj_storage.materials
            .iter()
            .find(|m| m.name() == name)
    }
    pub fn get_mut_material(&mut self, name: &str) -> Option<&mut LgMaterial> {
        self.obj_storage.materials
            .iter_mut()
            .find(|m| m.name() == name)
    }
}