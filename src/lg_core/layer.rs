use winit::window::Window;
use crate::MyError;

use super::{event::LgEvent, input::LgInput, lg_types::reference::Rfc};

pub trait Layer {
    fn init(&mut self, window: Rfc<Window>) -> Result<(), MyError>;
    fn on_update(&mut self, input: &LgInput) -> Result<(), MyError>;
    fn on_event(&mut self, event: &LgEvent) -> Result<(), MyError>;
    fn destroy(&mut self) -> Result<(), MyError>;
}