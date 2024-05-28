use crate::StdError;
use super::{application::ApplicationCore, entity::LgEntity, layer::Layer, lg_types::reference::Rfc, uuid::UUID};

pub struct TestLayer {
    core: Option<Rfc<ApplicationCore>>,
    entities: Vec<LgEntity>
}
impl TestLayer {
    pub fn new() -> Self {
        Self { 
            core: None, 
            entities: Vec::new() 
        }
    }
}
impl Layer for TestLayer {
    fn init(&mut self, app_core: Rfc<ApplicationCore>) -> Result<(), StdError> {
        self.core = Some(app_core);
        self.entities.push(LgEntity::new(
            UUID::from_u128(94175893682642414160568079829868456088),
            UUID::from_u128(0),
        ));
        
        Ok(())
    }

    fn on_update(&mut self) {
        unsafe {
            self.core.as_mut().unwrap()
                .borrow_mut()
                .renderer
                .draw_entity(&self.entities[0]).unwrap();
        }
    }

    fn on_event(&mut self, event: super::event::LgEvent) -> bool {

        false
    }

    fn shutdown(&mut self) -> Result<(), StdError> {

        Ok(())
    }
}