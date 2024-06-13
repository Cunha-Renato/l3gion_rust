use crate::lg_core::{entity::LgEntity, ui::{is_inside, UiPosition, UiSize, UiTotalOffset, UiUnit, UI_LAYOUT}};
use super::{UiComponent, UiComponentCreateInfo, UiComponentPublic, UiManageComponent, UI_MATERIAL, UI_MESH};
use nalgebra_glm as glm;

pub struct UiButton {
    pub(crate) entity: LgEntity,

    // Visual
    offset: UiTotalOffset,
    local_position: UiPosition,
    ss_position: UiPosition, // Always in Pixels
    local_scale: UiSize,
    ss_scale: UiSize, // Always in Pixels
    
    // Interaction
    is_hover: bool,
    is_active: bool,
}
// Public(crate)
impl UiButton {
    pub(crate) fn new(info: &UiComponentCreateInfo) -> Self {
        let mut result = Self::default();
        result.offset = info.offset;
        result.local_scale = info.scale;
        
        result
    }
}
impl UiComponent for UiButton {}
impl UiComponentPublic for UiButton {
    fn is_hover(&self) -> bool {
        self.is_hover
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn position(&self) -> UiPosition {
        self.local_position
    }

    fn scale(&self) -> UiSize {
        self.local_scale
    }

    fn set_scale(&mut self, new_scale: UiSize) {
        self.local_scale = new_scale
    }

    fn set_offset(&mut self, amount: UiUnit, direction: crate::lg_core::ui::UiDirection, mode: crate::lg_core::ui::UiOffsetMode) {
        todo!()
    }

    fn get_offset(&self) -> &UiTotalOffset {
        &self.offset
    }
}
impl UiManageComponent for UiButton {
    fn on_update(&mut self) {
        
        self.is_active = false; // Testing
    }

    fn on_event(&mut self, event: &crate::lg_core::event::LgEvent) -> bool {
        match event {
            crate::lg_core::event::LgEvent::MouseEvent(me) => {
                match me {
                    crate::lg_core::event::MouseEvent::ButtonEvent(mbe) => self.is_active = self.is_hover && mbe.pressed,
                    crate::lg_core::event::MouseEvent::MoveEvent(mme) => {
                        self.is_hover = is_inside(
                            (mme.position.0 as u32, mme.position.0 as u32), 
                            &self.ss_position, 
                            &self.ss_scale,
                        );
                    },
                    _ => (),
                }
            },
            _ => (),
        };
        
        self.is_active
    }

    fn set_normalized_position(&mut self, new_pos: nalgebra_glm::Vec2) {
        self.entity.set_position(glm::vec3(new_pos.x, new_pos.y, 0.0));
    }

    fn set_normalized_size(&mut self, new_size: nalgebra_glm::Vec2) {
        self.entity.set_scale(glm::vec3(new_size.x, new_size.y, 1.0));
    }

    fn set_local_position(&mut self, new_pos: UiPosition) {
        self.local_position = new_pos;
    }

    fn set_ss_position(&mut self, new_pos: UiPosition) {
        self.ss_position = new_pos;
    }
    
    fn set_ss_scale(&mut self, new_scale: UiSize) {
        self.ss_scale = new_scale;
    }
}
impl Default for UiButton {
    fn default() -> Self {
        Self {
            entity: LgEntity::new(
                UI_MESH.clone(), 
                UI_MATERIAL.clone(), 
                glm::vec3(0.0, 0.0, 0.0)
            ),
            offset: UiTotalOffset::default(),
            local_position: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            ss_position: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            local_scale: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            ss_scale: (UiUnit::PIXEL(0), UiUnit::PIXEL(0)),
            
            is_hover: false,
            is_active: false,
        }
    }
}