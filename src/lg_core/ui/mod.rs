pub mod manager;
pub mod component;
pub mod layout;

#[derive(Debug, Clone, Copy)]
pub enum UiUnit {
    PIXEL(u32),
    PERCENTAGE(f32),
}

#[derive(Debug, Clone, Copy)]
pub enum UiDirection {
    ALL,
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,
}
impl Default for UiDirection {
    fn default() -> Self {
        Self::ALL
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UiOffsetMode {
    PADDING,
    MARGIN
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct UiOffset {
    top: UiUnit,
    bottom: UiUnit,
    left: UiUnit,
    right: UiUnit,
}
impl UiOffset {
    fn set_all(&mut self, unit: UiUnit) {
        self.top = unit;
        self.bottom = unit;
        self.left = unit;
        self.right = unit;
    }
}
impl Default for UiOffset {
    fn default() -> Self {
        let zero = UiUnit::PIXEL(0);
        Self {
            top: zero,
            bottom: zero,
            left: zero,
            right: zero,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UiTotalOffset {
    pub(crate) padding: UiOffset,
    pub(crate) margin: UiOffset,
}

pub type UiPosition = (UiUnit, UiUnit, UiUnit);
pub type UiSize = (UiUnit, UiUnit);