use std::collections::HashMap;
use crate::{lg_core::{event::LgEvent, lg_types::reference::Rfc, window::LgWindow}, StdError};
use super::{component::{frame::UiFrame, UiComponentCreateInfo, UiComponentPublic, UiManageComponent}, to_normalized_position, to_normalized_size, UiUnit};
use nalgebra_glm as glm;

pub struct Ui {
    window: Rfc<LgWindow>,

    pub(crate) frames: HashMap<String, UiFrame>
}
// Public
impl Ui {
    pub fn add_frame(&mut self, info: UiComponentCreateInfo) -> &mut UiFrame {
        let name = info.name.clone();
        let mut frame = UiFrame::new(info);
        frame.set_position((UiUnit::PIXEL(100), UiUnit::PIXEL(100)));

        self.frames.entry(name).or_insert(frame)
    }
}
// Public(crate)
impl Ui {
    pub(crate) fn new(window: Rfc<LgWindow>) -> Self {
        Self {
            window,
            frames: HashMap::default(),
        }
    }
    pub(crate) fn on_event(&mut self, event: &LgEvent) -> bool {
        let mut block = false;

        for (_, f) in &mut self.frames {
            if f.on_event(event) {
                block = true;
                break;
            }
        }
        
        block
    }
    pub(crate) fn on_update(&mut self) {
        for (_, f) in &mut self.frames {
            f.on_update();
        }
    }
    pub(crate) fn update(&mut self) -> Result<(), StdError> {
        let window_size = self.window.borrow().size();
        for (_, f) in &mut self.frames {
            let n_pos = to_normalized_position(&window_size, &f.position());
            let n_size = to_normalized_size(&window_size, &f.scale());

            f.set_normalized_position(n_pos + glm::vec2(n_size.x * 0.5, -n_size.y * 0.5));
            f.set_normalized_size(n_size);
        }
        
        Ok(())
    }
}