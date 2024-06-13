use std::collections::HashMap;
use nalgebra_glm as glm;

use crate::lg_core::{entity::LgEntity, event::LgEvent, input::LgInput, lg_types::reference::Rfc, ui::{is_inside, layout::vertical::VerticalLayout, UiDirection, UiOffsetMode, UiTotalOffset, UiUnit, UI_LAYOUT}};

use super::{UiComponent, UiComponentCreateInfo, UiComponentPublic, UiManageComponent, UiPosition, UiSize, UI_MATERIAL, UI_MESH};

pub struct UiFrame {
    pub(crate) entity: LgEntity,

    // Visual
    layout: UI_LAYOUT,
    offset: UiTotalOffset,
    position: UiPosition,
    scale: UiSize,
    
    // Interaction
    is_hover: bool,
    is_active: bool,

    mouse_position: (u64, u64),
    move_frame: bool,
    resize_frame: bool,

    children: HashMap<String, Rfc<dyn UiComponent>>,
}
// Public(crate)
impl UiFrame {
    pub(crate) fn new(info: &UiComponentCreateInfo) -> Self {
        let mut result = Self::default();
        result.offset = info.offset;
        result.scale = info.scale;

        result
    }
    pub(crate) fn add(&mut self, name: String, component: Rfc<dyn UiComponent>) -> Rfc<dyn UiComponent>{
        self.children.entry(name.to_string()).or_insert(component).clone()
    }
}
impl UiComponentPublic for UiFrame {
    fn is_hover(&self) -> bool {
        self.is_hover
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
    
    fn position(&self) -> UiPosition {
        self.position
    }

    fn scale(&self) -> UiSize {
        self.scale
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
impl UiManageComponent for UiFrame {
    fn on_update(&mut self) {
        for (_, c) in &self.children {
            c.borrow_mut().on_update();
        }
    }

    fn on_event(&mut self, event: &LgEvent) -> bool {
        let mut block = false;
        match event {
            LgEvent::MouseEvent(me) => match me {
                crate::lg_core::event::MouseEvent::ButtonEvent(mbe) => {
                    let mp = LgInput::get_locked().unwrap().get_mouse_position();

                    // Resize
                    match self.position {
                        (UiUnit::PIXEL(x), UiUnit::PIXEL(y)) => {
                            match self.scale {
                                (UiUnit::PIXEL(width), UiUnit::PIXEL(height)) => {

                                    let resize_position = (
                                        x + width - 5,                                        
                                        y + height - 5,
                                    );
                                    
                                    self.resize_frame = mbe.pressed && is_inside(
                                        (mp.x as u32, mp.y as u32), 
                                        &(UiUnit::PIXEL(resize_position.0), UiUnit::PIXEL(resize_position.1)),
                                        &(UiUnit::PIXEL(10), UiUnit::PIXEL(10))
                                    );
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }

                    // Move
                    self.move_frame = mbe.pressed && !self.resize_frame && self.is_hover;
                },
                crate::lg_core::event::MouseEvent::MoveEvent(mme) => {
                    self.is_hover = is_inside(
                        (mme.position.0 as u32, mme.position.1 as u32), 
                        &self.position, 
                        &self.scale
                    );

                    let delta = (
                        self.mouse_position.0 as i32 - mme.position.0 as i32,
                        self.mouse_position.1 as i32 - mme.position.1 as i32
                    );

                    // Move
                    if self.move_frame {
                        match &mut self.position {
                            (UiUnit::PIXEL(x), UiUnit::PIXEL(y)) => {
                                *x = *x as i32 - delta.0;
                                *y = *y as i32 - delta.1;
                            },
                            _ => todo!(),
                        };
                    }
                    self.mouse_position = mme.position;
                    
                    // Resize
                    if self.resize_frame {
                        match &mut self.scale {
                            (UiUnit::PIXEL(width), UiUnit::PIXEL(height)) => {
                                *width = *width as i32 - delta.0;                                
                                *height = *height as i32 - delta.1;
                            },
                            _ => (),
                        }
                    }
                    
                    self.is_active = self.resize_frame || self.move_frame;
                    block = self.is_active;
                },
                _ => (),
            },
            _ => (),
        }

        for (_, c) in &self.children {
            c.borrow_mut().on_event(event);
        }
        
        block
    }

    fn set_normalized_position(&mut self, new_pos: nalgebra_glm::Vec2) {
        self.entity.set_position(glm::vec3(new_pos.x, new_pos.y, 0.0));
    }
    
    fn set_normalized_size(&mut self, new_size: nalgebra_glm::Vec2) {
        self.entity.set_scale(glm::vec3(new_size.x, new_size.y, 1.0));
    }

    fn set_local_position(&mut self, new_pos: UiPosition) {
        self.position = new_pos;
    }
    
    fn set_ss_position(&mut self, new_pos: UiPosition) {
        self.position = new_pos;
    }
    
    fn set_ss_scale(&mut self, new_scale: UiSize) {
        self.scale = new_scale;
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
            layout: UI_LAYOUT::VERTICAL(VerticalLayout::new()),
            offset: UiTotalOffset::default(),
            position: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            scale: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            is_hover: false,
            is_active: false,
            children: HashMap::default(),
            mouse_position: (0, 0),
            move_frame: false,
            resize_frame: false,
        }
    }
}