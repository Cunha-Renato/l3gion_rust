use imgui_glow_renderer::TextureMap;
use sllog::error;

use crate::StdError;

pub(crate) struct ImGuiCore {
    _gl_glow_context: glow::Context,
    _imgui_texture_map: imgui_glow_renderer::SimpleTextureMap,
    imgui_context: imgui::Context,
    imgui_winit: imgui_winit_support::WinitPlatform,
    imgui_renderer: imgui_glow_renderer::Renderer,
}
// Public
impl ImGuiCore {
    pub fn context(&mut self) -> &mut imgui::Context {
        &mut self.imgui_context
    }
}

// Public(crate)
impl ImGuiCore {
    pub(crate) fn handle_event<T>(&mut self, window: &winit::window::Window, event: &winit::event::Event<T>) {
        self.imgui_winit.handle_event(self.imgui_context.io_mut(), window, event);
    }
    
    pub(crate) fn new_frame(&mut self) -> *mut imgui::Ui {
        self.imgui_context.new_frame()
    }
    
    pub(crate) fn prepare_to_render(&mut self, ui: &mut imgui::Ui, window: &winit::window::Window) {
        self.imgui_winit.prepare_render(ui, window);
    }
}

// Public(super)
impl ImGuiCore {
    pub(super) fn new(
        _gl_glow_context: glow::Context,
        _imgui_texture_map: imgui_glow_renderer::SimpleTextureMap,
        imgui_context: imgui::Context,
        imgui_winit: imgui_winit_support::WinitPlatform,
        imgui_renderer: imgui_glow_renderer::Renderer,
    ) -> Self 
    {
        Self {
            _gl_glow_context,
            _imgui_texture_map,
            imgui_context,
            imgui_winit,
            imgui_renderer,
        }
    }

    pub(super) fn shutdown(&mut self) {
        let mut ini_contents = String::new();
        self.imgui_context
            .save_ini_settings(&mut ini_contents);
        
        if let Err(e) = std::fs::write("imgui.ini", ini_contents) {
            error!("Failed to write imgui.ini: {}", e);
        }
    }

    pub(super) fn update_delta_time(&mut self, delta: std::time::Duration) {
        self.imgui_context.io_mut()
            .update_delta_time(delta);
    }

    pub(super) fn prepare_frame(&mut self, window: &winit::window::Window) {
        self.imgui_winit
            .prepare_frame(self.imgui_context.io_mut(), window)
            .unwrap();
    }
    
    pub(super) fn render_imgui(&mut self) {
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
                    &self._imgui_texture_map,
                    dd,
                )
                .unwrap();
        }
    }
    
    pub(super) unsafe fn set_font(&mut self, bytes: &[u8], pixels: f32) -> Result<(), StdError> {
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
        fonts.tex_id = self._imgui_texture_map
            .register(tex)
            .unwrap();

        Ok(())
    }
}

pub(super) fn imgui_init(window: &winit::window::Window) -> (imgui::Context, imgui_winit_support::WinitPlatform) {
    let mut imgui_context = imgui::Context::create();

    // .ini file
    imgui_context.set_ini_filename(None);
    
    if let Ok(ini_contents) = std::fs::read_to_string("imgui.ini") {
        imgui_context.load_ini_settings(&ini_contents);
    }

    let mut imgui_winit = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
    imgui_winit.attach_window(
        imgui_context.io_mut(), 
        &window, 
        imgui_winit_support::HiDpiMode::Rounded
    );
    imgui_context.io_mut().font_global_scale = (1.0 / imgui_winit.hidpi_factor()) as f32;
    
    (imgui_context, imgui_winit)
}