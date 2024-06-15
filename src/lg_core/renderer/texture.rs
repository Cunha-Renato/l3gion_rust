use std::{hash::Hash, mem::size_of};
use lg_renderer::renderer_core::lg_texture::{TextureFormat, TextureType};

use crate::{lg_core::uuid::UUID, StdError};

#[derive(Debug)]
pub struct Texture {
    uuid: UUID,
    width: u32,
    height: u32,
    size: u64,
    mip_level: u32,
    format: TextureFormat,
    texture_type: TextureType,
    name: String, // TODO: Placeholder
    bytes: Vec<u8>,
}
impl Texture {
    pub fn new(name: &str, path: &str, format: TextureFormat, texture_type: TextureType) -> Result<Self, StdError> {
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
            texture_type,
            format,
        })
    }
    
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}
// Public(crate)
impl Texture {
    pub(crate) fn construct(
        uuid: UUID,
        name: &str,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        size: u64,
        mip_level: u32,
        texture_type: TextureType,
        format: TextureFormat,
    ) -> Self 
    {
        Self {
            uuid,
            width,
            height,
            size,
            mip_level,
            format,
            texture_type,
            name: name.to_string(),
            bytes,
        }
    }
}
impl Hash for Texture {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl lg_renderer::renderer_core::lg_texture::LgTexture for Texture {
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
    
    fn texture_type(&self) -> lg_renderer::renderer_core::lg_texture::TextureType {
        self.texture_type
    }
    
    fn texture_format(&self) -> lg_renderer::renderer_core::lg_texture::TextureFormat {
        self.format
    }
}