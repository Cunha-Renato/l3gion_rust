use std::mem::size_of;
use crate::{lg_core::uuid::UUID, StdError};

#[derive(Default)]
pub struct Texture {
    uuid: UUID,
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    size: u64,
    mip_level: u32,
}
impl Texture {
    pub fn new(path: &str) -> Result<Self, StdError> {
        let image = image::io::Reader::open(path)?.decode()?;

        let width = image.width();
        let height = image.height();
        let bytes = image.as_bytes().to_vec();
        let size = (bytes.len() * size_of::<u8>()) as u64;

        Ok(Self {
            uuid: UUID::generate(),
            width,
            height,
            bytes,
            size,
            mip_level: (width.max(height) as f32).log2().floor() as u32 + 1,
        })
    }
    
    // Get
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn mip_level(&self) -> u32 {
        self.mip_level
    }
}