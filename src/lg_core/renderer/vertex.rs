use std::{
    mem::size_of,
    hash::{
        Hash,
        Hasher,
    },
};
use vulkanalia::{
    prelude::v1_0::*, 
    vk,
};
pub use nalgebra_glm as glm;

pub trait VkVertex: PartialEq + Eq + Hash {
    fn binding_description() -> vk::VertexInputBindingDescription;
    fn attribute_descritptions() -> [vk::VertexInputAttributeDescription; 3];
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    position: glm::Vec3,
    color: glm::Vec3,
    tex_coord: glm::Vec2,
}
impl Vertex {
    pub const fn new(position: glm::Vec3, color: glm::Vec3, tex_coord: glm::Vec2) -> Self {
        Self { position, color, tex_coord }
    }
}
impl VkVertex for Vertex {
    fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }
    
    fn attribute_descritptions() -> [vk::VertexInputAttributeDescription; 3] {
        let position = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<glm::Vec3>() as u32)
            .build();
        
        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<glm::Vec3>() + size_of::<glm::Vec3>()) as u32)
            .build();
        
        [position, color, tex_coord]
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.color == other.color
            && self.tex_coord == other.tex_coord
    }
}
impl Eq for Vertex {}
impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position[0].to_bits().hash(state);
        self.position[1].to_bits().hash(state);
        self.position[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}

