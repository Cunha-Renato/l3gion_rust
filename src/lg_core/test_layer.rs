use nalgebra_glm as glm;
use sllog::*;
use winit::window::Window;
use crate::utils::tools::to_radians;

use super::{input::LgInput, layer::Layer, lg_types::reference::Ref, renderer::{camera::Camera, object::Object, vertex::Vertex, Renderer}};

pub struct TestLayer {
    renderer: Ref<Renderer>,
    objects: Vec<Ref<Object<Vertex>>>,
    main_camera: Ref<Camera>,   
}
impl TestLayer {
    pub fn new(renderer: Ref<Renderer>) -> Self {        
        Self {
            objects: Vec::new(),
            main_camera: Ref::default(),
            renderer,
        }
    }
}
impl Layer for TestLayer {
    fn init(&mut self, window: Ref<Window>) {
        let vertices = [
            Vertex::new(glm::vec3(-0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3(0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3(0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let vertices2 = [
            Vertex::new(glm::vec3(-0.3, -0.5, 1.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3(0.8, -0.5, 1.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3(0.8, 0.5, 1.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.3, 0.5, 1.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let indices = [0, 1, 2, 2, 3, 0];

        let object = Object::new(vertices.to_vec(), indices.to_vec());
        let object2 = Object::new(vertices2.to_vec(), indices.to_vec());
        
        let objects = vec![
            Ref::new(object),
            Ref::new(object2),
        ];
        
        self.objects = objects;
        self.main_camera = Ref::new(Camera::new(
            to_radians(45.0) as f32, 
            window.borrow().inner_size().width as f32, 
            window.borrow().inner_size().height as f32, 
            0.1, 
            1000.0
        ));
        self.renderer.borrow_mut().set_camera(self.main_camera.clone());
    }

    fn on_update(&mut self, input: &LgInput) {
        self.main_camera.borrow_mut().on_update(input);
        unsafe {
            for object in &self.objects {
                self.renderer.borrow_mut().draw(object.clone()).unwrap();
            }
        }
    }

    fn on_event(&mut self, event: &super::event::LgEvent) {
        self.main_camera.borrow_mut().on_event(event);
    }

    fn destroy(&mut self) {
    }
}