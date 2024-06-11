use crate::lg_core::ui::component::frame::UiFrame;

use super::UiLayout;

pub(crate) struct VerticalLayout {
    window_size: (u32, u32),
}
impl VerticalLayout {
    pub(crate) fn new(window_size: (u32, u32)) -> Self {
        Self {
            window_size,
        }
    }
}
impl UiLayout for VerticalLayout {
    fn arrange(parent: &mut UiFrame, components: &mut [Box<dyn crate::lg_core::ui::component::UiComponent>]) {
        todo!()
    }
}
