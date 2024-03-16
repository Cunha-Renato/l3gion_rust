use std::sync::Arc;

use winit::{
    event_loop::EventLoop, window::WindowBuilder
};

use crate::MyError;

pub fn get_event_loop() -> winit::event_loop::EventLoop<()> {
    EventLoop::new()
}
pub fn get_window(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Arc<winit::window::Window>, MyError> {
    Ok(Arc::new(WindowBuilder::new().build(event_loop)?))
}