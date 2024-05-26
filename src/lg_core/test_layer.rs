use crate::StdError;
use super::{application::ApplicationCore, entity::LgEntity, layer::Layer, lg_types::reference::Rfc};

pub struct TestLayer {
    core: Option<Rfc<ApplicationCore>>,
    entities: Vec<LgEntity>
}
impl TestLayer {
    fn new() -> Self {
        Self { 
            core: None, 
            entities: Vec::new() 
        }
    }
}
impl Layer for TestLayer {
    fn init(&mut self, app_core: Rfc<ApplicationCore>) -> Result<(), StdError> {
        self.core = Some(app_core);
        
        Ok(())
    }

    fn on_update(&mut self) {
        todo!()
    }

    fn on_event(&mut self, event: super::event::LgEvent) -> bool {

        false
    }

    fn shutdown(&mut self) -> Result<(), StdError> {
        todo!();

        Ok(())
    }
}