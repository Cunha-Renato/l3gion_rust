use std::sync::{Mutex, MutexGuard};
use std::{collections::HashMap, sync::OnceLock};

use nalgebra_glm as glm;

use crate::StdError;

use super::event::{
    LgKeyCode,
    MouseButton,
};

static INPUT: OnceLock<Mutex<LgInput>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct LgInput {
    key_states: HashMap<LgKeyCode, bool>,    
    mouse_states: HashMap<MouseButton, bool>,
    mouse_position: glm::Vec2,
}
impl LgInput {
    fn new() -> Self {
        Self { 
            key_states: HashMap::new(),
            mouse_states: HashMap::new(),
            mouse_position: glm::vec2(0.0, 0.0),
        }
    }
    
    pub fn get() -> Result<MutexGuard<'static, LgInput>, StdError> {
        if INPUT.get().is_none() {
            match INPUT.set(Mutex::new(LgInput::new())) {
                Err(_) => return Err("Failed to create Input! (LgInput)".into()),
                _ => ()
            }
        }

        Ok(INPUT.get().unwrap().lock()?)
    }

    pub fn set_mouse_state(&mut self, button: MouseButton, state: bool) {
        self.mouse_states.insert(button, state);
    }
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        *self.mouse_states.get(&button)
            .unwrap_or_else(|| &false)
    }
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position.x = x;
        self.mouse_position.y = y;
    }
    pub fn get_mouse_position(&self) -> glm::Vec2 {
        self.mouse_position
    }

    pub fn set_key_state(&mut self, key_code: LgKeyCode, state: bool) {
        self.key_states.insert(key_code, state);
    }
    pub fn is_key_pressed(&self, key: LgKeyCode) -> bool {
        *self.key_states.get(&key)
            .unwrap_or_else(|| &false)
    }
}