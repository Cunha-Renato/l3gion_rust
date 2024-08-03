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
    SRGB8,
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
            2 => Ok(Self::SRGB8),
            _ => Err("Failed to convert from u32! (TextureFormat)".into())
        }
    }
    
    pub fn to_opengl(&self) -> gl::types::GLenum {
        match &self {
            TextureFormat::RGB => gl::RGB,
            TextureFormat::RGBA => gl::RGBA,
            TextureFormat::SRGB8 => gl::SRGB8
        }
    }

    pub fn to_opengl_internal(&self) -> gl::types::GLenum {
        match &self {
            TextureFormat::RGB => gl::RGB,
            TextureFormat::RGBA => gl::RGBA,
            TextureFormat::SRGB8 => gl::RGB
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

#[derive(Debug)]
pub struct Texture {
    uuid: UUID,
    width: u32,
    height: u32,
    size: u64,
    mip_level: u32,
    specs: TextureSpecs,
    name: String, // TODO: Placeholder
    bytes: Vec<u8>,
    
    pub(crate) gl_texture: Option<GlTexture>,
}
impl Texture {
    pub fn new(name: &str, path: &str, specs: TextureSpecs) -> Result<Self, StdError> {
        let image = image::ImageReader::open(path)?.decode()?;

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
            gl_texture: None,
        })
    }
    
    pub fn gl_id(&self) -> Option<gl::types::GLuint> {
        if let Some(gl_tex) = &self.gl_texture {
            return Some(gl_tex.id);
        }

        None
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
            gl_texture: None
        }
    }
}

// Public(crate)
impl Texture {
    pub(crate) fn init_opengl(&mut self) -> Result<(), StdError> {
        if self.gl_texture.is_some() { return Ok(()); }

        let gl_tex = GlTexture::new()?;
        gl_tex.bind()?;
        gl_tex.load(&self)?;
        gl_tex.unbind()?;

        self.gl_texture = Some(gl_tex);

        Ok(())
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