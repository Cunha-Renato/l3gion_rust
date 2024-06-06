use std::sync::{Arc, Mutex, MutexGuard};
use std::{collections::HashMap, sync::OnceLock};

use nalgebra_glm as glm;

use crate::{profile_function, StdError};

use super::event::{
    LgKeyCode,
    MouseButton,
};

static INPUT: OnceLock<Arc<Mutex<LgInput>>> = OnceLock::new();

#[derive(Default, Debug, Clone)]
pub struct LgInput {
    key_states: HashMap<LgKeyCode, bool>,    
    mouse_states: HashMap<MouseButton, bool>,
    mouse_position: glm::Vec2,
}
impl LgInput {
    pub(crate) fn init() -> Result<(), StdError> {
        profile_function!();
        match INPUT.set(Arc::new(Mutex::new(LgInput::default()))) {
            Err(_) => return Err("Failed to create Input! (LgInput)".into()),
            _ => Ok(())
        }
    }
    
    pub fn get() -> Result<Arc<Mutex<LgInput>>, StdError> {
        profile_function!();
        Ok(Arc::clone(INPUT.get().ok_or("Failed to get Input! (LgInput)")?))
    }
    pub fn get_locked() -> Result<MutexGuard<'static, LgInput>, StdError> {
        INPUT.get().unwrap().lock().or(Err("Failed to get Input! (LgInput)".into()))
    }

    pub(crate) fn set_mouse_state(&mut self, button: MouseButton, state: bool) {
        profile_function!();
        self.mouse_states.insert(button, state);
    }
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        profile_function!();
        *self.mouse_states.get(&button)
            .unwrap_or_else(|| &false)
    }
    pub(crate) fn set_mouse_position(&mut self, x: f32, y: f32) {
        profile_function!();
        self.mouse_position.x = x;
        self.mouse_position.y = y;
    }
    pub fn get_mouse_position(&self) -> glm::Vec2 {
        profile_function!();
        self.mouse_position
    }

    pub(crate) fn set_key_state(&mut self, key_code: LgKeyCode, state: bool) {
        profile_function!();
        self.key_states.insert(key_code, state);
    }
    pub fn is_key_pressed(&self, key: LgKeyCode) -> bool {
        profile_function!();
        *self.key_states.get(&key)
            .unwrap_or_else(|| &false)
    }
}