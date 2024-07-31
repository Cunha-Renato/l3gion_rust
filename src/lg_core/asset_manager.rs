use std::collections::HashMap;
use crate::{lg_core::glm, StdError};
use super::{renderer::{material::Material, mesh::Mesh, shader::{Shader, ShaderStage}, texture::{Texture, TextureFilter, TextureFormat, TextureSpecs, TextureType}, vertex::Vertex}, uuid::UUID};

const ASSETS_DIR: &str = "assets";
const TEXTURES_DIR: &str = "textures";
const MESHES_DIR: &str = "meshes";
const SHADERS_DIR: &str = "shaders/src";
const MATERIALS_DIR: &str = "materials";

#[derive(Default)]
struct AssetsPath {
    textures: HashMap<UUID, String>,
    meshes: HashMap<UUID, String>,
    shaders: HashMap<UUID, String>,
    materials: HashMap<UUID, String>,
}

#[derive(Default)]
pub struct AssetManager {
    assets_path: AssetsPath,
    textures: HashMap<UUID, Texture>,
    meshes: HashMap<UUID, Mesh>,
    shaders: HashMap<UUID, Shader>,
    materials: HashMap<UUID, Material>,
}
// Public
impl AssetManager {
    pub fn get_texture(&mut self, uuid: &UUID) -> Result<*const Texture, StdError> {
        match self.textures.get(uuid) {
            Some(tex) => return Ok(tex as *const Texture),
            None => ()
        };

        let path = self.assets_path.textures
            .get(uuid)
            .ok_or(std::format!("{} is an invalid texture UUID!", uuid))?
            .clone();
        
        Ok(self.load_texture(std::path::Path::new(&path))?)
    }

    pub fn get_mesh(&mut self, uuid: &UUID) -> Result<*const Mesh, StdError> {
        match self.meshes.get(uuid) {
            Some(mesh) => return Ok(mesh),
            None => ()
        };

        let path = self.assets_path.meshes
            .get(uuid)
            .ok_or(std::format!("{} is an invalid mesh UUID!", uuid))?
            .clone();
        
        Ok(self.load_mesh(std::path::Path::new(&path))?)
    }
    
    pub fn get_shader(&mut self, uuid: &UUID) -> Result<*const Shader, StdError> {
        match self.shaders.get(uuid) {
            Some(shader) => return Ok(shader),
            None => ()
        };

        let path = self.assets_path.shaders
            .get(uuid)
            .ok_or(std::format!("{} is an invalid shader UUID!", uuid))?
            .clone();
        
        Ok(self.load_shader(std::path::Path::new(&path))?)
    }

    pub fn get_material(&mut self, uuid: &UUID) -> Result<*const Material, StdError> {
        match self.materials.get(uuid) {
            Some(material) => return Ok(material),
            None => ()
        };

        let path = self.assets_path.materials
            .get(uuid)
            .ok_or(std::format!("{} is an invalid material UUID!", uuid))?
            .clone();

        Ok(self.load_material(std::path::Path::new(&path))?)
    }
    
    pub fn create_material(&mut self, name: &str, textures: Vec<String>, shaders: Vec<String>) -> Result<*const Material, StdError> {
        let path = std::format!("{ASSETS_DIR}\\{MATERIALS_DIR}\\{name}.lgmat");

        let mut mat_node = serializer::YamlNode {
            name: name.to_string(),
            node_type: "MATERIAL".to_string(),
            ..Default::default()
        };

        let uuid = UUID::from_string(&path)?;
        let textures = textures.join(",");

        mat_node.push(serializer::YamlNode { 
            name: "uuid".to_string(),
            value: uuid.get_value().to_string(), 
            ..Default::default()
        });
        
        mat_node.push(serializer::YamlNode {
            name: "textures".to_string(),
            value: textures,
            ..Default::default()
        });
        
        mat_node.push(serializer::YamlNode {
            name: "vertex_shader".to_string(),
            value: shaders[0].clone(),
            ..Default::default()
        });

        mat_node.push(serializer::YamlNode {
            name: "fragment_shader".to_string(),
            value: shaders[1].clone(),
            ..Default::default()
        });
        
        mat_node.serialize_full(&path)?;

        self.assets_path.materials.entry(uuid.clone()).or_insert(path);
        self.get_material(&uuid)        
    }
}

// Public(crate)
impl AssetManager {
    pub(crate) fn init(&mut self) -> Result<(), StdError> {
        self.read_dir(ASSETS_DIR)
    }
}

// Private
impl AssetManager {
    fn read_dir(&mut self, dir: &str) -> Result<(), StdError> {
        let entries = std::fs::read_dir(dir)?;
        
        for entry in entries {
            let path = entry?.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    self.store_asset_path(
                        path.to_str().unwrap(),
                        extension.to_str().unwrap()
                    )?;
                }
            }
            else {
                self.read_dir(path.to_str().unwrap())?;
            }
        }

        Ok(())
    }
    
    fn store_asset_path(&mut self, path: &str, extension: &str) -> Result<(), StdError> {
        let str_path = path.to_string();
        let uuid = UUID::from_string(&str_path)?;

        match extension {
            "png" |
            "jpg" |
            "jpeg" => self.assets_path.textures.insert(uuid, str_path.clone()),

            "obj" => self.assets_path.meshes.insert(uuid, str_path.clone()),

            "vert" |
            "frag" => self.assets_path.shaders.insert(uuid, str_path.clone()),

            "lgmat" => self.assets_path.materials.insert(uuid, str_path.clone()),

            _ => return Err(std::format!("{} is an invalid asset path!", str_path).into()),
        };

        Ok(())
    }
    
    fn load_texture(&mut self, path: &std::path::Path) -> Result<&mut Texture, StdError> {
        let extension = path.extension().unwrap().to_str().unwrap();

        let tex_format = match extension {
            "png" => TextureFormat::RGBA,
            _ => TextureFormat::RGB,
        };

        let tex_specs = TextureSpecs {
            tex_format,
            tex_type: TextureType::UNSIGNED_BYTE,
            tex_filter: TextureFilter::LINEAR,
        };
        
        let texture = Texture::new(
            path.file_stem().unwrap().to_str().unwrap(),
            path.to_str().unwrap(),
            tex_specs,
        )?;
        
        Ok(self.textures.entry(texture.uuid().clone()).or_insert(texture))
    }
    
    fn load_mesh(&mut self, path: &std::path::Path) -> Result<&mut Mesh, StdError> {
        let tobj_mesh = &mut tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?.0[0].mesh;

        let uuid = UUID::from_string(path.to_str().unwrap())?;
        let name = path.file_stem().unwrap().to_str().unwrap();
        let positions = std::mem::take(&mut tobj_mesh.positions);
        let normals = std::mem::take(&mut tobj_mesh.normals);
        let tex_coords = std::mem::take(&mut tobj_mesh.texcoords);
        let indices = std::mem::take(&mut tobj_mesh.indices);

        let vertices = positions.chunks(3)
            .zip(tex_coords.chunks(2))
            .zip(normals.chunks(3))
            .map(|((p, tc), n)| {
                Vertex {
                    position: glm::vec3(p[0], p[1], p[2]),
                    normal: glm::vec3(n[0], n[1], n[2]),
                    tex_coord: glm::vec2(-tc[0], -tc[1]),
                }
            })
            .collect::<Vec<_>>();

        let mesh = Mesh::new(
            uuid, 
            name, 
            vertices, 
            indices
        );

        Ok(self.meshes.entry(uuid).or_insert(mesh))
    }
    
    fn load_shader(&mut self, path: &std::path::Path) -> Result<&mut Shader, StdError> {
        let uuid = UUID::from_string(path.to_str().unwrap())?;
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let stage = ShaderStage::from_str(path.extension().unwrap().to_str().unwrap())?;
        let src_code = crate::utils::tools::file_to_string(path.to_str().unwrap())?;
        
        let shader = Shader::new(
            uuid, 
            name, 
            vec![], 
            stage, 
            src_code
        );
        
        Ok(self.shaders.entry(uuid).or_insert(shader))
    }
    
    fn load_material(&mut self, path: &std::path::Path) -> Result<&mut Material, StdError> {
        let material_node = serializer::YamlNode::deserialize_full_path(path.to_str().unwrap())?;
        
        let name = material_node.name;
        let mut uuid = 0;
        let mut textures = Vec::new();
        let mut shaders = Vec::new();

        for child_node in material_node.children {
            let value = child_node.value;
            
            match child_node.name.as_str() {
                "uuid" => uuid = value.parse::<u128>()?,
                "textures" => if !value.is_empty() { 
                    textures = value.split(",")
                    . map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
                },
                "vertex_shader" | "fragment_shader" => shaders.push(value.trim().to_string()),

                _ => return Err("Material configuration file has wrong format! (ResourceManager)".into())
            }
        }

        let textures = textures
            .iter()
            .map(|t| {
                UUID::from_string(&*t).unwrap()
            })
            .collect::<Vec<_>>();

        let shaders = shaders
            .iter()
            .map(|s| {
                UUID::from_string(&*s).unwrap()
            })
            .collect::<Vec<_>>();

        let material = Material::new(
            UUID::from_u128(uuid), 
            &name,
            shaders,
            textures, 
            vec![]
        );

        Ok(self.materials.entry(UUID::from_u128(uuid)).or_insert(material))
    }
}