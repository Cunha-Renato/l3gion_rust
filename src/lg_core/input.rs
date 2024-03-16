use std::collections::HashMap;

use nalgebra_glm as glm;

use super::event::{
    VKeyCode,
    MouseButton,
};

#[derive(Debug, Clone)]
pub struct Input {
    key_states: HashMap<VKeyCode, bool>,    
    mouse_states: HashMap<MouseButton, bool>,
    mouse_position: glm::Vec2,
}
impl Input {
    pub fn new() -> Self {
        Self { 
            key_states: HashMap::new(),
            mouse_states: HashMap::new(),
            mouse_position: glm::vec2(0.0, 0.0),
        }
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
    pub fn get_mouse_position(&self) -> &glm::Vec2 {
        &self.mouse_position
    }

    pub fn set_key_state(&mut self, key_code: VKeyCode, state: bool) {
        self.key_states.insert(key_code, state);
    }
    pub fn is_key_pressed(&self, key: VKeyCode) -> bool {
        *self.key_states.get(&key)
            .unwrap_or_else(|| &false)
    }
}