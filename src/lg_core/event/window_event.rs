#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum WindowEvent {
    Resize(u32, u32),
    Focused(bool),
    Close,
}