#![allow(non_camel_case_types)]

use std::collections::HashSet;

use self::texture::LgTexture;

use super::uuid::UUID;
pub mod vertex;
pub mod mesh;
pub mod material;
pub mod texture;
pub mod shader;

pub struct LgRenderer {
    renderer: lg_renderer::renderer::LgRenderer<UUID>,
    a: HashSet<LgTexture>
}