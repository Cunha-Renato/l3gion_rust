use crate::lg_core::{entity::LgEntity, glm, ui::{ui_manager::Ui, Condition, UiFlags, UiPosition, UiSize}, uuid::UUID};

use super::constants::{WINDOW_MATERIAL, WINDOW_MESH};

#[derive(Default, Clone)]
pub(crate) struct WindowConfig {
    pub(crate) _window_entity: LgEntity,
    pub(crate) _title_bar_entity: LgEntity,

    pub(crate) name: String,
    pub(crate) flags: UiFlags,
    pub(crate) position: UiPosition,
    pub(crate) size: UiSize,

    pub(crate) focused: bool, // Foreground.
    pub(crate) active: bool, // Mouse is pressed on it.
    pub(crate) hover: bool, // Mouse is on top of it.
}

pub struct Window<'ui> {
    ui: &'ui mut Ui,
    flags: UiFlags,
    condition: Condition,

    name: String,
    position: UiPosition,
    size: UiSize,
}
// Public
impl<'ui> Window<'ui> {
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
        let config = WindowConfig {
            _window_entity: LgEntity::new(
                WINDOW_MESH.clone(),
                WINDOW_MATERIAL.clone(),
                glm::vec3(0.0, 0.0, 0.0)
            ),
            _title_bar_entity: LgEntity::new(
                WINDOW_MESH.clone(),
                WINDOW_MATERIAL.clone(),
                glm::vec3(0.0, 0.0, 0.0)
            ),
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
impl<'ui> Window<'ui> {
    pub(crate) fn new(ui: &'ui mut Ui, label: &str, condition: Condition) -> Self {
        let pos = ui.window.borrow().size() / 2.0;
        Self {
            ui,
            flags: UiFlags::NONE,
            condition,
            name: label.to_string(),
            position: pos,
            size: glm::vec2(0.0, 0.0),
        }
    }
}