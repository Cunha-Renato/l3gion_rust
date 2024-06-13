use std::collections::HashMap;
use crate::{as_dyn, lg_core::{event::LgEvent, lg_types::reference::Rfc, window::LgWindow}, profile_function, StdError};
use super::{component::{button::UiButton, frame::UiFrame, UiComponent, UiComponentCreateInfo, UiComponentPublic, UiManageComponent}, to_normalized_position, to_normalized_size, UiUnit};
use nalgebra_glm as glm;

// pub struct Ui {
//     window: Rfc<LgWindow>,
//
//     pub(crate) frames: HashMap<String, UiFrame>
// }
// // Public
// impl Ui {
//     pub fn add_frame(&mut self, info: UiComponentCreateInfo) -> &mut UiFrame {
//         let name = info.name.clone();
//         let mut frame = UiFrame::new(info);
//         frame.set_position((UiUnit::PIXEL(100), UiUnit::PIXEL(100)));
//
//         self.frames.entry(name).or_insert(frame)
//     }
// }
// // Public(crate)
// impl Ui {
//     pub(crate) fn new(window: Rfc<LgWindow>) -> Self {
//         Self {
//             window,
//             frames: HashMap::default(),
//         }
//     }
//     pub(crate) fn on_event(&mut self, event: &LgEvent) -> bool {
//         let mut block = false;
//
//         for (_, f) in &mut self.frames {
//             if f.on_event(event) {
//                 block = true;
//                 break;
//             }
//         }
//        
//         block
//     }
//     pub(crate) fn on_update(&mut self) {
//         for (_, f) in &mut self.frames {
//             f.on_update();
//         }
//     }
//     pub(crate) fn update(&mut self) -> Result<(), StdError> {
//         let window_size = self.window.borrow().size();
//         for (_, f) in &mut self.frames {
//             let n_pos = to_normalized_position(&window_size, &f.position());
//             let n_size = to_normalized_size(&window_size, &f.scale());
//
//             f.set_normalized_position(n_pos + glm::vec2(n_size.x * 0.5, -n_size.y * 0.5));
//             f.set_normalized_size(n_size);
//         }
//        
//         Ok(())
//     }
// }

// Similar to Dear ImGui, but worse
pub struct Ui {
    window: Rfc<LgWindow>,
    
    writing_frame: bool,

    pub(crate) frames_in_use: Vec<Rfc<UiFrame>>,
    frame_bank: HashMap<String, Rfc<UiFrame>>,
}
// Public
impl Ui {
    // Frame
    pub fn begin_frame(&mut self, info: &UiComponentCreateInfo) {
        profile_function!();

        let window_size = self.window.borrow().size();
        let frame = self.frame_bank.entry(info.name.clone()).or_insert_with(|| {
            let frame = Rfc::new(UiFrame::new(info));
            update_frames_position(window_size, &frame);
            
            frame
        }).clone();
        
        self.frames_in_use.push(frame);
        
        self.writing_frame = true;
    }

    pub fn end_frame(&mut self) {
        profile_function!();
        self.writing_frame = false;
    }
    
    // Button
    pub fn button(&mut self, info: &UiComponentCreateInfo) -> Rfc<dyn UiComponent> {
        assert!(self.writing_frame && !self.frames_in_use.is_empty());
        
        let frame = self.frames_in_use.last().unwrap();
        let button = as_dyn!(UiButton::new(info), dyn UiComponent);

        frame.borrow_mut().add(info.name.clone(), button)
    }
    
    pub(crate) fn on_update(&mut self) {
        let window_size = self.window.borrow().size();

        for f in &self.frames_in_use {
            if f.borrow().is_active() {
                update_frames_position(window_size, f);
            }
            f.borrow_mut().on_update();
        }
        
        self.frames_in_use.clear();
    }

    pub(crate) fn on_resize(&self) {
        let window_size = self.window.borrow().size();
        for f in &self.frames_in_use {
            update_frames_position(window_size, f);
        }
    }

    pub(crate) fn on_event(&mut self, event: &LgEvent) -> bool {
        let mut block = false;

        for f in &self.frames_in_use {
            if f.borrow_mut().on_event(event) {
                block = true;
                break;
            }
        }
        
        block
    }
}

// Public(crate)
impl Ui {
    pub(crate) fn new(window: Rfc<LgWindow>) -> Self {
        Self {
            window,
            writing_frame: false,
            frames_in_use: Vec::new(),
            frame_bank: HashMap::new(),
        }
    }
}

fn update_frames_position(window_size: (u32, u32), f: &Rfc<UiFrame>) {
    let n_pos = to_normalized_position(&window_size, &f.borrow().position());
    let n_size = to_normalized_size(&window_size, &f.borrow().scale());

    f.borrow_mut().set_normalized_position(n_pos + glm::vec2(n_size.x * 0.5, -n_size.y * 0.5));
    f.borrow_mut().set_normalized_size(n_size);
}