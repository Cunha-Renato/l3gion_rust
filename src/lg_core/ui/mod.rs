extern crate bitflags;
extern crate nalgebra_glm;

use bitflags::bitflags;
use nalgebra_glm as glm;

pub mod ui_manager;
pub mod component;

pub type UiPosition = (i32, i32);
pub type UiSize = (u32, u32);

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
    }
}

pub(crate) fn is_inside(point: (u32, u32), position: &UiPosition, size: &UiSize) -> bool {
    let point = (point.0 as i32, point.1 as i32);
    let size = (size.0 as i32, size.1 as i32);

    // Good for now
    point.0 >= position.0 && point.0 <= position.0 + size.0 &&
    point.1 >= position.1 && point.1 <= position.1 + size.1
}

pub(crate) fn to_normalized_position(screen_space: &(u32, u32), position: &UiPosition) -> glm::Vec2 {
    let width = position.0 as f32 / screen_space.0 as f32;
    let height = position.1 as f32 / screen_space.1 as f32;

    glm::vec2(width * 2.0 - 1.0, height * -2.0 + 1.0)                        
}
pub(crate) fn to_normalized_size(screen_space: &(u32, u32), size: &UiPosition) -> glm::Vec2 {
        // Mesh Dimensions
        let pixel_quad_width = screen_space.0 as f32 * 0.5;
        let pixel_quad_height = screen_space.1 as f32 * 0.5;

        let width = size.0 as f32 / pixel_quad_width;
        let height = size.1 as f32 / pixel_quad_height;

        glm::vec2(width, height)
}