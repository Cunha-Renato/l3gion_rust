use std::{borrow::Borrow, collections::HashMap};

use lg_renderer::lg_vertex;
use sllog::info;

use crate::{lg_core::{entity::LgEntity, event::{LgEvent, MouseEvent}, glm, lg_types::reference::Rfc, renderer::LgRenderer, ui::component::constants::{self, WINDOW_COLOR, WINDOW_TITTLE_HEIGHT}, uuid::UUID, window::LgWindow}, profile_function, StdError};
use super::{component::{constants::{WINDOW_MATERIAL, WINDOW_MESH}, window::{Window, WindowConfig}}, is_inside, to_normalized_position, to_normalized_size, Condition, UiFlags};

// Similar to Dear ImGui, but worse
pub struct Ui {
    pub(super) window: Rfc<LgWindow>,
    screen_entity: LgEntity,

    mouse_delta: glm::Vec2,
    mouse_position: glm::Vec2,
    
    current_window: Option<String>,

    windows_vec: Vec<Rfc<WindowConfig>>,
    windows_hash: HashMap<String, Rfc<WindowConfig>>,
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
            current_window: None,
            windows_vec: Vec::new(),
            windows_hash: HashMap::new(),
        }
    }
    pub fn window(&mut self, label: &str, condition: Condition) -> Window {
        Window::new(self, label, condition)
    }
}

// Public(super)
impl Ui {
    pub(super) fn insert_window(&mut self, config: WindowConfig, condition: Condition) {
        profile_function!();
        let new_config = match self.windows_hash.entry(config.name.clone()) {
            std::collections::hash_map::Entry::Occupied(val) => {
                val.into_mut()
            },
            std::collections::hash_map::Entry::Vacant(entry) => {
                let config = entry.insert(Rfc::new(config.clone()));
                self.windows_vec.push(config.clone());
                
                config
            },
        };
        new_config.borrow_mut().flags = config.flags;

        if condition == Condition::ALWAYS {
            *new_config.borrow_mut() = config;
        }
    }
    pub(super) fn set_current_window(&mut self, config: &mut WindowConfig) {
        profile_function!();

        if let Some(previous) = &mut self.current_window {
            if *previous == config.name { return; }
            let p_window_config = self.windows_hash.get_mut(previous).unwrap();
            
            p_window_config.borrow_mut().focused = false;
            p_window_config.borrow_mut().active = false;
            p_window_config.borrow_mut().hover = false;
        }

        self.current_window = Some(config.name.clone());
    }
}

// Public(crate)
impl Ui {
    pub(crate) fn on_update(&mut self, renderer: &mut LgRenderer) -> Result<(), StdError> {
        profile_function!();

        #[derive(Clone, Copy)]
        struct UiInst {
            window_color: glm::Vec4,
            window_title_color: glm::Vec4,
            window_position_and_size: glm::Vec4,
            window_title_position_and_size: glm::Vec4,
        }
        lg_vertex!(UiInst, window_color, window_title_color, window_position_and_size, window_title_position_and_size);

        // Drawing
        let mut inst_data = renderer.begin_instancing::<UiInst>();
        for window in &self.windows_vec {
            let window = &mut window.borrow_mut();
            if !window.flags.contains(UiFlags::SHOW) { continue; } // If isn't visible then don't calculate

            renderer.queue_instance(
                &self.screen_entity, 
                &mut inst_data, 
                |_| {
                    let mut title_bar_position = window.position;
                    title_bar_position.y -= WINDOW_TITTLE_HEIGHT;
                    let mut title_bar_size = window.size;
                    title_bar_size.y = WINDOW_TITTLE_HEIGHT;


                    let window_title_color = if window.focused { constants::WINDOW_TITTLE_COLOR_FOCUSED } else { constants::WINDOW_TITTLE_COLOR };

                    let screen_size = self.window.borrow().size();
                    let window_position_and_size = glm::vec4(
                        window.position.x,
                        screen_size.y - window.position.y,
                        window.size.x,
                        window.size.y,
                    );
                    let window_title_position_and_size = glm::vec4(
                        title_bar_position.x,
                        screen_size.y - title_bar_position.y,
                        title_bar_size.x,
                        title_bar_size.y,
                    );
                    UiInst { 
                        window_color: WINDOW_COLOR, 
                        window_title_color, 
                        window_position_and_size,  
                        window_title_position_and_size,
                    }
                }
            )?;
        }

        renderer.end_instancing(&mut inst_data)?;
        
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
        let mut windows_vec = std::mem::take(&mut self.windows_vec); // Hey I know this is ugly as fuck, and I know heap alocation and lookup is slow.
        for window in &windows_vec {
            let window = &mut window.borrow_mut();

            if !window.flags.contains(UiFlags::SHOW) { continue; }

            match event {
                LgEvent::WindowEvent(_) => (),
                LgEvent::KeyEvent(_) => (),

                // FIXME: If the flag SHOW is set to false when draging the window and if the button is released
                // the window will keep folowing the cursor because the window is not aware of the mouse button release event, so it will still 
                // be marked as active.
                LgEvent::MouseEvent(me) => match me {
                    crate::lg_core::event::MouseEvent::ButtonEvent(mbe) => {
                        if window.hover {
                            if mbe.pressed {
                                window.focused = true;
                                window.active = true;
                                self.set_current_window(window);
                            } else {
                                window.active = false;
                            }
                        } else {
                            window.focused = false;
                            window.active = false;
                        }
                    },
                    crate::lg_core::event::MouseEvent::MoveEvent(mme) => {
                        window.hover = is_inside(
                                &glm::vec2(mme.position.x as f32, mme.position.y as f32),
                                &window.position, 
                                &window.size,
                            );
                        
                        if window.active {
                            window.position -= self.mouse_delta;
                        }
                    },
                    _ => (),
                },
            }   
            if window.active { break; }
        }
        if let Some(active) = &self.current_window {
            let window = self.windows_hash.get(active).unwrap();
            
            if let Some(pos) = windows_vec.iter().position(|w| w.borrow().name == window.borrow().name) {
                let focused_window = windows_vec.remove(pos);
                windows_vec.insert(0, focused_window);
            }
        }
        self.windows_vec = windows_vec;

        false
    }
}