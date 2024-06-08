use lg_renderer::renderer::LgRendererCreationInfo;

use crate::{as_dyn, lg_core::frame_time::FrameTime, profile_function, profile_scope, StdError};
use super::{event::{KeyEvent, LgEvent, MouseButton, MouseButtonEvent, MouseEvent, MouseMoveEvent, MouseScrollEvent}, input::LgInput, layer::Layer, lg_types::reference::Rfc, renderer::LgRenderer, window::LgWindow};

pub struct ApplicationCreateInfo<'a> {
    pub renderer_api: lg_renderer::renderer::CreationApiInfo,
    pub window_info: lg_renderer::renderer::CreationWindowInfo<'a>,
}

pub struct L3gion {
    app: Application,
    event_loop: winit::event_loop::EventLoop<()>,
}
impl L3gion {
    pub fn new(info: ApplicationCreateInfo) -> Result<Self, StdError> {
        let event_loop = winit::event_loop::EventLoop::new()?;
        let mut info = info;

        info.window_info.event_loop = Some(&event_loop);
        let mut application = Application::new(info)?;
        application.init()?;

        Ok(Self {
            app: application,
            event_loop
        })
    }

    pub fn get_app(&self) -> &Application {
        &self.app
    }
    pub fn get_app_mut(&mut self) -> &mut Application {
        &mut self.app
    }

    pub fn run(mut self) -> Result<(), StdError> {
        self.event_loop.run(move |event, window_target| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        self.app.shutdown().unwrap();
                        window_target.exit()
                    },
                    winit::event::WindowEvent::Resized(window_size) => {
                        if window_size.width > 0 && window_size.height > 0 {
                            self.app.resize(window_size.into()).unwrap()
                        }
                    },
                    winit::event::WindowEvent::RedrawRequested => {
                        self.app.on_update().unwrap();
                    },
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        match event.physical_key {
                            winit::keyboard::PhysicalKey::Code(key_code) => {
                                self.app.on_event(LgEvent::KeyEvent( KeyEvent {
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
    
                        self.app.on_event(LgEvent::MouseEvent(MouseEvent::ButtonEvent(MouseButtonEvent {
                            button,
                            pressed: state.is_pressed(),
                        })));
                    },
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        self.app.on_event(LgEvent::MouseEvent(MouseEvent::MoveEvent(MouseMoveEvent {
                            position: (position.x as u64, position.y as u64),
                        })));
                    },
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        if let winit::event::MouseScrollDelta::LineDelta(x, y) = delta {
                            self.app.on_event(LgEvent::MouseEvent(MouseEvent::ScrollEvent(MouseScrollEvent {
                                delta: (x, y),
                            })));
                        }
                    },
                    _ => ()
                },
                winit::event::Event::AboutToWait => self.app.request_redraw(),
                _ => (),
            }
        })?;
        
        Ok(())
    }
}

pub struct ApplicationCore {
    pub window: LgWindow,
    pub renderer: LgRenderer,
}
pub struct Application {
    core: Rfc<ApplicationCore>,
    layers: Vec<Rfc<dyn Layer>>
}
// Public
impl Application {
    pub fn push_layer(&mut self, mut layer: impl Layer + 'static) -> Result<(), StdError> {
        layer.on_attach(self.core.clone())?;
        self.layers.push(as_dyn!(layer, dyn Layer));

        Ok(())
    }
    
    pub fn pop_layer(&mut self) -> Result<(), StdError> {
        match self.layers.pop() {
            Some(layer) => layer.borrow_mut().on_detach()?,
            None => (),
        };

        Ok(())
    }
}
// Private
impl Application {
    fn new(info: ApplicationCreateInfo) -> Result<Self, StdError> {
        let (window, renderer) = lg_renderer::renderer::LgRenderer::new(LgRendererCreationInfo {
            renderer_api: info.renderer_api,
            window_info: info.window_info,
        })?;

        let core = Rfc::new(ApplicationCore {
            window: LgWindow::new(window),
            renderer: LgRenderer::new(renderer)?,
        });
        
        Ok(Self {
            core,
            layers: Vec::new()
        })
    }

    fn init(&mut self) -> Result<(), StdError> {
        LgInput::init()?;
        FrameTime::init()?;
        self.core.borrow_mut().renderer.init()
    }

    fn shutdown(&mut self) -> Result<(), StdError> {
        profile_function!();

        while let Some(layer) = self.layers.pop() {
            layer.borrow_mut().on_detach()?;
        }

        self.core.borrow_mut().renderer.shutdown()
    }
    
    fn request_redraw(&self) {
        profile_function!();
        self.core.borrow().window.request_redraw();
    }
    
    fn on_event(&mut self, event: LgEvent) {
        profile_function!();
        LgInput::get_locked().unwrap().on_event(&event);

        for layer in &self.layers {
            if layer.borrow_mut().on_event(&event) {
                break;
            }
        }
    }
    
    fn on_update(&mut self) -> Result<(), StdError> {
        optick::next_frame();
        profile_function!();
        
        FrameTime::start()?;

        unsafe { 
            profile_scope!("render_begin");
            self.core.borrow().renderer.begin()?;
        }
        {
            profile_scope!("layers_on_update");
            for layer in &self.layers {
                layer.borrow_mut().on_update()?;
            }
        }

        unsafe { 
            profile_scope!("render_end");
            self.core.borrow_mut().renderer.end()?; 
        }
        
        FrameTime::end()?;

        Ok(())
    }

    fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError>{
        profile_function!();
        unsafe {
            self.core.borrow().renderer.resize(new_size)?;
            
            Ok(())
        }
    }
}