use std::{cell::RefCell, rc::Rc};

use sllog::*;
use winit::window::Window;

use crate::{lg_core::test_layer::TestLayer, MyError};
use super::{
    event::LgEvent, input::LgInput, layer::Layer, lg_types::reference::Rfc, renderer::Renderer
};
pub struct AppCore {
    window: Rfc<Window>,
    renderer: Rfc<Renderer>,
    pub input: LgInput,
}

pub struct Application {
    pub resized: bool,
    pub core: AppCore,
    layers: Vec<Rfc<dyn Layer>>,
}
impl Application {
    pub fn new(window: Window) -> Result<Self, MyError> {
        // optick::start_capture();
        optick::event!();
        
        let (renderer, window) = unsafe {Renderer::init(window)?};
        let input = LgInput::new();

        let renderer = Rfc::new(renderer);
        let core = AppCore {
            window: window.clone(),
            renderer: renderer.clone(),
            input,
        };
        
        let layers = vec![Rc::new(RefCell::new(TestLayer::new(renderer.clone())))];
        let layers: Vec<Rfc<dyn Layer>> = layers
            .iter()
            .map(|layer| Rfc::from_rc_refcell(&(layer.clone() as Rc<RefCell<dyn Layer>>)))
            .collect();
        
        for layer in &layers {
            layer.borrow_mut().init(window.clone())?;
        }

        Ok(Self {
            resized: false,
            core,
            layers,
        })
    }
    
    pub fn destroy(&mut self) -> Result<(), MyError>{
        optick::event!();
        for layer in &self.layers {
            layer.borrow_mut().destroy()?;
        }

        unsafe { self.core.renderer.borrow_mut().destroy().unwrap() }
        optick::stop_capture("profiling");
        
        Ok(())
    }
     
    pub fn on_update(&mut self) -> Result<(), MyError>{
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

    pub fn on_event(&mut self, event: &LgEvent) -> Result<(), MyError>{
        optick::event!();
        for layer in &self.layers {
            layer.borrow_mut().on_event(event)?;
        }
        
        Ok(())
    }
}