use std::collections::HashMap;
use nalgebra_glm as glm;
use crate::profile_function;

use super::event::{
    LgEvent, LgKeyCode, MouseButton
};

static mut INPUT: once_cell::sync::OnceCell<LgInput> = once_cell::sync::OnceCell::new();

#[derive(Debug)]
pub struct LgInput {
    key_states: HashMap<LgKeyCode, bool>,    
    mouse_states: HashMap<MouseButton, bool>,
    mouse_position: glm::DVec2,
    mouse_delta: glm::DVec2,
}

// Public
impl LgInput {
    pub fn get() -> Option<&'static Self> {
        profile_function!();
        unsafe { INPUT.get() }
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        profile_function!();
        *self.mouse_states.get(&button)
            .unwrap_or(&false)
    }
    
    pub fn mouse_position(&self) -> glm::DVec2 {
        profile_function!();
        self.mouse_position
    }
    
    pub fn mouse_delta(&self) -> glm::DVec2 {
        self.mouse_delta
    }
    
    pub fn is_key_pressed(&self, key: LgKeyCode) -> bool {
        profile_function!();
        *self.key_states.get(&key)
            .unwrap_or(&false)
    }
}

// Public(crate)
impl LgInput {
    pub(crate) fn init() {
        unsafe {
            INPUT.get_or_init(|| LgInput {
                key_states: HashMap::default(),
                mouse_states: HashMap::default(),
                mouse_position: glm::vec2(0.0, 0.0),
                mouse_delta: glm::vec2(0.0, 0.0),
            });
        }
    }

    pub(crate) fn on_event(event: &LgEvent) {
        profile_function!();
        let input = unsafe { INPUT.get_mut().unwrap() };

        match event {
            LgEvent::KeyEvent(ke) => { input.key_states.insert(ke.key, ke.pressed); },
            LgEvent::MouseEvent(me) => match me {
                super::event::MouseEvent::ButtonEvent(mbe) => { input.mouse_states.insert(mbe.button, mbe.pressed); },
                super::event::MouseEvent::MoveEvent(mme) => {
                    input.mouse_delta = input.mouse_position - mme.position;
                    input.mouse_position = mme.position;
                },
                _ => (),
            },
            _ => (),
        };
    }
}