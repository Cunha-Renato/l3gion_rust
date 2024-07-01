use crate::lg_core::{event::LgEvent, glm, input::LgInput, ui::{is_inside, ui_manager::Ui, Condition, UiFlags, UiPosition, UiSize}};

use super::constants;

#[derive(Default, Clone)]
pub(crate) struct Window {
    pub(crate) name: String,
    pub(crate) flags: UiFlags,
    pub(crate) position: UiPosition,
    pub(crate) size: UiSize,

    pub(crate) focused: bool, // Foreground.
    pub(crate) active: bool, // Mouse is pressed on it.
    pub(crate) hover: bool, // Mouse is on top of it.
}
impl Window {
    pub(crate) fn color(&self) -> glm::Vec4 {
        if self.focused { 
            constants::WINDOW_COLOR_FOCUSED
        } 
        else { 
            constants::WINDOW_COLOR_UNFOCUSED
        }
    }

    pub(crate) fn title_color(&self) -> glm::Vec4 {
        if self.focused { 
            constants::WINDOW_TITTLE_COLOR_FOCUSED
        } 
        else { 
            constants::WINDOW_TITTLE_COLOR_UNFOCUSED
        }
    }
    
    pub(crate) fn on_event(&mut self, event: &LgEvent) -> bool {
        if !self.flags.contains(UiFlags::SHOW) { 
            self.hover = false;
            self.active = false;
            self.focused = false; 

            return false; 
        }

        let m_position = LgInput::get().unwrap().mouse_position();
        self.hover = is_inside(
            &glm::vec2(m_position.x as f32, m_position.y as f32),
            &self.position, 
            &self.size,
        );

        match event {
            LgEvent::WindowEvent(_) => (),
            LgEvent::KeyEvent(_) => (),

            LgEvent::MouseEvent(me) => match me {
                crate::lg_core::event::MouseEvent::ButtonEvent(mbe) => {
                    if self.hover {
                        if mbe.pressed {
                            self.focused = true;
                            self.active = true;
                        } else {
                            self.active = false;
                        }
                    } else {
                        self.focused = false;
                        self.active = false;
                    }
                },
                crate::lg_core::event::MouseEvent::MoveEvent(_) => {
                    if self.active {
                        let mouse_delta = LgInput::get().unwrap().mouse_delta();
                        self.position -= glm::vec2(mouse_delta.x as f32, mouse_delta.y as f32);
                    }
                },
                _ => (),
            },
        }

        false
    }
}

pub struct WindowBuilder<'ui> {
    ui: &'ui mut Ui,
    flags: UiFlags,
    condition: Condition,

    name: String,
    position: UiPosition,
    size: UiSize,
}
// Public
impl<'ui> WindowBuilder<'ui> {
    pub fn position(mut self, position: UiPosition) -> Self {
        self.position = position;
        self
    }

    pub fn size(mut self, size: UiSize) -> Self {
        self.size = size;
        self
    }
    
    pub fn flags(mut self, flags: UiFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn insert<F: FnOnce()>(self, f: F) {
        let config = Window {
            name: self.name.clone(),
            flags: self.flags,
            position: self.position,
            size: self.size,

            focused: false,
            active: false,
            hover: false,
        };

        self.ui.insert_window(config, self.condition);

        f()
    }
}
// Public(crate)
impl<'ui> WindowBuilder<'ui> {
    pub(crate) fn new(ui: &'ui mut Ui, label: &str, condition: Condition) -> Self {
        Self {
            ui,
            flags: UiFlags::NONE,
            condition,
            name: label.to_string(),
            position: glm::vec2(0.0, 0.0),
            size: glm::vec2(0.0, 0.0),
        }
    }
}