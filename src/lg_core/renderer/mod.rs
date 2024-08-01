use std::{collections::HashMap, ffi::CString, sync::{mpsc::{Receiver, Sender}, Arc, Mutex, MutexGuard}};
use command::{ReceiveRendererCommand, SendDrawData, SendInstanceDrawData, SendRendererCommand};
use glutin::{display::GlDisplay, surface::GlSurface};
use imgui_config::imgui_init;
use imgui_glow_renderer::TextureMap;
use opengl::{gl_buffer::GlBuffer, gl_init::{init_opengl, init_window}, gl_program::GlProgram, gl_shader::GlShader, gl_texture::{self, GlTexture}, gl_vertex_array::GlVertexArray, GlSpecs};
use render_target::{FramebufferFormat, RenderTarget, RenderTargetSpecs};
use sllog::error;
use texture::Texture;
use uniform::Uniform;
use vertex::{LgVertex, VertexInfo};
use crate::{lg_core::glm, StdError};
use super::{asset_manager::{self, AssetManager}, uuid::UUID, window::LgWindow};

pub mod mesh;
pub mod material;
pub mod texture;
pub mod shader;
pub mod uniform;
pub mod buffer;
pub mod vertex;
pub mod render_target;
pub mod command;
mod imgui_config;
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
    core: Arc<Mutex<RendererCore>>,
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
    ) -> Result<(Self, LgWindow, Arc<Mutex<AssetManager>>), StdError> 
    {
        let am = Arc::new(Mutex::new(AssetManager::default()));
        let asset_manager = Arc::clone(&am);

        let (w_sender, w_receiver) = std::sync::mpsc::channel();
        let (core_sender, core_receiver) = std::sync::mpsc::channel();

        let (s_sender, s_receiver) = std::sync::mpsc::channel();
        let (r_sender, r_receiver) = std::sync::mpsc::channel();

        let (window, gl_config) = init_window(window_info)?;
        std::thread::spawn(move || {
            asset_manager.lock().unwrap().init().unwrap();

            let (window, specs) = init_opengl(window, gl_config).unwrap();

            let (imgui_context, imgui_winit) = imgui_init(&window);
            let renderer_core = Arc::new(Mutex::new(RendererCore::new(
                specs, 
                Arc::clone(&asset_manager),
                imgui_context,
                imgui_winit,
            ).unwrap()));

            let r_core = Arc::clone(&renderer_core);

            core_sender.send(renderer_core).unwrap();
            w_sender.send(window).unwrap();

            loop {
                while let Ok(msg) = s_receiver.recv() {
                    let mut r_core = r_core.lock().unwrap();
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
                        SendRendererCommand::GET_PASS_DEPTH_TEXTURE_LG(_name) => todo!(),
                        SendRendererCommand::BEGIN_RENDER_PASS(name) => {
                            let target = r_core.render_passes.get(&name).unwrap();
                            r_core.set_render_target(target.framebuffer, &target.specs);
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
                        SendRendererCommand::_SHUTDOWN => {
                            r_sender.send(ReceiveRendererCommand::_SHUTDOWN_DONE).unwrap();

                            // ImGui .ini file writing.
                            let mut ini_contents = String::new();
                            r_core.imgui_context
                                .save_ini_settings(&mut ini_contents);
                            
                            if let Err(e) = std::fs::write("imgui.ini", ini_contents) {
                                error!("Failed to write imgui.ini: {}", e);
                            }
                        },
                        SendRendererCommand::_BEGIN => todo!(),
                        SendRendererCommand::_END => {
                            r_sender.send(ReceiveRendererCommand::_END_DONE).unwrap();
                            r_core.swap_buffers().unwrap();
                        },
                        SendRendererCommand::_DRAW_IMGUI => {
                            r_core.render_imgui();
                            r_sender.send(ReceiveRendererCommand::_IMGUI_DONE).unwrap();
                        },
                        SendRendererCommand::_DRAW_BACKBUFFER => r_core.draw_backbuffer().unwrap(),
                        SendRendererCommand::SET_FONT(bytes, size) => {
                            r_core.set_font_imgui(&bytes, size).unwrap();
                        },
                    }
                }}
            }
        });
        
        let window = LgWindow::new(w_receiver.recv()?);
        let core = core_receiver.recv()?;

        Ok((
            Self {
                core,
                sender: s_sender,
                receiver: r_receiver,
                last_frame: Vec::new(),
            },
            window,
            am,
        ))
    }
    
    pub(crate) fn core(&self) -> MutexGuard<RendererCore> {
        self.core.lock().unwrap()
    }

    pub(crate) fn update_imgui_delta_time(&self, delta: std::time::Duration) {
        let mut core = self.core.lock().unwrap();        
        
        core.imgui_context
            .io_mut()
            .update_delta_time(delta);
    }
    
    pub(crate) fn prepare_imgui_frame(&self, window: &winit::window::Window) {
        let mut core = self.core.lock().unwrap();
        core.prepare_imgui_frame(window);
    }
    
    pub(crate) fn handle_imgui_event<T>(&self, window: &winit::window::Window, event: &winit::event::Event<T>) {
        let mut core = self.core.lock().unwrap();
        
        core.handle_imgui_event(window, event);
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

pub(crate) struct RendererCore {
    _gl_glow_context: glow::Context,
    _imgui_textures: imgui_glow_renderer::SimpleTextureMap,
    pub(crate) imgui_context: imgui::Context,
    pub(crate) imgui_winit: imgui_winit_support::WinitPlatform,
    pub(crate) imgui_renderer: imgui_glow_renderer::Renderer,

    asset_manager: Arc<Mutex<AssetManager>>,
    gl_specs: GlSpecs,

    // Material, Mesh, Data
    draw_data: HashMap<UUID, HashMap<UUID, DrawData>>,
    render_passes: HashMap<String, RenderTarget>,
    active_pass: String,
    
    vsync: bool
}
unsafe impl Send for RendererCore {}
unsafe impl Sync for RendererCore {}

impl RendererCore {
    pub(crate) fn handle_imgui_event<T>(&mut self, window: &winit::window::Window, event: &winit::event::Event<T>) {
        self.imgui_winit
            .handle_event(self.imgui_context.io_mut(), window, event);
    }

    pub(crate) fn new_imgui_frame(&mut self) -> *mut imgui::Ui {
        self.imgui_context.new_frame()
    }
    
    pub(crate) fn prepare_to_render(&mut self, ui: &mut imgui::Ui, window: &winit::window::Window) {
        self.imgui_winit
            .prepare_render(ui, window);
    }
}
impl RendererCore {
    fn new(
        specs: GlSpecs, 
        asset_manager: Arc<Mutex<AssetManager>>,
        mut imgui_context: imgui::Context,
        imgui_winit: imgui_winit_support::WinitPlatform,
    ) -> Result<Self, StdError> 
    {
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            specs.gl_display.get_proc_address(symbol.as_c_str()).cast()
        });
        
        unsafe {
            // Debug
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());

            // Depth, Blend
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
        }

        let mut gl_glow = unsafe {
            glow::Context::from_loader_function_cstr(|s| specs.gl_display.get_proc_address(s).cast())
        };

        // let imgui_renderer = imgui_glow_renderer::AutoRenderer::initialize(gl_glow, &mut imgui_context)?;
        let mut simple_textures = imgui_glow_renderer::SimpleTextureMap {};
        let imgui_renderer = imgui_glow_renderer::Renderer::initialize(
            &mut gl_glow, 
            &mut imgui_context, 
            &mut simple_textures,
            true
        )?;

        Ok(Self {
            _gl_glow_context: gl_glow,
            _imgui_textures: simple_textures,
            imgui_context,
            imgui_winit,
            imgui_renderer,

            asset_manager,
            gl_specs: specs,
            draw_data: HashMap::new(),
            render_passes: HashMap::new(),
            active_pass: String::new(),
            vsync: false,
        })
    }
    
    fn prepare_imgui_frame(&mut self, window: &winit::window::Window) {
        self.imgui_winit
            .prepare_frame(self.imgui_context.io_mut(), window)
            .unwrap();
    }

    fn render_imgui(&mut self) {
        let size = (
            self.gl_specs.gl_surface.width().unwrap(),
            self.gl_specs.gl_surface.height().unwrap(),
        );

        let specs = RenderTargetSpecs {
            clear: true,
            clear_color: glm::vec4(0.0, 0.0, 0.0, 1.0),
            viewport: (0, 0, size.0 as i32, size.1 as i32),
            depth_test: false,
            ..Default::default()
        };

        unsafe { self.set_render_target(0, &specs); }

        let dd = self.imgui_context.render();

        let mut should_render = false;
        for dl in dd.draw_lists() {
            if !dl.vtx_buffer().is_empty() {
                should_render = true;
            }
        }

        if should_render {
            self.imgui_renderer
                .render(
                    &self._gl_glow_context,
                    &self._imgui_textures,
                    dd,
                )
                .unwrap();
        }
    }

    unsafe fn set_font_imgui(&mut self, bytes: &[u8], pixels: f32) -> Result<(), StdError> {
        self.imgui_context.fonts().clear();
        let fonts = self.imgui_context.fonts();
        fonts.add_font(&[imgui::FontSource::TtfData { 
            data: bytes,
            size_pixels: pixels, 
            config: None 
        }]);

        let atlas_texture = fonts.build_rgba32_texture();
        
        let mut gl_texture = 0;
        gl::GenTextures(1, &mut gl_texture);
        gl::BindTexture(gl::TEXTURE_2D, gl_texture);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as _,
        );
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::SRGB8_ALPHA8 as _,
            atlas_texture.width as _,
            atlas_texture.height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            atlas_texture.data.as_ptr() as *const _,
        );
        
        let tex = glow::NativeTexture(std::num::NonZero::new(gl_texture).unwrap());
        fonts.tex_id = self._imgui_textures
            .register(tex).unwrap();

        Ok(())
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

    unsafe fn set_render_target(&self, fb_target: gl::types::GLuint, specs: &RenderTargetSpecs) {
        let viewport = specs.viewport;
        if specs.framebuffer_format == FramebufferFormat::SRGB {
            gl::Enable(gl::FRAMEBUFFER_SRGB);
        } 
        else {
            gl::Disable(gl::FRAMEBUFFER_SRGB);
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, fb_target);
        gl::Viewport(viewport.0, viewport.1, viewport.2, viewport.3);
        gl::ClearColor(specs.clear_color.x, specs.clear_color.y, specs.clear_color.z, specs.clear_color.w);
        
        if specs.clear {
            if specs.depth_test {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(gl::LESS);
                
                gl::ClearDepth(specs.clear_depth);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            } else {
                gl::Disable(gl::DEPTH_TEST);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
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

    unsafe fn draw_backbuffer(&mut self) -> Result<(), StdError> {
        let size = (
            self.gl_specs.gl_surface.width().unwrap(),
            self.gl_specs.gl_surface.height().unwrap(),
        );

        let specs = RenderTargetSpecs {
            clear: true,
            clear_color: glm::vec4(0.0, 0.0, 0.0, 1.0),
            viewport: (0, 0, size.0 as i32, size.1 as i32),
            depth_test: false,
            ..Default::default()
        };

        self.set_render_target(0, &specs);
        let program = self.set_program(FINAL_PASS_MATERIAL.clone())?;
        let vao = self.set_vao(FINAL_PASS_MESH.clone())?;
        program.use_prog()?;
        vao.bind()?;
        vao.vertex_buffer().bind()?;
        vao.index_buffer().bind()?;
        
        let last_pas = self.render_passes.get(&self.active_pass).unwrap();

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, last_pas.color_texture);
        
        {
            let mut am = self.asset_manager.lock().unwrap();
            let mesh = am.get_mesh(&FINAL_PASS_MESH)?
                .as_ref()
                .unwrap();

            gl::DrawElements(gl::TRIANGLES, mesh.indices().len() as i32, gl::UNSIGNED_INT, std::ptr::null());
        }

        vao.unbind_buffers()?;
        vao.unbind()?;
        program.unuse()?;
        
        Ok(())
    }

    unsafe fn swap_buffers(&mut self) -> Result<(), StdError> {
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

    if severity != gl::DEBUG_SEVERITY_NOTIFICATION {
        let message_str = unsafe { 
            std::str::from_utf8(std::ffi::CStr::from_ptr(message).to_bytes()).unwrap() 
        };
        error!("{}", message_str);
        error!(
            "OpenGL Debug Message:\n  Source: {}\n  Type: {}\n  ID: {}\n  Severity: {}\n  Message: {}",
            source_str, gltype_str, id, severity_str, message_str
        );
    }
}