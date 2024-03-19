use winit::window::Window;

use crate::{utils::tools::to_radians, MyError};
use super::{
    event::Event, input::Input, renderer::{ 
        camera::Camera, Renderer
    }
};
pub struct AppCore {
    window: Window,
    renderer: Renderer,
    pub input: Input,
}

pub struct Application {
    pub core: AppCore,
    main_camera: Camera,
}
impl Application {
    pub fn new(window: Window) -> Result<Self, MyError> {
        let mut renderer = Renderer::init(&window)?;
        let input = Input::new();
        let main_camera = Camera::new(
            to_radians(45.0) as f32,
            window.inner_size().width as f32, 
            window.inner_size().height as f32, 
            0.1, 
            1000.0
        );

        renderer.core.set_camera(&main_camera);

        let core = AppCore {
            window,
            renderer,
            input,
        };
        
        Ok(Self {
            core,
            main_camera,
        })
    }
    
    pub fn destroy(&mut self) {
        unsafe { self.core.renderer.core.destroy() }
    }
     
    pub fn on_update(&mut self) {
        self.main_camera.on_update(&self.core.input);
        self.core.renderer.core.set_camera(&self.main_camera);

        // Rendering
        unsafe { self.core.renderer.core.render(&self.core.window).unwrap(); }
    }

    pub fn on_event(&mut self, event: &Event) {
        self.main_camera.on_event(event);
    }
}