pub(crate) mod macros;
pub(crate) mod gl_init;
pub(crate) mod gl_buffer;
pub(crate) mod gl_shader;
pub(crate) mod gl_texture;
pub(crate) mod gl_vertex_array;
pub(crate) mod gl_program;

#[derive(Clone, Debug)]
pub enum GlError {
    Error(String, String),
}
impl std::fmt::Display for GlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(api_e, desc) => write!(f, "GlError: Api: {}, descriprion: {}", api_e, desc),
        }
    }
}
impl std::error::Error for GlError {}

pub struct GlSpecs {
    pub gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    pub gl_display: glutin::display::Display, 
    pub gl_context: glutin::context::PossiblyCurrentContext,
}