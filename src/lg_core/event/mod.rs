pub mod keyboard_event;
pub mod mouse_event;
pub mod window_event;
pub use keyboard_event::*;
pub use mouse_event::*;
pub use window_event::*;

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum LgEvent {
    WindowEvent(WindowEvent),
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
}