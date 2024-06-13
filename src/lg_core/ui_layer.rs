use crate::profile_function;

use super::{application::ApplicationCore, layer::Layer, lg_types::reference::Rfc, ui::component::UiComponent};
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
        let core = self.core();
        
        let mut instance_data = core.renderer.borrow_mut().begin_instancing::<UiInstanceVertex>();

        for f in &core.ui.borrow_mut().frames_in_use {
            core.renderer.borrow_mut().queue_instance(&f.borrow().entity, &mut instance_data, |e| {
                let model = e.model();
                let row_0 = glm::vec4(model[(0, 0)], model[(0, 1)], model[(0, 2)], model[(0, 3)]);
                let row_1 = glm::vec4(model[(1, 0)], model[(1, 1)], model[(1, 2)], model[(1, 3)]);
                let row_2 = glm::vec4(model[(2, 0)], model[(2, 1)], model[(2, 2)], model[(2, 3)]);
                let color = glm::vec4(1.0, 1.0, 1.0, 1.0);
                
                UiInstanceVertex {
                    color,
                    row_0,
                    row_1,
                    row_2,
                }
            })?;
        }
        
        core.renderer.borrow_mut().end_instancing(&mut instance_data)?;
        core.ui.borrow_mut().on_update();

        Ok(())
    }

    fn on_event(&mut self, event: &super::event::LgEvent) -> bool {
        self.core().ui.borrow_mut().on_event(event)
    }
}