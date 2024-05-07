extern crate gl;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use std::{collections::HashMap, ffi::CString};
use glutin::{display::GlDisplay, surface::GlSurface};
use sllog::info;
use crate::lg_core::renderer::uniform::LgUniform;
use crate::{gl_check, lg_core::{entity::LgEntity, lg_types::reference::Rfc, renderer::{material::LgMaterial, Renderer}}, StdError};
use super::opengl_uniform_buffer::GlUniformBuffer;
use super::{opengl_buffer::GlBuffer, opengl_mesh::GlMesh, opengl_program::GlProgram, opengl_shader::GlShader, opengl_texture::GlTexture, opengl_vertex_array::GlVertexArray, utils};

pub(crate) struct GlSpecs {
    pub(crate) gl_context: glutin::context::PossiblyCurrentContext,
    pub(crate) gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    pub(crate) gl_display: glutin::display::Display, 
}

#[derive(Default, Debug)]
struct GlStorage {
    shaders: HashMap<u128, Rfc<GlShader>>,
    textures: HashMap<u128, Rfc<GlTexture>>,
    programs: HashMap<u128, GlProgram>,
    vaos: HashMap<u128, GlVertexArray>,
    entity_ubos: HashMap<u128, Vec<GlUniformBuffer>>,
    material_ubos: HashMap<u128, Vec<GlUniformBuffer>>,
}

#[derive(Default)]
struct DrawSpec {
    texture_uuid: Option<u128>,
    vao_uuid: u128,
    indices_len: usize,
    program_uuid: u128,
    entity_uuid: u128,
}

pub(crate) struct GlRenderer {
    specs: GlSpecs,
    storage: GlStorage,
    to_render: Vec<DrawSpec>
}
impl GlRenderer {
    pub fn new(specs: GlSpecs) -> Self {
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            specs.gl_display.get_proc_address(symbol.as_c_str()).cast()
        });
        
        unsafe {
            gl_check!(gl::Enable(gl::DEBUG_OUTPUT));
            gl_check!(gl::DebugMessageCallback(Some(utils::debug_callback), std::ptr::null()));
            
            gl_check!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
            gl_check!(gl::Enable(gl::BLEND));
            
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        
        Self {
            specs,
            storage: GlStorage::default(),
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
        let mesh = entity.mesh().borrow();
        let material = entity.material().borrow();

        // Shaders
        let mut shaders = Vec::new();
        for s in material.shaders() {
            shaders.push(match self.storage.shaders.entry(s.borrow().uuid().get_value()) {
                std::collections::hash_map::Entry::Occupied(shdr) => shdr.into_mut(),
                std::collections::hash_map::Entry::Vacant(entry) => {
                    info!("[OpenGL]: GlShader: {}, being chached!", s.borrow().uuid().get_value());
                    entry.insert(Rfc::new(GlShader::new(s.clone())?))
                },
            }.clone());
        }

        // Texture
        let (_gl_tex, gl_tex_id) = if let Some(texture) = material.texture() {
            (Some(match self.storage.textures.entry(texture.borrow().uuid().get_value()) {
                std::collections::hash_map::Entry::Occupied(tex) => tex.into_mut(),
                std::collections::hash_map::Entry::Vacant(entry) => {
                    info!("[OpenGL]: GlTexture, FROM LGTEXTURE: {}, being chached!", texture.borrow().uuid().get_value());
                    let texture = Rfc::new(GlTexture::new(texture.clone()));

                    texture.borrow().bind();
                    texture.borrow().load();

                    entry.insert(texture)
                },
            }), Some(texture.borrow().uuid().get_value()))
        } else {
            (None, None)
        };

        // Shader Program
        let program = match self.storage.programs.entry(material.uuid().get_value()) {
            std::collections::hash_map::Entry::Occupied(prg) => prg.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let program = GlProgram::builder()
                    .add_shaders(shaders)
                    .build()?;
                
                info!("[OpenGL]: GlProgram, FROM MATERIAL: {}, being cached!", material.uuid().get_value());
                
                entry.insert(program)
            }
        };
        if self.storage.material_ubos.get(&material.uuid().get_value()).is_none() {
            program.bind();
            let ubos: Vec<GlUniformBuffer> = material.uniforms
                .iter()
                .map(|u| {
                    let usage = match u.u_type() {
                        crate::lg_core::renderer::uniform::LgUniformType::STRUCT => gl::UNIFORM_BUFFER,
                        crate::lg_core::renderer::uniform::LgUniformType::STORAGE_BUFFER => gl::SHADER_STORAGE_BUFFER,
                        _ => gl::UNIFORM_BUFFER
                    };

                    let buffer = Rfc::new(GlBuffer::new(usage));
                    let ubo = GlUniformBuffer::new(buffer, u.binding());
                    ubo.bind();
                    ubo.bind_base();
                    ubo.buffer.borrow().set_data_full(
                        u.data.size(), 
                        u.data(), 
                        gl::STATIC_DRAW
                    );
                    ubo.unbind();
                    info!("[OpenGL]: Uniform Buffer 
                        name: {}
                        type: {:?}
                        set: {}
                        binding: {}", u.name(), u.u_type(), u.set(), u.binding()
                    );
                    ubo
                }).collect();
            program.unbind();

            self.storage.material_ubos.insert(material.uuid().get_value(), ubos);
        }

        program.bind();
        if let Some(ubos) = self.storage.entity_ubos.get(&entity.uuid().get_value()) {
            // Update buffers
            for (i, ubo) in ubos.iter().enumerate() {
                let entity_uniform = &entity.uniforms[i];

                ubo.bind();
                ubo.bind_base();
                ubo.buffer.borrow().set_data_full(
                    entity_uniform.data.size(), 
                    entity_uniform.data(), 
                    gl::STATIC_DRAW
                );
                ubo.unbind();
            }
                
        } else {
            let ubos: Vec<GlUniformBuffer> = entity.uniforms
                .iter()
                .map(|u| {
                    let usage = match u.u_type() {
                        crate::lg_core::renderer::uniform::LgUniformType::STRUCT => gl::UNIFORM_BUFFER,
                        crate::lg_core::renderer::uniform::LgUniformType::STORAGE_BUFFER => gl::SHADER_STORAGE_BUFFER,
                        _ => gl::UNIFORM_BUFFER
                    };

                    let buffer = Rfc::new(GlBuffer::new(usage));
                    let ubo = GlUniformBuffer::new(buffer, u.binding());
                    ubo.bind();
                    ubo.bind_base();
                    ubo.buffer.borrow().set_data_full(
                        u.data.size(), 
                        u.data(), 
                        gl::STATIC_DRAW
                    );
                    ubo.unbind();
                    info!("[OpenGL]: Uniform Buffer 
                        name: {}
                        type: {:?}
                        set: {}
                        binding: {}", u.name(), u.u_type(), u.set(), u.binding()
                    );
                    ubo
                }).collect();

            self.storage.entity_ubos.insert(entity.uuid().get_value(), ubos);
        }
        program.unbind();

        // One VertexArray per Mesh
        let _vao = match self.storage.vaos.entry(mesh.uuid().get_value()) {
            std::collections::hash_map::Entry::Occupied(vao) => vao.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let vao = entry.insert(GlVertexArray::new());
                vao.bind();
                vao.bind_buffers();
                mesh.set_attrib_locations(vao, program)?;
                
                // Update the vertex_buffer and the index_buffer.
                vao.vertex_buffer().set_data(mesh.vertices(), gl::STATIC_DRAW);
                vao.index_buffer().set_data(mesh.indices(), gl::STATIC_DRAW);
                vao.unbind();

                info!("[OpenGL]: VAO, FROM MESH: {}, being cached!", mesh.uuid().get_value());

                vao
            }
        };
        
        self.to_render.push(DrawSpec {
            texture_uuid: gl_tex_id,
            program_uuid: material.uuid().get_value(),
            entity_uuid: entity.uuid().get_value(),
            vao_uuid: mesh.uuid().get_value(),
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
                let material_ubos = self.storage.material_ubos.get(&r.program_uuid).unwrap();

                let entity_ubos = match self.storage.entity_ubos.get(&r.entity_uuid) {
                    Some(val) => val.clone(),
                    None => Vec::new()
                };


                if let Some(tex_id) = r.texture_uuid { 
                    let texture = self.storage.textures.get(&tex_id).unwrap();
                    texture.borrow().bind();
                };
                
                vao.bind();
                program.bind();
                material_ubos.iter().for_each(|u| u.bind_base());
                entity_ubos.iter().for_each(|u| u.bind_base());

                gl_check!(gl::DrawElements(
                    gl::TRIANGLES,
                    r.indices_len as i32,
                    gl::UNSIGNED_INT,
                    std::ptr::null()
                ));
                vao.unbind();
                program.unbind();
            });
        self.to_render.clear();

        self.specs.gl_surface.swap_buffers(&self.specs.gl_context)?;

        Ok(())
    }
    
    unsafe fn read_buffer<T: Clone>(&mut self, material: &LgMaterial, uniform: usize) -> Result<T, StdError> {
        gl_check!(gl::MemoryBarrier(gl::UNIFORM_BARRIER_BIT));
        if let Some(ubos) = self.storage.material_ubos.get(&material.uuid().get_value()) {
            if let Some(ubo) = ubos.get(uniform) {
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