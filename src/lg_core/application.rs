use crate::{glm, lg_core::{frame_time::FrameTime, renderer::Renderer}, profile_function, profile_scope, StdError};
use super::{event::{KeyEvent, LgEvent, MouseButton, MouseButtonEvent, MouseEvent, MouseMoveEvent, MouseScrollEvent}, input::LgInput, layer::Layer, lg_types::reference::Rfc, renderer::CreationWindowInfo,  window::LgWindow};

pub struct PersistentApplicationInfo {
    pub v_sync: bool,
}

pub struct ApplicationCreateInfo<'a> {
    pub persistant_info: PersistentApplicationInfo,
    pub window_info: CreationWindowInfo<'a>,
}

pub struct L3gion {
    app: Application,
    _event_loop: winit::event_loop::EventLoop<()>,
}
impl L3gion {
    pub fn new(info: ApplicationCreateInfo) -> Result<Self, StdError> {
        profile_function!();
        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut info = info;
        info.window_info.event_loop = Some(&event_loop);

        let mut application = Application::new(info)?;
        application.init()?;

        Ok(Self {
            app: application,
            _event_loop: event_loop,
        })
    }

    pub fn get_app(&self) -> &Application {
        &self.app
    }
    pub fn get_app_mut(&mut self) -> &mut Application {
        &mut self.app
    }

    pub fn run(mut self) -> Result<(), StdError> {
        let mut last_frame = std::time::Instant::now();

        self._event_loop.run(move |event, window_target| {
            match &event {
                event => {
                    self.app.core.renderer.borrow().handle_imgui_event(
                        &self.app.core.window.borrow().window, 
                        event
                    );
                }
            }
            match event {
                /* winit::event::Event::NewEvents(cause) => match cause {
                    winit::event::StartCause::Poll => {
                        optick::next_frame();
                        self.app.on_update().unwrap();
                    },
                    _ => (),
                }, */
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        self.app.shutdown().unwrap();
                        window_target.exit()
                    },
                    winit::event::WindowEvent::Resized(window_size) => {
                        self.app.on_event(LgEvent::WindowEvent(super::event::WindowEvent::Resize(window_size.width, window_size.height)));

                        if window_size.width > 0 && window_size.height > 0 {
                            self.app.resize(window_size.into()).unwrap();
                        }
                    },
                    winit::event::WindowEvent::RedrawRequested => {
                        optick::next_frame();
                        let now = std::time::Instant::now();

                        self.app.core.renderer
                            .borrow()
                            .update_imgui_delta_time(now.duration_since(last_frame));

                        last_frame = now;

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
                            position: glm::vec2(position.x, position.y),
                        })));
                    },
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        if let winit::event::MouseScrollDelta::LineDelta(x, y) = delta {
                            self.app.on_event(LgEvent::MouseEvent(MouseEvent::ScrollEvent(MouseScrollEvent {
                                delta: glm::vec2(x, y),
                            })));
                        }
                    },
                    _ => ()
                },
                winit::event::Event::AboutToWait => {
                    self.app.core.renderer.borrow_mut().prepare_imgui_frame(
                        &self.app.core.window.borrow().window
                    );
                    self.app.core.window.borrow().request_redraw();
                },
                _ => (),
            }
        })?;
        
        Ok(())
    }
}

#[derive(Clone)]
pub struct ApplicationCore {
    pub window: Rfc<LgWindow>,
    pub renderer: Rfc<Renderer>,
}
pub struct Application {
    core: ApplicationCore,
    layers: Vec<Rfc<dyn Layer>>
}
// Public
impl Application {
    pub fn core(&self) -> ApplicationCore {
        self.core.clone()
    }

    pub fn push_layer(&mut self, layer: Rfc<dyn Layer>) -> Result<(), StdError> {
        layer.borrow_mut().on_attach()?;

        self.layers.push(layer);
        
        Ok(())
    }

    pub fn push_new_layer<F>(&mut self, func: F) -> Result<(), StdError> 
        where F: FnOnce(ApplicationCore) -> Rfc<dyn Layer>
    {
        let layer = func(self.core.clone());
        self.push_layer(layer)
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
        profile_function!();
        let (renderer, window) = Renderer::new(&info.window_info)?;
        renderer.send(crate::lg_core::renderer::command::SendRendererCommand::_INIT);
        let renderer = Rfc::new(renderer);
        let window = Rfc::new(window);

        let core = ApplicationCore {
            window,
            renderer,
        };

        Ok(Self {
            core,
            layers: vec![]
        })
    }

    fn init(&mut self) -> Result<(), StdError> {
        profile_function!();

        // Singletons
        LgInput::init();
        FrameTime::init()?;
        
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), StdError> {
        profile_function!();

        while let Some(layer) = self.layers.pop() {
            layer.borrow_mut().on_detach()?;
        }

        self.core.renderer.borrow_mut().shutdown();
        
        Ok(())
    }
    
    fn on_event(&mut self, event: LgEvent) {
        profile_function!();
        LgInput::on_event(&event);

        for layer in &self.layers {
            if layer.borrow_mut().on_event(&event) {
                return;
            }
        }
    }
    
    fn on_update(&mut self) -> Result<(), StdError> {
        profile_function!();
        FrameTime::end()?;
        FrameTime::start()?;
        
        for layer in &self.layers {
            layer.borrow_mut().on_update()?;
        }

        {
            profile_scope!("ImGui");
            let ui = unsafe { 
                self.core
                    .renderer
                    .borrow_mut()
                    .core()
                    .imgui()
                    .new_frame()
                    .as_mut()
                    .unwrap()
            };

            for layer in &self.layers {
                layer.borrow_mut().on_imgui(ui);
            }
            
            self.core
                .renderer
                .borrow_mut()
                .core()
                .imgui()
                .prepare_to_render(ui, &self.core.window.borrow().window);
        }

        let mut renderer = self.core.renderer.borrow_mut();
        // TODO: Deal with editor / runtime builds
        // renderer.send(crate::lg_core::renderer::command::SendRendererCommand::_DRAW_BACKBUFFER);
        renderer.send(crate::lg_core::renderer::command::SendRendererCommand::_DRAW_IMGUI);
        renderer.end();

        Ok(())
    }

    fn resize(&self, new_size: (u32, u32)) -> Result<(), StdError> {
        profile_function!();
        self.core.renderer.borrow_mut().resize(new_size);
        
        Ok(())
    }
}