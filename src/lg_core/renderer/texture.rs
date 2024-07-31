use std::{hash::Hash, mem::size_of};
use crate::{lg_core::uuid::UUID, StdError};

use super::opengl::gl_texture::GlTexture;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureType {
    UNSIGNED_BYTE,
}
impl Default for TextureType {
    fn default() -> Self {
        Self::UNSIGNED_BYTE
    }
}
impl TextureType {
    pub fn from(value: u32) -> Result<Self, StdError> {
        match value {
            0 => Ok(Self::UNSIGNED_BYTE),
            _ => Err("Failed to convert from u32! (TextureFormat)".into())
        }
    }

    pub fn to_opengl(&self) -> gl::types::GLenum {
        match &self {
            TextureType::UNSIGNED_BYTE => gl::UNSIGNED_BYTE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureFormat {
    RGB,
    RGBA,
    SRGBA,
}
impl Default for TextureFormat {
    fn default() -> Self {
        Self::RGBA
    }
}
impl TextureFormat {
    pub fn from(value: u32) -> Result<Self, StdError> {
        match value {
            0 => Ok(Self::RGB),
            1 => Ok(Self::RGBA),
            2 => Ok(Self::SRGBA),
            _ => Err("Failed to convert from u32! (TextureFormat)".into())
        }
    }
    
    pub fn to_opengl(&self) -> gl::types::GLenum {
        match &self {
            TextureFormat::RGB => gl::RGB,
            TextureFormat::RGBA => gl::RGBA,
            TextureFormat::SRGBA => gl::SRGB_ALPHA
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureFilter {
    LINEAR,
}
impl Default for TextureFilter {
    fn default() -> Self {
        Self::LINEAR
    }
}
impl TextureFilter {
    pub fn to_opengl(&self) -> gl::types::GLenum{
        match &self {
            TextureFilter::LINEAR => gl::LINEAR,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TextureSpecs {
    pub tex_format: TextureFormat,
    pub tex_type: TextureType,
    pub tex_filter: TextureFilter,
}

#[derive(Debug, Clone)]
pub struct Texture {
    uuid: UUID,
    width: u32,
    height: u32,
    size: u64,
    mip_level: u32,
    specs: TextureSpecs,
    name: String, // TODO: Placeholder
    bytes: Vec<u8>,
}
impl Texture {
    pub fn new(name: &str, path: &str, specs: TextureSpecs) -> Result<Self, StdError> {
        let image = image::io::Reader::open(path)?.decode()?;

        let width = image.width();
        let height = image.height();
        let bytes = image.as_bytes().to_vec();
        let size = (bytes.len() * size_of::<u8>()) as u64;
        let mip_level = (width.max(height) as f32).log2().floor() as u32 + 1;

        Ok(Self {
            uuid: UUID::from_string(path)?,
            name: String::from(name),
            width,
            height,
            bytes,
            size,
            mip_level,
            specs,
        })
    }
    
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    
    pub fn name(&self) -> &str {
        &self.name
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
    
    pub fn specs(&self) -> TextureSpecs {
        self.specs
    }

    pub fn construct(
        uuid: UUID,
        name: &str,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        size: u64,
        mip_level: u32,
        specs: TextureSpecs
    ) -> Self 
    {
        Self {
            uuid,
            width,
            height,
            size,
            mip_level,
            specs,
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
impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}