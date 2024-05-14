use crate::lg_core::{input::LgInput, renderer::uniform::SSBO};

use super::{
    application::ApplicationCore, entity::LgEntity, event::{LgEvent, MouseEvent}, lg_types::reference::Rfc, renderer::uniform::Data
};
use lg_renderer::renderer::lg_uniform::{LgUniform, LgUniformType};
use nalgebra_glm as glm;

pub struct TestScene {
    app_core: Rfc<ApplicationCore>,
    
    pick_object: bool,
    
    big: LgEntity,
    smol: LgEntity,
}
impl TestScene {
    pub fn new(app_core: Rfc<ApplicationCore>) -> Self {
        let mut smol = LgEntity::new("med_quad", "obj_picker");
        let mut big = LgEntity::new("big_quad", "obj_picker");

        let data = Data {
            mouse_position: glm::vec2(0.0, 0.0),
            uuid: 0
        };
        smol.uniforms.push(LgUniform::new(
            "data", 
            LgUniformType::STRUCT, 
            0, 
            0, 
            true,
            data.clone()
        ));
        big.uniforms.push(LgUniform::new(
            "data", 
            LgUniformType::STRUCT, 
            0, 
            0, 
            true,
            data
        ));

        Self {
            app_core,
            
            pick_object: false,
            big,
            smol,
        }
    }
    pub fn init(&mut self) {
    }
    fn update_entity(&mut self) {
        let mut pos = LgInput::get().unwrap().get_mouse_position();
        pos.y = self.app_core.borrow().window.size().1 as f32 - pos.y;
        let mut data = Data {
            mouse_position: pos,
            uuid: 150
        };
        self.smol.uniforms[0].set_data(data.clone());
        
        data.uuid = 255;
        self.big.uniforms[0].set_data(data);
    }
    pub fn on_update(&mut self) {
        unsafe { 
            self.app_core.borrow_mut().renderer
                .draw_entity(&self.big).unwrap();
            self.app_core.borrow_mut().renderer
                .draw_entity(&self.smol).unwrap();
        }

        if self.pick_object {
            let ssbo = unsafe { self.app_core.borrow()
                .renderer
                .read_material_ubo::<SSBO>("obj_picker", "ssbo")
                .unwrap()
            };
            self.pick_object = false;
            
            println!("{:?}", ssbo.data);
        }
        else {
            unsafe { self.app_core.borrow().renderer.reset_material_ubo("obj_picker", "ssbo").unwrap(); }
        }

        self.update_entity();
    }
    pub fn on_event(&mut self, event: &LgEvent) {
        match event {
            LgEvent::WindowEvent(_) => (),
                LgEvent::KeyEvent(_) => (),
                LgEvent::MouseEvent(event) => {
                    if let MouseEvent::ButtonEvent(e) = event {
                        self.pick_object = e.pressed;
                    }
                },
        }
    }
    pub fn destroy(&mut self) {

    }
}