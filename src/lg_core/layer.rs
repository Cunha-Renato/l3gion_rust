use crate::StdError;

use super:: event::LgEvent;

pub trait Layer {
    fn debug_name(&self) -> &str;
    fn on_attach(&mut self) -> Result<(), StdError>;
    fn on_detach(&mut self) -> Result<(), StdError>;
    fn on_update(&mut self) -> Result<(), StdError>;
    fn on_event(&mut self, event: &LgEvent) -> bool;
    fn on_imgui(&mut self, ui: &mut imgui::Ui);
}