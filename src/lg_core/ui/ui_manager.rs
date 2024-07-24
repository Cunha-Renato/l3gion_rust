use std::{borrow::Borrow, collections::HashMap};

use crate::{lg_core::{entity::LgEntity, event::{LgEvent, MouseEvent}, glm, input::LgInput, lg_types::reference::Rfc, renderer::Renderer, ui::component::constants::{self, WINDOW_TITTLE_HEIGHT}, uuid::UUID, window::LgWindow}, profile_function, StdError};
use super::{component::{constants::{WINDOW_MATERIAL, WINDOW_MESH}, window::{Window, WindowBuilder}}, is_inside, to_normalized_position, to_normalized_size, Condition, UiFlags};

// Similar to Dear ImGui, but worse
pub struct Ui {
    pub(super) window: Rfc<LgWindow>,
    screen_entity: LgEntity,

    mouse_delta: glm::Vec2,
    mouse_position: glm::Vec2,
    
    windows_vec: Vec<Rfc<Window>>,
    windows_hash: HashMap<String, Rfc<Window>>,
}

// Public
impl Ui {
    pub fn new(window: Rfc<LgWindow>) -> Self {
        Self {
            window,
            screen_entity: LgEntity::new(
                WINDOW_MESH.clone(), 
                WINDOW_MATERIAL.clone(), 
                glm::vec3(0.0, 0.0, 0.0),
            ),
            mouse_delta: glm::vec2(0.0, 0.0),
            mouse_position: glm::vec2(0.0, 0.0),
            windows_vec: Vec::new(),
            windows_hash: HashMap::new(),
        }
    }
    pub fn window(&mut self, label: &str, condition: Condition) -> WindowBuilder {
        WindowBuilder::new(self, label, condition)
    }
}

// Public(super)
impl Ui {
    pub(super) fn insert_window(&mut self, window: Window, condition: Condition) {
        profile_function!();
        let new_config = match self.windows_hash.entry(window.name.clone()) {
            std::collections::hash_map::Entry::Occupied(val) => {
                val.into_mut()
            },
            std::collections::hash_map::Entry::Vacant(entry) => {
                let window = entry.insert(Rfc::new(window.clone()));
                self.windows_vec.push(window.clone());
                
                window
            },
        };
        new_config.borrow_mut().flags = window.flags;

        if condition == Condition::ALWAYS {
            *new_config.borrow_mut() = window;
        }
    }
}

// Public(crate)
impl Ui {
    pub(crate) fn on_update(&mut self, renderer: &mut Renderer) -> Result<(), StdError> {
        /*profile_function!();

        #[derive(Clone, Copy)]
        struct UiInst {
            window_color: glm::Vec4,
            window_title_color: glm::Vec4,
            window_position_and_size: glm::Vec4,
            window_title_height: f32,
        }
        lg_vertex!(UiInst, window_color, window_title_color, window_position_and_size, window_title_height);

        // Drawing
        let mut inst_data = renderer.begin_instancing::<UiInst>();
        for window in &self.windows_vec {
            let window = &mut window.borrow_mut();
            if !window.flags.contains(UiFlags::SHOW) { continue; } // If isn't visible then don't calculate

            let screen_size = self.window.borrow().size();
            renderer.queue_instance(
                &self.screen_entity, 
                &mut inst_data, 
                |_| {
                    let window_color = window.color();
                    let window_title_color = window.title_color();

                    let window_position_and_size = glm::vec4(
                        window.position.x,
                        screen_size.y - window.position.y,
                        window.size.x,
                        window.size.y,
                    );

                    UiInst { 
                        window_color,
                        window_title_color, 
                        window_position_and_size,  
                        window_title_height: WINDOW_TITTLE_HEIGHT,
                    }
                }
            )?;
        }

        renderer.end_instancing(&mut inst_data)?;*/
        
        Ok(())
    }
    
    pub(crate) fn on_event(&mut self, event: &LgEvent) -> bool {
        profile_function!();

        // Self
        match event {
            LgEvent::MouseEvent(MouseEvent::MoveEvent(mme)) => {
                let mme_position = glm::vec2(mme.position.x as f32, mme.position.y as f32);
                self.mouse_delta = self.mouse_position - mme_position;
                self.mouse_position = mme_position;
            },
            _ => (),
        }

        // Windows
        let mut active_window = None;
        for window in &self.windows_vec {
            window.borrow_mut().on_event(event);

            if window.borrow().active { 
                active_window = Some(window.clone());
                break; 
            }
        }

        if let Some(window) = active_window {
            // Setting proper rendering order (Not ideal)
            if let Some(pos) = self.windows_vec.iter().position(|w| w.borrow().name == window.borrow().name) {
                let focused_window = self.windows_vec.remove(pos);
                self.windows_vec.insert(0, focused_window);
            }
        }

        false
    }
}