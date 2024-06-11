use super::{component::{frame::UiFrame, UiComponent}, UiPosition, UiSize, UiUnit};
use nalgebra_glm as glm;

pub(crate) mod vertical;

pub(crate) trait UiLayout {
    fn arrange(parent: &mut UiFrame, components: &mut [Box<dyn UiComponent>]);
}

pub(crate) fn is_inside(point: (u32, u32), position: &UiPosition, size: &UiSize) -> bool {
    match position {
        (UiUnit::PIXEL(x), UiUnit::PIXEL(y)) => {
            match size {
                (UiUnit::PIXEL(s_width), UiUnit::PIXEL(s_height)) => {
                    let mut pos = (*x as i32, *y as i32);
                    let size = (*s_width as i32, *s_height as i32);
                    let point = (point.0 as i32, point.1 as i32);

                    pos.0 -= size.0 / 2;
                    pos.1 -= size.0 / 2;
                    
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
        (UiUnit::PIXEL(width), UiUnit::PIXEL(height)) => {
            let width = *width as f32 / screen_space.0 as f32;
            let height = *height as f32 / screen_space.1 as f32;

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