use crate::lg_core::input::LgInput;

use super::{
    application::ApplicationCore, entity::LgEntity, event::{LgEvent, MouseEvent}, lg_types::reference::Rfc, renderer::uniform::Data
};
use lg_renderer::renderer::lg_uniform::LgUniform;
use nalgebra_glm as glm;

pub struct TestScene {
    app_core: Rfc<ApplicationCore>,
    
    big: LgEntity,
    smol: LgEntity,
}
impl TestScene {
    pub fn new(app_core: Rfc<ApplicationCore>) -> Self {
        let big = LgEntity::new("big_quad", "grid");
        let smol = LgEntity::new("med_quad", "viking");

        Self {
            app_core,
            
            big,
            smol,
        }
    }
    pub fn init(&mut self) {
    }
    fn update_entity(&mut self) {
    }
    pub fn on_update(&mut self) {
        self.update_entity();
        
        unsafe { 
            self.app_core.borrow_mut().renderer
                .draw_entity(&self.big).unwrap();
            self.app_core.borrow_mut().renderer
                .draw_entity(&self.smol).unwrap();
        }
    }
    pub fn on_event(&mut self, event: &LgEvent) {
        match event {
            LgEvent::WindowEvent(_) => (),
                LgEvent::KeyEvent(_) => (),
                LgEvent::MouseEvent(event) => {
                    if let MouseEvent::ButtonEvent(_) = event {
                        /* let ssbo = unsafe { self.app_core.borrow()
                            .renderer.borrow_mut()
                            .read_buffer::<SSBO>(&self.materials.obj_picker.borrow(), 0)
                            .unwrap()
                        };
                        
                        println!("{:?}", ssbo.data); */
                    }
                },
        }
    }
    pub fn destroy(&mut self) {

    }
}