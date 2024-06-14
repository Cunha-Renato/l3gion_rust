#![allow(non_camel_case_types)]

use std::{borrow::BorrowMut, collections::{HashMap, HashSet}};
use lg_renderer::{lg_vertex, renderer::{lg_uniform::LgUniform, lg_vertex::LgVertex}};
use uniform::Uniform;
use crate::{profile_function, profile_scope, StdError};
use self::texture::Texture;
use super::{entity::LgEntity, asset_manager::AssetManager, uuid::UUID};
use nalgebra_glm as glm;

pub mod vertex;
pub mod mesh;
pub mod material;
pub mod texture;
pub mod shader;
pub mod uniform;
pub mod buffer;
pub mod mt_renderer;

pub struct RendererConfig {
    pub v_sync: bool,
}

pub struct LgRenderer {
    renderer: lg_renderer::renderer::LgRenderer<UUID>,
    config: RendererConfig,
    asset_manager: AssetManager,
    
    // (Material UUID, Data)
    instance_draw_data: HashMap<UUID, HashMap<UUID, Vec<InstanceVertex>>>,
    in_use_instance_data: InstanceData,
    
    // TODO: Just for testing
    global_uniform: Option<Uniform>,
    
    // TODO: Testing
    pub draw_calls: usize,
}
impl LgRenderer {
    pub fn new(renderer: lg_renderer::renderer::LgRenderer<UUID>, config: RendererConfig) -> Result<Self, StdError> {

        Ok(Self {
            renderer,
            config,
            asset_manager: AssetManager::default(),
            instance_draw_data: HashMap::default(),
            in_use_instance_data: InstanceData::default(),
            global_uniform: None,
            draw_calls: 0,
        })
    }

    pub fn set_resource_folder(&mut self, path: &std::path::Path) -> Result<(), StdError> {
        self.asset_manager.read_asset_paths(path)
    }

    pub fn init(&mut self) -> Result<(), StdError> {
        profile_function!();
        // self.asset_manager.process_folder(std::path::Path::new("assets"))?;
        self.asset_manager.init()?;
        
        self.set_vsync(self.config.v_sync);
        self.renderer.init()
    }
    
    pub fn set_vsync(&mut self, v_sync: bool) {
        self.config.v_sync = v_sync;

        if self.config.v_sync != self.renderer.is_vsync() {
            self.renderer.set_vsync(v_sync);
        }
    }
    pub fn is_vsync(&self) -> bool {
        self.config.v_sync
    }

    pub fn shutdown(&mut self) -> Result<(), StdError> {
        self.renderer.shutdown()
    }

    pub fn draw_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        profile_function!();

        self.asset_manager.prepare_mesh(&entity.mesh)?;
        self.asset_manager.prepare_material(&entity.material)?;
        
        let mesh = self.asset_manager.get_mesh(&entity.mesh).unwrap();
        let material = self.asset_manager.get_material(&entity.material).unwrap();

        let texture: Option<(UUID, &Texture)> = None;

        let shaders = material.shaders().iter()
            .map(|s| (s.clone(), self.asset_manager.get_shader(s).unwrap()))
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
    pub fn begin(&self) -> Result<(), StdError>{
        profile_function!();
        self.renderer.begin()?;
        Ok(())
    }
    pub fn end(&mut self) -> Result<(), StdError> {
        profile_function!();
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
    pub fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError> {
        self.renderer.resize(new_size)
    }
}

// ****************************************************************************************************
// -------------------------------------------- TEST_INSTANCING --------------------------------------------
// ****************************************************************************************************

pub struct TestInstanceData<V: LgVertex> {
    data: HashMap<UUID, HashMap<UUID, Vec<V>>>,
}

impl LgRenderer {
    pub fn begin_instancing<V: LgVertex>(&self) -> TestInstanceData<V> {
        TestInstanceData {
            data: HashMap::new(),
        }
    }
    pub fn queue_instance<V: LgVertex, F>(&mut self, entity: &LgEntity, instance_data: &mut TestInstanceData<V>, f: F) -> Result<(), StdError>
    where F: FnOnce(&LgEntity) -> V
    {
        profile_function!();

        {
            profile_scope!("preparing resources");
            self.asset_manager.prepare_mesh(&entity.mesh)?;
            self.asset_manager.prepare_material(&entity.material)?;
        }

        let material = self.asset_manager.get_material(&entity.material).ok_or("Failed to get Material in flush! (Renderer)")?;

        let data = f(entity);

        let mesh_map = instance_data.data.entry(entity.material.clone()).or_insert_with(HashMap::new);
        match mesh_map.entry(entity.mesh.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(data);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(vec![data]);
            }
        }
        
        assert!(material.texture().len() <= INSTANCING_CONSTRAINTS.max_textures as usize);
        
        for tex_id in material.texture() {
            self.in_use_instance_data.textures.insert(tex_id.clone());
        }

        self.in_use_instance_data.instances += 1;
        if self.in_use_instance_data.instances >= INSTANCING_CONSTRAINTS.max_instances || self.in_use_instance_data.textures.len() >= INSTANCING_CONSTRAINTS.max_textures as usize {
            self.end_instancing(instance_data)?;
        }

        Ok(())
    }
    pub fn end_instancing<V: LgVertex>(&mut self, instance_data: &mut TestInstanceData<V>) -> Result<(), StdError> {
        profile_function!();
        for (mat_uuid, dd) in &instance_data.data {
            let material = self.asset_manager.get_material(mat_uuid).ok_or("Failed to get Material in flush! (Renderer)")?;

            let mut textures: Vec<(UUID, &Texture, u32)> = Vec::new();
            for (location, tex_id) in material.texture().iter().enumerate() {
                textures.push(
                (
                    tex_id.clone(),
                    self.asset_manager.get_texture(tex_id).unwrap(),
                    location as u32,
                ));
            }

            let shaders = material.shaders().iter()
                .map(|s| (s.clone(), self.asset_manager.get_shader(s).unwrap()))
                .collect::<Vec<_>>();

            let mut ubos = material.uniforms.iter()
                .map(|ubo| (ubo.buffer.uuid().clone(), ubo))
                .collect::<Vec<_>>();

            // TODO: Testing
            if let Some(uniform) = self.global_uniform.as_ref() {
                ubos.push((uniform.buffer.uuid().clone(), uniform));
            }

            for dd in dd {
                let mesh = self.asset_manager.get_mesh(dd.0).ok_or("Failed to get Mesh in flush! (Renderer)")?;
                
                let vertices = mesh.vertices();
                let indices = mesh.indices();
                
                self.draw_calls += 1;
                self.renderer.borrow_mut().draw_instanced(
                    (mesh.uuid().clone(), vertices, indices), 
                    &textures,
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

// ****************************************************************************************************
// -------------------------------------------- INSTANCING --------------------------------------------
// ****************************************************************************************************

#[derive(Clone, Debug)]
struct InstanceVertex {
    row_0: glm::Vec4,
    row_1: glm::Vec4,
    row_2: glm::Vec4,
    tex_index: i32,
}
lg_vertex!(InstanceVertex, row_0, row_1, row_2, tex_index);

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
    max_textures: 32,
};

impl LgRenderer {
    pub fn instance_entity(&mut self, entity: &LgEntity) -> Result<(), StdError> {
        profile_function!();

        {
            profile_scope!("preparing resources");
            self.asset_manager.prepare_mesh(&entity.mesh)?;
            self.asset_manager.prepare_material(&entity.material)?;
        }

        let material = self.asset_manager.get_material(&entity.material).ok_or("Failed to get Material in flush! (Renderer)")?;

        let model = entity.model();
        let row_0 = glm::vec4(model[(0, 0)], model[(0, 1)], model[(0, 2)], model[(0, 3)]);
        let row_1 = glm::vec4(model[(1, 0)], model[(1, 1)], model[(1, 2)], model[(1, 3)]);
        let row_2 = glm::vec4(model[(2, 0)], model[(2, 1)], model[(2, 2)], model[(2, 3)]);
        let data = InstanceVertex {
            row_0,
            row_1,
            row_2,
            tex_index: if material.texture().len() > 0 { 0 } else { -1 }, // TODO: For now is only one texture
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
        
        assert!(material.texture().len() <= INSTANCING_CONSTRAINTS.max_textures as usize);
        
        for tex_id in material.texture() {
            self.in_use_instance_data.textures.insert(tex_id.clone());
        }

        self.in_use_instance_data.instances += 1;
        if self.in_use_instance_data.instances >= INSTANCING_CONSTRAINTS.max_instances || self.in_use_instance_data.textures.len() >= INSTANCING_CONSTRAINTS.max_textures as usize {
            self.flush()?;
        }

        Ok(())
    }
    fn flush(&mut self) -> Result<(), StdError> {
        profile_function!();
        for (mat_uuid, dd) in &self.instance_draw_data {
            let material = self.asset_manager.get_material(mat_uuid).ok_or("Failed to get Material in flush! (Renderer)")?;

            let mut textures: Vec<(UUID, &Texture, u32)> = Vec::new();
            for (location, tex_id) in material.texture().iter().enumerate() {
                textures.push(
                (
                    tex_id.clone(),
                    self.asset_manager.get_texture(tex_id).unwrap(),
                    location as u32,
                ));
            }

            let shaders = material.shaders().iter()
                .map(|s| (s.clone(), self.asset_manager.get_shader(s).unwrap()))
                .collect::<Vec<_>>();

            let mut ubos = material.uniforms.iter()
                .map(|ubo| (ubo.buffer.uuid().clone(), ubo))
                .collect::<Vec<_>>();

            // TODO: Testing
            if let Some(uniform) = self.global_uniform.as_ref() {
                ubos.push((uniform.buffer.uuid().clone(), uniform));
            }

            for dd in dd {
                let mesh = self.asset_manager.get_mesh(dd.0).ok_or("Failed to get Mesh in flush! (Renderer)")?;
                
                let vertices = mesh.vertices();
                let indices = mesh.indices();
                
                self.draw_calls += 1;
                self.renderer.borrow_mut().draw_instanced(
                    (mesh.uuid().clone(), vertices, indices), 
                    &textures,
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