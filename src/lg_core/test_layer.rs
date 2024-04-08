use nalgebra_glm as glm;
use sllog::*;
use winit::window::Window;
use crate::{utils::tools::to_radians, MyError};

use super::{input::LgInput, layer::Layer, lg_types::reference::Rfc, renderer::{camera::Camera, object::{Object, Transformation}, texture::Texture, vertex::Vertex, Renderer}};

#[derive(Default)]
struct Textures {
    grid: Rfc<Texture>,
    viking: Rfc<Texture>,
    black: Rfc<Texture>,
    white: Rfc<Texture>,
}

pub struct TestLayer {
    renderer: Rfc<Renderer>,
    objects: Vec<Vec<Rfc<Object<Vertex>>>>,
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
        let black = Rfc::new(Texture::new("assets/textures/black.png")?);
        let white = Rfc::new(Texture::new("assets/textures/white.png")?);

        self.textures.grid = grid;
        self.textures.viking = viking;
        self.textures.black = black;
        self.textures.white = white;


        let vertices = [
            Vertex::new(glm::vec3(-0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3( 0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3( 0.5,  0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5,  0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let indices = [0, 1, 2, 2, 3, 0];

        let object = Object::new(
            self.textures.white.clone(), 
            Transformation {
                position: glm::vec3(-3.0, 3.0, 0.0),
                scale: glm::vec3(1.0, 1.0, 1.0),
                angle: 0.0,
                rotation_axis: glm::vec3(0.0, 0.0, 1.0),
            },
            vertices.to_vec(), 
            indices.to_vec()
        );
        
        let mut objects = Vec::new();
        for i in 0..5 {
            let mut objs = Vec::new();
            for j in 0..5 {
                let mut new_obj = object.replicate();
                new_obj.transform.position.x += j as f32 + j as f32 * 0.015;
                new_obj.transform.position.y -= i as f32 + i as f32 * 0.015;

                objs.push(Rfc::new(new_obj));                
            }
            objects.push(objs);
        }
        
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
            for vec in &self.objects {
                for obj in vec {
                    self.renderer.borrow_mut().draw(obj.clone())?;
                }
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