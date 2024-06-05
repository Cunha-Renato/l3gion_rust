#![allow(non_camel_case_types)]

pub type StdError = Box<dyn std::error::Error>;
pub mod utils;
pub mod lg_core;

use lg_core::{
    application::Application, 
    event::{
        KeyEvent, 
        LgEvent, 
        MouseButton, 
        MouseButtonEvent, 
        MouseEvent, 
        MouseMoveEvent, 
        MouseScrollEvent
    }, 
    input::LgInput, layer::Layer
};

pub enum RendererAPI {
    OPEN_GL,
    VULKAN
}

pub struct L3gion {
    event_loop: winit::event_loop::EventLoop<()>,
    application: Application,
}
impl L3gion {
    pub fn new(api: RendererAPI) -> Result<Self, StdError> {
        let event_loop = winit::event_loop::EventLoop::new()?;
        let application = match api {
            RendererAPI::OPEN_GL => Application::new_opengl(&event_loop)?,
            RendererAPI::VULKAN => Application::new_vulkan(),
        };
        
        Ok(Self {
            application,
            event_loop
        })
    }
    pub fn push_layer(&mut self, layer: impl Layer + 'static) -> Result<(), StdError> {
        self.application.push_layer(layer)
    }
    pub fn run(mut self) {
        let _ = self.event_loop.run(move |event, window_target| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        self.application.shutdown().unwrap();
                        window_target.exit()
                    },
                    winit::event::WindowEvent::Resized(window_size) => {
                        if window_size.width > 0 && window_size.height > 0 {
                            self.application.resize(window_size.into()).unwrap()
                        }
                    },
                    winit::event::WindowEvent::RedrawRequested => {
                        self.application.on_update().unwrap();
                    },
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        match event.physical_key {
                            winit::keyboard::PhysicalKey::Code(key_code) => {
                                LgInput::get().unwrap().set_key_state(
                                    key_code.into(), 
                                    event.state.is_pressed()
                                );
    
                                self.application.on_event(LgEvent::KeyEvent( KeyEvent {
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
    
                        self.application.on_event(lg_core::event::LgEvent::MouseEvent(MouseEvent::ButtonEvent(MouseButtonEvent {
                            button,
                            pressed: state.is_pressed(),
                        })));
                    },
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        LgInput::get().unwrap().set_mouse_position(position.x as f32, position.y as f32);
    
                        self.application.on_event(lg_core::event::LgEvent::MouseEvent(MouseEvent::MoveEvent(MouseMoveEvent {
                            position: (position.x, position.y),
                        })));
                    },
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        if let winit::event::MouseScrollDelta::LineDelta(x, y) = delta {
                            self.application.on_event(lg_core::event::LgEvent::MouseEvent(MouseEvent::ScrollEvent(MouseScrollEvent {
                                delta: (x, y),
                            })));
                        }
                    },
                    _ => ()
                },
                winit::event::Event::AboutToWait => self.application.request_redraw(),
                _ => (),
            }
        });            
    }
}