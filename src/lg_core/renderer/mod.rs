#![allow(non_camel_case_types)]

use std::{borrow::BorrowMut, collections::{HashMap, HashSet}, path::Path};
use lg_renderer::{lg_vertex, renderer::{lg_shader::ShaderStage, lg_uniform::{LgUniform, LgUniformType}}};
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
    instance_draw_data: HashMap<UUID, InstanceDrawData>,
    in_use_instance_data: InstanceData,
    
    // TODO: Just for testing
    global_uniform: Option<Uniform>,
    
    // TODO: Testing
    pub draw_calls: usize,
}
impl LgRenderer {
    pub fn new(renderer: lg_renderer::renderer::LgRenderer<UUID>) -> Result<Self, StdError> {
        Ok(Self {
            renderer,
            resource_manager: ResourceManager::default(),
            instance_draw_data: HashMap::default(),
            in_use_instance_data: InstanceData::default(),
            global_uniform: None,
            draw_calls: 0,
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

        self.flush()?;
        self.draw_calls = 0;
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

        self.draw_calls += 1;
        // TODO: I need a shader Program, so different materials can use the same program, and not require to recreate it
        self.renderer.draw(
            (mesh.uuid().clone(), mesh.vertices(), mesh.indices()), 
            texture, 
            (UUID::from_u128(86545322955764439664055660664792965181), &shaders), // Shader Program, I need a UUID placeholder
            ubos,
        )
    }
}

struct InstanceVertex {
    row_0: glm::Vec4,
    row_1: glm::Vec4,
    row_2: glm::Vec4,
}
lg_vertex!(InstanceVertex, row_0, row_1, row_2);

struct InstancingConstraints {
    max_instances: u32,
    max_textures: u32,
}

#[derive(Default, Debug)]
struct InstanceData {
    instances: u32,
    textures: HashSet<UUID>,
}

#[derive(Default)]
struct InstanceDrawData {
    instance_data: HashMap<UUID, Vec<InstanceVertex>>
}

static INSTANCING_CONSTRAINTS: InstancingConstraints = InstancingConstraints {
    max_instances: 1000,
    max_textures: 20,
};

// Batch
impl LgRenderer {
    pub fn set_uniform(&mut self, uniform: Uniform) {
        self.global_uniform = Some(uniform);
    }
    pub fn update_uniform<D>(&mut self, name: &str, data: &D) {
        match &mut self.global_uniform {
            Some(u) => if u.name() == name {
                u.set_data(data);
            },
            None => (),
        };
    }
    pub unsafe fn instance_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        profile_function!();

        {
            profile_scope!("preparing resources");
            self.resource_manager.prepare_mesh(&entity.mesh)?;
            self.resource_manager.prepare_material(&entity.material)?;
        }
        
        // Transform
        let data = {
            profile_scope!("transform");

            let identity = glm::Mat4::identity();
            let translation = glm::translate(&identity, &entity.position);
            let rotation = glm::rotate(&identity, entity.rotation_angle, &entity.rotation_axis);
            let scale = glm::scale(&identity, &entity.scale);
            let model = translation * rotation * scale;

            let row_0 = glm::vec4(model[(0, 0)], model[(0, 1)], model[(0, 2)], model[(0, 3)]);
            let row_1 = glm::vec4(model[(1, 0)], model[(1, 1)], model[(1, 2)], model[(1, 3)]);
            let row_2 = glm::vec4(model[(2, 0)], model[(2, 1)], model[(2, 2)], model[(2, 3)]);
            InstanceVertex {
                row_0,
                row_1,
                row_2,
            }
        };

        {
            profile_scope!("match statement");
            match self.instance_draw_data.entry(entity.material.clone()) {
                std::collections::hash_map::Entry::Occupied(val) => {
                    let dd = val.into_mut();

                    match dd.instance_data.entry(entity.mesh.clone()) {
                        std::collections::hash_map::Entry::Occupied(val) => val.into_mut().push(data),
                        std::collections::hash_map::Entry::Vacant(entry) => { entry.insert(vec![data]); },
                    }
                },
                std::collections::hash_map::Entry::Vacant(entry) => {
                    let instances = entry.insert(InstanceDrawData::default());                
                    instances.instance_data.insert(entity.mesh.clone(), vec![data]);
                },
            }
        }

        self.in_use_instance_data.instances += 1;
        if self.in_use_instance_data.instances >= INSTANCING_CONSTRAINTS.max_instances {
            self.flush()?;
        }

        Ok(())
    }
    pub unsafe fn flush(&mut self) -> Result<(), StdError> {
        profile_function!();

        for (mat_uuid, dd) in &self.instance_draw_data {
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

            for dd in &dd.instance_data {
                let mesh = self.resource_manager.get_mesh(dd.0).ok_or("Failed to get Mesh in flush! (Renderer)")?;
                
                let vertices = mesh.vertices();
                let indices = mesh.indices();
                
                self.draw_calls += 1;
                self.renderer.borrow_mut().draw_instanced(
                    (mesh.uuid().clone(), vertices, indices), 
                    texture.clone(), 
                    (UUID::from_u128(86545322955764439664055660664792965181), &shaders), 
                    ubos.clone(),
                    dd.1
                )?;
            }
        }


        self.in_use_instance_data.instances = 0;
        self.in_use_instance_data.textures.clear();
        self.instance_draw_data.clear();
        
        Ok(())
    }
}