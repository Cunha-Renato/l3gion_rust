use std::collections::HashMap;
use nalgebra_glm as glm;

use crate::lg_core::{entity::LgEntity, event::LgEvent, input::LgInput, ui::{layout::{is_inside, to_normalized_position}, UiDirection, UiOffsetMode, UiTotalOffset, UiUnit}};

use super::{UiComponent, UiComponentCreateInfo, UiPosition, UiSize, UI_MATERIAL, UI_MESH};

pub struct UiFrame {
    pub(crate) entity: LgEntity,

    offset: UiTotalOffset,
    position: UiPosition,
    scale: UiSize,
    
    is_hover: bool,
    is_active: bool,

    children: HashMap<String, Box<dyn UiComponent>>,
    
    mouse_position: (u64, u64),
    move_frame: bool,
}
impl UiFrame {
    pub(crate) fn new(info: UiComponentCreateInfo) -> Self {
        Self {
            entity: LgEntity::new(
                UI_MESH.clone(), 
                UI_MATERIAL.clone(), 
                glm::vec3(0.0, 0.0, 0.0)
            ),
            offset: info.offset,
            position: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            scale: info.scale,
            is_hover: false,
            is_active: false,
            children: HashMap::default(),
            mouse_position: (0, 0),
            move_frame: false,
        }
    }
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
        match event {
            LgEvent::MouseEvent(me) => match me {
                crate::lg_core::event::MouseEvent::ButtonEvent(mbe) => {
                    let mp = LgInput::get_locked().unwrap().get_mouse_position();

                    self.move_frame = mbe.pressed && is_inside(
                        (mp.x as u32, mp.y as u32),
                        &self.position, 
                        &self.scale
                    );
                },
                crate::lg_core::event::MouseEvent::MoveEvent(mme) => {
                    if self.move_frame {
                        let delta = (
                            self.mouse_position.0 as i32 - mme.position.0 as i32,
                            self.mouse_position.1 as i32 - mme.position.1 as i32
                        );

                        match &mut self.position {
                            (UiUnit::PIXEL(x), UiUnit::PIXEL(y)) => {
                                *x = (*x as i32 - delta.0) as u32;
                                *y = (*y as i32 - delta.1) as u32;
                            },
                            _ => todo!(),
                        };
                    }
                    self.mouse_position = mme.position;
                },
                _ => (),
            },
            _ => (),
        }

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
    
    fn set_normalized_position(&mut self, new_pos: nalgebra_glm::Vec2) {
        self.entity.set_position(glm::vec3(new_pos.x, new_pos.y, 0.0));
    }
    
    fn set_normalized_size(&mut self, new_size: nalgebra_glm::Vec2) {
        self.entity.set_scale(glm::vec3(new_size.x, new_size.y, 1.0));
    }
}
impl Default for UiFrame {
    fn default() -> Self {
        Self {
            entity: LgEntity::new(
                UI_MESH.clone(), 
                UI_MATERIAL.clone(), 
                glm::vec3(0.0, 0.0, 0.0)
            ),
            offset: UiTotalOffset::default(),
            position: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            scale: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            is_hover: false,
            is_active: false,
            children: HashMap::default(),
            mouse_position: (0, 0),
            move_frame: false,
        }
    }
}