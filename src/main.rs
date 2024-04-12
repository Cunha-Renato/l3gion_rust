use std::env;

use l3gion_rust::{
    lg_core::{
        application::Application,
        event::{
            KeyEvent,
            MouseButton,
            MouseButtonEvent,
            MouseEvent,
            MouseMoveEvent,
            MouseScrollEvent,
        }
    },
    window::get_event_loop
};
use l3gion_rust::MyError;

use sllog::{error, warn};
use winit::event::{ElementState, MouseScrollDelta};
use winit::{
    dpi::LogicalSize,
    event::{
        Event,
        WindowEvent
    },
    event_loop::ControlFlow,
    window::WindowBuilder,
};

fn main() -> Result<(), MyError> {
    env::set_var("LOG", "4");

    // Window
    let event_loop = get_event_loop();
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1080, 720))
        .build(&event_loop)?;

    // App
    let mut app = Application::new(window)?;
    let mut destroying = false;
    let mut minimized = false;
    
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared if !destroying && !minimized => {
                match app.on_update() {
                    Err(e) => warn!("{:#?}", e),
                    _ => ()
                }
            }
            Event::WindowEvent {event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        destroying = true;
                        *control_flow = ControlFlow::Exit;
                        app.destroy().unwrap();
                    },
                    WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                            minimized = true;
                        }
                        else {
                            minimized = false;
                            app.resized = true;
                        }
                    },
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(key_code) = input.virtual_keycode {
                            let state = input.state == ElementState::Pressed;

                            app.core.input.set_key_state(key_code.into(), state);
                            app.on_event(&l3gion_rust::lg_core::event::LgEvent::KeyEvent(KeyEvent {
                                code: input.scancode,
                                key: key_code.into(),
                                pressed: state,
                            })).unwrap();
                        }
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        let button = match button {
                            winit::event::MouseButton::Left => MouseButton::Left,
                            winit::event::MouseButton::Right => MouseButton::Right,
                            winit::event::MouseButton::Middle => MouseButton::Middle,
                            winit::event::MouseButton::Other(val) => MouseButton::Other(val),
                        };
                        let btn_state = state == ElementState::Pressed;

                        app.core.input.set_mouse_state(button, btn_state);
                        app.on_event(&l3gion_rust::lg_core::event::LgEvent::MouseEvent(MouseEvent::ButtonEvent(MouseButtonEvent {
                            button: button,
                            pressed: btn_state,
                        }))).unwrap();
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        app.core.input.set_mouse_position(position.x as f32, position.y as f32);
                        app.on_event(&l3gion_rust::lg_core::event::LgEvent::MouseEvent(MouseEvent::MoveEvent(MouseMoveEvent {
                            position: (position.x, position.y),
                        }))).unwrap();
                    },
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let MouseScrollDelta::LineDelta(x, y) = delta {
                            app.on_event(&l3gion_rust::lg_core::event::LgEvent::MouseEvent(MouseEvent::ScrollEvent(MouseScrollEvent {
                                delta: (x, y),
                            }))).unwrap();
                        }
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    });
}

