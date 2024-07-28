use std::{collections::HashMap, ffi::CString, sync::{mpsc::{Receiver, Sender}, Arc, Mutex}};
use command::{ReceiveRendererCommand, SendDrawData, SendInstanceDrawData, SendRendererCommand};
use glutin::{display::GlDisplay, surface::GlSurface};
use opengl::{gl_buffer::GlBuffer, gl_init::{init_opengl, init_window}, gl_program::GlProgram, gl_shader::GlShader, gl_texture::GlTexture, gl_vertex_array::GlVertexArray, GlSpecs};
use render_target::RenderTarget;
use sllog::{error, info, warn};
use texture::Texture;
use uniform::Uniform;
use vertex::{LgVertex, VertexInfo};
use crate::StdError;
use super::{am::AssetManager, uuid::UUID, window::LgWindow};

pub mod mesh;
pub mod material;
pub mod texture;
pub mod shader;
pub mod uniform;
pub mod buffer;
pub mod vertex;
pub mod render_target;
pub mod command;
mod opengl;

const FINAL_PASS_MESH: UUID = UUID::from_u128(252411435688744967694609164507863584779);
const FINAL_PASS_MATERIAL: UUID = UUID::from_u128(315299335240398778209169027697428014904);

pub struct CreationWindowInfo<'a> {
    pub event_loop: Option<&'a winit::event_loop::EventLoop<()>>,
    pub title: String,
    pub width: u32,
    pub height: u32,
}

pub struct Renderer {
    sender: Sender<SendRendererCommand>,
    receiver: Receiver<ReceiveRendererCommand>,
    last_frame: Vec<ReceiveRendererCommand>,
}
impl Renderer {
    pub fn send(&self, msg: SendRendererCommand) {
        self.sender.send(msg).unwrap();
    }
    
    /// Will block until the receiver is empty.
    pub fn send_and_wait(&mut self, msg: SendRendererCommand) {
        self.send(msg);
        
        while let Ok(msg) = self.receiver.try_recv() {
            self.last_frame.push(msg);
        }
    }
    
    // Will always wait(block)
    pub fn get_pass_color_texture_gl(&mut self, name: String) -> Option<gl::types::GLuint> {
        while let Ok(msg) = self.receiver.recv() {
            match &msg {
                ReceiveRendererCommand::RENDER_TARGET_COLOR_TEXTURE_GL(tex, tex_name) => {
                    if name == *tex_name {
                        return Some(*tex);
                    }
                    
                    self.last_frame.push(msg);
                }
                _ => self.last_frame.push(msg),
            }
        }
        
        None
    }    
    /// Will always wait(block).
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.send(SendRendererCommand::SET_SIZE(new_size));

        while let Ok(msg) = self.receiver.recv() {
            match msg {
                ReceiveRendererCommand::_RESIZE_DONE => break,
                _ => self.last_frame.push(msg),
            }
        }
    }

    /// Will always wait(block).
    pub fn end(&mut self) {
        self.send(SendRendererCommand::_END);
        self.last_frame.clear();

        while let Ok(msg) = self.receiver.recv() {
            match msg {
                ReceiveRendererCommand::_END_DONE => break,
                _ => self.last_frame.push(msg),
            }
        }
    }
    
    /// Will always wait(block).
    pub fn shutdown(&mut self) {
        self.send(SendRendererCommand::_SHUTDOWN);
        
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                ReceiveRendererCommand::_SHUTDOWN_DONE => break,
                _ => (),
            }
        }
    }
}
impl Renderer {
    pub(crate) fn new(
        window_info: &CreationWindowInfo, 
        asset_manager: Arc<Mutex<AssetManager>>,
    ) -> Result<(Self, LgWindow), StdError> 
    {
        let (w_sender, w_receiver) = std::sync::mpsc::channel();

        let (s_sender, s_receiver) = std::sync::mpsc::channel();
        let (r_sender, r_receiver) = std::sync::mpsc::channel();

        let (window, gl_config) = init_window(window_info)?;
        std::thread::spawn(move || {
            let (window, specs) = init_opengl(window, gl_config).unwrap();
            w_sender.send(window).unwrap();
            let mut r_core = RendererCore::new(specs, Arc::clone(&asset_manager)).unwrap();
            
            loop {
                while let Ok(msg) = s_receiver.recv() {
                    unsafe { match msg {
                        SendRendererCommand::SET_VSYNC(val) => r_core.set_vsync(val),
                        SendRendererCommand::GET_VSYNC => r_sender.send(ReceiveRendererCommand::VSYNC(r_core.vsync)).unwrap(),
                        SendRendererCommand::SET_SIZE(new_size) => {
                            r_core.resize(new_size);
                            r_sender.send(ReceiveRendererCommand::_RESIZE_DONE).unwrap();
                        },

                        SendRendererCommand::CREATE_NEW_RENDER_PASS(name, specs) => {
                            let target = RenderTarget::new(specs);
                            r_core.render_passes.insert(name, target);
                        },
                        SendRendererCommand::RESIZE_RENDER_PASS(name, new_size) => {
                            let mut specs = r_core.render_passes.get_mut(&name).unwrap().specs.clone();
                            specs.viewport = (0, 0, new_size.0, new_size.1);
                            r_core.render_passes.insert(name.clone(), RenderTarget::new(specs));
                        },
                        SendRendererCommand::GET_PASS_COLOR_TEXTURE_GL(name) => {
                            let target = r_core.render_passes.get(&name).unwrap();
                            r_sender.send(ReceiveRendererCommand::RENDER_TARGET_COLOR_TEXTURE_GL(target.color_texture, name)).unwrap();
                        },
                        SendRendererCommand::GET_PASS_DEPTH_TEXTURE_GL(name) => {
                            let target = r_core.render_passes.get(&name).unwrap();
                            r_sender.send(ReceiveRendererCommand::RENDER_TARGET_DEPTH_TEXTURE_GL(target.depth_texture.unwrap(), name)).unwrap();
                        },
                        SendRendererCommand::GET_PASS_COLOR_TEXTURE_LG(name) => {
                            let target = r_core.render_passes.get(&name).unwrap();
                            let specs = &target.specs.color_texture_specs;
                            gl::BindTexture(gl::TEXTURE_2D, target.color_texture);
                            
                            let (mut width, mut height) = (0, 0);
                            gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut width);
                            gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut height);

                            let mut pixels = vec![0u8; (width * height) as usize];
                            gl::GetTexImage(
                                gl::TEXTURE_2D, 
                                0, 
                                specs.tex_format.to_opengl(),
                                specs.tex_type.to_opengl(),
                                pixels.as_mut_ptr() as *mut gl::types::GLvoid
                            );
                            let size = (pixels.len() * std::mem::size_of::<u8>()) as u64;

                            let texture = Texture::construct(
                                UUID::generate(), 
                                &std::format!("{}_color_texture", name), 
                                width as u32, 
                                height as u32, 
                                pixels, 
                                size, 
                                0, 
                                specs.clone()
                            );
                            
                            r_sender.send(ReceiveRendererCommand::RENDER_TARGET_COLOR_TEXTURE_LG(texture, name)).unwrap();
                        },
                        SendRendererCommand::GET_PASS_DEPTH_TEXTURE_LG(name) => {
                            let target = r_core.render_passes.get(&name).unwrap();
                            
                            todo!();
                        },
                        SendRendererCommand::BEGIN_RENDER_PASS(name) => {
                            let target = r_core.render_passes.get(&name).unwrap();
                            r_core.set_render_target(target.framebuffer, target.specs.viewport, target.specs.depth_test);
                            r_core.active_pass = name;
                        },

                        SendRendererCommand::SEND_INSTANCE_DATA(dd) => r_core.send_data(dd).unwrap(),
                        SendRendererCommand::DRAW_INSTANCED => r_core.draw_instanced().unwrap(),
                        SendRendererCommand::SEND_DRAW_DATA(dd) => r_core.draw(dd).unwrap(),
                        SendRendererCommand::_INIT => {
                            asset_manager.lock()
                                .unwrap()
                                .init()
                                .unwrap();
                        },
                        SendRendererCommand::_SHUTDOWN => r_sender.send(ReceiveRendererCommand::_SHUTDOWN_DONE).unwrap(),
                        SendRendererCommand::_BEGIN => todo!(),
                        SendRendererCommand::_END => {
                            r_sender.send(ReceiveRendererCommand::_END_DONE).unwrap();
                            r_core.swap_buffers().unwrap();
                        },
                    }
                }}
            }
        });
        
        let window = LgWindow::new(w_receiver.recv()?);

        Ok((
            Self {
                sender: s_sender,
                receiver: r_receiver,
                last_frame: Vec::new(),
            },
            window,
        ))
    }
}

#[derive(Debug)]
struct DrawData {
    uniforms: Vec<GlBuffer>,
    textures: Vec<GlTexture>,
    instance_data: (u32, VertexInfo, Vec<u8>),

    program: GlProgram,
    vao: GlVertexArray,
    indices_len: usize,
    first_location: u32,
}

struct RendererCore {
    asset_manager: Arc<Mutex<AssetManager>>,
    gl_specs: GlSpecs,

    // Material, Mesh, Data
    draw_data: HashMap<UUID, HashMap<UUID, DrawData>>,
    render_passes: HashMap<String, RenderTarget>,
    active_pass: String,
    
    vsync: bool
}
impl RendererCore {
    fn new(specs: GlSpecs, asset_manager: Arc<Mutex<AssetManager>>) -> Result<Self, StdError> {
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            specs.gl_display.get_proc_address(symbol.as_c_str()).cast()
        });
        
        unsafe {
            // Debug
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());

            // Depth, Blend
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
        }

        asset_manager.lock().unwrap().create_material("post_processing_pass", vec![], vec![
            "assets\\shaders\\src\\final_pass_v.vert".to_string(), 
            "assets\\shaders\\src\\post_processing_f.frag".to_string(), 
        ])?;

        Ok(Self {
            asset_manager,
            gl_specs: specs,
            draw_data: HashMap::new(),
            render_passes: HashMap::new(),
            active_pass: String::new(),
            vsync: false,
        })
    }
    
    fn set_vsync(&mut self, op: bool) {
        self.vsync = op;
        if op {
            self.gl_specs.gl_surface.set_swap_interval(&self.gl_specs.gl_context, glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap())).unwrap();
        } else {
            self.gl_specs.gl_surface.set_swap_interval(&self.gl_specs.gl_context, glutin::surface::SwapInterval::DontWait).unwrap();
        } 
    }

    unsafe fn set_program(&self, material: UUID) -> Result<GlProgram, StdError> {
        let mut am = self.asset_manager.lock().unwrap();
        let shaders = am.get_material(&material)?
            .as_ref()
            .unwrap()
            .shaders()
            .to_vec();

        let shaders = vec![
            am.get_shader(&shaders[0])?.as_ref().unwrap(),
            am.get_shader(&shaders[1])?.as_ref().unwrap(),
        ];
        
        let gl_shaders = vec![
            GlShader::new(shaders[0].src_code(), shaders[0].stage().to_gl_stage())?,
            GlShader::new(shaders[1].src_code(), shaders[1].stage().to_gl_stage())?,
        ];

        let mut program = GlProgram::new()?;
        program.set_shaders(gl_shaders)?;
        program.link()?;
        program.use_prog()?;
        
        Ok(program)
    }

    unsafe fn set_vao(&self, mesh: UUID) -> Result<GlVertexArray, StdError> {
        let mut am = self.asset_manager.lock().unwrap();
        let mesh = am.get_mesh(&mesh)?.as_ref().unwrap();

        let vao = GlVertexArray::new()?;
        vao.bind()?;

        // Vertices
        let vertex_info = mesh.vertices()[0].vertex_info();
        vao.vertex_buffer().bind()?;
        vao.vertex_buffer().set_data(mesh.vertices(), gl::STATIC_DRAW)?;
        for info in &vertex_info.gl_info {
            vao.set_attribute(info.0, info.1, vertex_info.stride, info.2)?;
        }
        
        // Indices
        vao.index_buffer().bind()?;
        vao.index_buffer().set_data(mesh.indices(), gl::STATIC_DRAW)?;
        vao.unbind_buffers()?;

        Ok(vao)
    }

    unsafe fn set_uniforms(&self, uniforms: &[Uniform]) -> Result<Vec<GlBuffer>, StdError> {
        let mut gl_ubos = Vec::with_capacity(uniforms.len());

        for u in uniforms {
            let buffer = GlBuffer::new(u.u_type().to_opengl())?;
            
            buffer.bind()?;
            buffer.bind_base(u.binding())?;
            buffer.set_data_full(
                u.data_size(), 
                u.get_raw_data(),
                gl::STATIC_DRAW
            )?;
            buffer.unbind()?;

            gl_ubos.push(buffer);
        }
        
        Ok(gl_ubos)
    }

    unsafe fn set_textures(&self, textures: &[UUID]) -> Result<Vec<GlTexture>, StdError> {
        let mut am = self.asset_manager.lock().unwrap();
        let mut gl_textures = Vec::with_capacity(textures.len());

        for tex in textures.iter().enumerate() {
            let texture = am.get_texture(tex.1)?.as_ref().unwrap();
            let gl_tex = GlTexture::new()?;
            gl_tex.bind(tex.0 as u32)?;
            gl_tex.load(&texture)?;
            
            gl::Uniform1i(tex.0 as i32, tex.0 as i32);
            gl_tex.unbind()?;
            
            gl_textures.push(gl_tex);
        }

        Ok(gl_textures)
    }

    unsafe fn draw(&self, dd: SendDrawData) -> Result<(), StdError> {
        let mesh = self.asset_manager
            .lock()
            .unwrap()
            .get_mesh(&dd.mesh)?;
        
        let program = self.set_program(dd.material.clone())?;
        let vao = self.set_vao(dd.mesh.clone())?;
        let ubos = self.set_uniforms(&dd.uniforms)?;
        
        program.use_prog()?;
        vao.bind()?;
        vao.vertex_buffer().bind()?;
        vao.index_buffer().bind()?;
        
        for ubo in &ubos {
            ubo.bind()?;
        }

        for (_location, tex_op) in dd.textures.iter().enumerate() {
            match tex_op {
                command::TextureOption::UUID(_) => todo!(),
                command::TextureOption::LG_TEXTURE(_) => todo!(),
                command::TextureOption::GL_TEXTURE(tex) => {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, *tex);
                    gl::Uniform1i(0, 0);
                },
            }
        }
        
        gl::DrawElements(gl::TRIANGLES, mesh.as_ref().unwrap().indices().len() as i32, gl::UNSIGNED_INT, std::ptr::null());
        
        vao.unbind_buffers()?;
        vao.unbind()?;
        program.unuse()?;

        Ok(())
    }

    unsafe fn draw_instanced(&mut self) -> Result<(), StdError> {
        for (_, dd) in &self.draw_data {
            let instance_vbo = GlBuffer::new(gl::ARRAY_BUFFER)?;
            for (_, d) in dd {
                d.program.use_prog()?;
                d.vao.bind()?;
                d.vao.vertex_buffer().bind()?;
                d.vao.index_buffer().bind()?;
                
                let last_location = d.first_location;
                instance_vbo.bind()?;
                instance_vbo.set_data(&d.instance_data.2, gl::STATIC_DRAW)?;
                
                for info in &d.instance_data.1.gl_info {
                    let location = info.0 + last_location + 1;
                    d.vao.set_attribute(location, info.1, d.instance_data.1.stride, info.2)?;
                    
                    gl::VertexAttribDivisor(location, 1);
                }

                for ubo in &d.uniforms {
                    ubo.bind()?;
                }
                for tex in d.textures.iter().enumerate() {
                    tex.1.bind(tex.0 as u32)?;
                }
                
                gl::DrawElementsInstanced(
                    gl::TRIANGLES, 
                    d.indices_len as i32, 
                    gl::UNSIGNED_INT, 
                    std::ptr::null(), 
                    d.instance_data.0 as i32
                );
                
                d.vao.unbind_buffers()?;
                d.vao.unbind()?;
                d.program.unuse()?;
            }
            
            instance_vbo.unbind()?;
        }
        
        self.draw_data.clear();

       Ok(())
    }

    unsafe fn set_render_target(&self, fb_target: gl::types::GLuint, viewport: (i32, i32, i32, i32), depth_test: bool) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, fb_target);
        gl::Viewport(viewport.0, viewport.1, viewport.2, viewport.3);
        gl::ClearColor(0.5, 0.1, 0.2, 1.0);
        
        if depth_test {
            gl::ClearDepth(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        } else {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
    
    unsafe fn send_data(&mut self, mut dd: SendInstanceDrawData) -> Result<(), StdError> {
        let new_data;
        match self.draw_data.entry(dd.material) {
            std::collections::hash_map::Entry::Occupied(val) => {
                new_data = !val.into_mut().contains_key(&dd.mesh);
            },
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(HashMap::default());
                new_data = true;
            },
        }

        if new_data {
            let program = self.set_program(dd.material.clone())?;
            let vao = self.set_vao(dd.mesh.clone())?;
            
            let textures = self.asset_manager
                .lock()
                .unwrap()
                .get_material(&dd.material)?
                .as_ref().unwrap()
                .texture()
                .to_vec();

            let textures = self.set_textures(&textures)?;
            let uniforms = self.set_uniforms(&dd.uniforms)?;
            let instance_data = (
                1,
                dd.instance_data.0,
                dd.instance_data.1,
            );
            let mut am = self.asset_manager
                .lock()
                .unwrap();
            let mesh = am.get_mesh(&dd.mesh)?
                .as_ref()
                .unwrap();

            let draw_data = DrawData {
                uniforms,
                textures,
                instance_data,
                program,
                vao,
                indices_len: mesh.indices().len(),
                first_location: mesh.vertices()[0].vertex_info().gl_info.last().unwrap().0,
            };            

            let mat_map = self.draw_data.get_mut(&dd.material).unwrap();
            mat_map.insert(dd.mesh, draw_data);
        }
        else {
            let mat_map = self.draw_data.get_mut(&dd.material).unwrap();
            match mat_map.entry(dd.mesh) {
                std::collections::hash_map::Entry::Occupied(val) => {
                    let val = val.into_mut();
                    val.instance_data.0 += 1;
                    val.instance_data.2.append(&mut dd.instance_data.1);
                },
                std::collections::hash_map::Entry::Vacant(_) => (),
            }
        }
        
        Ok(())
    }

    unsafe fn swap_buffers(&self) -> Result<(), StdError> {
        let size = (
            self.gl_specs.gl_surface.width().unwrap(),
            self.gl_specs.gl_surface.height().unwrap(),
        );
        self.set_render_target(0, (0, 0, size.0 as i32, size.1 as i32), true);
        let program = self.set_program(FINAL_PASS_MATERIAL.clone())?;
        let vao = self.set_vao(FINAL_PASS_MESH.clone())?;
        program.use_prog()?;
        vao.bind()?;
        vao.vertex_buffer().bind()?;
        vao.index_buffer().bind()?;
        
        let last_pas = self.render_passes.get(&self.active_pass).unwrap();

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, last_pas.color_texture);
        
        let mut am = self.asset_manager.lock().unwrap();
        let mesh = am.get_mesh(&FINAL_PASS_MESH)?
            .as_ref()
            .unwrap();

        gl::DrawElements(gl::TRIANGLES, mesh.indices().len() as i32, gl::UNSIGNED_INT, std::ptr::null());

        vao.unbind_buffers()?;
        vao.unbind()?;
        program.unuse()?;
        
        self.gl_specs.gl_surface.swap_buffers(&self.gl_specs.gl_context)?;
        
        Ok(())
    }
    
    unsafe fn resize(&self, new_size: (u32, u32)) {
        self.gl_specs.gl_surface.resize(
            &self.gl_specs.gl_context, 
            std::num::NonZeroU32::new(new_size.0).unwrap(), 
            std::num::NonZeroU32::new(new_size.1).unwrap(), 
        );
    }
}
impl Drop for RendererCore {
    fn drop(&mut self) {
        loop {
            let err = unsafe { gl::GetError() };
            if err == gl::NO_ERROR {
                break;
            }
            
            println!("OpenGL error {:08x}", err)
        }
    }
}

extern "system" fn debug_callback(
    source: gl::types::GLenum,
    gltype: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut std::ffi::c_void,
) {
    let source_str = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        gl::DEBUG_SOURCE_APPLICATION => "Application",
        _ => "Unknown",
    };

    let severity_str = match severity {
        gl::DEBUG_SEVERITY_HIGH => "High",
        gl::DEBUG_SEVERITY_MEDIUM => "Medium",
        gl::DEBUG_SEVERITY_LOW => "Low",
        gl::DEBUG_SEVERITY_NOTIFICATION => "Notification",
        _ => "Unknown",
    };

    let gltype_str = match gltype {
        gl::DEBUG_TYPE_ERROR => "Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
        gl::DEBUG_TYPE_PORTABILITY => "Portability",
        gl::DEBUG_TYPE_PERFORMANCE => "Performance",
        gl::DEBUG_TYPE_OTHER => "Other",
        gl::DEBUG_TYPE_MARKER => "Marker",
        gl::DEBUG_TYPE_PUSH_GROUP => "Push Group",
        gl::DEBUG_TYPE_POP_GROUP => "Pop Group",
        _ => "Unknown",
    };

    let message_str = unsafe { 
        std::str::from_utf8(std::ffi::CStr::from_ptr(message).to_bytes()).unwrap() 
    };
    error!(
        "OpenGL Debug Message:\n  Source: {}\n  Type: {}\n  ID: {}\n  Severity: {}\n  Message: {}",
        source_str, gltype_str, id, severity_str, message_str
    );
}