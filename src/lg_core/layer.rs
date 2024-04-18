use winit::window::Window;
use crate::StdError;

use super::{event::LgEvent, input::LgInput, lg_types::reference::Rfc};

pub trait Layer {
    fn init(&mut self, window: Rfc<Window>) -> Result<(), StdError>;
    fn on_update(&mut self, input: &LgInput) -> Result<(), StdError>;
    fn on_event(&mut self, event: &LgEvent) -> Result<(), StdError>;
    fn destroy(&mut self) -> Result<(), StdError>;
}