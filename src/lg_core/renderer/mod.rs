#![allow(non_camel_case_types)]

use std::{borrow::BorrowMut, collections::{HashMap, HashSet}, sync::{Arc, Mutex}};
use lg_renderer::{lg_vertex, renderer::lg_uniform::LgUniform};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use uniform::Uniform;
use crate::{profile_function, profile_scope, StdError};
use self::texture::Texture;
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

pub struct LgRenderer {
    renderer: lg_renderer::renderer::LgRenderer<UUID>,
    resource_manager: ResourceManager,
    
    // (Material UUID, Data)
    instance_draw_data: HashMap<UUID, HashMap<UUID, Vec<InstanceVertex>>>,
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
        // self.resource_manager.process_folder(std::path::Path::new("resources"))?;
        self.resource_manager.init()?;
        
        Ok(())
    }

    pub unsafe fn draw_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        profile_function!();

        self.resource_manager.prepare_mesh(&entity.mesh)?;
        self.resource_manager.prepare_material(&entity.material)?;
        
        let mesh = self.resource_manager.get_mesh(&entity.mesh).unwrap();
        let material = self.resource_manager.get_material(&entity.material).unwrap();

        let texture: Option<(UUID, &Texture)> = None;

        let shaders = material.shaders().iter()
            .map(|s| (s.clone(), self.resource_manager.get_shader(s).unwrap()))
            .collect::<Vec<_>>();

        let mut ubos = entity.uniforms
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
            (material.uuid().clone(), &shaders), // Shader Program, I need a UUID placeholder
            ubos,
        )
    }
    pub unsafe fn begin(&self) {
        self.renderer.begin()
    }
    pub unsafe fn end(&mut self) -> Result<(), StdError> {
        self.flush()?;
        self.draw_calls = 0;
        {
            profile_scope!("gl_swap_buffers");
            self.renderer.end()
        }
    }
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
    pub unsafe fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError> {
        self.renderer.resize(new_size)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
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

static INSTANCING_CONSTRAINTS: InstancingConstraints = InstancingConstraints {
    max_instances: 100_000,
    max_textures: 20,
};

impl LgRenderer {
    pub unsafe fn instance_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        profile_function!();

        {
            profile_scope!("preparing resources");
            self.resource_manager.prepare_mesh(&entity.mesh)?;
            self.resource_manager.prepare_material(&entity.material)?;
        }

        let model = entity.model();
        let row_0 = glm::vec4(model[(0, 0)], model[(0, 1)], model[(0, 2)], model[(0, 3)]);
        let row_1 = glm::vec4(model[(1, 0)], model[(1, 1)], model[(1, 2)], model[(1, 3)]);
        let row_2 = glm::vec4(model[(2, 0)], model[(2, 1)], model[(2, 2)], model[(2, 3)]);
        let data = InstanceVertex {
            row_0,
            row_1,
            row_2,
        };

        let mesh_map = self.instance_draw_data.entry(entity.material.clone()).or_insert_with(HashMap::new);
        match mesh_map.entry(entity.mesh.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(data);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(vec![data]);
            }
        }

        self.in_use_instance_data.instances += 1;
        if self.in_use_instance_data.instances >= INSTANCING_CONSTRAINTS.max_instances {
            self.flush()?;
        }

        Ok(())
    }
    unsafe fn flush(&mut self) -> Result<(), StdError> {
        profile_function!();
        for (mat_uuid, dd) in &self.instance_draw_data {
            let material = self.resource_manager.get_material(mat_uuid).ok_or("Failed to get Material in flush! (Renderer)")?;
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

            for dd in dd {
                let mesh = self.resource_manager.get_mesh(dd.0).ok_or("Failed to get Mesh in flush! (Renderer)")?;
                
                let vertices = mesh.vertices();
                let indices = mesh.indices();
                
                self.draw_calls += 1;
                self.renderer.borrow_mut().draw_instanced(
                    (mesh.uuid().clone(), vertices, indices), 
                    texture.clone(), 
                    (mat_uuid.clone(), &shaders), 
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