pub mod frame;
pub mod button;

use crate::lg_core::{event::LgEvent, uuid::UUID};
use super::{UiDirection, UiOffsetMode, UiPosition, UiSize, UiTotalOffset, UiUnit};
use nalgebra_glm as glm;

pub(crate) const UI_MESH: UUID = UUID::from_u128(316691656959075038046595414025328715723);
pub(crate) const UI_MATERIAL: UUID = UUID::from_u128(4);

pub struct UiComponentCreateInfo {
    pub name: String,
    pub offset: UiTotalOffset,
    pub scale: UiSize,
}

pub trait UiComponent: UiComponentPublic + UiManageComponent {}

pub trait UiComponentPublic {
    fn is_hover(&self) -> bool;
    fn is_active(&self) -> bool;

    fn position(&self) -> UiPosition;

    fn scale(&self) -> UiSize;
    fn set_scale(&mut self, new_scale: UiSize);
    
    fn set_offset(&mut self, amount: UiUnit, direction: UiDirection, mode: UiOffsetMode);
    fn get_offset(&self) -> &UiTotalOffset;
}
pub(crate) trait UiManageComponent {
    fn on_update(&mut self);
    fn on_event(&mut self, event: &LgEvent) -> bool;
    
    fn set_normalized_position(&mut self, new_pos: glm::Vec2);
    fn set_normalized_size(&mut self, new_size: glm::Vec2);

    fn set_local_position(&mut self, new_pos: UiPosition);
    fn set_ss_position(&mut self, new_pos: UiPosition);
    
    fn set_ss_scale(&mut self, new_scale: UiSize);
}
