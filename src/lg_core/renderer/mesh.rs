use std::hash::Hash;
use crate::{lg_core::uuid::UUID, StdError};

use super::{opengl::gl_vertex_array::GlVertexArray, vertex::{LgVertex, Vertex}};


#[derive(Debug)]
pub struct Mesh {
    uuid: UUID,
    name: String, // TODO: Placeholder
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    
    pub(crate) gl_vao: Option<GlVertexArray>
}
impl Mesh {
    pub fn new(
        uuid: UUID,
        name: &str,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Self
    {
        Self {
            uuid,
            name: String::from(name),
            vertices,
            indices,
            
            gl_vao: None,
        }
    }
    
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
    
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

// Public(crate)
impl Mesh {
    pub(crate) fn init_opengl(&mut self) -> Result<(), StdError> {
        if self.gl_vao.is_some() { return Ok(()); }

        let vao = GlVertexArray::new()?;
        vao.bind()?;

        // Vertices
        let vertex_info = self.vertices()[0].vertex_info();
        vao.vertex_buffer().bind()?;
        vao.vertex_buffer().set_data(self.vertices(), gl::STATIC_DRAW)?;
        for info in &vertex_info.gl_info {
            vao.set_attribute(info.0, info.1, vertex_info.stride, info.2)?;
        }
        
        // Indices
        vao.index_buffer().bind()?;
        vao.index_buffer().set_data(self.indices(), gl::STATIC_DRAW)?;
        vao.unbind_buffers()?;
        vao.unbind()?;
        
        self.gl_vao = Some(vao);

        Ok(())
    }
}
impl Hash for Mesh {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}