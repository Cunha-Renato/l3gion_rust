use std::{hash::Hash, mem::size_of};
use crate::{lg_core::uuid::UUID, StdError};

#[derive(Debug, Default)]
pub struct LgTexture {
    uuid: UUID,
    name: String, // TODO: Placeholder
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    size: u64,
    mip_level: u32,
}
impl lg_renderer::renderer::lg_texture::Texture for LgTexture {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn size(&self) -> u64 {
        self.size
    }
    fn mip_level(&self) -> u32 {
        self.mip_level
    }
    
    fn texture_type(&self) -> lg_renderer::renderer::lg_texture::TextureType {
        lg_renderer::renderer::lg_texture::TextureType::UNSIGNED_BYTE
    }
    
    fn texture_format(&self) -> lg_renderer::renderer::lg_texture::TextureFormat {
        lg_renderer::renderer::lg_texture::TextureFormat::RGBA
    }
}
impl LgTexture {
    pub fn new(name: &str, path: &str) -> Result<Self, StdError> {
        let image = image::io::Reader::open(path)?.decode()?;

        let width = image.width();
        let height = image.height();
        let bytes = image.as_bytes().to_vec();
        let size = (bytes.len() * size_of::<u8>()) as u64;

        Ok(Self {
            uuid: UUID::generate(),
            name: String::from(name),
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
    pub fn name(&self) -> &str {
        &self.name
    }
}
impl Hash for LgTexture {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}