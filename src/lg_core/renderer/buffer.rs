use lg_renderer::renderer::lg_buffer::LgBufferData;

use crate::lg_core::uuid::UUID;

pub struct Buffer {
    uuid: UUID,
    data: Box<dyn LgBufferData>
}
impl Buffer {
    pub fn new(data: impl LgBufferData) -> Self {
        Self {
            uuid: UUID::generate(),
            data: Box::new(data) as Box<dyn LgBufferData>,
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
}
impl lg_renderer::renderer::lg_buffer::LgBuffer for Buffer {
    fn data_size(&self) -> usize {
        self.data.size()
    }

    fn get_raw_data(&self) -> *const std::ffi::c_void {
        self.data.as_c_void()
    }

    fn set_data(&mut self, data: impl LgBufferData) {
        self.data = Box::new(data) as Box<dyn LgBufferData>;
    }
}