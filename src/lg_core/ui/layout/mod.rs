use super::{component::{frame::UiFrame, UiComponent}, UiPosition, UiSize, UiUnit};
use nalgebra_glm as glm;

pub(crate) mod vertical;

pub(crate) trait UiLayout {
    fn arrange(parent: &mut UiFrame, components: &mut [Box<dyn UiComponent>]);
}

