use std::{cell::RefCell, rc::Rc};

use sllog::*;
use winit::window::Window;
use nalgebra_glm as glm;

use crate::{utils::tools::to_radians, MyError};
use super::{
    event::Event, input::Input, renderer::{ 
        camera::Camera, object::Object, vertex::Vertex, Renderer
    }
};
pub struct AppCore {
    window: Rc<Window>,
    renderer: Renderer,
    pub input: Input,
}

pub struct Application {
    pub core: AppCore,
    main_camera: Rc<RefCell<Camera>>,
}
impl Application {
    pub fn new(window: Window) -> Result<Self, MyError> {
        let (mut renderer, window) = unsafe {Renderer::init(window)?};
        let input = Input::new();
        let main_camera = Rc::new(RefCell::new(Camera::new(
            to_radians(45.0) as f32,
            window.inner_size().width as f32, 
            window.inner_size().height as f32, 
            0.1, 
            1000.0
        )));

        // set camera as a Rc

        let core = AppCore {
            window,
            renderer,
            input,
        };
        
        Ok(Self {
            core,
            main_camera,
        })
    }
    
    pub fn destroy(&mut self) {
        unsafe { self.core.renderer.destroy().unwrap() }
    }
     
    pub fn on_update(&mut self) {
        self.main_camera.borrow_mut().on_update(&self.core.input);
        //update camera
        let vertices = [
            Vertex::new(glm::vec3(-0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3(0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3(0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let indices = [0, 1, 2, 3, 4];
        let object = Object::new(vertices.to_vec(), indices.to_vec());

        unsafe {
            match self.core.renderer.draw(object) {
                Ok(_) => (),
                Err(e) => error!("{:?}", e),
            }
        };

        // Rendering
        unsafe { 
            match self.core.renderer.render() {
                Ok(_) => (),
                Err(e) => error!("{:?}", e),
            }
        }
    }

    pub fn on_event(&mut self, event: &Event) {
        self.main_camera.borrow_mut().on_event(event);
    }
}