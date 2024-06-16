use crate::{lg_core::{frame_time::FrameTime, lg_types::units_of_time::AsLgTime, ui::{component::UiComponentCreateInfo, UiOffset, UiTotalOffset, UiUnit}, window::LgWindow}, profile_function, profile_scope, profiler_begin, profiler_end, utils::tools::to_radians, StdError};
use super::{application::ApplicationCore, camera::Camera, entity::LgEntity, event::{LgEvent, LgKeyCode}, layer::Layer, lg_types::reference::Rfc, renderer::uniform::Uniform, uuid::UUID};
use lg_renderer::lg_vertex;
use nalgebra_glm as glm;

pub struct TestLayer {
    _debug_name: String,

    core: Option<ApplicationCore>,
    camera: Camera,
    entities: Vec<LgEntity>,
    profile: bool,
}
impl TestLayer {
    pub fn new() -> Self {
        Self { 
            _debug_name: "TestLayer".to_string(),
            core: None, 
            camera: Camera::default(),
            entities: Vec::new(),
            profile: false,
        }
    }
}
// Private
impl TestLayer {
    fn core(&self) -> &ApplicationCore {
        self.core.as_ref().unwrap()
    } 
}
impl Layer for TestLayer {
    fn debug_name(&self) -> &str {
        &self._debug_name
    }

    fn on_attach(&mut self, app_core: ApplicationCore) -> Result<(), StdError> {
        profile_function!();
        let vp = app_core.window.borrow().size();

        self.camera = Camera::new(
            to_radians(45.0) as f32, 
            vp.0 as f32, 
            vp.1 as f32, 
            0.1, 
            1000.0
        );
        app_core.renderer.borrow_mut().set_uniform(Uniform::new(
            "ViewMatrix", 
            lg_renderer::renderer_core::lg_uniform::LgUniformType::STRUCT, 
            0, 
            0, 
            true,
        ));

        self.entities = vec![
            LgEntity::new(
                UUID::from_u128(316691656959075038046595414025328715723), 
                UUID::from_u128(1), 
                glm::vec3(0.0, 0.0, 0.0)
            ),
            LgEntity::new(
                UUID::from_u128(316691656959075038046595414025328715723), 
                UUID::from_u128(1), 
                glm::vec3(0.5, 0.0, 0.0)
            )];

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

        self.core().ui.borrow_mut().begin_frame(&UiComponentCreateInfo {
            name: "frame1".to_string(),
            offset: UiTotalOffset::default(),
            scale: (UiUnit::PIXEL(400), UiUnit::PIXEL(100)),
        });
        self.core().ui.borrow_mut().end_frame();
        self.core().ui.borrow_mut().begin_frame(&UiComponentCreateInfo {
            name: "frame2".to_string(),
            offset: UiTotalOffset::default(),
            scale: (UiUnit::PIXEL(100), UiUnit::PIXEL(150)),
        });
        self.core().ui.borrow_mut().end_frame();
        self.core().ui.borrow_mut().begin_frame(&UiComponentCreateInfo {
            name: "frame3".to_string(),
            offset: UiTotalOffset::default(),
            scale: (UiUnit::PIXEL(600), UiUnit::PIXEL(150)),
        });
        self.core().ui.borrow_mut().end_frame();

        // Update uniform
        struct ViewProj {
            view: glm::Mat4,
            proj: glm::Mat4,
        }
        let view_proj = ViewProj {
            view: self.camera.get_view_matrix(),
            proj: self.camera.get_projection_matrix()
            // view: glm::Mat4::identity(),
            // proj: glm::Mat4::identity(),
        };
        self.core().renderer.borrow_mut().update_uniform("ViewMatrix", &view_proj);
        
        // TESTING
        #[derive(Clone, Debug)]
        struct InstanceVertex {
            row_0: glm::Vec4,
            row_1: glm::Vec4,
            row_2: glm::Vec4,
            tex_index: i32,
        }
        lg_vertex!(InstanceVertex, row_0, row_1, row_2, tex_index);

        let renderer = &mut self.core().renderer.borrow_mut();
        let mut instance_data = renderer.begin_instancing::<InstanceVertex>();

        for e in &self.entities {
            /* self.core.as_ref().unwrap()
                .borrow_mut().renderer.instance_entity(&e)?; */
            renderer.queue_instance(e, &mut instance_data, |e| {
                let model = e.model();
                let row_0 = glm::vec4(model[(0, 0)], model[(0, 1)], model[(0, 2)], model[(0, 3)]);
                let row_1 = glm::vec4(model[(1, 0)], model[(1, 1)], model[(1, 2)], model[(1, 3)]);
                let row_2 = glm::vec4(model[(2, 0)], model[(2, 1)], model[(2, 2)], model[(2, 3)]);
                InstanceVertex {
                    row_0,
                    row_1,
                    row_2,
                    tex_index: 0// TODO: For now is only one texture
                }
            })?;
        }
        
        renderer.end_instancing(&mut instance_data)?;
        
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
                if e.key == LgKeyCode::V {
                    let renderer = &mut self.core().renderer.borrow_mut();
                    if renderer.is_vsync() {
                        renderer.set_vsync(false);
                    } else {
                        renderer.set_vsync(true);
                    }
                }
                if e.key == LgKeyCode::I {
                }
            },
            _ => (),
        }

        false
    }
}