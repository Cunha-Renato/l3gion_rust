use crate::{lg_core::{am::AssetManager, frame_time::FrameTime, lg_types::units_of_time::AsLgTime, renderer::{command::{ReceiveRendererCommand, SendDrawData, SendInstanceDrawData, SendRendererCommand, TextureOption}, render_target::RenderTargetSpecs, texture::TextureSpecs, uniform::LgUniformType}, window::LgWindow}, lg_vertex, profile_function, profile_scope, profiler_begin, profiler_end, utils::tools::to_radians, StdError};
use super::{application::ApplicationCore, camera::Camera, entity::LgEntity, event::{LgEvent, LgKeyCode}, layer::Layer, lg_types::reference::Rfc, renderer::{render_target::RenderTarget, uniform::Uniform}, uuid::UUID};
use crate::lg_core::renderer::vertex::LgVertex;
use nalgebra_glm as glm;
use sllog::info;

pub struct TestLayer {
    _debug_name: String,

    core: Option<ApplicationCore>,
    camera: Camera,
    entities: Vec<LgEntity>,
    profile: bool,
    
    geometry_pass: RenderTargetSpecs,
    post_processing_pass: RenderTargetSpecs,
}
impl TestLayer {
    pub fn new() -> Self {
        Self { 
            _debug_name: "TestLayer".to_string(),
            core: None, 
            camera: Camera::default(),
            entities: Vec::new(),
            profile: false,
            
            geometry_pass: RenderTargetSpecs::default(),
            post_processing_pass: RenderTargetSpecs::default(),
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
        
        let specs = RenderTargetSpecs {
            clear: true,
            clear_color: glm::vec4(0.5, 0.1, 0.2, 1.0),
            clear_depth: 1.0,
            viewport: (0, 0, vp.x as i32, vp.y as i32),
            depth_test: true,
            depth_filter: crate::lg_core::renderer::texture::TextureFilter::LINEAR,
            color_texture_specs: TextureSpecs {
                tex_format: crate::lg_core::renderer::texture::TextureFormat::RGBA,
                tex_type: crate::lg_core::renderer::texture::TextureType::UNSIGNED_BYTE,
                tex_filter: crate::lg_core::renderer::texture::TextureFilter::LINEAR,
            },
        };
        self.geometry_pass = specs.clone();
        self.post_processing_pass = specs;
        
        app_core.renderer.borrow().send(SendRendererCommand::CREATE_NEW_RENDER_PASS("GEOMETRY".to_string(), self.geometry_pass.clone()));
        app_core.renderer.borrow().send(SendRendererCommand::CREATE_NEW_RENDER_PASS("POST".to_string(), self.post_processing_pass.clone()));

        self.camera = Camera::new(
            to_radians(45.0) as f32, 
            vp.x,
            vp.y,
            0.1, 
            1000.0
        );

        self.entities = vec![
            LgEntity::new(
                UUID::from_u128(82133816883675309422823400350076070065), 
                UUID::from_u128(229355871321227895111753443892732218389), 
                glm::vec3(0.0, 0.0, 0.0)
            ),
        ];

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
        #[derive(Clone)]
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

        // TESTING
        #[derive(Debug, Clone, Copy)]
        struct InstanceVertex {
            row_0: glm::Vec4,
            row_1: glm::Vec4,
            row_2: glm::Vec4,
            tex_index: i32,
        }
        lg_vertex!(InstanceVertex, row_0, row_1, row_2, tex_index);


        let renderer = &mut self.core().renderer.borrow_mut();

        renderer.send(SendRendererCommand::BEGIN_RENDER_PASS("GEOMETRY".to_string()));
        for e in &self.entities {
            let instance_vertex = {
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
            };
            
            unsafe {
                renderer.send(SendRendererCommand::SEND_INSTANCE_DATA(SendInstanceDrawData {
                    mesh: e.mesh.clone(),
                    material: e.material.clone(),
                    instance_data: (instance_vertex.vertex_info(), vec![instance_vertex.clone()].align_to::<u8>().1.to_vec()),
                    uniforms: vec![Uniform::new_with_data(
                        "ViewMatrix", 
                        LgUniformType::STRUCT, 
                        0, 
                        0, 
                        true,
                        view_proj.clone(),
                    )],
                }));
            }
        }
        renderer.send(SendRendererCommand::DRAW_INSTANCED);

        // Post processing
        renderer.send(SendRendererCommand::GET_PASS_COLOR_TEXTURE_GL("GEOMETRY".to_string()));
        if let Some(geo_tex) = renderer.get_pass_color_texture_gl("GEOMETRY".to_string()) {
            renderer.send(SendRendererCommand::BEGIN_RENDER_PASS("POST".to_string()));

            renderer.send(SendRendererCommand::SEND_DRAW_DATA(SendDrawData {
                mesh: UUID::from_string("assets\\objects\\ui_screen.obj")?,
                material: UUID::from_string("assets\\materials\\post_processing_pass.lgmat")?,
                uniforms: vec![],
                textures: vec![TextureOption::GL_TEXTURE(geo_tex)],
            }));
        }

        Ok(())
    }

    fn on_event(&mut self, event: &LgEvent) -> bool {
        profile_function!();
        self.camera.on_event(event);

        match event {
            LgEvent::WindowEvent(e) => match e {
                crate::lg_core::event::WindowEvent::Resize(width, height) => {
                    let renderer = self.core().renderer.borrow();
                    renderer.send(SendRendererCommand::RESIZE_RENDER_PASS("GEOMETRY".to_string(), (*width as i32, *height as i32)));
                    renderer.send(SendRendererCommand::RESIZE_RENDER_PASS("POST".to_string(), (*width as i32, *height as i32)));
                },
                _ => (),
            },
            LgEvent::KeyEvent(e) => if e.pressed {
                if e.key == LgKeyCode::P  {
                    match self.profile {
                        true => {
                            info!("Ending Profile!");
                            self.profile = false;
                            profiler_end!("profiles/test_layer");
                        },
                        false => {
                            info!("Begining Profile!");
                            self.profile = true;
                            profiler_begin!();
                        },
                    }
                }
                if e.key == LgKeyCode::K {
                    for i in 0..2_000u128 {
                        let shader: u128 = if i % 2 == 0 { 229355871321227895111753443892732218389 } else { 325699289483174847292149352498212715256 };
                        self.entities.push(LgEntity::new(
                            UUID::from_u128(280168720002063226134650013125607437790), 
                            UUID::from_u128(shader as u128), 
                            glm::vec3((i * 2) as f32, 0.0, 0.0)
                        ));
                    }
                }
                if e.key == LgKeyCode::V {
                    static mut V_SYNC: bool = false;
                    unsafe { 
                        V_SYNC = !V_SYNC; 
                        self.core().renderer.borrow().send(SendRendererCommand::SET_VSYNC(V_SYNC));
                    }
                }
            },
            _ => (),
        }

        false
    }
}