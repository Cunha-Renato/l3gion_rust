use crate::StdError;

use super::{
    lg_types::reference::Rfc, 
    renderer::{
        opengl::opengl_init, 
        LgRenderer, Renderer
    }, 
    test_scene::TestScene
};

pub struct ApplicationCore {
    pub _window: winit::window::Window,
    pub renderer: Rfc<LgRenderer>,
}
pub struct Application {
    core: Rfc<ApplicationCore>,
    scene: TestScene,
}
impl Application {
    pub fn new_vulkan() -> Self {
        todo!()
    }
    pub fn new_opengl(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self, StdError> {
        let window_builder = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize{ width: 1080, height: 720 })
            .with_title("L3gion");

        let (window, gl_specs) = opengl_init::init_opengl(event_loop, window_builder)?;

        let renderer = Rfc::new(LgRenderer::opengl(gl_specs));
        let core = Rfc::new(ApplicationCore {
            _window: window,
            renderer
        });
        let scene = TestScene::new(core.clone());
        
        Ok(Self {
            core,
            scene,
        })
    } 
    pub fn request_redraw(&self) {
        self.core.borrow()._window.request_redraw();
    }
    pub fn on_update(&mut self) -> Result<(), StdError>{
        self.scene.on_update();

        unsafe {
            self.core.borrow().renderer.borrow_mut().render()?;
            
            Ok(())
        }
    }
    pub fn on_event(&mut self, event: winit::event::Event<()>) {
        match event {

            _ => ()            
        }
    }
    pub fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError>{
        unsafe {
            self.core.borrow().renderer.borrow().resize(new_size)?;
            
            Ok(())
        }
    }
    pub fn destroy(&mut self) {
        self.scene.destroy();
        unsafe { self.core.borrow_mut().renderer.borrow_mut().destroy().unwrap(); }
    }
}