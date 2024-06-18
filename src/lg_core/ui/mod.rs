extern crate bitflags;
extern crate nalgebra_glm;

use bitflags::bitflags;
use nalgebra_glm as glm;

pub mod ui_manager;
pub mod component;

pub type UiPosition = glm::Vec2;
pub type UiSize = glm::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    ALWAYS,
    FIRST_TIME,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct UiFlags: u32 {
        const NONE = 0;
        const ON_HOVER = 1;
        const ON_ACTIVE = 2;
        const ON_KEYBOARD = 3;

        const SHOW = 4;
    }
}

pub(crate) fn is_inside(point: &glm::Vec2, position: &UiPosition, size: &UiSize) -> bool {
    let point = (point.x as f32, point.y as f32);

    // Good for now
    point.0 >= position.x && point.0 <= position.x + size.x &&
    point.1 >= position.y && point.1 <= position.y + size.y
}

pub(crate) fn to_normalized_position(screen_space: &glm::Vec2, position: &UiPosition, size: &UiSize) -> glm::Vec2 {
    let pos = position.component_div(screen_space);
    let size = size.component_div(screen_space);
    let dimensions = pos + (size / 2.0);

    glm::vec2(dimensions.x * 2.0 - 1.0, dimensions.y * -2.0 + 1.0)
}
pub(crate) fn to_normalized_size(screen_space: &glm::Vec2, size: &UiPosition) -> glm::Vec2 {
        // Mesh Dimensions
        let screen_space = screen_space * 0.5;
        size.component_div(&screen_space)
}