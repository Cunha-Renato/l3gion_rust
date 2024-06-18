use std::{borrow::Borrow, collections::HashMap};

use lg_renderer::lg_vertex;
use sllog::info;

use crate::{lg_core::{event::{LgEvent, MouseEvent}, glm, lg_types::reference::Rfc, renderer::LgRenderer, ui::component::constants::{self, WINDOW_COLOR, WINDOW_TITTLE_HEIGHT}, uuid::UUID, window::LgWindow}, profile_function, StdError};
use super::{component::window::{Window, WindowConfig}, is_inside, to_normalized_position, to_normalized_size, Condition, UiFlags};

// Similar to Dear ImGui, but worse
pub struct Ui {
    pub(super) window: Rfc<LgWindow>,

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
            row_0: glm::Vec4,
            row_1: glm::Vec4,
            row_2: glm::Vec4,            
        }
        lg_vertex!(UiInst, window_color, row_0, row_1, row_2);

        // Drawing
        let mut inst_data = renderer.begin_instancing::<UiInst>();
        for window in &self.windows_vec {
            let window = &mut window.borrow_mut();
            if !window.flags.contains(UiFlags::SHOW) { continue; } // If isn't visible then don't calculate

            renderer.queue_instance(
                &window._title_bar_entity, 
                &mut inst_data, 
                |_| {
                    let mut title_bar_position = window.position;
                    title_bar_position.y -= WINDOW_TITTLE_HEIGHT;
                    let mut title_bar_size = window.size;
                    title_bar_size.y = WINDOW_TITTLE_HEIGHT;

                    let identity = glm::Mat4::identity();
                    let translation = glm::translate(
                        &identity, 
                        &glm::vec2_to_vec3(&to_normalized_position(&self.window.borrow().size(), &title_bar_position, &title_bar_size))
                    );
                    let scale = glm::scale(
                        &identity, 
                        &glm::vec2_to_vec3(&to_normalized_size(&self.window.borrow().size(), &title_bar_size))
                    );

                    let model = translation * scale;
                    let row_0 = glm::vec4(model[(0, 0)], model[(0, 1)], model[(0, 2)], model[(0, 3)]);
                    let row_1 = glm::vec4(model[(1, 0)], model[(1, 1)], model[(1, 2)], model[(1, 3)]);
                    let row_2 = glm::vec4(model[(2, 0)], model[(2, 1)], model[(2, 2)], model[(2, 3)]);

                    let title_bar_color = if window.focused { constants::WINDOW_TITTLE_COLOR_FOCUSED } else { constants::WINDOW_TITTLE_COLOR };

                    UiInst { 
                        window_color: title_bar_color,
                        row_0,
                        row_1, 
                        row_2,
                    }
                }
            )?;

            renderer.queue_instance(
                &window._window_entity, 
                &mut inst_data, 
                |_| {
                    let identity = glm::Mat4::identity();
                    let translation = glm::translate(
                        &identity, 
                        &glm::vec2_to_vec3(&to_normalized_position(&self.window.borrow().size(), &window.position, &window.size))
                    );
                    let scale = glm::scale(
                        &identity, 
                        &glm::vec2_to_vec3(&to_normalized_size(&self.window.borrow().size(), &window.size))
                    );

                    let model = translation * scale;
                    let row_0 = glm::vec4(model[(0, 0)], model[(0, 1)], model[(0, 2)], model[(0, 3)]);
                    let row_1 = glm::vec4(model[(1, 0)], model[(1, 1)], model[(1, 2)], model[(1, 3)]);
                    let row_2 = glm::vec4(model[(2, 0)], model[(2, 1)], model[(2, 2)], model[(2, 3)]);

                    let window_color = WINDOW_COLOR;

                    UiInst { 
                        window_color,
                        row_0,
                        row_1, 
                        row_2,
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
        let mut windows_vec = self.windows_vec.clone(); // Hey I know this is ugly as fuck, and I know heap alocation and lookup is slow.
        // println!("+===============================================+");
        for window in &windows_vec {
            let window = &mut window.borrow_mut();
            // println!("{}", window.name);

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