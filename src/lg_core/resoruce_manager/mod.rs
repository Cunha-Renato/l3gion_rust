use std::{collections::HashMap, mem::size_of};
use lg_renderer::renderer::{lg_shader::ShaderStage, lg_texture::{TextureFormat, TextureType}};
use crate::StdError;
use super::{renderer::{mesh::Mesh, shader::Shader, texture::Texture}, uuid::UUID};

const RESOURCE_PATH_YAML: &str = "engine_resources/YAML";
const RESOURCE_PATH_BINARY: &str = "engine_resources/bin";
const YAML_FILE_EXTENSION: &str = "yaml";

const TEXTURE_FORMATS: [&str; 1] = ["png"];
const SHADER_FORMATS: [&str; 2] = ["vert", "frag"];

const UUID_YAML:        &str = "uuid";
const WIDTH_YAML:       &str = "width";
const HEIGHT_YAML:      &str = "height";
const BYTES_YAML:       &str = "bytes";
const MIP_LEVEL_YAML:   &str = "mip_level";
const FORMAT_YAML:      &str = "format";
const TYPE_YAML:        &str = "type";
const SIZE_YAML:        &str = "size";


// UUID / file path
#[derive(Default)]
struct ResourcePaths {
    textures: HashMap<UUID, String>,
    meshes: HashMap<UUID, String>,
    shaders: HashMap<UUID, String>,
}

#[derive(Default)]
struct LoadedResources {
    textures: HashMap<UUID, Texture>,
    meshes: HashMap<UUID, Mesh>,
    shaders: HashMap<UUID, Shader>
}

#[derive(Default)]
pub(crate) struct ResourceManager {
    resource_folders: Vec<String>,
    resource_paths: ResourcePaths,
    loaded: LoadedResources,
}
impl ResourceManager {
    /// Add a folder of pre-processed resource files
    pub(crate) fn add_folder(&mut self, folder_path: &std::path::Path) -> Result<(), StdError> {
        

        Ok(())
    }
    pub(crate) fn get_texture(&mut self, texture_uuid: &UUID) -> Result<&Texture, StdError> {
        if self.loaded.textures.contains_key(&texture_uuid) {
           Ok(self.loaded.textures.get(&texture_uuid).unwrap())
        }
        else {
            self.load_texture(&texture_uuid)?;
            Ok(self.loaded.textures.get(&texture_uuid).unwrap())
        }
    }
    pub(crate) fn process_folder(&mut self, folder_path: &std::path::Path) -> Result<(), StdError> {
        let entries = std::fs::read_dir(folder_path)?;
        
        for entry in entries {
            let path = entry?.path();

            if path.is_file() {
                let path = path.as_path();
                let extension = match path.extension() {
                    Some(ext) => ext.to_str().unwrap(),
                    None => return Err("File doesn't have an extension! (ResourceManager)".into()),
                };
                
                if TEXTURE_FORMATS.contains(&extension) {
                    self.process_texture(path)?;
                }
                else if SHADER_FORMATS.contains(&extension) {
                    self.process_shader(path)?;
                }
            }
            else { self.process_folder(&path)?; }
        }
        
        Ok(())
    }
}
impl ResourceManager {
    fn load_texture(&mut self, texture_uuid: &UUID) -> Result<(), StdError>{
        let texture_path = match self.resource_paths.textures.get(&texture_uuid) {
            Some(path) => path,
            None => return Err("Failed to load texture! (ResourceManager)".into()),
        };
        
        let texture_node = serializer::YamlNode::deserialize_full_path(&texture_path)?;
        
        let name = texture_node.name;
        let mut uuid = 0;
        let mut width = 0;
        let mut height = 0;
        let mut size = 0;
        let mut mip_level = 0;
        let mut format = TextureFormat::RGBA;
        let mut texture_type = TextureType::UNSIGNED_BYTE;
        let mut bytes = Vec::new();
        
        for child_node in texture_node.children {
            let value = child_node.value;
            match child_node.name.as_str() {
                UUID_YAML       => uuid = value.parse::<u128>()?,
                WIDTH_YAML      => width = value.parse()?,
                HEIGHT_YAML     => height = value.parse()?,
                SIZE_YAML       => size = value.parse::<u64>()?,
                MIP_LEVEL_YAML  => mip_level = value.parse()?,
                FORMAT_YAML     => format = TextureFormat::from(value.parse()?)?,
                TYPE_YAML       => texture_type = TextureType::from(value.parse()?)?,
                BYTES_YAML      => bytes = value.split(",")
                                    .map(|s| s.trim().parse::<u8>().unwrap())
                                    .collect::<Vec<_>>(),

                _ => return Err("Texture configuration file has wrong format! (ResourceManager)".into())
            }
        }
        
        assert!(uuid == texture_uuid.get_value() && texture_uuid.is_valid());
        
        let texture = Texture::construct(UUID::from_u128(uuid), &name, width, height, bytes, size, mip_level, texture_type, format);
        self.loaded.textures.insert(UUID::from_u128(uuid), texture);
        
        Ok(())
    }
}
impl ResourceManager {
    /// Only use files with .png, .jpg, .jpeg, etc
    fn process_texture(&mut self, file_path: &std::path::Path) -> Result<(), StdError> {
        // Using YAML        
        let image = image::io::Reader::open(file_path)?.decode()?;
        
        let width = image.width();
        let height = image.height();
        let bytes = image.as_bytes().to_vec();
        let size = (bytes.len() * size_of::<u8>()) as u64;
        let mip_level = (width.max(height) as f32).log2().floor() as u32 + 1; // No idea if this is rigth
        let tex_format = match file_path.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "png" => TextureFormat::RGBA,
                _ => return Err("Texture format not suported, try .png! (ResourceManager)".into())
            },
            None => return Err("Texture file doesn't have an extension! (ResourceManager)".into()),
        };
        let tex_type = TextureType::UNSIGNED_BYTE;
        let texture_name = file_path.file_stem().unwrap()
            .to_string_lossy()
            .to_string();
        let texture_uuid = UUID::from_string(file_path.to_str().unwrap())?;

        // Serializing
        let mut texture_node = serializer::YamlNode {
            name: texture_name.clone(),
            node_type: "TEXTURE".to_string(),
            ..Default::default()
        };
        texture_node.push(serializer::YamlNode { 
            name: UUID_YAML.to_string(), 
            value: texture_uuid.get_value().to_string(),
            ..Default::default()
        });
        texture_node.push(serializer::YamlNode { 
            name: WIDTH_YAML.to_string(), 
            value: width.to_string(), 
            ..Default::default()
        });
        texture_node.push(serializer::YamlNode { 
            name: HEIGHT_YAML.to_string(), 
            value: height.to_string(), 
            ..Default::default()
        });
        texture_node.push(serializer::YamlNode { 
            name: SIZE_YAML.to_string(), 
            value: size.to_string(), 
            ..Default::default()
        });
        texture_node.push(serializer::YamlNode { 
            name: MIP_LEVEL_YAML.to_string(), 
            value: mip_level.to_string(), 
            ..Default::default()
        });
        texture_node.push(serializer::YamlNode { 
            name: FORMAT_YAML.to_string(), 
            value: (tex_format as u32).to_string(), 
            ..Default::default()
        });
        texture_node.push(serializer::YamlNode { 
            name: TYPE_YAML.to_string(), 
            value: (tex_type as u32).to_string(), 
            ..Default::default()
        });
        let bytes_string = bytes.iter()
            .map(|&byte| byte.to_string())
            .collect::<Vec<String>>()
            .join(",");
        texture_node.push(serializer::YamlNode { 
            name: BYTES_YAML.to_string(), 
            value: bytes_string, 
            ..Default::default()
        });
        
        let tex_resources_path = std::format!("{}/textures", RESOURCE_PATH_YAML);
        texture_node.serialize(&tex_resources_path, &texture_name)?;

        Ok(())
    }
    
    /// Only use files with .vert and .frag
    fn process_shader(&mut self, file_path: &std::path::Path) -> Result<(), StdError> {
        let shader_name = file_path.file_stem().unwrap().to_string_lossy().to_string();
        let shader_stage = match file_path.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "vert" => ShaderStage::VERTEX,
                "frag" => ShaderStage::FRAGMENT,
                _ => return Err("Shader format not supported! (ResourceManager)".into()) 
            },
            None => return Err("Shader file doesn't have an extension! (ResourceManager)".into()),
        };
        
        let mut shader_node = serializer::YamlNode {
            name: shader_name.clone(),
            node_type: "SHADER".to_string(),
            ..Default::default()
        };
        let shader_uuid = UUID::from_string(file_path.to_str().unwrap())?;
        shader_node.push(serializer::YamlNode { 
            name: UUID_YAML.to_string(), 
            value: shader_uuid.get_value().to_string(), 
            ..Default::default()
        });
        shader_node.push(serializer::YamlNode { 
            name: "stage".to_string(), 
            value: (shader_stage as u32).to_string(),
            ..Default::default()
        });
        shader_node.push(serializer::YamlNode { 
            name: BYTES_YAML.to_string(), 
            ..Default::default()
        });
        let src_code = crate::utils::tools::file_to_string(file_path.to_str().unwrap())?;
        shader_node.push(serializer::YamlNode { 
            name: "src_code".to_string(), 
            value: src_code,
            ..Default::default()
        });

        let path = std::format!("{}/shaders", RESOURCE_PATH_YAML);
        shader_node.serialize(&path, &shader_name)?;

        Ok(())
    }

    /// Placeholder
    fn process_mesh(&mut self, mesh: &Mesh) -> Result<(), StdError> {
        let mut mesh_node = serializer::YamlNode {
            name: mesh.name().to_string(),
            node_type: "MESH".to_string(),
            ..Default::default()
        };
        mesh_node.push(serializer::YamlNode { 
            name: UUID_YAML.to_string(), 
            value: mesh.uuid().get_value().to_string(), 
            ..Default::default()
        });

        let mut vertex_node = serializer::YamlNode { 
            name: "vertices".to_string(), 
            ..Default::default()
        };

        let mut positions = Vec::new();
        let mut tex_coords = Vec::new();
        for vertex in mesh.vertices() {
            let position = vertex.position;
            let tex_coord = vertex.tex_coord;
            
            let position = position.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let position = std::format!("[{}]", position);
            let tex_coord = tex_coord.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let tex_coord = std::format!("[{}]", tex_coord);
            
            positions.push(position);
            tex_coords.push(tex_coord);
        }
        vertex_node.push(serializer::YamlNode { 
            name: "positions".to_string(), 
            value: positions.join(","), 
            ..Default::default()
        });
        vertex_node.push(serializer::YamlNode { 
            name: "tex_coords".to_string(), 
            value: tex_coords.join(","), 
            ..Default::default()
        });
        mesh_node.push(vertex_node);
        
        let path = std::format!("{}/meshes", RESOURCE_PATH_YAML);
        mesh_node.serialize(&path, mesh.name())?;

        Ok(())
    }
}