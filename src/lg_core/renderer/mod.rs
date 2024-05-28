#![allow(non_camel_case_types)]

use std::path::Path;
use lg_renderer::renderer::{lg_shader::ShaderStage, lg_uniform::{LgUniform, LgUniformType}};
use crate::StdError;
use self::{material::LgMaterial, mesh::Mesh, shader::Shader, texture::Texture, uniform_struct::SSBO, vertex::Vertex};
use super::{entity::LgEntity, resoruce_manager::ResourceManager, uuid::{self, UUID}};
use nalgebra_glm as glm;

pub mod vertex;
pub mod mesh;
pub mod material;
pub mod texture;
pub mod shader;
pub mod uniform;
pub mod buffer;
pub mod uniform_struct;

pub struct LgRenderer {
    renderer: lg_renderer::renderer::LgRenderer<UUID>,
    resource_manager: ResourceManager,
}
impl LgRenderer {
    pub fn new(renderer: lg_renderer::renderer::LgRenderer<UUID>) -> Result<Self, StdError> {
        Ok(Self {
            renderer,
            resource_manager: ResourceManager::default()
        })
    }

    pub fn init(&mut self) -> Result<(), StdError> {
        self.resource_manager.process_folder(std::path::Path::new("resources"))?;
        self.resource_manager.init()?;
        
        Ok(())
    }

    pub unsafe fn draw_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        let s_uuid1 = UUID::from_u128(86545322955764439664055660664792965181);
        let s_uuid2 = UUID::from_u128(216783183541451875125666792462885461737);

        self.resource_manager.prepare_mesh(&entity.mesh)?;
        self.resource_manager.prepare_shader(&s_uuid1)?;
        self.resource_manager.prepare_shader(&s_uuid2)?;

        let mesh = self.resource_manager.get_mesh(&entity.mesh).unwrap();
        // let material = self.resource_manager.get_material(&entity.material);

        // let texture = if let Some(texture) = material.texture() {
            // self.resource_manager.get_texture(&material.texture)?
        // } else { None };
        let texture: Option<(UUID, &Texture)> = None;

        let shaders = vec![
            (s_uuid1.clone(), self.resource_manager.get_shader(&s_uuid1).unwrap()), 
            (s_uuid2.clone(), self.resource_manager.get_shader(&s_uuid2).unwrap())
        ];
        /* for shader in material.shaders() {
            shaders.push(self.resource_manager.get_shader(&shader));
        } */
        let ubos = entity.uniforms
            .iter()
            // .chain(material.uniforms.iter())
            .map(|ubo| (ubo.buffer.uuid().clone(), ubo))
            .collect::<Vec<_>>();

        self.renderer.draw(
            (mesh.uuid().clone(), mesh.vertices(), mesh.indices()), 
            texture, 
            (UUID::from_u128(86545322955764439664055660664792965181), &shaders), // Shader Program, I need a UUID placeholder
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
        /* let material = self.get_material(material_name).unwrap();
        let (index, _) = material.uniforms
            .iter()
            .enumerate()
            .find(|(_, u)| {
                u.name() == uniform_name
            }).unwrap();
        
        self.renderer.read_uniform_buffer::<T>(material.uniforms[index].buffer.uuid().clone(), index) */
        Err("Not available".into())
    }
    pub unsafe fn reset_material_ubo(&self, material_name: &str, uniform_name: &str) -> Result<(), StdError> {
        /* let material = self.get_material(material_name).unwrap();
        let (index, uniform) = material.uniforms
            .iter()
            .enumerate()
            .find(|(_, u)| {
                u.name() == uniform_name
            }).unwrap(); */

        // self.renderer.set_uniform_buffer(uniform.buffer.uuid().clone(), index, uniform)
        Ok(())
    }
}