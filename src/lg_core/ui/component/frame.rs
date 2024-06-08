use std::collections::HashMap;

use crate::lg_core::{event::LgEvent, ui::{UiDirection, UiOffsetMode, UiTotalOffset, UiUnit}};

use super::{UiComponent, UiPosition, UiSize};

pub struct UiFrame {
    offset: UiTotalOffset,
    position: UiPosition,
    scale: UiSize,
    
    is_hover: bool,
    is_active: bool,

    children: HashMap<String, Box<dyn UiComponent>>,
}
impl UiComponent for UiFrame {
    fn is_hover(&self) -> bool {
        self.is_hover
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn on_update(&mut self) {
        for (_, c) in &mut self.children {
            c.on_update();
        }
    }

    fn on_event(&mut self, event: &LgEvent) -> bool {
        for (_, c) in &mut self.children {
            c.on_event(event);
        }
        
        false
    }

    fn position(&self) -> UiPosition {
        self.position
    }

    fn scale(&self) -> UiSize {
        self.scale
    }

    fn set_position(&mut self, new_pos: UiPosition) {
        self.position = new_pos;
    }

    fn set_scale(&mut self, new_scale: UiSize) {
        self.scale = new_scale;
    }

    fn set_offset(&mut self, amount: UiUnit, direction: UiDirection, mode: UiOffsetMode) {
        match mode {
            UiOffsetMode::PADDING => match direction {
                UiDirection::ALL => self.offset.padding.set_all(amount),
                UiDirection::TOP => self.offset.padding.top = amount,
                UiDirection::BOTTOM => self.offset.padding.bottom = amount,
                UiDirection::LEFT => self.offset.padding.left = amount,
                UiDirection::RIGHT => self.offset.padding.right = amount,
            },
            UiOffsetMode::MARGIN => match direction {
                UiDirection::ALL => self.offset.margin.set_all(amount),
                UiDirection::TOP => self.offset.margin.top = amount,
                UiDirection::BOTTOM => self.offset.margin.bottom = amount,
                UiDirection::LEFT => self.offset.margin.left = amount,
                UiDirection::RIGHT => self.offset.margin.right = amount,
            },
        }
    }

    fn get_offset(&self) -> &UiTotalOffset {
        &self.offset
    }
}
impl Default for UiFrame {
    fn default() -> Self {
        Self {
            offset: UiTotalOffset::default(),
            position: (UiUnit::PIXEL(0), UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            scale: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            is_hover: false,
            is_active: false,
            children: HashMap::default(),
        }
    }
}