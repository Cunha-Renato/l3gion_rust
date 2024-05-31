#![allow(non_camel_case_types)]

use std::{borrow::BorrowMut, collections::{HashMap, HashSet}, path::Path};
use lg_renderer::renderer::{lg_shader::ShaderStage, lg_uniform::{LgUniform, LgUniformType}};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use uniform::Uniform;
use crate::{profile_function, profile_scope, StdError};
use self::{material::Material, mesh::Mesh, shader::Shader, texture::Texture, uniform_struct::SSBO, vertex::Vertex};
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
    
    // (Material UUID, Data)
    batch_draw_data: HashMap<UUID, BatchDrawData>,
    in_use_batch_data: BatchData,
    
    // TODO: Just for testing
    global_uniform: Option<Uniform>,
}
impl LgRenderer {
    pub fn new(renderer: lg_renderer::renderer::LgRenderer<UUID>) -> Result<Self, StdError> {
        Ok(Self {
            renderer,
            resource_manager: ResourceManager::default(),
            batch_draw_data: HashMap::default(),
            in_use_batch_data: BatchData::default(),
            global_uniform: None,
        })
    }

    pub fn set_resource_folder(&mut self, path: &std::path::Path) -> Result<(), StdError> {
        self.resource_manager.read_resource_paths(path)
    }

    pub fn init(&mut self) -> Result<(), StdError> {
        self.resource_manager.process_folder(std::path::Path::new("resources"))?;
        self.resource_manager.init()?;
        
        Ok(())
    }

    pub unsafe fn draw_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        profile_function!();

        self.resource_manager.prepare_mesh(&entity.mesh)?;
        self.resource_manager.prepare_material(&entity.material)?;

        self.draw(&entity.mesh, &entity.material, &entity.uniforms)
    }
    pub unsafe fn begin(&self) {
        profile_function!();

        self.renderer.begin()
    }
    pub unsafe fn end(&mut self) -> Result<(), StdError> {
        profile_function!();

        self.flush_batch()?;
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
        todo!();
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
        todo!();
    }
}
impl LgRenderer {
    unsafe fn draw(&mut self, mesh: &UUID, material: &UUID, uniforms: &[Uniform]) -> Result<(), StdError> {
        let mesh = self.resource_manager.get_mesh(mesh).unwrap();
        let material = self.resource_manager.get_material(material).unwrap();

        let texture: Option<(UUID, &Texture)> = None;

        let shaders = material.shaders().iter()
            .map(|s| (s.clone(), self.resource_manager.get_shader(s).unwrap()))
            .collect::<Vec<_>>();

        let mut ubos = uniforms
            .iter()
            .chain(material.uniforms.iter())
            .map(|ubo| (ubo.buffer.uuid().clone(), ubo))
            .collect::<Vec<_>>();

        // TODO: Testing
        if let Some(uniform) = self.global_uniform.as_ref() {
            ubos.push((uniform.buffer.uuid().clone(), uniform));
        }

        // TODO: I need a shader Program, so different materials can use the same program, and not require to recreate it
        self.renderer.draw(
            (mesh.uuid().clone(), mesh.vertices(), mesh.indices()), 
            texture, 
            (UUID::from_u128(86545322955764439664055660664792965181), &shaders), // Shader Program, I need a UUID placeholder
            ubos
        )
    }
}

struct BatchConstraints {
    uuid: UUID,
    max_meshes: u32,
    max_textures: u32,
}

#[derive(Default, Debug)]
struct BatchData {
    meshes: u32,
    textures: HashSet<UUID>,
}

#[derive(Default)]
struct BatchDrawData {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

static BATCH_CONSTRAINTS: BatchConstraints = BatchConstraints {
    uuid: UUID::from_u128(3127841985126510201947169420),
    max_meshes: 1000,
    max_textures: 20,
};

// Batch
impl LgRenderer {
    pub fn set_uniform(&mut self, uniform: Uniform) {
        self.global_uniform = Some(uniform);
    }
    pub unsafe fn batch_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        profile_function!();

        self.resource_manager.prepare_mesh(&entity.mesh)?;
        self.resource_manager.prepare_material(&entity.material)?;

        if self.in_use_batch_data.meshes + 1 > BATCH_CONSTRAINTS.max_meshes {
            self.flush_batch()?;
        }
        
        let (mut vertices, mut indices) = {
            let mesh = self.resource_manager.get_mesh(&entity.mesh).unwrap();
            
            (mesh.vertices().to_vec(), mesh.indices().to_vec())
        };
        
        // Transform
        let identity = glm::Mat4::identity();
        let translation = glm::translate(&identity, &entity.position);
        let rotation = glm::rotate(&identity, entity.rotation_angle, &entity.rotation_axis);
        let scale = glm::scale(&identity, &entity.scale);

        let transform = translation * rotation * scale;
        {
            profile_scope!("transform_loop");
            vertices
                .par_iter_mut()
                .for_each(|v| {
                    let og_position = v.position;
                    let v_position = glm::vec4(og_position.x, og_position.y, og_position.z, 1.0);
                    let transformed = transform * v_position;
                    
                    v.position = glm::vec3(transformed.x, transformed.y, transformed.z);
                });
        }
        
        let draw_data = self.batch_draw_data.entry(entity.material.clone()).or_default();
        { 
            profile_scope!("updating_indices");

            for i in &mut indices {
                *i += draw_data.vertices.len() as u32;
            }
        }

        {
            profile_scope!("extending_draw_data");

            self.in_use_batch_data.meshes += 1;
            let mut new_vertices = Vec::with_capacity(draw_data.vertices.len() + vertices.len());
            new_vertices.extend_from_slice(&draw_data.vertices);
            new_vertices.extend_from_slice(&vertices);

            let mut new_indices = Vec::with_capacity(draw_data.indices.len() + indices.len());
            new_indices.extend_from_slice(&draw_data.indices);
            new_indices.extend_from_slice(&indices);

            draw_data.vertices = new_vertices;
            draw_data.indices = new_indices;
        }

        Ok(())
    }
    pub unsafe fn flush_batch(&mut self) -> Result<(), StdError> {
        profile_function!();

        for (mat_uuid, dd) in &self.batch_draw_data {
            let material = self.resource_manager.get_material(mat_uuid).unwrap();
            let texture: Option<(UUID, &Texture)> = None;
            let shaders = material.shaders().iter()
                .map(|s| (s.clone(), self.resource_manager.get_shader(s).unwrap()))
                .collect::<Vec<_>>();

            let mut ubos = material.uniforms.iter()
                .map(|ubo| (ubo.buffer.uuid().clone(), ubo))
                .collect::<Vec<_>>();

            // TODO: Testing
            if let Some(uniform) = self.global_uniform.as_ref() {
                ubos.push((uniform.buffer.uuid().clone(), uniform));
            }

            self.renderer.borrow_mut().draw(
                (BATCH_CONSTRAINTS.uuid.clone(), &dd.vertices, &dd.indices), 
                texture, 
                (UUID::from_u128(86545322955764439664055660664792965181), &shaders), 
                ubos
            )?;
        }


        self.in_use_batch_data.meshes = 0;
        self.in_use_batch_data.textures.clear();
        self.batch_draw_data.clear();
        
        Ok(())
    }
}