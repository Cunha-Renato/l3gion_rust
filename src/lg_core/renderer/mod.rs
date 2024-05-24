#![allow(non_camel_case_types)]

use std::path::Path;
use lg_renderer::renderer::{lg_shader::ShaderStage, lg_uniform::{LgUniform, LgUniformType}};
use crate::StdError;
use self::{material::LgMaterial, mesh::Mesh, shader::Shader, texture::Texture, uniform_struct::SSBO, vertex::Vertex};
use super::{entity::LgEntity, resoruce_manager::ResourceManager, uuid::UUID};
use nalgebra_glm as glm;

pub mod vertex;
pub mod mesh;
pub mod material;
pub mod texture;
pub mod shader;
pub mod uniform;
pub mod buffer;
pub mod uniform_struct;

struct ObjectStorage {
    meshes: Vec<Mesh>,
    shaders: Vec<Shader>,
    textures: Vec<Texture>,
    materials: Vec<LgMaterial>
}
impl ObjectStorage {
    fn new() -> Result<Self, StdError> {
        // Meshes
        let meshes = vec![
            Mesh::new(
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
            Mesh::new(
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
            Mesh::new(
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
            Shader::builder("std_v")
                .stage(ShaderStage::VERTEX)
                .src_code(Path::new("resources/shaders/src/std_v.vert")).unwrap()
                .build(),
            Shader::builder("std_f")
                .stage(ShaderStage::FRAGMENT)
                .src_code(Path::new("resources/shaders/src/std_f.frag")).unwrap()
                .build(),
            Shader::builder("uniform_v")
                .stage(ShaderStage::VERTEX)
                .src_code(Path::new("resources/shaders/src/uniform_v.vert")).unwrap()
                .build(),
            Shader::builder("uniform_f")
                .stage(ShaderStage::FRAGMENT)
                .src_code(Path::new("resources/shaders/src/uniform_f.frag")).unwrap()
                .build(),
            Shader::builder("obj_picker_v")
                .stage(ShaderStage::VERTEX)
                .src_code(Path::new("resources/shaders/src/obj_picker_v.vert")).unwrap()
                .build(),
            Shader::builder("obj_picker_f")
                .stage(ShaderStage::FRAGMENT)
                .src_code(Path::new("resources/shaders/src/obj_picker_f.frag")).unwrap()
                .build(),
        ];

        // Textures
        let textures = vec![
            Texture::new(
                "grid", 
                "resources/textures/grid.png", 
                lg_renderer::renderer::lg_texture::TextureFormat::RGBA, 
                lg_renderer::renderer::lg_texture::TextureType::UNSIGNED_BYTE
            )?,
            Texture::new(
                "viking", 
                "resources/textures/viking.png", 
                lg_renderer::renderer::lg_texture::TextureFormat::RGBA,
                lg_renderer::renderer::lg_texture::TextureType::UNSIGNED_BYTE,
            )?,
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
                vec![self::uniform::Uniform::new_with_data(
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
            self.obj_storage.textures
                .iter()
                .map(|t| (t.uuid().clone(), t))
                .find(|(_, t)| t.name() == texture)
        } else { None };

        let mut shaders = Vec::new();
        for shader in material.shaders() {
            shaders.push(self.obj_storage.shaders
                .iter()
                .find(|s| s.name() == shader)
                .map(|s| (s.uuid().clone(), s)).unwrap()
            );
        }
        let ubos = entity.uniforms
            .iter()
            .chain(material.uniforms.iter())
            .map(|ubo| (ubo.buffer.borrow().uuid().clone(), ubo))
            .collect::<Vec<_>>();

        self.renderer.draw(
            (mesh.uuid().clone(), mesh.vertices(), mesh.indices()), 
            texture, 
            (material.uuid().clone(), &shaders), 
            ubos
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
        
        self.renderer.read_uniform_buffer::<T>(material.uniforms[index].buffer.borrow().uuid().clone(), index)
    }
    pub unsafe fn reset_material_ubo(&self, material_name: &str, uniform_name: &str) -> Result<(), StdError> {
        let material = self.get_material(material_name).unwrap();
        let (index, uniform) = material.uniforms
            .iter()
            .enumerate()
            .find(|(_, u)| {
                u.name() == uniform_name
            }).unwrap();

        // self.renderer.set_uniform_buffer(uniform.buffer.uuid().clone(), index, uniform)
        Ok(())
    }
}
impl LgRenderer {
    pub fn get_mesh(&self, name: &str) -> Option<&Mesh> {
        self.obj_storage.meshes
            .iter()
            .find(|m| m.name() == name)
    }
    pub fn get_mut_mesh(&mut self, name: &str) -> Option<&mut Mesh> {
        self.obj_storage.meshes
            .iter_mut()
            .find(|m| m.name() == name)
    }
    pub fn get_shader(&self, name: &str) -> Option<&Shader> {
        self.obj_storage.shaders
            .iter()
            .find(|s| s.name() == name)
    }
    pub fn get_mut_shader(&mut self, name: &str) -> Option<&mut Shader> {
        self.obj_storage.shaders
            .iter_mut()
            .find(|s| s.name() == name)
    }
    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.obj_storage.textures
            .iter()
            .find(|t| t.name() == name)
    }
    pub fn get_mut_texture(&mut self, name: &str) -> Option<&mut Texture> {
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