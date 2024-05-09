use std::collections::HashMap;

use sllog::info;

use crate::{lg_core::{entity::LgEntity, lg_types::reference::Rfc, renderer::{material::LgMaterial, shader::LgShader, texture::LgTexture, uniform::LgUniform}, uuid::UUID}, StdError};

use super::{opengl_buffer::GlBuffer, opengl_material::GlMaterial, opengl_program::GlProgram, opengl_shader::GlShader, opengl_texture::GlTexture, opengl_uniform_buffer::GlUniformBuffer, opengl_vertex_array::GlVertexArray};

#[derive(Default)]
pub(crate) struct GlObjectsCache {
    pub(crate) shaders: HashMap<UUID, Rfc<GlShader>>,
    pub(crate) textures: HashMap<UUID, Rfc<GlTexture>>,
    pub(crate) programs: HashMap<UUID, Rfc<GlProgram>>,
    pub(crate) vaos: HashMap<UUID, GlVertexArray>,
    pub(crate) materials: HashMap<UUID, GlMaterial>,
    pub(crate) entity_ubos: HashMap<UUID, Vec<GlUniformBuffer>>,
}
impl GlObjectsCache {
    pub(crate) unsafe fn get_shader(&mut self, shader: &Rfc<LgShader>) -> Result<Rfc<GlShader>, StdError> 
    {
        let shader = match self.shaders.entry(shader.borrow().uuid().clone()) {
            std::collections::hash_map::Entry::Occupied(sdr) => sdr.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Rfc::new(GlShader::new(shader.clone())?))
            },
        };
        
        Ok(shader.clone())
    }
    pub(crate) unsafe fn get_texture(&mut self, texture: &Rfc<LgTexture>) -> Rfc<GlTexture> {
        let texture = match self.textures.entry(texture.borrow().uuid().clone()) {
            std::collections::hash_map::Entry::Occupied(tex) => tex.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let texture = Rfc::new(GlTexture::new(texture.clone()));
                texture.borrow().bind();
                texture.borrow().load();

                entry.insert(texture)
            },
        };
        
        texture.clone()
    }
    pub(crate) unsafe fn get_program(&mut self, uuid: UUID, shaders: &[Rfc<GlShader>]) -> Result<Rfc<GlProgram>, StdError> {
        let program = match self.programs.entry(uuid) {
            std::collections::hash_map::Entry::Occupied(prog) => prog.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut program = GlProgram::new();
                program.set_shaders(Vec::from(shaders));
                program.link()?;
                
                entry.insert(Rfc::new(program))
            },
        };
        
        Ok(program.clone())
    }
    pub(crate) unsafe fn set_entity_ubos(&mut self, entity: &LgEntity) {
        match self.entity_ubos.entry(entity.uuid().clone()) {
            std::collections::hash_map::Entry::Occupied(_) => (),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let ubos = get_uniform_buffers(&entity.uniforms);
                
                entry.insert(ubos);
            }
        };
    }
    pub(crate) unsafe fn set_material(&mut self, material: &LgMaterial) -> Result<(), StdError> 
    {
        let empty = self.materials.get(material.uuid()).is_some();

        if !empty {
            let mut gl_material = GlMaterial::new();

            // Adding the shaders
            for s in material.shaders() {
                gl_material.shaders.push(self.get_shader(s)?);
            }

            // Adding the texture
            gl_material.texture = if let Some(texture) = material.texture() {
                Some(self.get_texture(texture))
            } else {
                None
            };

            // Adding the program
            gl_material.program = self.get_program(
                material.uuid().clone(), 
                &gl_material.shaders
            )?;
            
            // Adding the uniform buffers
            gl_material.ubos = get_uniform_buffers(&material.uniforms);

            self.materials.insert(material.uuid().clone(), gl_material);
        }
        
        Ok(())
    }
    pub(crate) unsafe fn set_vao(&mut self, uuid: UUID) {
        match self.vaos.entry(uuid) {
            std::collections::hash_map::Entry::Occupied(vao) => vao.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(GlVertexArray::new())
            },
        };
    }
}
unsafe fn get_uniform_buffers(uniforms: &[LgUniform]) -> Vec<GlUniformBuffer> {
    uniforms.iter()
        .map(|u| {
            let usage = match u.u_type() {
                crate::lg_core::renderer::uniform::LgUniformType::STRUCT => gl::UNIFORM_BUFFER,
                crate::lg_core::renderer::uniform::LgUniformType::STORAGE_BUFFER => gl::SHADER_STORAGE_BUFFER,
                crate::lg_core::renderer::uniform::LgUniformType::COMBINED_IMAGE_SAMPLER => gl::SAMPLER_2D,
            };
            
            info!("[OpenGL]: Uniform Buffer 
                name: {}
                type: {:?}
                set: {}
                binding: {}", u.name(), u.u_type(), u.set(), u.binding()
            );

            GlUniformBuffer::new(Rfc::new(GlBuffer::new(usage)), u.clone())
        })
        .collect()
}
