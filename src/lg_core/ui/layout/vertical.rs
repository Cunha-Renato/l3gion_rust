use crate::lg_core::ui::component::frame::UiFrame;

use super::UiLayout;

pub(crate) struct VerticalLayout {
}
impl VerticalLayout {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
impl UiLayout for VerticalLayout {
    fn arrange(parent: &mut UiFrame, components: &mut [Box<dyn crate::lg_core::ui::component::UiComponent>]) {
        todo!()
    }
}
