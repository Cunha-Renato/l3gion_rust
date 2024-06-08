use super::component::UiComponent;

pub(crate) trait UiLayout {
    fn arrange(components: &[Box<dyn UiComponent>]);
}