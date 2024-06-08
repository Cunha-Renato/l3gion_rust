use std::{collections::HashMap, mem::size_of};
use lg_renderer::renderer::{lg_shader::ShaderStage, lg_texture::{TextureFormat, TextureType}};
use crate::{lg_core::renderer::vertex::Vertex, StdError};
use super::{renderer::{material::Material, mesh::Mesh, shader::Shader, texture::Texture}, uuid::UUID};
use nalgebra_glm as glm;

// TODO: Make so that this path below is always present and with the necessary assets loaded.
const CORE_ASSET_FOLDER:      &str = "engine_assets/core/";
const ASSET_FOLDER:           &str = "engine_assets/";
const ASSET_FOLDER_BINARY:    &str = "bin/";
const ASSET_FOLDER_YAML:      &str = "YAML/";
const ASSET_FOLDER_MESHES:    &str = "meshes/";
const ASSET_FOLDER_TEXTURES:  &str = "textures/";
const ASSET_FOLDER_SHADERS:   &str = "shaders/";
const ASSET_FOLDER_MATERIALS: &str = "materials/";

const TEXTURE_FORMATS:      [&str; 1] = ["png"];
const SHADER_FORMATS:       [&str; 2] = ["vert", "frag"];
const MESH_FORMAT:          &str = "obj";
const YAML_FILE_EXTENSION:  &str = "yaml";

const TEXTURE_YAML:         &str = "TEXTURE";
const SHADER_YAML:          &str = "SHADER";
const MESH_YAML:            &str = "MESH";
const MATERIAL_YAML:        &str = "MATERIAL";

const UUID_YAML:            &str = "uuid";
const SHADER_STAGE_YAML:    &str = "stage";
const SHADER_SRC_CODE_YAML: &str = "src_code";
const WIDTH_YAML:           &str = "width";
const HEIGHT_YAML:          &str = "height";
const BYTES_YAML:           &str = "bytes";
const MIP_LEVEL_YAML:       &str = "mip_level";
const FORMAT_YAML:          &str = "format";
const TYPE_YAML:            &str = "type";
const SIZE_YAML:            &str = "size";
const POSITIONS_YAML:       &str = "positions";
const INDICES_YAML:         &str = "indices";
const NORMALS_YAML:         &str = "normals";
const TEX_COORDS_YAML:      &str = "tex_coords";
const TEXTURES_YAML:        &str = "textures";
const VERTEX_SHADER_YAML:   &str = "vertex_shader";
const FRAGMENT_SHADER_YAML: &str = "fragment_shader";


// UUID / file path
#[derive(Default, Debug)]
struct AssetPaths {
    textures: HashMap<UUID, String>,
    meshes: HashMap<UUID, String>,
    shaders: HashMap<UUID, String>,
    materials: HashMap<UUID, String>,
}

#[derive(Default, Debug)]
struct LoadedAssets {
    textures: HashMap<UUID, Texture>,
    meshes: HashMap<UUID, Mesh>,
    shaders: HashMap<UUID, Shader>,
    materials: HashMap<UUID, Material>,
}

#[derive(Default, Debug)]
pub(crate) struct AssetManager {
    resource_folders: Vec<String>,
    resource_paths: AssetPaths,
    loaded: LoadedAssets,
}
// Public
impl AssetManager {
    pub(crate) fn init(&mut self) -> Result<(), StdError> {
        self.init_folders()?;

        // YAML
        let core_path = std::format!("{}{}", CORE_ASSET_FOLDER, ASSET_FOLDER_YAML);
        let normal_path = std::format!("{}{}", ASSET_FOLDER, ASSET_FOLDER_YAML);
        self.read_asset_paths(std::path::Path::new(&core_path))?;
        self.read_asset_paths(std::path::Path::new(&normal_path))
    }

    pub(crate) fn read_asset_paths(&mut self, path: &std::path::Path) -> Result<(), StdError> {
        // TODO: Only reading the YAML files
        let entries = std::fs::read_dir(path)?;
        
        for entry in entries {
            let path = entry?.path();

            if path.is_file() && path.extension().unwrap().to_str().unwrap() == YAML_FILE_EXTENSION {
                let node = serializer::YamlNode::deserialize_full_path(path.to_str().unwrap())?;

                for child in node.children {
                    match child.name.as_str() {
                        UUID_YAML => {
                            let uuid = UUID::from_u128(child.value.parse::<u128>()?);
                            match node.node_type.as_str() {
                                TEXTURE_YAML => { let _ = self.resource_paths.textures.insert(uuid, path.to_string_lossy().to_string()); },
                                SHADER_YAML => { let _ = self.resource_paths.shaders.insert(uuid, path.to_string_lossy().to_string()); },
                                MESH_YAML => { let _ = self.resource_paths.meshes.insert(uuid, path.to_string_lossy().to_string()); },
                                MATERIAL_YAML => { let _ = self.resource_paths.materials.insert(uuid, path.to_string_lossy().to_string()); },
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
            } else { self.read_asset_paths(&path)?; }
        }
        
        Ok(())
    }
    /// Add a folder of pre-processed resource files
    pub(crate) fn add_folder(&mut self, folder_path: &std::path::Path) -> Result<(), StdError> {
        

        Ok(())
    }

    pub(crate) fn prepare_texture(&mut self, texture_uuid: &UUID) -> Result<(), StdError> {
        if !self.loaded.textures.contains_key(texture_uuid) {
            self.load_texture(texture_uuid)?;
        }
        
        Ok(())
    }

    pub(crate) fn prepare_shader(&mut self, shader_uuid: &UUID) -> Result<(), StdError> {
        if !self.loaded.shaders.contains_key(shader_uuid) {
            self.load_shader(shader_uuid)?;
        }
        Ok(())
    }

    pub(crate) fn prepare_mesh(&mut self, mesh_uuid: &UUID) -> Result<(), StdError> {
        if !self.loaded.meshes.contains_key(mesh_uuid) {
            self.load_mesh(mesh_uuid)?;
        }
        
        Ok(())
    }

    pub(crate) fn prepare_material(&mut self, material_uuid: &UUID) -> Result<(), StdError> {
        if !self.loaded.materials.contains_key(material_uuid) {
            self.load_material(material_uuid)?;
        }
        
        Ok(())
    }

    pub(crate) fn get_texture(&self, texture_uuid: &UUID) -> Option<&Texture> {
        self.loaded.textures.get(texture_uuid)
    }

    pub(crate) fn get_shader(&self, shader_uuid: &UUID) -> Option<&Shader> {
        self.loaded.shaders.get(shader_uuid)
    }

    pub(crate) fn get_mesh(&self, mesh_uuid: &UUID) -> Option<&Mesh> {
        self.loaded.meshes.get(mesh_uuid)
    }

    pub(crate) fn get_material(&self, material_uuid: &UUID) -> Option<&Material> {
        self.loaded.materials.get(material_uuid)
    }

    pub(crate) fn process_folder(&mut self, folder_path: &std::path::Path) -> Result<(), StdError> {
        self.init_folders()?;

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
                else if MESH_FORMAT == extension {
                    self.process_mesh(path)?;
                }
            }
            else { self.process_folder(&path)?; }
        }
        
        Ok(())
    }
}
// Private
impl AssetManager {
    // ------------------------- Loading ------------------------- 
    fn load_texture(&mut self, texture_uuid: &UUID) -> Result<(), StdError> {
        let texture_path = match self.resource_paths.textures.get(texture_uuid) {
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
    
    // TODO: Rename vertices in the mesh.yaml to position
    fn load_mesh(&mut self, mesh_uuid: &UUID) -> Result<(), StdError> {
        let mesh_path = match self.resource_paths.meshes.get(mesh_uuid) {
            Some(path) => path,
            None => return Err("Failed to load Mesh! (ResourceManager)".into()),
        };
        
        let mesh_node = serializer::YamlNode::deserialize_full_path(&mesh_path)?;
        
        let name = mesh_node.name;
        let mut uuid = 0;
        let mut positions = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();
        
        for child_node in mesh_node.children {
            let value = child_node.value;
            match child_node.name.as_str() {
                UUID_YAML => uuid = value.parse::<u128>()?,
                POSITIONS_YAML => positions = value.split(",").map(|s| s.trim().parse::<f32>().unwrap()).collect::<Vec<_>>(),
                INDICES_YAML => indices = value.split(",").map(|s| s.trim().parse::<u32>().unwrap()).collect::<Vec<_>>(),
                TEX_COORDS_YAML => tex_coords = value.split(",").map(|s| s.trim().parse::<f32>().unwrap()).collect::<Vec<_>>(), 
                NORMALS_YAML => (),

                _ => return Err("Mesh configuration file has wrong format! (ResourceManager)".into())
            }
        }

        assert!(uuid == mesh_uuid.get_value() && mesh_uuid.is_valid());
        let chunk_pos = positions.chunks(3);
        let chunk_tex_coords = tex_coords.chunks(2);

        let vertices = chunk_pos.zip(chunk_tex_coords)
            .map(|(p, tc)| Vertex {
                position: glm::vec3(p[0], p[1], p[2]),
                tex_coord: glm::vec2(tc[0], tc[1])
        }).collect::<Vec<_>>();

        let mesh = Mesh::new(
            UUID::from_u128(uuid),
            &name, 
            vertices, 
            indices
        );
        self.loaded.meshes.insert(UUID::from_u128(uuid), mesh);
        
        Ok(())
    }

    fn load_shader(&mut self, shader_uuid: &UUID) -> Result<(), StdError> {
        let shader_path = match self.resource_paths.shaders.get(shader_uuid) {
            Some(path) => path,
            None => return Err("Failed to load shader! (ResourceManager)".into()),
        };

        let shader_node = serializer::YamlNode::deserialize_full_path(&shader_path)?;
        
        let name = shader_node.name;
        let mut uuid = 0;
        let mut shader_stage = ShaderStage::VERTEX;
        let mut src_code = String::new();
        let mut bytes = Vec::new();
        
        for child_node in shader_node.children {
            let value = child_node.value;
            
            match child_node.name.as_str() {
                UUID_YAML => uuid = value.parse::<u128>()?,
                SHADER_STAGE_YAML => shader_stage = ShaderStage::from_u32(value.parse::<u32>()?)?,
                SHADER_SRC_CODE_YAML => src_code = value,
                BYTES_YAML => if !value.is_empty() { bytes = value.split(",").map(|s| s.trim().parse::<u8>().unwrap()).collect::<Vec<_>>() },

                _ => return Err("Shader configuration file has wrong format! (ResourceManager)".into())
            }
        }

        assert!(uuid == shader_uuid.get_value() && shader_uuid.is_valid());

        let shader = Shader::new(
            UUID::from_u128(uuid),
            name,
            bytes,
            shader_stage,
            src_code
        );

        self.loaded.shaders.insert(UUID::from_u128(uuid), shader);

        Ok(())
    }
    
    fn load_material(&mut self, material_uuid: &UUID) -> Result<(), StdError> {
        let material_path = match self.resource_paths.materials.get(material_uuid) {
            Some(path) => path,
            None => return Err("Failed to load material! (ResourceManager)".into()),
        };

        let material_node = serializer::YamlNode::deserialize_full_path(&material_path)?;
        
        let name = material_node.name;
        let mut uuid = 0;
        let mut textures = Vec::new();
        let mut shaders = Vec::new();

        for child_node in material_node.children {
            let value = child_node.value;
            
            match child_node.name.as_str() {
                UUID_YAML => uuid = value.parse::<u128>()?,
                TEXTURES_YAML => if !value.is_empty() { 
                    textures = value.split(",")
                    . map(|s| UUID::from_u128(s.trim().parse::<u128>().unwrap()))
                    .collect::<Vec<_>>()
                },
                VERTEX_SHADER_YAML | FRAGMENT_SHADER_YAML => shaders.push(UUID::from_u128(value.parse::<u128>().unwrap())),

                _ => return Err("Material configuration file has wrong format! (ResourceManager)".into())
            }
        }

        assert!(uuid == material_uuid.get_value() && material_uuid.is_valid());

        for tex_uuid in &textures {
            self.prepare_texture(tex_uuid)?;
        }
        for shader_uuid in &shaders {
            self.prepare_shader(shader_uuid)?;
        }

        let material = Material::new(
            UUID::from_u128(uuid), 
            &name,
            shaders,
            textures, 
            vec![]
        );

        self.loaded.materials.insert(UUID::from_u128(uuid), material);

        Ok(())
    }
    
    // ------------------------- Processing ------------------------- 
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
            node_type: TEXTURE_YAML.to_string(),
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
        
        let tex_resources_path = std::format!("{}{}{}", ASSET_FOLDER, ASSET_FOLDER_YAML, ASSET_FOLDER_TEXTURES);
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
            node_type: SHADER_YAML.to_string(),
            ..Default::default()
        };
        let shader_uuid = UUID::from_string(file_path.to_str().unwrap())?;
        shader_node.push(serializer::YamlNode { 
            name: UUID_YAML.to_string(), 
            value: shader_uuid.get_value().to_string(), 
            ..Default::default()
        });
        shader_node.push(serializer::YamlNode { 
            name: SHADER_STAGE_YAML.to_string(), 
            value: (shader_stage as u32).to_string(),
            ..Default::default()
        });
        shader_node.push(serializer::YamlNode { 
            name: BYTES_YAML.to_string(), 
            ..Default::default()
        });
        let src_code = crate::utils::tools::file_to_string(file_path.to_str().unwrap())?;
        shader_node.push(serializer::YamlNode { 
            name: SHADER_SRC_CODE_YAML.to_string(), 
            value: src_code,
            ..Default::default()
        });

        let path = std::format!("{}{}{}", ASSET_FOLDER, ASSET_FOLDER_YAML, ASSET_FOLDER_SHADERS);
        shader_node.serialize(&path, &shader_name)?;

        Ok(())
    }

    /// Placeholder
    fn process_mesh(&mut self, file_path: &std::path::Path) -> Result<(), StdError> {
        let mesh_name = file_path.file_stem().unwrap().to_string_lossy().to_string();
        
        let mut mesh_node = serializer::YamlNode {
            name: mesh_name.clone(),
            node_type: MESH_YAML.to_string(),
            ..Default::default()
        };

        let mesh_uuid = UUID::from_string(file_path.to_str().unwrap())?;
        mesh_node.push(serializer::YamlNode { 
            name: UUID_YAML.to_string(), 
            value: mesh_uuid.clone().get_value().to_string(), 
            ..Default::default()
        });
        
        let mesh = &tobj::load_obj(file_path, &tobj::GPU_LOAD_OPTIONS)?.0[0].mesh;

        let positions = mesh.positions.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let indices = mesh.indices.iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let tex_coords = mesh.texcoords.iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let normals = mesh.normals.iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");

        mesh_node.push(serializer::YamlNode { 
            name: POSITIONS_YAML.to_string(), 
            value: positions,
            ..Default::default()
        });

        mesh_node.push(serializer::YamlNode { 
            name: INDICES_YAML.to_string(), 
            value: indices,
            ..Default::default()
        });
        mesh_node.push(serializer::YamlNode { 
            name: TEX_COORDS_YAML.to_string(), 
            value: tex_coords,
            ..Default::default()
        });
        mesh_node.push(serializer::YamlNode { 
            name: NORMALS_YAML.to_string(), 
            value: normals,
            ..Default::default()
        });

        let mesh_resources_path = std::format!("{}{}{}", ASSET_FOLDER, ASSET_FOLDER_YAML, ASSET_FOLDER_MESHES);
        mesh_node.serialize(&mesh_resources_path, &mesh_name)?;

        Ok(())
    }

    fn process_material(&mut self, file_path: &std::path::Path) -> Result<(), StdError> {
        todo!()
    }
    
    // ------------------------- Init ------------------------- 
    fn init_folders(&self) -> Result<(), StdError> {
        for root in [CORE_ASSET_FOLDER, ASSET_FOLDER] {
            for format in [ASSET_FOLDER_YAML, ASSET_FOLDER_BINARY] {
                let meshes_string = std::format!("{}{}{}", root, format, ASSET_FOLDER_MESHES);
                let shaders_string = std::format!("{}{}{}", root, format, ASSET_FOLDER_SHADERS);
                let textures_string = std::format!("{}{}{}", root, format, ASSET_FOLDER_TEXTURES);
                let materials_string = std::format!("{}{}{}", root, format, ASSET_FOLDER_MATERIALS);

                let meshes = std::path::Path::new(&meshes_string);
                let shaders = std::path::Path::new(&shaders_string);
                let textures = std::path::Path::new(&textures_string);
                let materials = std::path::Path::new(&materials_string);

                std::fs::create_dir_all(meshes)?;
                std::fs::create_dir_all(shaders)?;
                std::fs::create_dir_all(textures)?;
                std::fs::create_dir_all(materials)?;
            }
        }
        
        Ok(())
    }
}