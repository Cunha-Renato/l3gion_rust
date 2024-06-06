use crate::{profile_function, profiler_begin, profiler_end, utils::tools::to_radians, StdError};
use super::{application::ApplicationCore, camera::Camera, entity::LgEntity, event::{LgEvent, LgKeyCode}, layer::Layer, lg_types::reference::Rfc, renderer::uniform::Uniform, uuid::UUID};
use nalgebra_glm as glm;

pub struct TestLayer {
    core: Option<Rfc<ApplicationCore>>,
    camera: Camera,
    entities: Vec<LgEntity>,
    profile: bool,
}
impl TestLayer {
    pub fn new() -> Self {
        Self { 
            core: None, 
            camera: Camera::default(),
            entities: Vec::new(),
            profile: false,
        }
    }
}
impl Layer for TestLayer {
    fn on_attach(&mut self, app_core: Rfc<ApplicationCore>) -> Result<(), StdError> {
        profile_function!();
        let vp = app_core.borrow().window.size();
        self.camera = Camera::new(
            to_radians(45.0) as f32, 
            vp.0 as f32, 
            vp.1 as f32, 
            0.1, 
            1000.0
        );
        app_core.borrow_mut().renderer.set_uniform(Uniform::new(
            "ViewMatrix", 
            lg_renderer::renderer::lg_uniform::LgUniformType::STRUCT, 
            0, 
            0, 
            true,
        ));

        self.core = Some(app_core);
        
        Ok(())
    }
    
    fn on_detach(&mut self) -> Result<(), StdError> {
        profile_function!();
        if self.profile {
            profiler_end!("profiles/test_layer");
        }

        Ok(())
    }

    fn on_update(&mut self) -> Result<(), StdError> {
        profile_function!();
        self.camera.on_update();
        
        // Update uniform
        struct ViewProj {
            view: glm::Mat4,
            proj: glm::Mat4,
        }
        let view_proj = ViewProj {
            view: self.camera.get_view_matrix().clone(),
            proj: self.camera.get_projection_matrix()
        };
        self.core.as_mut().unwrap().borrow_mut().renderer.update_uniform("ViewMatrix", &view_proj);

        unsafe {
            for e in &self.entities {
                self.core.as_ref().unwrap()
                    .borrow_mut().renderer.instance_entity(&e)?;
            }
        }
        
        Ok(())
    }

    fn on_event(&mut self, event: &LgEvent) -> bool {
        profile_function!();
        self.camera.on_event(event);

        match event {
            LgEvent::KeyEvent(e) => if e.pressed {
                if e.key == LgKeyCode::F12  {
                    match self.profile {
                        true => {
                            self.profile = false;
                            profiler_end!("profiles/test_layer");
                        },
                        false => {
                            self.profile = true;
                            profiler_begin!();
                        },
                    }
                }
                if e.key == LgKeyCode::K {
                    for i in 0..2_000 {
                        let shader = if i % 2 == 0 { 1 } else { 2 };
                        self.entities.push(LgEntity::new(
                            UUID::from_u128(280168720002063226134650013125607437790), 
                            UUID::from_u128(shader as u128), 
                            glm::vec3((i * 2) as f32, 0.0, 0.0)
                        ));
                    }
                }
                if e.key == LgKeyCode::J {
                    
                }
                if e.key == LgKeyCode::I {
                }
            },
            _ => (),
        }

        false
    }
}