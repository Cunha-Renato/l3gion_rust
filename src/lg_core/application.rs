use crate::{as_dyn, profile_function, profile_scope, StdError};
use super::{event::LgEvent, input::LgInput, layer::Layer, lg_types::reference::Rfc, renderer::LgRenderer, window::LgWindow};

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

pub struct ApplicationCore {
    _window: winit::window::Window,
    pub window: LgWindow,
    pub renderer: LgRenderer,
}
pub struct Application {
    core: Rfc<ApplicationCore>,
    layers: Vec<Rfc<dyn Layer>>
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

        let result = Self {
            core,
            layers: Vec::new(),
        };

        LgInput::init()?;
        result.core.borrow_mut().renderer.init()?;
      
        Ok(result)
    } 
    pub fn shutdown(&mut self) -> Result<(), StdError> {
        profile_function!();

        while let Some(layer) = self.layers.pop() {
            layer.borrow_mut().on_detach()?;
        }
        
        Ok(())
    }
    pub fn push_layer(&mut self, mut layer: impl Layer + 'static) -> Result<(), StdError> {
        layer.on_attach(self.core.clone())?;
        self.layers.push(as_dyn!(layer, dyn Layer));

        Ok(())
    }
    pub fn pop_layer(&mut self) -> Result<(), StdError> {
        match self.layers.pop() {
            Some(layer) => layer.borrow_mut().on_detach()?,
            None => (),
        };

        Ok(())
    }
    pub fn request_redraw(&self) {
        profile_function!();
        self.core.borrow()._window.request_redraw();
    }
    pub fn on_update(&mut self) -> Result<(), StdError> {
        optick::next_frame();
        profile_function!();

        unsafe { 
            profile_scope!("render_begin");
            self.core.borrow().renderer.begin(); 
        }
        {
            profile_scope!("layers_on_update");
            for layer in &self.layers {
                layer.borrow_mut().on_update()?;
            }
        }

        unsafe { 
            profile_scope!("render_end");
            self.core.borrow_mut().renderer.end()?; 
        }

        Ok(())
    }
    pub fn on_event(&mut self, event: LgEvent) {
        profile_function!();
        for layer in &self.layers {
            if layer.borrow_mut().on_event(&event) {
                break;
            }
        }
    }
    pub fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError>{
        profile_function!();
        unsafe {
            self.core.borrow_mut().window.set_size(new_size);
            self.core.borrow().renderer.resize(new_size)?;
            
            Ok(())
        }
    }
}