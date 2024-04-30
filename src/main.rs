use std::env;

use l3gion_rust::lg_core::{
        application::Application, event::{
            KeyEvent,
            MouseButton,
            MouseButtonEvent,
            MouseEvent,
            MouseMoveEvent,
            MouseScrollEvent,
        }, input::LgInput
    };
use l3gion_rust::StdError;

use winit::{event::{ElementState, MouseScrollDelta}, event_loop};
use winit::{
    dpi::PhysicalSize,
    event::{
        Event,
        WindowEvent
    },
    window::WindowBuilder,
};

fn main() -> Result<(), StdError> {
    env::set_var("LOG", "4");
    env::set_var("OPEN_GL", "1");
    env::remove_var("VULKAN");

    let event_loop = winit::event_loop::EventLoop::new()?;
    
    let mut app = if env::var("OPEN_GL").is_ok() {
        Application::new_opengl(&event_loop)?
    } else if env::var("VULKAN").is_ok() {
        Application::new_vulkan()
    } else {
        return Err("A graphics API must be specified! (Set env var as OPEN_GL = 1, or VULKAN = 1)".into());
    };

    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    app.destroy();
                    window_target.exit()
                },
                winit::event::WindowEvent::Resized(window_size) => {
                    if window_size.width > 0 && window_size.height > 0 {
                        app.resize(window_size.into()).unwrap()
                    }
                },
                winit::event::WindowEvent::RedrawRequested => {
                    app.on_update().unwrap();
                },
                _ => ()
            },
            winit::event::Event::AboutToWait => app.request_redraw(),
            _ => (),
        }
    });
    
    Ok(())
}

