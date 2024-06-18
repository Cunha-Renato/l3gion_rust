use crate::profile_function;

use super::{application::ApplicationCore, layer::Layer, lg_types::reference::Rfc};
use lg_renderer::lg_vertex;
use nalgebra_glm as glm;

#[derive(Debug, Clone)]
struct UiInstanceVertex {
    color: glm::Vec4,
    row_0: glm::Vec4,
    row_1: glm::Vec4,
    row_2: glm::Vec4,
}
lg_vertex!(UiInstanceVertex, color, row_0, row_1, row_2);

#[derive(Default)]
pub(crate) struct UiLayer {
    _debug_name: String,

    core: Option<ApplicationCore>,
}
// Public(crate)
impl UiLayer {
    pub(crate) fn new() -> Self {
        Self {
            _debug_name: "UiLayer".to_string(),
            core: None,
        }
    }
}
// Private
impl UiLayer {
    fn core(&self) -> &ApplicationCore {
        self.core.as_ref().unwrap()
    }
}

impl Layer for UiLayer {
    fn debug_name(&self) -> &str {
        &self._debug_name
    }

    fn on_attach(&mut self, app_core: ApplicationCore) -> Result<(), crate::StdError> {
        self.core = Some(app_core);
        
        Ok(())
    }

    fn on_detach(&mut self) -> Result<(), crate::StdError> {
        
        Ok(())
    }

    fn on_update(&mut self) -> Result<(), crate::StdError> {
        profile_function!();
        self.core().ui.borrow_mut().on_update(&mut self.core().renderer.borrow_mut())?;
        Ok(())
    }

    fn on_event(&mut self, event: &super::event::LgEvent) -> bool {
        self.core().ui.borrow_mut().on_event(event)
    }
}