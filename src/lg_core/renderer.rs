pub mod camera;
pub mod renderer_core;
pub mod model;

use std::rc::Rc;

use renderer_core::{
    RCoreData,
    RCore,
    Vertex,
    glm,
};
use model::Model;
use winit::window::Window;

use crate::MyError;

pub struct Renderer {
    core: Rc<RCore>,
    data: RCoreData,
}
impl Renderer {
    pub fn init(window: &Window) -> Result<(Self, Renderer2D), MyError>{
        let mut data = RCoreData::default();
        let core = Rc::new(unsafe { RCore::create(window, &mut data)? });

        let renderer2d = Renderer2D::init(core.clone());

        Ok((Self {
            core,
            data: data,
            }, 
            renderer2d,
        ))
    }
    
    pub fn vsync(&mut self, option: bool) {
        todo!()
    }
    pub fn msaa(&mut self, value: u32) {
        todo!()
    }
    
    pub fn upload_models(&mut self, models: &[Model]) {
        todo!()
    }
}
pub struct DrawInfo {
    transform: glm::Mat4,
    color: glm::Vec4,
}
pub struct CircleInfo {
    draw_info: DrawInfo,
    radius: f32,
    thickness: Option<f32>,
}

struct VerticesInfo {
    vertices: [Vertex; 4],
    indices: [u16; 5],
}
pub struct Renderer2D {
    core: Rc<RCore>,
    vertices_info: VerticesInfo
}
impl Renderer2D {
    fn init(core: Rc<RCore>) -> Self {
        let vertices = [
            Vertex::new(glm::vec3(-0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3(0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3(0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let indices = [0, 1, 2, 3, 4];
        Self {
            core,
            vertices_info: VerticesInfo {
                vertices,
                indices,
            }
        }
    }
    pub fn shutdown(&mut self) {
        todo!()
    }

    pub fn begin_batch(&mut self) {
        todo!()
    }
    pub fn end_batch(&mut self) {
        
    }
    
    pub fn submit_quad(&mut self, info: &DrawInfo) {
        todo!()
    }
    pub fn submit_circle(&mut self, info: &CircleInfo) {
        todo!()
    }
}