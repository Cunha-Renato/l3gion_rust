use std::collections::HashMap;

use crate::lg_core::window::LgWindow;
use super::component::frame::UiFrame;

pub struct UiManager {
    window: LgWindow,
    
    frames: HashMap<String, UiFrame>
}
impl UiManager {
    fn new(window: LgWindow) -> Self {
        Self {
            window,
            frames: HashMap::default(),
        }
    }
}