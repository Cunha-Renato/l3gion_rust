use std::collections::HashMap;

use crate::{lg_core::{event::LgEvent, lg_types::reference::Rfc, uuid::UUID, window::LgWindow}, StdError};
use super::{component::window::{Window, WindowConfig}, is_inside, Condition};

const UI_MESH: UUID = UUID::from_u128(316691656959075038046595414025328715723);
const UI_MATERIAL: UUID = UUID::from_u128(4);

// Similar to Dear ImGui, but worse
pub struct Ui {
    window: Rfc<LgWindow>,
    
    current_window: Option<String>,
    pub(super) windows_config: HashMap<String, WindowConfig>,
}

// Public
impl Ui {
    pub fn new(window: Rfc<LgWindow>) -> Self {
        Self {
            window,
            current_window: None,
            windows_config: HashMap::new(),
        }
    }
    pub fn window(&mut self, label: &str, condition: Condition) -> Window {
        Window::new(self, label, condition)
    }
}

// Public(super)
impl Ui {
    pub(super) fn set_current_window(&mut self, label: String) {
        if let Some(previous) = self.current_window {
            let p_window_config = self.windows_config.get(&previous).unwrap();
            
            p_window_config.focused = false;
            p_window_config.active = false;
        }

        self.current_window = Some(label);
    }
}

// Public(crate)
impl Ui {
    pub(crate) fn on_update(&mut self) -> Result<(), StdError> {
        
        Ok(())
    }
    
    pub(crate) fn on_event(&mut self, event: &LgEvent) -> bool {
        false
    }
}
// Private 
impl Ui {
    fn window_on_event(&mut self, window: &mut WindowConfig, event: LgEvent) -> bool {
        match event {
            LgEvent::WindowEvent(_) => (),
            LgEvent::KeyEvent(_) => (),

            LgEvent::MouseEvent(me) => match me {
                crate::lg_core::event::MouseEvent::ButtonEvent(mbe) => {
                    if window.hover {
                        window.focused = true;
                        window.active = true;
                        
                        self.set_current_window(window.name);
                    }
                },
                crate::lg_core::event::MouseEvent::MoveEvent(mme) => {
                    window.hover = is_inside(
                        (mme.position.0 as u32, mme.position.1 as u32),
                            &window.position, 
                            &window.size,
                        );
                    
                    if window.active {
                        
                    }
                },
                _ => (),
            },
        }   

        false
    }
}