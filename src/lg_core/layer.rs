use crate::StdError;

use super::{application::ApplicationCore, event::LgEvent};

pub trait Layer {
    fn on_attach(&mut self, app_core: ApplicationCore) -> Result<(), StdError>;
    fn on_detach(&mut self) -> Result<(), StdError>;
    fn on_update(&mut self) -> Result<(), StdError>;
    fn on_event(&mut self, event: &LgEvent) -> bool;
}