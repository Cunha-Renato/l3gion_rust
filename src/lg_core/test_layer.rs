use nalgebra_glm as glm;
use sllog::*;
use winit::window::Window;
use crate::{utils::tools::to_radians, MyError};

use super::{input::LgInput, layer::Layer, lg_types::reference::Rfc, renderer::{camera::Camera, object::Object, texture::Texture, vertex::Vertex, Renderer}};

#[derive(Default)]
struct Textures {
    grid: Rfc<Texture>,
    viking: Rfc<Texture>,
}

pub struct TestLayer {
    renderer: Rfc<Renderer>,
    objects: Vec<Rfc<Object<Vertex>>>,
    main_camera: Rfc<Camera>,
    textures: Textures,
}
impl TestLayer {
    pub fn new(renderer: Rfc<Renderer>) -> Self {        
        Self {
            objects: Vec::new(),
            main_camera: Rfc::default(),
            renderer,
            textures: Textures::default(),
        }
    }
}
impl Layer for TestLayer {
    fn init(&mut self, window: Rfc<Window>) -> Result<(), MyError>{
        // Load textures
        let grid = Rfc::new(Texture::new("assets/textures/grid.png")?);
        let viking = Rfc::new(Texture::new("assets/textures/viking_room.png")?);
        self.textures.grid = grid;
        self.textures.viking = viking;


        let vertices = [
            Vertex::new(glm::vec3(-0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3( 0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3( 0.5,  0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5,  0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let vertices2 = [
            Vertex::new(glm::vec3(-0.3, -0.5, 1.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3( 0.8, -0.5, 1.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3( 0.8,  0.5, 1.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.3,  0.5, 1.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let indices = [0, 1, 2, 2, 3, 0];

        let object = Object::new(self.textures.grid.clone(), vertices.to_vec(), indices.to_vec());
        let object2 = Object::new(self.textures.viking.clone(), vertices2.to_vec(), indices.to_vec());
        
        let objects = vec![
            Rfc::new(object),
            Rfc::new(object2),
        ];
        
        self.objects = objects;
        self.main_camera = Rfc::new(Camera::new(
            to_radians(45.0) as f32, 
            window.borrow().inner_size().width as f32, 
            window.borrow().inner_size().height as f32, 
            0.1, 
            1000.0
        ));
        self.renderer.borrow_mut().set_camera(self.main_camera.clone());
        
        Ok(())
    }

    fn on_update(&mut self, input: &LgInput) -> Result<(), MyError>{
        self.main_camera.borrow_mut().on_update(input);
        unsafe {
            for object in &self.objects {
                self.renderer.borrow_mut().draw(object.clone())?;
            }
        }
        
        Ok(())
    }

    fn on_event(&mut self, event: &super::event::LgEvent) -> Result<(), MyError> {
        self.main_camera.borrow_mut().on_event(event);
        
        Ok(())
    }

    fn destroy(&mut self) -> Result<(), MyError> {
        
        Ok(())
    }
}