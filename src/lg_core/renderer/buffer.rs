use std::os::raw::c_void;

use crate::lg_core::uuid::UUID;

#[derive(Debug, Default, Clone)]
pub struct Buffer {
    uuid: UUID,
    data: Vec<u8>
}
impl Buffer {
    pub unsafe fn new<D>(data: &D) -> Self {
        let size = std::mem::size_of::<D>();
        let data = data as *const D;
        
        let bytes = core::slice::from_raw_parts(data as *const u8, size).to_vec();
        
        Self::from_bytes(bytes)
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            uuid: UUID::generate(),
            data: bytes
        }
    }
    pub fn get_raw_data(&self) -> *const std::ffi::c_void {
        self.data.as_ptr() as *const c_void
    }
    pub unsafe fn set_data<D>(&mut self, data: &D) {
        let size = std::mem::size_of::<D>();
        let data = data as *const D;
        
        self.data = core::slice::from_raw_parts(data as *const u8, size).to_vec();
        
    }
    pub fn data_size(&self) -> usize {
        self.data.len() * std::mem::size_of::<u8>()
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
}