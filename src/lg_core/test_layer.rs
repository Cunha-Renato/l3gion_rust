use crate::{utils::tools::to_radians, StdError};
use super::{application::ApplicationCore, camera::Camera, entity::LgEntity, layer::Layer, lg_types::reference::Rfc, renderer::uniform::Uniform, uuid::UUID};
use nalgebra_glm as glm;

pub struct TestLayer {
    core: Option<Rfc<ApplicationCore>>,
    entities: Vec<LgEntity>,
    camera: Camera,
}
impl TestLayer {
    pub fn new() -> Self {
        Self { 
            core: None, 
            entities: Vec::new(),
            camera: Camera::default(),
        }
    }
}
impl Layer for TestLayer {
    fn init(&mut self, app_core: Rfc<ApplicationCore>) -> Result<(), StdError> {
        self.core = Some(app_core);
        
        let window = self.core.as_ref().unwrap().borrow().window.size();
        self.camera = Camera::new(
            to_radians(45.0) as f32,
            window.0 as f32,
            window.1 as f32,
            0.1,
            1000.0
        );
        self.entities.push(LgEntity::new(
            UUID::from_u128(279637899307357088197043655395897281162),
            UUID::from_u128(1),
            glm::vec3(0.5, 0.0, 0.0)
        ));
        self.entities.push(LgEntity::new(
            UUID::from_u128(94175893682642414160568079829868456088),
            UUID::from_u128(1),
            glm::vec3(-0.5, 0.0, 0.0)
        ));
        
        Ok(())
    }

    fn on_update(&mut self) {
        self.camera.on_update();

        struct ViewProj {
            view: glm::Mat4,
            proj: glm::Mat4,
        }

        let view_proj = ViewProj {
            view: self.camera.get_view_matrix().clone(),
            proj: self.camera.get_projection_matrix()
        };
        unsafe {
            self.core.as_mut().unwrap().borrow_mut().renderer.set_uniform(Uniform::new_with_data(
                "ViewMatrix", 
                lg_renderer::renderer::lg_uniform::LgUniformType::STRUCT, 
                0, 
                0, 
                true, 
                view_proj
            ));
        }

        unsafe {
            self.entities.iter().for_each(|e| self.core.as_mut().unwrap().borrow_mut().renderer.batch_entity(e).unwrap());
        }
    }

    fn on_event(&mut self, event: super::event::LgEvent) -> bool {
        self.camera.on_event(&event);

        false
    }

    fn shutdown(&mut self) -> Result<(), StdError> {

        Ok(())
    }
}