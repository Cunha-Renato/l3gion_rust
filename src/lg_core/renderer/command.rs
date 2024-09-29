use crate::lg_core::uuid::UUID;
use super::{texture::Texture, uniform::Uniform, vertex::VertexInfo};

#[derive(Debug, Clone)]
pub struct SendInstanceDrawData {
    pub mesh: UUID,
    pub material: UUID,
    pub instance_data: (VertexInfo, Vec<u8>),
    pub uniforms: Vec<Uniform>,
}

#[derive(Debug)]
pub struct SendDrawData {
    pub mesh: UUID,
    pub material: UUID,
    pub uniforms: Vec<Uniform>,
    pub textures: Vec<TextureOption>,
}

#[derive(Debug)]
pub enum TextureOption {
    UUID(UUID),
    LG_TEXTURE(Texture),
    GL_TEXTURE(gl::types::GLuint),
    PREVIOUS_PASS,
}

#[derive(Debug, PartialEq)]
pub(super) enum RendererCommand {
    _IMGUI_DONE,
    _RESIZE_DONE,
    _SHUTDOWN_DONE,
    _END_DONE,
}