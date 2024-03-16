use winit::window::Window;

use crate::MyError;
use super::{
    input::Input,
    renderer::{ 
        Renderer, 
        camera::Camera,
    },
    event::Event,
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
        let mut renderer = unsafe { Renderer::create(&window)? };
        let input = Input::new();
        let main_camera = Camera::new(
            vmm::to_radians(45.0) as f32,
            window.inner_size().width as f32, 
            window.inner_size().height as f32, 
            0.1, 
            1000.0
        );

        renderer.set_camera(main_camera.clone());

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
        unsafe { self.core.renderer.destroy() }
    }
     
    pub fn on_update(&mut self) {
        self.main_camera.on_update(&self.core.input);
        self.core.renderer.set_camera(self.main_camera);

        // Rendering
        unsafe { self.core.renderer.render(&self.core.window).unwrap(); }
    }

    pub fn on_event(&mut self, event: &Event) {
        self.main_camera.on_event(event);
    }
}