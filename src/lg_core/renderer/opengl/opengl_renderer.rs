extern crate gl;
use std::ffi::CString;
use glutin::{display::GlDisplay, surface::GlSurface};
use crate::{gl_check, lg_core::{entity::LgEntity, renderer::{material::LgMaterial, Renderer}, uuid::UUID}, StdError};
use super::opengl_object_chache::GlObjectsCache;
use super::{opengl_mesh::GlMesh, utils};

pub(crate) struct GlSpecs {
    pub(crate) gl_context: glutin::context::PossiblyCurrentContext,
    pub(crate) gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    pub(crate) gl_display: glutin::display::Display, 
}

#[derive(Default)]
struct DrawSpec {
    texture_uuid: Option<UUID>,
    vao_uuid: UUID,
    indices_len: usize,
    program_uuid: UUID,
    entity_uuid: UUID,
    material_uuid: UUID,
}

pub(crate) struct GlRenderer {
    specs: GlSpecs,
    storage: GlObjectsCache,
    to_render: Vec<DrawSpec>
}
impl GlRenderer {
    pub fn new(specs: GlSpecs) -> Self {
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            specs.gl_display.get_proc_address(symbol.as_c_str()).cast()
        });
        
        unsafe {
            if cfg!(debug_assertions) {
                gl_check!(gl::Enable(gl::DEBUG_OUTPUT));
                gl_check!(gl::DebugMessageCallback(Some(utils::debug_callback), std::ptr::null()));
            }
            
            gl_check!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
            gl_check!(gl::Enable(gl::BLEND));
            
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        
        Self {
            specs,
            storage: GlObjectsCache::default(),
            to_render: Vec::new(),
        }
    }
}
impl Renderer for GlRenderer {
    unsafe fn begin_batch(&mut self) -> Result<(), StdError>{
        
        Ok(())
    }
    unsafe fn end_batch(&mut self) -> Result<(), StdError>{
        
        Ok(())
    }
    unsafe fn draw(
        &mut self, 
        entity: &LgEntity
    ) -> Result<(), StdError> 
    {   
        let mesh = entity.mesh.borrow();
        let material = entity.material.borrow();

        self.storage.set_vao(mesh.uuid().clone());
        self.storage.set_material(&material)?;
        self.storage.set_entity_ubos(entity);
        
        let vao = self.storage.vaos.get(mesh.uuid()).unwrap();
        let gl_material = self.storage.materials.get(material.uuid()).unwrap();

        // Texture
        let tex_id = if gl_material.texture.is_some() {
           Some(material.texture().as_ref().unwrap().borrow().uuid().clone())
        } else {
            None
        };
        
        // gl_material.program.borrow().use_prog();
        gl_material.ubos
            .iter()
            .for_each(|u| {
                u.bind();
                u.bind_base();
                u.set_data_full(gl::STATIC_DRAW);
                u.unbind();
            });

        // Entities Ubos
        if let Some(ubos) = self.storage.entity_ubos.get_mut(entity.uuid()) {
            if ubos.len() == entity.uniforms.len() {
                for i in 0..ubos.len() {
                    ubos[i].specs = entity.uniforms[i].clone();
                }
            }
        
            ubos
                .iter()
                .for_each(|u| {
                    u.bind();
                    u.bind_base();
                    u.set_data_full(gl::DYNAMIC_DRAW);
                    u.unbind();
                });
        }
        // gl_material.program.borrow().unuse();

        vao.bind();
        vao.bind_buffers();
        mesh.set_attrib_locations(&vao, &mut gl_material.program.borrow_mut())?;
        vao.vertex_buffer().set_data(mesh.vertices(), gl::STATIC_DRAW);
        vao.index_buffer().set_data(mesh.indices(), gl::STATIC_DRAW);
        vao.unbind();
        
        self.to_render.push(DrawSpec {
            texture_uuid: tex_id,
            program_uuid: material.uuid().clone(),
            entity_uuid: entity.uuid().clone(),
            material_uuid: material.uuid().clone(),
            vao_uuid: mesh.uuid().clone(),
            indices_len: mesh.indices().len(),
        });

        Ok(())
    }
    unsafe fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError>{
        self.specs.gl_surface.resize(
            &self.specs.gl_context, 
            std::num::NonZeroU32::new(new_size.0).unwrap(),
            std::num::NonZeroU32::new(new_size.1).unwrap(),
        );

        gl_check!(gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32));

        Ok(())
    }
    
    unsafe fn render(&mut self) -> Result<(), StdError> {
        gl_check!(gl::ClearColor(0.5, 0.1, 0.2, 1.0));
        gl_check!(gl::Clear(gl::COLOR_BUFFER_BIT));
        
        self.to_render
            .iter()
            .for_each(|r| {
                let vao = self.storage.vaos.get(&r.vao_uuid).unwrap();
                let program = self.storage.programs.get(&r.program_uuid).unwrap();
                let material_ubos = self.storage.materials.get(&r.material_uuid).unwrap();

                let entity_ubos = self.storage.entity_ubos.get(&r.entity_uuid).unwrap();


                if let Some(tex_id) = &r.texture_uuid { 
                    let texture = self.storage.textures.get(&tex_id).unwrap();
                    texture.borrow().bind();
                };
                
                vao.bind();
                program.borrow().use_prog();
                material_ubos.ubos.iter().for_each(|u| u.bind_base());
                entity_ubos.iter().for_each(|u| u.bind_base());

                gl_check!(gl::DrawElements(
                    gl::TRIANGLES,
                    r.indices_len as i32,
                    gl::UNSIGNED_INT,
                    std::ptr::null()
                ));
                vao.unbind();
                program.borrow().unuse();
            });
        self.to_render.clear();

        self.specs.gl_surface.swap_buffers(&self.specs.gl_context)?;

        Ok(())
    }
    
    unsafe fn read_buffer<T: Clone>(&mut self, material: &LgMaterial, uniform: usize) -> Result<T, StdError> {
        gl_check!(gl::MemoryBarrier(gl::UNIFORM_BARRIER_BIT));
        if let Some(material) = self.storage.materials.get(&material.uuid()) {
            if let Some(ubo) = material.ubos.get(uniform) {
                ubo.bind();
                let data = ubo.buffer.borrow().map(gl::READ_ONLY) as *const T;
                let result = (*data).clone();
                ubo.buffer.borrow().unmap();

                return Ok(result);
            }
        }

        Err("Couldn't find buffer! (OpenGL)".into())
    }    

    unsafe fn destroy(&mut self) -> Result<(), StdError> {
        
        Ok(())
    }
}