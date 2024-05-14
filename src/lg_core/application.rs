use crate::StdError;

use super::{event::LgEvent, lg_types::reference::Rfc, renderer::LgRenderer, test_scene::TestScene, window::LgWindow};

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

pub struct ApplicationCore {
    _window: winit::window::Window,
    pub window: LgWindow,
    pub renderer: LgRenderer,
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
            .with_inner_size(winit::dpi::PhysicalSize{ width: WIDTH, height: HEIGHT })
            .with_title("L3gion");

        let (window, renderer) = lg_renderer::renderer::LgRenderer::new_opengl(event_loop, window_builder)?;

        let core = Rfc::new(ApplicationCore {
            _window: window,
            window: LgWindow::new(WIDTH, HEIGHT),
            renderer: LgRenderer::new(renderer)?,
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
        unsafe { self.core.borrow().renderer.begin(); }
        self.scene.on_update();
        unsafe { self.core.borrow().renderer.end()?; }
        Ok(())
    }
    pub fn on_event(&mut self, event: LgEvent) {
        self.scene.on_event(&event);
    }
    pub fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError>{
        unsafe {
            self.core.borrow_mut().window.set_size(new_size);
            self.core.borrow().renderer.resize(new_size)?;
            
            Ok(())
        }
    }
    pub fn destroy(&mut self) {
        self.scene.destroy();
    }
}