use crate::StdError;

use self::{material::LgMaterial, mesh::LgMesh, opengl::{opengl_renderer::{GlRenderer, GlSpecs}, opengl_vertex::GlVertex}, vertex::LgVertex};

pub mod vertex;
pub mod mesh;
pub mod material;
pub mod texture;
pub mod shader;

pub(crate) mod opengl;

pub struct LgRenderer {
    opengl: Option<GlRenderer>,
    // vulkan: Option<VkRenderer>
}
impl LgRenderer {
    pub(crate) fn opengl(specs: GlSpecs) -> Self {
        Self {
            opengl: Some(GlRenderer::new(specs))
        }
    }
    /* pub(crate) fn vulkan() -> Self {
        todo!()
    } */
    pub(crate) fn get(&self) -> Result<&impl Renderer, StdError> {
        match &self.opengl {
            Some(gl) => Ok(gl),
            None => Err("No Renderer available! Make sure to set env var as: OPEN_GL = 1, or VULKAN = 1".into()) // Match Vulkan
        }
    }
    pub(crate) fn get_mut(&mut self) -> Result<&mut impl Renderer, StdError> {
        match &mut self.opengl {
            Some(gl) => Ok(gl),
            None => Err("No Renderer available! Make sure to set env var as: OPEN_GL = 1, or VULKAN = 1".into()) // Match Vulkan
        }
    }
}
impl Renderer for LgRenderer {
    unsafe fn begin_batch(&mut self) -> Result<(), StdError> {
        self.get_mut()?.begin_batch()
    }

    unsafe fn end_batch(&mut self) -> Result<(), StdError> {
        self.get_mut()?.end_batch()
    }

    unsafe fn draw<T: LgVertex + GlVertex>(&mut self, mesh: &LgMesh<T>, material: &LgMaterial) -> Result<(), StdError> {
        self.get_mut()?.draw(mesh, material)
    }

    unsafe fn render(&mut self) -> Result<(), StdError> {
        self.get_mut()?.render()
    }

    unsafe fn destroy(&mut self) -> Result<(), StdError> {
        self.get_mut()?.destroy()
    }

    unsafe fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError> {
        self.get()?.resize(new_size)
    }
}

pub(crate) trait Renderer {
    unsafe fn begin_batch(&mut self) -> Result<(), StdError>;
    unsafe fn end_batch(&mut self) -> Result<(), StdError>;
    unsafe fn draw<T: LgVertex + GlVertex>(&mut self, mesh: &LgMesh<T>, material: &LgMaterial) -> Result<(), StdError>;
    unsafe fn render(&mut self) -> Result<(), StdError>;
    unsafe fn destroy(&mut self) -> Result<(), StdError>;
    unsafe fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError>;
}