use std::{fs::File, io::{Read, Write}};

use serializer::YamlNode;
use shaderc::{Compiler, ShaderKind};
use crate::StdError;
use spirv_cross::{spirv::Type, *};
use vulkanalia::vk::{self, DeviceV1_0, HasBuilder};

const STORE_PATH: &str = "resources/shaders/reflected";

#[derive(Debug, Clone, Copy)]
pub enum ShaderPrimitiveTypes {
    None,
    Unit,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    pub shader_stage: vk::ShaderStageFlags,
    pub ds_type: vk::DescriptorType,
    pub binding: u32,
    pub set: u32,
}

pub struct Shader {
    pub module: vk::ShaderModule,
    pub info: vk::PipelineShaderStageCreateInfo,
    pub name: String,
    pub descriptors: Vec<ShaderDescriptor>,
    pub bytes: Vec<u8>,
}
impl Shader {
    pub unsafe fn new(
        device: &vulkanalia::Device,
        path: &std::path::Path,
    ) -> Result<Self, StdError>
    {
        let name = if let Some(stem) = path.file_stem() {
            String::from(match stem.to_str() {
                Some(name) => name,
                None => return Err("Could not find shader name in path! (Shader)".into()),
            })
        } else {
            return Err("Could not find shader name in path! (Shader)".into())
        };

        let mut bytes = Vec::new();
        File::open(path)?.read_to_end(&mut bytes)?;

        let module = spirv::Module::from_words(words_from_bytes(&bytes));

        let (descriptors, stage) = serialize_and_get_descriptors(&module, &name)?;

        let bytecode = vulkanalia::bytecode::Bytecode::new(&bytes)?;
        let info = vk::ShaderModuleCreateInfo::builder()
            .code_size(bytecode.code_size())
            .code(bytecode.code());
        
        let module = device.create_shader_module(&info, None)?;
        
        let info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(string_to_shader_stage(&stage)?)
            .module(module)
            .name(b"main\0")
            .build();

        Ok(Self {
            module,
            info,
            name,
            descriptors,
            bytes
        })
    }
    pub unsafe fn destroy_module(&self, device: &vulkanalia::Device) {
        device.destroy_shader_module(self.module, None);
    }
}
unsafe fn serialize_and_get_descriptors(module: &spirv::Module, name: &str) -> Result<(Vec<ShaderDescriptor>, String), StdError> {
    let mut ast = spirv::Ast::<spirv_cross::glsl::Target>::parse(module)?;

    let mut shader_node = YamlNode {
        name: name.to_string(),
        node_type: get_shader_stage(&ast)?,
        ..Default::default()
    };
    let ast_resources = ast.get_shader_resources()?;
    
    shader_node.push(serialize_resource("Input", &ast_resources.stage_inputs));
    shader_node.push(serialize_resource("Output", &ast_resources.stage_outputs));
    shader_node.push(serialize_resource("UniformBuffer", &ast_resources.uniform_buffers));
    shader_node.push(serialize_resource("StorageBuffer", &ast_resources.storage_buffers));
    shader_node.push(serialize_resource("CombinedImageSampler", &ast_resources.sampled_images));
    shader_node.push(serialize_resource("StorageImage", &ast_resources.storage_images));
    
    let stage = shader_node.node_type.clone();
    Ok((clean_serialization(&mut ast, shader_node, STORE_PATH, name)?, stage))
}
fn clean_serialization(ast: &mut spirv::Ast<spirv_cross::glsl::Target>, node: YamlNode, path: &str, name: &str) -> Result<Vec<ShaderDescriptor>, StdError> {
    let mut new_node = YamlNode {
        name: node.name,
        node_type: node.node_type,
        ..Default::default()
    };
    
    let mut result = Vec::new();
    
    for main_children in &node.children {
        for ds_children in &main_children.children {
            // Serializing
            let mut new_main_children = YamlNode::default();
            new_main_children.name = ds_children.name.clone();
            let vulkan_type = match get_descriptor_type(&main_children.name, ds_children.name.contains("DYNAMIC")) {
                Ok(nd) => nd,
                Err(_) => continue
            };
            new_main_children.node_type = vulkan_type.clone().as_raw().to_string();

            // Descriptors and Serializing
            let mut ds_id = 0;
            for ds_type in &ds_children.children {
                match ds_type.name.as_str() {
                    "Id" => {
                        result.push(ShaderDescriptor {
                            shader_stage: string_to_shader_stage(&new_node.node_type)?,
                            ds_type: vulkan_type,
                            binding: ast.get_decoration(ds_type.value.parse()?, spirv::Decoration::Binding)?,
                            set: ast.get_decoration(ds_type.value.parse()?, spirv::Decoration::DescriptorSet)?,
                        });
                    }
                    "BaseTypeId" => ds_id = ds_type.value.parse()?,
                    _ => (),
                } 
            }
            
            // If it is a struct then serialize it's members
            match ast.get_type(ds_id) {
                Ok(Type::Struct { member_types, .. }) => {
                    for (i, ty) in member_types.iter().enumerate() {
                        let mut new_types = YamlNode::default();
                        new_types.name = ast.get_member_name(ds_id, i as u32)?;
                        new_types.node_type = convert_types(ast, ty)?;

                        new_main_children.push(new_types);
                    }
                }
                _ => (),
            }
            new_node.push(new_main_children);
        }
    }
    
    new_node.serialize(&path, name)?;

    Ok(result)
}
fn serialize_resource(name: &str, resources: &Vec<spirv::Resource>) -> YamlNode {
    let mut node = YamlNode {
        name: name.to_string(),
        ..Default::default()
    };

    resources
        .iter()
        .for_each(|res| {
            node.push(YamlNode {
                name: res.name.clone(),
                children: vec![
                    YamlNode {
                        name: "Id".to_string(),
                        value: res.id.to_string(),
                        ..Default::default()
                    },
                    YamlNode {
                        name: "TypeId".to_string(),
                        value: res.type_id.to_string(),
                        ..Default::default()
                    },
                    YamlNode {
                        name: "BaseTypeId".to_string(),
                        value: res.base_type_id.to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            });
        });
    
    node
}
fn get_descriptor_type(ds_type: &str, dynamic: bool) -> Result<vk::DescriptorType, StdError> {
    Ok(match (ds_type, dynamic) {
        ("UniformBuffer", false) => vk::DescriptorType::UNIFORM_BUFFER,
        ("UniformBuffer", true) => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        ("StorageBuffer", false) => vk::DescriptorType::STORAGE_BUFFER,
        ("StorageBuffer", true) => vk::DescriptorType::STORAGE_BUFFER_DYNAMIC,
        ("CombinedImageSampler", _) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        ("StorageImage", _) => vk::DescriptorType::STORAGE_IMAGE,
        
        (_, _) => return Err(format!("Descriptor type of {} not suported! (shader)", ds_type).into())        
    })
}
fn get_shader_stage(ast: &spirv::Ast<spirv_cross::glsl::Target>) -> Result<String, StdError> {
    // Now I dont use more entry points so this is fine
    let entries = ast.get_entry_points()?;
    
    for entry in &entries {
        if entry.name == "main".to_string() {
            return Ok(match entry.execution_model {
                spirv::ExecutionModel::Vertex => "Vertex",
                spirv::ExecutionModel::TessellationControl => "TessellationControl",
                spirv::ExecutionModel::TessellationEvaluation => "TessellationEvaluation",
                spirv::ExecutionModel::Geometry => "Geometry",
                spirv::ExecutionModel::Fragment => "Fragment",
                spirv::ExecutionModel::GlCompute => "GlCompute",
                spirv::ExecutionModel::Kernel => "Kernel",
            }.to_string());
        }
    }
    
    Err("No valid entry point (Shader)".into())
}
fn get_type(vec_size: u32, columns: u32) -> ShaderPrimitiveTypes {
    match (vec_size, columns) {
        (4, 4) => ShaderPrimitiveTypes::Mat4,
        (3, 3) => ShaderPrimitiveTypes::Mat3,
        (2, 2) => ShaderPrimitiveTypes::Mat2,
        (1, _) => ShaderPrimitiveTypes::Unit,
        (4, 1) => ShaderPrimitiveTypes::Vec4,
        (3, 1) => ShaderPrimitiveTypes::Vec3,
        (2, 1) => ShaderPrimitiveTypes::Vec2,
        _ => ShaderPrimitiveTypes::None
    }
}
fn convert_types(ast: &spirv::Ast<glsl::Target>, ty: &u32) -> Result<String, StdError> {
    let var_type = match ast.get_type(*ty)? {
        Type::Int { vecsize, columns, .. } => {
            match get_type(vecsize, columns) {
                ShaderPrimitiveTypes::None => "Error",
                ShaderPrimitiveTypes::Unit => "i32",
                ShaderPrimitiveTypes::Vec2 => "nalgebra_glm::IVec2",
                ShaderPrimitiveTypes::Vec3 => "nalgebra_glm::IVec3",
                ShaderPrimitiveTypes::Vec4 => "nalgebra_glm::IVec4",
                ShaderPrimitiveTypes::Mat2 => "nalgebra_glm::Mat2",
                ShaderPrimitiveTypes::Mat3 => "nalgebra_glm::Mat3",
                ShaderPrimitiveTypes::Mat4 => "nalgebra_glm::Mat4",
            }
        },
        Type::UInt { vecsize, columns, .. } => {
            match get_type(vecsize, columns) {
                ShaderPrimitiveTypes::None => "Error",
                ShaderPrimitiveTypes::Unit => "u32",
                ShaderPrimitiveTypes::Vec2 => "nalgebra_glm::UVec2",
                ShaderPrimitiveTypes::Vec3 => "nalgebra_glm::UVec3",
                ShaderPrimitiveTypes::Vec4 => "nalgebra_glm::UVec4",
                ShaderPrimitiveTypes::Mat2 => "nalgebra_glm::Mat2",
                ShaderPrimitiveTypes::Mat3 => "nalgebra_glm::Mat3",
                ShaderPrimitiveTypes::Mat4 => "nalgebra_glm::Mat4",
            }
        },
        Type::Int64 { vecsize, .. } => {
            match get_type(vecsize, 0) {
                ShaderPrimitiveTypes::None => "Error",
                ShaderPrimitiveTypes::Unit => "i64",
                ShaderPrimitiveTypes::Vec2 => "nalgebra_glm::I64Vec2",
                ShaderPrimitiveTypes::Vec3 => "nalgebra_glm::I64Vec3",
                ShaderPrimitiveTypes::Vec4 => "nalgebra_glm::I64Vec4",
                ShaderPrimitiveTypes::Mat2 => "nalgebra_glm::DMat2",
                ShaderPrimitiveTypes::Mat3 => "nalgebra_glm::DMat3",
                ShaderPrimitiveTypes::Mat4 => "nalgebra_glm::DMat4",
            }
        },
        Type::UInt64 { vecsize, .. } => {
            match get_type(vecsize, 0) {
                ShaderPrimitiveTypes::None => "Error",
                ShaderPrimitiveTypes::Unit => "u64",
                ShaderPrimitiveTypes::Vec2 => "nalgebra_glm::U64Vec2",
                ShaderPrimitiveTypes::Vec3 => "nalgebra_glm::U64Vec3",
                ShaderPrimitiveTypes::Vec4 => "nalgebra_glm::U64Vec3",
                ShaderPrimitiveTypes::Mat2 => "nalgebra_glm::DMat2",
                ShaderPrimitiveTypes::Mat3 => "nalgebra_glm::DMat3",
                ShaderPrimitiveTypes::Mat4 => "nalgebra_glm::DMat4",
            }
        },
        Type::Float { vecsize, columns, .. } => {
            match get_type(vecsize, columns) {
                ShaderPrimitiveTypes::None => "Error",
                ShaderPrimitiveTypes::Unit => "f32",
                ShaderPrimitiveTypes::Vec2 => "nalgebra_glm::Vec2",
                ShaderPrimitiveTypes::Vec3 => "nalgebra_glm::Vec3",
                ShaderPrimitiveTypes::Vec4 => "nalgebra_glm::Vec4",
                ShaderPrimitiveTypes::Mat2 => "nalgebra_glm::Mat2",
                ShaderPrimitiveTypes::Mat3 => "nalgebra_glm::Mat3",
                ShaderPrimitiveTypes::Mat4 => "nalgebra_glm::Mat4",
            }
        },
        Type::Double { vecsize, columns, .. } => {
            match get_type(vecsize, columns) {
                ShaderPrimitiveTypes::None => "Error",
                ShaderPrimitiveTypes::Unit => "f64",
                ShaderPrimitiveTypes::Vec2 => "nalgebra_glm::DVec2",
                ShaderPrimitiveTypes::Vec3 => "nalgebra_glm::DVec3",
                ShaderPrimitiveTypes::Vec4 => "nalgebra_glm::DVec4",
                ShaderPrimitiveTypes::Mat2 => "nalgebra_glm::DMat2x2",
                ShaderPrimitiveTypes::Mat3 => "nalgebra_glm::DMat3",
                ShaderPrimitiveTypes::Mat4 => "nalgebra_glm::DMat4",
            }
        },
        _ => "Error"
    };
    
    Ok(var_type.to_string())
}
pub fn string_to_shader_stage(name: &str) -> Result<vk::ShaderStageFlags, StdError> {
    match name {
        "Vertex" => Ok(vk::ShaderStageFlags::VERTEX),
        "Fragment" => Ok(vk::ShaderStageFlags::FRAGMENT),
        _ => Err("Invalid Shader Stage".into())
    }
}

#[allow(clippy::cast_ptr_alignment)]
pub fn words_from_bytes(buf: &[u8]) -> &[u32] {
    unsafe {
        std::slice::from_raw_parts(
            buf.as_ptr() as *const u32,
            buf.len() / std::mem::size_of::<u32>(),
        )
    }
}

// Compilation
fn compilation_get_shader_stage(extension: &str) -> Result<ShaderKind, StdError> {
    Ok(match extension {
        "vert" => ShaderKind::Vertex,
        "frag" => ShaderKind::Fragment,
        _ => return Err("Shader extension not suported! try (.vert/.frag)".into())
    })
}
pub fn read_folder(src_folder: &str, dst_folder: &str) -> Result<(), StdError> {
    let files: std::fs::ReadDir = std::fs::read_dir(src_folder)?;
    for file in files {
        let file = file?.path();

        if !file.is_file() { continue }

        let file_name = file.file_stem().unwrap().to_str().unwrap();
        let file_extension = match file.extension() {
            Some(ext) => ext.to_str().unwrap(),
            None => continue
        };
        let dst_path = format!("{}/{}.spv", dst_folder, file_name);

        if let Ok(shader_stage) = compilation_get_shader_stage(file_extension) {
            compile(file.to_str().unwrap(), &dst_path, shader_stage)?;
        }
    }
    
    Ok(())
}
fn compile(src_path: &str, dst_path: &str, shader_stage: ShaderKind) -> Result<(), StdError> {
    // Reading
    let mut src_file = File::open(src_path)?;
    let mut src_code = String::new();
    src_file.read_to_string(&mut src_code)?;
    
    // Compiling
    let compiler = match Compiler::new() {
        Some(c) => c,
        None => return Err("Failed to create shader compiler! (shader)".into()),
    };    
    let binary = compiler.compile_into_spirv(
        &src_code, 
        shader_stage, 
        src_path, 
        "main", 
        None
    )?;

    // Writing
    let mut dst_file = File::create(dst_path)?;
    dst_file.write_all(binary.as_binary_u8())?;

    Ok(())
}