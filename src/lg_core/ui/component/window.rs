use crate::lg_core::ui::{ui_manager::Ui, Condition, UiFlags, UiPosition, UiSize};

#[derive(Debug, Default)]
pub(crate) struct WindowConfig {
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

    pub fn insert<F: FnOnce()>(mut self, f: Option<F>) {
        let config = self.ui.windows_config.entry(self.name.clone())
            .or_insert(WindowConfig {
                name: self.name,
                flags: self.flags,
                position: self.position,
                size: self.size,

                focused: true,
                active: false,
                hover: false,
            });

        if self.condition == Condition::ALWAYS {
            config.flags = self.flags;
            config.position = self.position;
            config.size = self.size;
        }

        self.ui.set_current_window(self.name);

        if let Some(f) = f { f(); }
    }
}
// Public(crate)
impl<'ui> Window<'ui> {
    pub(crate) fn new(ui: &mut Ui, label: &str, condition: Condition) -> Self {
        Self {
            ui,
            flags: UiFlags::NONE,
            condition,
            name: label.to_string(),
            position: (0, 0),
            size: (0, 0),
        }
    }
}