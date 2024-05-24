use crate::StdError;

use super::{application::ApplicationCore, event::LgEvent, lg_types::reference::Rfc};

pub trait Layer {
    fn init(&mut self, app_core: Rfc<ApplicationCore>) -> Result<(), StdError>;
    fn on_update(&mut self);
    fn on_event(&mut self, event: LgEvent) -> bool;
    fn shutdown(&mut self) -> Result<(), StdError>;
}