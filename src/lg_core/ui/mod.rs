use layout::vertical::VerticalLayout;
use nalgebra_glm as glm;

pub mod ui_manager;
pub mod component;
pub mod layout;

#[derive(Debug, Clone, Copy)]
pub enum UiUnit {
    PIXEL(i32),
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

pub type UiPosition = (UiUnit, UiUnit);
pub type UiSize = (UiUnit, UiUnit);

enum UI_LAYOUT {
    VERTICAL(VerticalLayout),
}

pub(crate) fn is_inside(m_point: (u32, u32), position: &UiPosition, size: &UiSize) -> bool {
    match position {
        (UiUnit::PIXEL(x), UiUnit::PIXEL(y)) => {
            match size {
                (UiUnit::PIXEL(s_width), UiUnit::PIXEL(s_height)) => {
                    let pos = (*x as f32, *y as f32);
                    let size = (*s_width as f32, *s_height as f32);
                    let point = (m_point.0 as f32, m_point.1 as f32);
                    
                    // Good for now
                    point.0 >= pos.0 && point.0 <= pos.0 + size.0 &&
                    point.1 >= pos.1 && point.1 <= pos.1 + size.1
                }
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}

pub(crate) fn to_normalized_position(screen_space: &(u32, u32), position: &UiPosition) -> glm::Vec2 {
    match position {
        (UiUnit::PIXEL(x), UiUnit::PIXEL(y)) => {
            let width = *x as f32 / screen_space.0 as f32;
            let height = *y as f32 / screen_space.1 as f32;

            glm::vec2(width * 2.0 - 1.0, height * -2.0 + 1.0)                        
        }
        _ => todo!(),
    }
}
pub(crate) fn to_normalized_size(screen_space: &(u32, u32), size: &UiPosition) -> glm::Vec2 {
    match size {
        (UiUnit::PIXEL(width), UiUnit::PIXEL(height)) => {
            // Mesh Dimensions
            let pixel_quad_width = screen_space.0 as f32 * 0.5;
            let pixel_quad_height = screen_space.1 as f32 * 0.5;

            let width = *width as f32 / pixel_quad_width;
            let height = *height as f32 / pixel_quad_height;

            glm::vec2(width, height)
        }
        _ => todo!(),
    }
}