use crate::{profiler_begin, profiler_end, utils::tools::to_radians, StdError};
use super::{application::ApplicationCore, camera::Camera, entity::LgEntity, event::{LgEvent, LgKeyCode}, layer::Layer, lg_types::reference::Rfc, renderer::uniform::Uniform, uuid::UUID};
use nalgebra_glm as glm;

pub struct TestLayer {
    core: Option<Rfc<ApplicationCore>>,
    entities: Vec<LgEntity>,
    camera: Camera,
    profile: bool,
}
impl TestLayer {
    pub fn new() -> Self {
        Self { 
            core: None, 
            entities: Vec::new(),
            camera: Camera::default(),
            profile: false,
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
            UUID::from_u128(94175893682642414160568079829868456088),
            UUID::from_u128(1),
            glm::vec3(0.0, 0.0, 0.0)
        ));

        self.core.as_mut().unwrap().borrow_mut().renderer.set_uniform(Uniform::new(
            "ViewMatrix", 
            lg_renderer::renderer::lg_uniform::LgUniformType::STRUCT, 
            0, 
            0, 
            true, 
        ));
        
        Ok(())
    }

    fn on_update(&mut self) {
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
            self.entities.iter().for_each(|e| self.core.as_mut().unwrap().borrow_mut().renderer.instance_entity(e).unwrap());
        }
    }

    fn on_event(&mut self, event: &LgEvent) -> bool {
        self.camera.on_event(&event);

        static mut ADDED: f32 = 0.1;

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
                    unsafe {
                        self.entities.extend((0..2_000)
                            .map(|_| {
                                ADDED += 0.1;
                                LgEntity::new(
                                    UUID::from_u128(94175893682642414160568079829868456088),
                                    UUID::from_u128(1),
                                    glm::vec3(-ADDED, 0.0, 0.0)
                                )
                            })
                            .collect::<Vec<_>>()
                        );
                    }
                }
                if e.key == LgKeyCode::J {
                    unsafe {
                        self.entities.extend((0..2)
                            .map(|_| {
                                ADDED += 0.1;
                                LgEntity::new(
                                    UUID::from_u128(279637899307357088197043655395897281162),
                                    UUID::from_u128(3),
                                    glm::vec3(ADDED * 5.0, 0.0, 0.0)
                                )
                            })
                            .collect::<Vec<_>>()
                        );
                    }
                }
                if e.key == LgKeyCode::I {
                    println!("Num entities: {}", self.entities.len());
                }
            },
            _ => (),
        }

        false
    }

    fn shutdown(&mut self) -> Result<(), StdError> {
        if self.profile {
            profiler_end!("profiles/test_layer");
        }

        Ok(())
    }
}