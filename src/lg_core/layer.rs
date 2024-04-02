use winit::window::Window;
use super::{event::LgEvent, input::LgInput, lg_types::reference::Ref};

pub trait Layer {
    fn init(&mut self, window: Ref<Window>);
    fn on_update(&mut self, input: &LgInput);
    fn on_event(&mut self, event: &LgEvent);
    fn destroy(&mut self);
}