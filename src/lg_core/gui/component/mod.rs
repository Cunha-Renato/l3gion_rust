pub mod frame;

use crate::lg_core::event::LgEvent;
use super::{UiDirection, UiOffsetMode, UiPosition, UiSize, UiTotalOffset, UiUnit};

pub trait UiComponent {
    fn is_hover(&self) -> bool;
    fn is_active(&self) -> bool;
    fn on_update(&mut self);
    fn on_event(&mut self, event: &LgEvent) -> bool;

    fn position(&self) -> UiPosition;
    fn scale(&self) -> UiSize;

    fn set_position(&mut self, new_pos: UiPosition);
    fn set_scale(&mut self, new_scale: UiSize);
    
    fn set_offset(&mut self, amount: UiUnit, direction: UiDirection, mode: UiOffsetMode);
    
    fn get_offset(&self) -> &UiTotalOffset;
}
