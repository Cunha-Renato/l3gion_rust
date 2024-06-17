extern crate nalgebra_glm;
use nalgebra_glm as glm;

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum MouseEvent {
    ButtonEvent(MouseButtonEvent),
    MoveEvent(MouseMoveEvent),
    ScrollEvent(MouseScrollEvent),
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub pressed: bool,
}
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct MouseMoveEvent {
    pub position: glm::DVec2,
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct MouseScrollEvent {
    pub delta: glm::Vec2,
}