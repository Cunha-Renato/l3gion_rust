use std::{cell::RefCell, rc::Rc};

use winit::window::Window;

use crate::{as_dyn, lg_core::test_layer::TestLayer, StdError};
use super::{
    event::LgEvent, input::LgInput, layer::Layer, lg_types::reference::Rfc, renderer::Renderer
};
pub struct AppCore {
    _window: Rfc<Window>,
    renderer: Rfc<Renderer>,
    pub input: LgInput,
}

pub struct Application {
    pub resized: bool,
    pub core: AppCore,
    layers: Vec<Rfc<dyn Layer>>,
}
impl Application {
    pub fn new(window: Window) -> Result<Self, StdError> {
        // optick::start_capture();
        optick::event!();
        
        let (renderer, window) = unsafe {Renderer::init(window)?};
        let input = LgInput::new();

        let renderer = Rfc::new(renderer);
        let core = AppCore {
            _window: window.clone(),
            renderer: renderer.clone(),
            input,
        };
        
        let layers = vec![as_dyn!(TestLayer::new(renderer.clone()), dyn Layer)];
        
        for layer in &layers {
            layer.borrow_mut().init(window.clone())?;
        }

        Ok(Self {
            resized: false,
            core,
            layers,
        })
    }
    
    pub fn destroy(&mut self) -> Result<(), StdError>{
        optick::event!();
        for layer in &self.layers {
            layer.borrow_mut().destroy()?;
        }

        unsafe { self.core.renderer.borrow_mut().destroy().unwrap() }
        // optick::stop_capture("profiling");
        
        Ok(())
    }
     
    pub fn on_update(&mut self) -> Result<(), StdError>{
        optick::next_frame();
        optick::event!();
        
        self.core.renderer.borrow_mut().resized = self.resized;
        self.resized = false;

        for layer in &self.layers {
            layer.borrow_mut().on_update(&self.core.input)?;
        }

        // Rendering
        unsafe { 
            self.core.renderer.borrow_mut().render()?;
        }
        
        Ok(())
    }

    pub fn on_event(&mut self, event: &LgEvent) -> Result<(), StdError>{
        optick::event!();
        for layer in &self.layers {
            layer.borrow_mut().on_event(event)?;
        }
        
        Ok(())
    }
}