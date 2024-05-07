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

use winit::{event::MouseScrollDelta, event_loop};
use winit::{
    dpi::PhysicalSize,
    event::{
        Event,
        WindowEvent
    },
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
                winit::event::WindowEvent::KeyboardInput { event, .. } => {
                    match event.physical_key {
                        winit::keyboard::PhysicalKey::Code(key_code) => {
                            LgInput::get().unwrap().set_key_state(
                                key_code.into(), 
                                event.state.is_pressed()
                            );

                            app.on_event(l3gion_rust::lg_core::event::LgEvent::KeyEvent( KeyEvent {
                                code: 0,
                                key: key_code.into(),
                                pressed: event.state.is_pressed(),
                            }))
                        },
                        _ => ()
                    }
                },
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    let button = match button {
                        winit::event::MouseButton::Left => MouseButton::Left,
                            winit::event::MouseButton::Right => MouseButton::Right,
                            winit::event::MouseButton::Middle => MouseButton::Middle,
                            winit::event::MouseButton::Other(val) => MouseButton::Other(val),
                            _ => MouseButton::Other(0)
                    };
                    LgInput::get().unwrap().set_mouse_state(button, state.is_pressed());

                    app.on_event(l3gion_rust::lg_core::event::LgEvent::MouseEvent(MouseEvent::ButtonEvent(MouseButtonEvent {
                        button,
                        pressed: state.is_pressed(),
                    })));
                },
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    LgInput::get().unwrap().set_mouse_position(position.x as f32, position.y as f32);

                    app.on_event(l3gion_rust::lg_core::event::LgEvent::MouseEvent(MouseEvent::MoveEvent(MouseMoveEvent {
                        position: (position.x, position.y),
                    })));
                },
                winit::event::WindowEvent::MouseWheel { delta, .. } => {
                    if let MouseScrollDelta::LineDelta(x, y) = delta {
                        app.on_event(l3gion_rust::lg_core::event::LgEvent::MouseEvent(MouseEvent::ScrollEvent(MouseScrollEvent {
                            delta: (x, y),
                        })));
                    }
                },
                _ => ()
            },
            winit::event::Event::AboutToWait => app.request_redraw(),
            _ => (),
        }
    });
    
    Ok(())
}

