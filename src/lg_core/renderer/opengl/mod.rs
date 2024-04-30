pub(crate) mod opengl_init;
pub(crate) mod opengl_renderer;
pub(crate) mod opengl_shader;
pub(crate) mod opengl_buffer;
pub(crate) mod opengl_texture;
pub(crate) mod utils;
pub mod opengl_program; // Leaking, for now I don't know how to not
pub mod opengl_vertex_array;
pub mod opengl_vertex;