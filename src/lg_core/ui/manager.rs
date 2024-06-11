use std::{cell::OnceCell, collections::HashMap, sync::{Arc, Mutex, OnceLock}};

use crate::{lg_core::{event::LgEvent, lg_types::reference::Rfc, window::LgWindow}, StdError};

use super::{component::{frame::UiFrame, UiComponent, UiComponentCreateInfo}, layout::{to_normalized_position, to_normalized_size, vertical::VerticalLayout}, UiUnit};

enum Layout {
    NONE,
    VERTICAL(VerticalLayout),
}

pub struct Ui {
    window: Rfc<LgWindow>,
    layout: Layout,

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
            layout: Layout::NONE,
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
            f.set_normalized_position(to_normalized_position(&window_size, &f.position()));
            f.set_normalized_size(to_normalized_size(&window_size, &f.scale()))
        }
        
        Ok(())
    }
}