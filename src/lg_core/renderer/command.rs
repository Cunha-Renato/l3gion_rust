use crate::lg_core::{glm, uuid::UUID, window::LgWindow};
use super::{render_target::{RenderTarget, RenderTargetSpecs}, texture::Texture, uniform::Uniform, vertex::VertexInfo};

#[derive(Debug, Clone)]
pub struct SendDrawData {
    pub mesh: UUID,
    pub material: UUID,
    pub instance_data: (VertexInfo, Vec<u8>),
    pub uniforms: Vec<Uniform>,
}

#[derive(Debug, Clone)]
pub enum SendRendererCommand {
    SET_VSYNC(bool),
    GET_VSYNC,

    SET_SIZE((u32, u32)),

    CREATE_RENDER_TARGET(RenderTargetSpecs),

    SET_CLEAR_COLOR(glm::Vec4),
    SET_CLEAR_DEPTH(f32),

    BEGIN_RENDER_PASS(RenderTargetSpecs),
    SEND_DATA(SendDrawData),
    DRAW_INSTANCED,
    DRAW,
    
//========================== INTERNAL ====================================== 
    _INIT,
    _SHUTDOWN,
    _BEGIN,
    _END,
}

#[derive(Debug, PartialEq)]
pub enum ReceiveRendererCommand {
    VSYNC(bool),
    RENDER_TARGET(RenderTarget),

//========================== INTERNAL ====================================== 
    _SHUTDOWN_DONE,
    _END_DONE,
}