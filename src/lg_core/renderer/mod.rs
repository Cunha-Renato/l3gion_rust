use std::{collections::HashMap, ffi::CString, sync::{mpsc::{Receiver, Sender}, Arc, Mutex, MutexGuard}, thread::JoinHandle};
use command::{RendererCommand, SendDrawData, SendInstanceDrawData};
use glutin::{display::GlDisplay, surface::GlSurface};
use imgui_config::{imgui_init, ImGuiCore};
use material::Material;
use mesh::Mesh;
use opengl::{gl_buffer::GlBuffer, gl_init::{init_opengl, init_window}, GlSpecs};
use render_target::{FramebufferFormat, RenderTarget, RenderTargetSpecs};
use shader::Shader;
use sllog::{error, warn};
use texture::{Texture, TextureSpecs};
use uniform::Uniform;
use vertex::{LgVertex, VertexInfo};

use crate::{gl_check, gl_check_and_print, glm, lg_types::no_check_option::NCOption, profile_function, profile_scope, StdError};
use super::{asset_manager::AssetManager, uuid::UUID, window::LgWindow};

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

type Job = Box<dyn FnOnce() -> bool + Send + 'static>;

const FINAL_PASS_MESH: UUID = UUID::from_u128(252411435688744967694609164507863584779);
const FINAL_PASS_MATERIAL: UUID = UUID::from_u128(315299335240398778209169027697428014904);

pub struct CreationWindowInfo<'a> {
    pub event_loop: Option<&'a winit::event_loop::EventLoop<()>>,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

pub struct Renderer {
    core: NCOption<Arc<Mutex<RendererCore>>>,
    job_sender: Sender<Job>,
    message_sender: Sender<RendererCommand>,
    receiver: Receiver<RendererCommand>,
    thread: JoinHandle<()>,
}
// Public
impl Renderer {
    /// Don't store this pointer, it points to a HashMap!
    pub fn get_texture(&self, uuid: &UUID) -> Result<*const Texture, StdError> {
        self.core
            .lock()
            .unwrap()
            .asset_manager
            .get_texture(uuid)
    }
    
    /// Don't store this pointer, it points to a HashMap!
    pub fn get_mesh(&self, uuid: &UUID) -> Result<*const Mesh, StdError> {
        self.core
            .lock()
            .unwrap()
            .asset_manager
            .get_mesh(uuid)
    }
    
    /// Don't store this pointer, it points to a HashMap!
    pub fn get_shader(&self, uuid: &UUID) -> Result<*const Shader, StdError> {
        self.core
            .lock()
            .unwrap()
            .asset_manager
            .get_shader(uuid)
    }
    
    /// Don't store this pointer, it points to a HashMap!
    pub fn get_material(&self, uuid: &UUID) -> Result<*const Material, StdError> {
        self.core
            .lock()
            .unwrap()
            .asset_manager
            .get_material(uuid)
    }
    
    /// Don't store this pointer, it points to a HashMap!
    pub fn create_material(&self, uuid: &UUID, name: &str, textures: Vec<String>, shaders: Vec<String>) -> Result<*const Material, StdError> {
        self.core
            .lock()
            .unwrap()
            .asset_manager
            .create_material(name, textures, shaders)
    }
    
    /// Don't store this pointer, it points to a HashMap!
    pub fn create_texture(&self, name: &str, path: &str, specs: TextureSpecs) -> Result<*const Texture, StdError> {
        self.core
            .lock()
            .unwrap()
            .asset_manager
            .create_texture(name, path, specs)
    }

    /// Locks the core mutex
    pub fn core(&self) -> MutexGuard<RendererCore> {
        self.core.lock().unwrap()
    }

    pub fn get_prev_frame_color_tex_gl(&self, name: &str) -> Option<gl::types::GLuint> {
        if let Some(tex) = self.core().render_passes.get(name) {
            return Some(tex.color_texture);
        }

        None
    }
    
    pub fn set_vsync(&self, val: bool) {
        let (r_core, _) = self.get_coms_data();

        self.job_sender.send(Box::new(move || {
            r_core.lock().unwrap().set_vsync(val);

            false
        })).unwrap();
    }

    /// Can block.
    pub fn get_vsync(&self) -> bool {
        self.core.lock().unwrap().vsync
    }

    pub fn set_size(&self, new_size: (u32, u32)) {
        profile_function!();
        let (r_core, _) = self.get_coms_data();

        self.job_sender.send(Box::new(move || unsafe {
            r_core.lock().unwrap().resize(new_size);
            
            false
        }))
        .unwrap();
    }

    pub fn create_render_pass(&self, name: String, specs: RenderTargetSpecs) {
        let (r_core, _) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || {
            r_core.lock().unwrap().render_passes.insert(name, RenderTarget::new(specs));
            
            false
        })).unwrap();
    }

    pub fn set_render_pass_size(&self, name: String, new_size: (i32, i32)) {
        let (r_core, _) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || {
            let mut r_core = r_core.lock().unwrap();

            let mut specs = r_core.render_passes.get_mut(&name).unwrap().specs.clone();
            specs.viewport = (0, 0, new_size.0, new_size.1);
            r_core.render_passes.insert(name, RenderTarget::new(specs));
            
            false
        }))
        .unwrap();
    }

    pub fn begin_render_pass(&self, name: String) {
        let (r_core, _) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || unsafe {
            let mut r_core = r_core.lock().unwrap();
            let target = r_core.render_passes.get(&name).unwrap();
            
            r_core.set_render_target(target.framebuffer, &target.specs);

            if !r_core.active_pass.is_empty() {
                let active_pass = std::mem::take(&mut r_core.active_pass);
                r_core.render_pipeline.push(active_pass);
            }
            
            r_core.active_pass = name;
            
            false
        }))
        .unwrap();
    }

    pub fn send_instance_data(&self, instance_data: SendInstanceDrawData) {
        let (r_core, _) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || unsafe {
            r_core.lock().unwrap().send_data(instance_data).unwrap();
                
            false
        }))
        .unwrap()
    }

    pub fn draw_instanced(&self) {
        let (r_core, _) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || unsafe {
            r_core.lock().unwrap().draw_instanced().unwrap(); 
            
            false
        }))
        .unwrap();
    }

    pub fn draw(&self, draw_data: SendDrawData) {
        let (r_core, _) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || unsafe {
            r_core.lock().unwrap().draw(draw_data).unwrap();
            
            false
        }))
        .unwrap();
    }

    pub fn set_fonts(&self) {
        let (r_core, _) = self.get_coms_data();

        self.job_sender.send(Box::new(move || unsafe {
            r_core.lock().unwrap().imgui_core.set_fonts().unwrap();
            
            false
        }))
        .unwrap();
    }
}
// Public(crate)
impl Renderer {
    pub(crate) fn new(
        window_info: CreationWindowInfo, 
    ) -> Result<(Self, LgWindow), StdError> 
    {
        profile_function!();

        let (w_sender, w_receiver) = std::sync::mpsc::channel();
        let (core_sender, core_receiver) = std::sync::mpsc::channel();

        let (s_sender, s_receiver) = std::sync::mpsc::channel::<Job>();
        let (r_sender, r_receiver) = std::sync::mpsc::channel();

        let (window, gl_config) = init_window(&window_info)?;
        let thread = std::thread::spawn(move || {
            optick::register_thread("render_thread");

            let (window, specs) = init_opengl(window, gl_config).unwrap();

            let (imgui_context, imgui_winit) = imgui_init(&window);
            let renderer_core = Arc::new(Mutex::new(RendererCore::new(
                specs, 
                AssetManager::default(),
                imgui_context,
                imgui_winit,
            ).unwrap()));

            renderer_core.lock().unwrap().set_vsync(window_info.vsync);

            // Sending outside this thread.
            core_sender.send(renderer_core).unwrap();
            w_sender.send(window).unwrap();

            while let Ok(job) = s_receiver.recv() {
                if job() {
                    break; 
                }
            }
        });
        
        let window = LgWindow::new(w_receiver.recv()?);
        let core = NCOption::Some(core_receiver.recv()?);

        Ok((
            Self {
                core,
                job_sender: s_sender,
                message_sender: r_sender,
                receiver: r_receiver,
                thread,
            },
            window,
        ))
    }
    
    pub(crate) fn begin(&self) {
        let (r_core, _) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || {
            let mut r_core = r_core.lock().unwrap();

            // TODO: Maybe don't do this
            r_core.asset_manager.init_gl_program().unwrap();
            r_core.asset_manager.init_gl_vao().unwrap();                                
            r_core.asset_manager.init_gl_texture().unwrap();                                
            
            false
        }))
        .unwrap();
    }

    /// Will always wait(block).
    pub(crate) fn end(&self) {
        profile_function!();
        let (r_core, message_sender) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || unsafe {
            let mut r_core = r_core.lock().unwrap();

            r_core.render_pipeline.clear();
            r_core.swap_buffers().unwrap();

            r_core.asset_manager.to_destroy();

            message_sender.send(RendererCommand::_END_DONE).unwrap();
            
            false
        }))
        .unwrap();

        while let Ok(msg) = self.receiver.recv() {
            match msg {
                RendererCommand::_END_DONE => break,
                _ => ()
            }
        }
    }

    pub(crate) fn draw_imgui(&self) {
        let (r_core, message_sender) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || {
            r_core.lock().unwrap().render_imgui();
            message_sender.send(RendererCommand::_IMGUI_DONE).unwrap();
            
            false
        }))
        .unwrap();
    }

    pub(crate) fn draw_backbuffer(&self) {
        let (r_core, _) = self.get_coms_data();
        
        self.job_sender.send(Box::new(move || unsafe {
            r_core.lock().unwrap().draw_backbuffer().unwrap();
            
            false
        })).unwrap();
    }

    pub(crate) fn update_imgui_delta_time(&self, delta: std::time::Duration) {
        profile_function!();
        let mut core = self.core.lock().unwrap();        
        
        core.imgui_core.update_delta_time(delta);
    }
    
    pub(crate) fn prepare_imgui_frame(&self, window: &winit::window::Window) {
        profile_function!();

        let mut core = self.core.lock().unwrap();
        core.imgui_core.prepare_frame(window);
    }
    
    pub(crate) fn handle_imgui_event<T>(&self, window: &winit::window::Window, event: &winit::event::Event<T>) {
        profile_function!();
        let mut core = self.core.lock().unwrap();
        
        core.imgui_core.handle_event(window, event);
    }
}
// Private
impl Renderer {
    fn get_coms_data(&self) -> (Arc<Mutex<RendererCore>>, Sender<RendererCommand>) {
        (Arc::clone(&self.core), self.message_sender.clone())
    }
    
    /// Will always block.
    fn shutdown(&mut self) {
        let (_, message_sender) = self.get_coms_data();
        let r_core = std::mem::take(&mut self.core);
        
        self.job_sender.send(Box::new(move || {
            r_core.lock().unwrap().imgui_core.shutdown();
            
            message_sender.send(RendererCommand::_SHUTDOWN_DONE).unwrap();
            
            true
        })).unwrap();
        
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                RendererCommand::_SHUTDOWN_DONE => break,
                _ => (),
            }
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.shutdown();
        while !self.thread.is_finished() {}
    }
}

#[derive(Debug)]
struct DrawData {
    uniforms: Vec<GlBuffer>,
    textures: Vec<UUID>,
    instance_data: (u32, VertexInfo, Vec<u8>),

    indices_len: usize,
    first_location: u32,
}
pub struct RendererCore {
    imgui_core: ImGuiCore,

    asset_manager:AssetManager,
    gl_specs: GlSpecs,

    // Material, Mesh, Data
    draw_data: HashMap<UUID, HashMap<UUID, DrawData>>,
    // Order of Render Passes
    render_pipeline: Vec<String>,
    render_passes: HashMap<String, RenderTarget>,
    active_pass: String,
    
    vsync: bool
}
impl RendererCore {
    pub fn imgui(&mut self) -> &mut ImGuiCore {
        &mut self.imgui_core
    }
}
impl RendererCore {
    fn new(
        specs: GlSpecs, 
        mut asset_manager: AssetManager,
        mut imgui_context: imgui::Context,
        imgui_winit: imgui_winit_support::WinitPlatform,
    ) -> Result<Self, StdError> 
    {
        let mut _symbol_debug = 0;

        gl::load_with(|symbol| {
            _symbol_debug += 1;

            let symbol = CString::new(symbol).unwrap();
            specs.gl_display.get_proc_address(symbol.as_c_str()) as *const _
        });
        
        warn!("OpenGL Symbols loaded: {}", _symbol_debug);
        
        // Debug
        gl_check_and_print!(gl::Enable(gl::DEBUG_OUTPUT));
        gl_check_and_print!(gl::DebugMessageCallback(Some(debug_callback), std::ptr::null()));

        // Depth, Blend
        gl_check_and_print!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
        gl_check_and_print!(gl::Enable(gl::BLEND));

        let mut gl_glow = unsafe {
            glow::Context::from_loader_function_cstr(|s| specs.gl_display.get_proc_address(s).cast())
        };

        let mut simple_textures = imgui_glow_renderer::SimpleTextureMap {};
        let imgui_renderer = imgui_glow_renderer::Renderer::initialize(
            &mut gl_glow, 
            &mut imgui_context, 
            &mut simple_textures,
            true
        )?;

        let imgui_core = ImGuiCore::new(
            gl_glow,
            simple_textures,
            imgui_context,
            imgui_winit,
            imgui_renderer,
        );

        asset_manager.init().unwrap();

        Ok(Self {
            imgui_core,
            asset_manager,
            gl_specs: specs,
            draw_data: HashMap::default(),

            render_pipeline: Vec::default(),
            render_passes: HashMap::default(),
            active_pass: String::default(),

            vsync: false,
        })
    }

    fn render_imgui(&mut self) {
        profile_function!();

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

        self.imgui_core.render_imgui();        
    }

    fn set_vsync(&mut self, op: bool) {
        profile_function!();

        self.vsync = op;
        if op {
            self.gl_specs.gl_surface.set_swap_interval(&self.gl_specs.gl_context, glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap())).unwrap();
        } else {
            self.gl_specs.gl_surface.set_swap_interval(&self.gl_specs.gl_context, glutin::surface::SwapInterval::DontWait).unwrap();
        } 
    }

    unsafe fn set_uniforms(&self, uniforms: &[Uniform]) -> Result<Vec<GlBuffer>, StdError> {
        profile_function!();

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

    unsafe fn draw(&mut self, dd: SendDrawData) -> Result<(), StdError> {
        profile_function!();

        let mesh = self.asset_manager
            .get_mesh(&dd.mesh)?
            .as_ref()
            .unwrap();

        let material = self.asset_manager
            .get_material(&dd.material)?
            .as_ref()
            .unwrap();
        
        // Program
        self.asset_manager.init_gl_program()?;
        let program = material.gl_program.as_ref().ok_or("Couldn't find GlProgram in Material!")?;
        program.use_prog()?;

        // VAO
        self.asset_manager.init_gl_vao()?;
        let vao = mesh.gl_vao.as_ref().ok_or("Couldn't find GlVertexArray in Mesh!")?;
        vao.bind()?;
        vao.vertex_buffer().bind()?;
        vao.index_buffer().bind()?;

        let ubos = self.set_uniforms(&dd.uniforms)?;
        
        for ubo in &ubos {
            ubo.bind()?;
        }

        for (location, tex_op) in dd.textures.iter().enumerate() {
            match tex_op {
                command::TextureOption::UUID(_) => todo!(),
                command::TextureOption::LG_TEXTURE(_) => todo!(),
                command::TextureOption::GL_TEXTURE(tex) => {
                    gl_check_and_print!(gl::ActiveTexture(gl::TEXTURE0 + location as u32));
                    gl_check_and_print!(gl::BindTexture(gl::TEXTURE_2D, *tex));
                    gl_check_and_print!(gl::Uniform1i(location as i32, location as i32));
                },
                command::TextureOption::PREVIOUS_PASS => {
                    let pass = self.render_pipeline.last().unwrap();
                    let tex = self.render_passes.get(pass).unwrap()
                        .color_texture;
                    
                    gl_check_and_print!(gl::ActiveTexture(gl::TEXTURE0 + location as u32));
                    gl_check_and_print!(gl::BindTexture(gl::TEXTURE_2D, tex));
                    gl_check_and_print!(gl::Uniform1i(location as i32, location as i32));
                },
            }
        }
        
        {
            profile_scope!("DrawElements");
            gl_check_and_print!(gl::DrawElements(gl::TRIANGLES, mesh.indices().len() as i32, gl::UNSIGNED_INT, std::ptr::null()));
        }
        
        vao.unbind_buffers()?;
        vao.unbind()?;
        program.unuse()?;

        Ok(())
    }

    unsafe fn draw_instanced(&mut self) -> Result<(), StdError> {
        profile_function!();

        for (material_uuid, dd) in &self.draw_data {
            let material = self.asset_manager.get_material(material_uuid)?
                    .as_ref()
                    .unwrap();
            let instance_vbo = GlBuffer::new(gl::ARRAY_BUFFER)?;

            for (mesh_uui, d) in dd {
                let mesh = self.asset_manager.get_mesh(mesh_uui)?
                    .as_ref()
                    .unwrap();
                let textures = d.textures.iter()
                    .map(|uuid| self.asset_manager.get_texture(uuid).unwrap() )
                    .collect::<Vec<_>>();

                // Program
                let program = material.gl_program.as_ref().ok_or("Couldn't find GlProgram in Material!")?;
                program.use_prog()?;

                // VAO
                let vao = mesh.gl_vao.as_ref().ok_or("Couldn't find GlVertexArray in Mesh!")?;
                vao.bind()?;
                vao.vertex_buffer().bind()?;
                vao.index_buffer().bind()?;
                
                // Textures
                self.asset_manager.init_gl_texture()?;

                let last_location = d.first_location;
                instance_vbo.bind()?;
                instance_vbo.set_data(&d.instance_data.2, gl::STATIC_DRAW)?;
                
                for info in &d.instance_data.1.gl_info {
                    let location = info.0 + last_location + 1;
                    vao.set_attribute(location, info.1, d.instance_data.1.stride, info.2)?;
                    
                    gl_check_and_print!(gl::VertexAttribDivisor(location, 1));
                }

                for ubo in &d.uniforms {
                    ubo.bind()?;
                }

                for tex in textures.iter().enumerate() {
                    let gl_texture = tex.1
                        .as_ref()
                        .unwrap()
                        .gl_texture
                        .as_ref()
                        .ok_or("Couldn't find GlTexture in Texture!")?;

                    gl_texture.activate(tex.0 as u32)?;
                    gl_texture.bind()?;
                }
                
                {
                    profile_scope!("DrawElementsInstanced");
                    gl_check_and_print!(gl::DrawElementsInstanced(
                        gl::TRIANGLES, 
                        d.indices_len as i32, 
                        gl::UNSIGNED_INT, 
                        std::ptr::null(), 
                        d.instance_data.0 as i32
                    ));
                }
                
                vao.unbind_buffers()?;
                vao.unbind()?;
                program.unuse()?;
            }
            
            instance_vbo.unbind()?;
        }
        
        self.draw_data.clear();

       Ok(())
    }

    unsafe fn set_render_target(&self, fb_target: gl::types::GLuint, specs: &RenderTargetSpecs) {
        profile_function!();

        let viewport = specs.viewport;
        if specs.framebuffer_format == FramebufferFormat::SRGB {
            gl_check_and_print!(gl::Enable(gl::FRAMEBUFFER_SRGB));
        } 
        else {
            gl_check_and_print!(gl::Disable(gl::FRAMEBUFFER_SRGB));
        }

        gl_check_and_print!(gl::BindFramebuffer(gl::FRAMEBUFFER, fb_target));
        gl_check_and_print!(gl::Viewport(viewport.0, viewport.1, viewport.2, viewport.3));
        gl_check_and_print!(gl::ClearColor(specs.clear_color.x, specs.clear_color.y, specs.clear_color.z, specs.clear_color.w));
        
        if specs.clear {
            if specs.depth_test {
                gl_check_and_print!(gl::Enable(gl::DEPTH_TEST));
                gl_check_and_print!(gl::DepthFunc(gl::LESS));
                
                gl_check_and_print!(gl::ClearDepth(specs.clear_depth));
                gl_check_and_print!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
            } else {
                gl_check_and_print!(gl::Disable(gl::DEPTH_TEST));
                gl_check_and_print!(gl::Clear(gl::COLOR_BUFFER_BIT));
            }
        }
    }
    
    unsafe fn send_data(&mut self, mut dd: SendInstanceDrawData) -> Result<(), StdError> {
        profile_function!();

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
            profile_scope!("new_data");

            let material = self.asset_manager.get_material(&dd.material)?
                .as_ref()
                .unwrap();
            
            // Program
            self.asset_manager.init_gl_program()?;
            let program = material.gl_program.as_ref().ok_or("Couldn't find GlProgram in Material!")?;
            program.use_prog()?;

            let mesh = self.asset_manager.get_mesh(&dd.mesh)?
                .as_ref()
                .unwrap();

            // VAO
            self.asset_manager.init_gl_vao()?;

            let textures = material
                .texture()
                .to_vec();

            let instance_data = (
                1,
                dd.instance_data.0,
                dd.instance_data.1,
            );

            let uniforms = self.set_uniforms(&dd.uniforms)?;

            let draw_data = DrawData {
                uniforms,
                textures,
                instance_data,
                indices_len: mesh.indices().len(),
                first_location: mesh.vertices()[0].vertex_info().gl_info.last().unwrap().0,
            };            

            let mat_map = self.draw_data.get_mut(&dd.material).unwrap();
            mat_map.insert(dd.mesh, draw_data);
        }
        else {
            profile_scope!("old_data");

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
        profile_function!();

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

        // Program
        let material = self.asset_manager.get_material(&FINAL_PASS_MATERIAL)?
            .as_ref()
            .unwrap();

        self.asset_manager.init_gl_program()?;
        let program = material.gl_program.as_ref().ok_or("Couldn't find GlProgram in Material!")?;
        program.use_prog()?;

        // VAO
        let mesh = self.asset_manager.get_mesh(&FINAL_PASS_MESH)?
            .as_ref()
            .unwrap();

        self.asset_manager.init_gl_vao()?;
        let vao = mesh.gl_vao.as_ref().ok_or("Couldn't find GlVertexArray in Mesh!")?;
        vao.bind()?;
        vao.vertex_buffer().bind()?;
        vao.index_buffer().bind()?;
        
        let last_pas = self.render_passes.get(&self.active_pass).unwrap();
        gl_check_and_print!(gl::ActiveTexture(gl::TEXTURE0));
        gl_check_and_print!(gl::BindTexture(gl::TEXTURE_2D, last_pas.color_texture));
        
        gl_check_and_print!(gl::DrawElements(gl::TRIANGLES, mesh.indices().len() as i32, gl::UNSIGNED_INT, std::ptr::null()));

        vao.unbind_buffers()?;
        vao.unbind()?;
        program.unuse()?;
        
        Ok(())
    }

    unsafe fn swap_buffers(&mut self) -> Result<(), StdError> {
        profile_function!();
        self.gl_specs.gl_surface.swap_buffers(&self.gl_specs.gl_context)?;
        
        Ok(())
    }
    
    unsafe fn resize(&self, new_size: (u32, u32)) {
        profile_function!();
        self.gl_specs.gl_surface.resize(
            &self.gl_specs.gl_context, 
            std::num::NonZeroU32::new(new_size.0).unwrap(), 
            std::num::NonZeroU32::new(new_size.1).unwrap(), 
        );
    }
}
impl Drop for RendererCore {
    fn drop(&mut self) {
        if cfg!(debug_assertions) {
            loop {
                let err = unsafe { gl::GetError() };
                if err == gl::NO_ERROR {
                    break;
                }
                
                println!("OpenGL error {:08x}", err)
            }
        }
    }
}
unsafe impl Send for RendererCore {}
unsafe impl Sync for RendererCore {}

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