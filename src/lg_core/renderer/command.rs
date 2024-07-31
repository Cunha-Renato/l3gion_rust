use crate::lg_core::uuid::UUID;
use super::{render_target::RenderTargetSpecs, texture::Texture, uniform::Uniform, vertex::VertexInfo};

#[derive(Debug, Clone)]
pub struct SendInstanceDrawData {
    pub mesh: UUID,
    pub material: UUID,
    pub instance_data: (VertexInfo, Vec<u8>),
    pub uniforms: Vec<Uniform>,
}

#[derive(Debug, Clone)]
pub struct SendDrawData {
    pub mesh: UUID,
    pub material: UUID,
    pub uniforms: Vec<Uniform>,
    pub textures: Vec<TextureOption>,
}

#[derive(Debug, Clone)]
pub enum TextureOption {
    UUID(UUID),
    LG_TEXTURE(Texture),
    GL_TEXTURE(gl::types::GLuint),
}

#[derive(Debug, Clone)]
pub enum SendRendererCommand {
    SET_VSYNC(bool),
    GET_VSYNC,

    SET_SIZE((u32, u32)),

    CREATE_NEW_RENDER_PASS(String, RenderTargetSpecs),
    RESIZE_RENDER_PASS(String, (i32, i32)),
    GET_PASS_COLOR_TEXTURE_GL(String),
    GET_PASS_DEPTH_TEXTURE_GL(String),
    GET_PASS_COLOR_TEXTURE_LG(String),
    GET_PASS_DEPTH_TEXTURE_LG(String),
    BEGIN_RENDER_PASS(String),

    SEND_INSTANCE_DATA(SendInstanceDrawData),
    DRAW_INSTANCED,
    SEND_DRAW_DATA(SendDrawData),
    
//========================== INTERNAL ====================================== 
    _DRAW_IMGUI,
    _DRAW_BACKBUFFER,

    _INIT,
    _BEGIN,
    _END,
    _SHUTDOWN,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReceiveRendererCommand {
    VSYNC(bool),
    RENDER_TARGET_COLOR_TEXTURE_LG(Texture, String),
    RENDER_TARGET_DEPTH_TEXTURE_LG(Texture, String),
    RENDER_TARGET_COLOR_TEXTURE_GL(gl::types::GLuint, String),
    RENDER_TARGET_DEPTH_TEXTURE_GL(gl::types::GLuint, String),

//========================== INTERNAL ====================================== 
    _IMGUI_DONE,
    _RESIZE_DONE,
    _SHUTDOWN_DONE,
    _END_DONE,
}