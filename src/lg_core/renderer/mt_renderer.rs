pub enum SendRendererCommand {
    V_SYNC(bool),
    SET_UNIFORM(), // TODO: Use Material Uniforms
    RESIZE((u32, u32)),
    
}

pub enum ReceiveRendererCommand {
    V_SYNC(bool),
}

pub(crate) enum CoreSendRendererCommand {
    INIT,
    SHUTDOWN,
    BEGIN,
    END,
}