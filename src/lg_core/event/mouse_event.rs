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
    pub position: (u64, u64),
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct MouseScrollEvent {
    pub delta: (f32, f32),
}