use std::sync::{Arc, Mutex, MutexGuard};
use std::{collections::HashMap, sync::OnceLock};

use nalgebra_glm as glm;

use crate::{profile_function, StdError};

use super::event::{
    LgEvent, LgKeyCode, MouseButton
};

static INPUT: OnceLock<Arc<Mutex<LgInput>>> = OnceLock::new();

#[derive(Default, Debug, Clone)]
pub struct LgInput {
    key_states: HashMap<LgKeyCode, bool>,    
    mouse_states: HashMap<MouseButton, bool>,
    mouse_position: glm::DVec2,
}
// Public
impl LgInput {
    pub fn get() -> Result<Arc<Mutex<LgInput>>, StdError> {
        profile_function!();
        Ok(Arc::clone(INPUT.get().ok_or("Failed to get Input! (LgInput)")?))
    }

    pub fn get_locked() -> Result<MutexGuard<'static, LgInput>, StdError> {
        INPUT.get().unwrap().lock().or(Err("Failed to get Input! (LgInput)".into()))
    }
    
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        profile_function!();
        *self.mouse_states.get(&button)
            .unwrap_or(&false)
    }
    
    pub fn get_mouse_position(&self) -> glm::DVec2 {
        profile_function!();
        self.mouse_position
    }
    
    pub fn is_key_pressed(&self, key: LgKeyCode) -> bool {
        profile_function!();
        *self.key_states.get(&key)
            .unwrap_or(&false)
    }
}
// Public(crate)
impl LgInput {
    pub(crate) fn init() -> Result<(), StdError> {
        profile_function!();
        match INPUT.set(Arc::new(Mutex::new(LgInput::default()))) {
            Err(_) => Err("Failed to create Input! (LgInput)".into()),
            _ => Ok(())
        }
    }
    
    pub(crate) fn on_event(&mut self, event: &LgEvent) {
        profile_function!();
        match event {
            LgEvent::KeyEvent(ke) => { self.key_states.insert(ke.key, ke.pressed); },
            LgEvent::MouseEvent(me) => match me {
                super::event::MouseEvent::ButtonEvent(mbe) => { self.mouse_states.insert(mbe.button, mbe.pressed); },
                super::event::MouseEvent::MoveEvent(mme) => {
                    self.mouse_position = mme.position;
                },
                _ => (),
            },
            _ => (),
        };
    }
}