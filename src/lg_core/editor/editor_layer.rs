// TODO: Move to it's own project

use sllog::info;
use crate::{lg_core::{application::ApplicationCore, asset_manager::AssetManager, camera::Camera, editor::{imgui_config::config_imgui, panels::status}, entity::LgEntity, event::{LgEvent, LgKeyCode}, frame_time::FrameTime, glm, layer::Layer, lg_types::units_of_time::{AsLgTime, LgTime}, renderer::{command::{ReceiveRendererCommand, SendDrawData, SendInstanceDrawData, SendRendererCommand, TextureOption}, render_target::{FramebufferFormat, RenderTargetSpecs}, texture::{self, TextureSpecs}, uniform::{LgUniformType, Uniform}}, timer::LgTimer, uuid::UUID, window::LgWindow}, lg_vertex, profile_function, profile_scope, profiler_begin, profiler_end, utils::tools::to_radians, StdError};
use crate::lg_core::renderer::vertex::LgVertex;

use super::panels::{self, assets::ImGuiAssetsPanel};

pub(crate) struct EditorLayer {
    _debug_name: String,
    viewport: [f32; 2],

    core: Option<ApplicationCore>,
    camera: Camera,
    entities: Vec<LgEntity>,

    // Misc / Testing
    profile: bool,
    light_position: glm::Vec3,
    
    render_imgui: bool,
    render_post_processing: bool,
    
    // Render Passes
    geometry_pass: RenderTargetSpecs,
    post_processing_pass: RenderTargetSpecs,
    imgui_correction_pass: RenderTargetSpecs,
    
    // Panels
    imgui_assets_panel: ImGuiAssetsPanel,
}
impl EditorLayer {
    pub(crate) fn new() -> Self {
        Self { 
            _debug_name: "EditorLayer".to_string(),
            viewport: [1080.0, 720.0],
            core: None, 
            camera: Camera::default(),
            entities: Vec::new(),

            profile: false,
            light_position: glm::vec3(-1.0, 0.0, 3.0),
            
            render_imgui: true,
            render_post_processing: true,
            geometry_pass: RenderTargetSpecs::default(),
            post_processing_pass: RenderTargetSpecs::default(),
            imgui_correction_pass: RenderTargetSpecs::default(),
            
            imgui_assets_panel: ImGuiAssetsPanel::new(),
        }
    }
}
// Private
impl EditorLayer {
    fn core(&self) -> &ApplicationCore {
        self.core.as_ref().unwrap()
    } 
}
impl Layer for EditorLayer {
    fn debug_name(&self) -> &str {
        &self._debug_name
    }

    fn on_attach(&mut self, app_core: ApplicationCore) -> Result<(), StdError> {
        profile_function!();
        self.imgui_assets_panel.init(app_core.renderer.borrow().asset_manager());

        let mut specs = RenderTargetSpecs {
            framebuffer_format: FramebufferFormat::RGB,
            clear: true,
            clear_color: glm::vec4(0.2, 0.2, 0.2, 1.0),
            clear_depth: 1.0,
            viewport: (0, 0, self.viewport[0] as i32, self.viewport[1] as i32),
            depth_test: true,
            depth_filter: texture::TextureFilter::LINEAR,
            color_texture_specs: TextureSpecs {
                tex_format: texture::TextureFormat::RGBA,
                tex_type: texture::TextureType::UNSIGNED_BYTE,
                tex_filter: texture::TextureFilter::LINEAR,
            },
        };
        self.geometry_pass = specs.clone();

        specs.depth_test = false;
        self.post_processing_pass = specs.clone();

        specs.framebuffer_format = FramebufferFormat::SRGB;
        specs.color_texture_specs.tex_format = texture::TextureFormat::SRGB8;
        self.imgui_correction_pass = specs;
        
        app_core.renderer.borrow().send(SendRendererCommand::CREATE_NEW_RENDER_PASS("GEOMETRY".to_string(), self.geometry_pass.clone()));
        app_core.renderer.borrow().send(SendRendererCommand::CREATE_NEW_RENDER_PASS("POST".to_string(), self.post_processing_pass.clone()));
        app_core.renderer.borrow().send(SendRendererCommand::CREATE_NEW_RENDER_PASS("IMGUI_CORRECTION".to_string(), self.imgui_correction_pass.clone()));

        self.camera = Camera::new(
            to_radians(45.0) as f32, 
            self.viewport[0],
            self.viewport[1],
            0.1, 
            1000.0
        );

        self.entities = vec![
            LgEntity::new(
                UUID::from_string("assets\\objects\\sphere.obj")?,
                UUID::from_string("assets\\materials\\BP_BRDF.lgmat")?,
                glm::vec3(0.0, 0.0, 0.0)
            ),
            LgEntity::new(
                UUID::from_string("assets\\objects\\sphere.obj")?,
                UUID::from_string("assets\\materials\\BP_BRDF.lgmat")?,
                glm::vec3(0.0, 0.0, 0.0)
            ),
        ];
        self.entities[1].set_scale(glm::vec3(0.3, 0.3, 0.3));

        config_imgui(&mut app_core.renderer.borrow_mut());
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
        self.entities[1].set_position(self.light_position);

        // Update uniform
        #[repr(C)]
        #[derive(Clone)]
        struct Camera {
            view: glm::Mat4,
            proj: glm::Mat4,
            dir: glm::Vec3,
            _padding: f32,
        }
        let view_proj = Camera {
            view: self.camera.get_view_matrix(),
            proj: self.camera.get_projection_matrix(),
            dir: self.camera.get_forward_direction().clone(),
            _padding: 0.0,
        };

        #[repr(C)]
        #[derive(Clone)]
        struct LightProperties {
            position: glm::Vec3,
            _padding1: f32,
            color: glm::Vec3,
            _padding2: f32,
        }
        let light_properties = LightProperties {
            position: self.light_position,
            _padding1: 0.0,
            color: glm::vec3(1.0, 1.0, 1.0),
            _padding2: 0.0,
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

        let uniforms = unsafe { vec![
            Uniform::new_with_data(
                "ViewProj", 
                LgUniformType::STRUCT, 
                0, 
                0, 
                true,
                view_proj.clone()
            ),
            Uniform::new_with_data(
                "LightProperties", 
                LgUniformType::STRUCT, 
                1, 
                1, 
                true,
                light_properties.clone()
            ),
        ]};

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
                    uniforms: uniforms.clone(),
                }));
            }
        }
        renderer.send(SendRendererCommand::DRAW_INSTANCED);

        // Post processing
        if self.render_post_processing {
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
        }

        // ImGui Gamma Correction
        if self.render_imgui {
            let pass = if self.render_post_processing { "POST".to_string() } else { "GEOMETRY".to_string() };
            renderer.send(SendRendererCommand::GET_PASS_COLOR_TEXTURE_GL(pass.clone()));
            if let Some(post_tex) = renderer.get_pass_color_texture_gl(pass) {
                renderer.send(SendRendererCommand::BEGIN_RENDER_PASS("IMGUI_CORRECTION".to_string()));

                renderer.send(SendRendererCommand::SEND_DRAW_DATA(SendDrawData {
                    mesh: UUID::from_string("assets\\objects\\ui_screen.obj")?,
                    material: UUID::from_string("assets\\materials\\IMGUI_CORRECTION.lgmat")?,
                    uniforms: vec![],
                    textures: vec![TextureOption::GL_TEXTURE(post_tex)],
                }));
            }
        }

        Ok(())
    }

    fn on_imgui(&mut self, ui: &mut imgui::Ui) {
        profile_function!();

        if !self.render_imgui { return; }

        unsafe {
            imgui::sys::igDockSpaceOverViewport(imgui::sys::igGetMainViewport(), 0, std::ptr::null());
        }

        self.imgui_viewport_panel(ui);
        self.imgui_settings_panel(ui);
        panels::status::imgui_status_panel(ui);
        self.imgui_assets_panel.imgui_assets_panel(ui);

        // ui.show_demo_window(&mut true);
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
                    renderer.send(SendRendererCommand::RESIZE_RENDER_PASS("IMGUI_CORRECTION".to_string(), (*width as i32, *height as i32)));
                },
                _ => (),
            },
            LgEvent::KeyEvent(e) => if e.pressed {
                match e.key {
                    LgKeyCode::W => self.light_position.y += 1.0,
                    LgKeyCode::A => self.light_position.x -= 1.0,
                    LgKeyCode::S => self.light_position.y -= 1.0,
                    LgKeyCode::D => self.light_position.x += 1.0,
                    LgKeyCode::Q => self.light_position.z += 1.0,
                    LgKeyCode::E => self.light_position.z -= 1.0,
                    LgKeyCode::P => {
                        if self.profile { profiler_end!("profiles/editor_layer"); }
                        else { profiler_begin!(); }

                        self.profile = !self.profile;
                    },
                    LgKeyCode::K => for i in 0..2_000u128 {
                        let mut entity = LgEntity::new(
                            UUID::from_string("assets\\objects\\cube.obj").unwrap(),
                            UUID::from_string("assets\\materials\\BP_BRDF.lgmat").unwrap(),
                            glm::vec3((i * 2) as f32, 0.0, 0.0)
                        );
                        entity.set_scale(glm::vec3(0.5, 0.5, 0.5));
                        self.entities.push(entity);
                    },
                    LgKeyCode::V => {
                        static mut V_SYNC: bool = false;
                        unsafe { 
                            V_SYNC = !V_SYNC; 
                            self.core().renderer.borrow().send(SendRendererCommand::SET_VSYNC(V_SYNC));
                        }
                    },
                    _ => (),
                };
            },
            _ => (),
        }

        false
    }
}

// ------------------------------------ ImGui panels ------------------------------------ 

impl EditorLayer {
    fn imgui_viewport_panel(&mut self, ui: &mut imgui::Ui) {
        let image = {    
            let mut renderer = self.core().renderer.borrow_mut();
            renderer.send(SendRendererCommand::GET_PASS_COLOR_TEXTURE_GL("IMGUI_CORRECTION".to_string()));
            renderer.get_pass_color_texture_gl("IMGUI_CORRECTION".to_string()).unwrap()
        };

        let _wp = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
        ui.window("Viewport")
            .size([200.0, 200.0], imgui::Condition::FirstUseEver)
            .bg_alpha(1.0)
            .title_bar(false)
            .collapsible(false)
            .draw_background(false)
            .scrollable(false)
            .scroll_bar(false)
            .build(|| {
                self.viewport = ui.content_region_avail();

                imgui::Image::new(
                    imgui::TextureId::new(image as usize),
                    self.viewport.clone()
                )
                .uv0([0.0, 1.0])
                .uv1([1.0, 0.0])
                .build(ui)
            });
    }
    
    fn imgui_settings_panel(&self, ui: &mut imgui::Ui) {
        ui.window("Settings")
            .build(|| {
                ui.text(std::format!("Profiling: {}", self.profile));
            });
    }
}