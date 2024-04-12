extern crate spirv_cross;
use std::{fs::File, io::Read, path::Path};
use spirv_cross::spirv;
use vulkanalia:: {
    bytecode::Bytecode, prelude::v1_2::*, vk
};
use crate::{lg_core::serializer::YamlNode, utils::tools, MyError};
use super::vk_device::VkDevice;

#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    pub shader_stage: vk::ShaderStageFlags,
    pub ds_type: vk::DescriptorType,
    pub binding: u32,
    pub set: u32,
}

pub struct Shader {
    module: vk::ShaderModule,
    pub info: vk::PipelineShaderStageCreateInfo,
    pub node: YamlNode,
    pub ast: spirv::Ast<spirv_cross::glsl::Target>,
}
impl Shader {
    // Public
    pub unsafe fn new(
        device: &VkDevice,
        path: &str,
    ) -> Result<Self, MyError> 
    {
        let device = device.get_device();

        let mut file = File::open(path).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        let name = String::from(Path::new(path)
            .file_stem()
            .and_then(|f| f.to_str())
            .unwrap());

        let ast = reflect_and_serialize(path, &name)?;
        let node = deserialize(&name)?;

        let module = Self::create_module(device, &bytes)?;
        let info = Self::get_stage_info(&module, string_to_shader_stage(&node.node_type)?);

        Ok(Self {
            module,
            info,
            node,
            ast,
        })
    }
    pub unsafe fn destroy_module(&mut self, device: &Device) {
        device.destroy_shader_module(self.module, None);
    }
    pub unsafe fn get_descriptors(&self) -> Result<Vec<ShaderDescriptor>, MyError> {
        let mut result = Vec::new();
        let ast = &self.ast;

        for main_children in &self.node.children {
            match main_children.name.as_str() {
                "UniformBuffer" => {
                    for ub_children in &main_children.children {
                        for ub_type in &ub_children.children {
                            let ds_type = if ub_children.name.contains("DYNAMIC") {
                                vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
                            } else {
                                vk::DescriptorType::UNIFORM_BUFFER
                            };
                            if ub_type.name == "Id" {
                                result.push(ShaderDescriptor {
                                    shader_stage: string_to_shader_stage(&self.node.node_type)?,
                                    ds_type,
                                    binding: ast.get_decoration(ub_type.value.parse()?, spirv::Decoration::Binding)?,
                                    set: ast.get_decoration(ub_type.value.parse()?, spirv::Decoration::DescriptorSet)?,
                                })
                            } 
                        }
                    }
                }
                "SampledImage" => {
                    for si_children in &main_children.children {
                        for si_type in &si_children.children {
                            if si_type.name == "Id" {
                                result.push(ShaderDescriptor {
                                    shader_stage: string_to_shader_stage(&self.node.node_type)?,
                                    ds_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                                    binding: ast.get_decoration(si_type.value.parse()?, spirv::Decoration::Binding)?,
                                    set: ast.get_decoration(si_type.value.parse()?, spirv::Decoration::DescriptorSet)?,
                                })
                            }
                        }
                    }
                }
                _ => ()
            }
        }
        
        Ok(result)
    }
    
    // Private
    unsafe fn create_module(
        device: &Device,
        bytecode: &[u8],
    ) -> Result<vk::ShaderModule, MyError>
    {
        let bytecode = Bytecode::new(bytecode)?;    
        
        let info = vk::ShaderModuleCreateInfo::builder()
            .code_size(bytecode.code_size())
            .code(bytecode.code());
        
        Ok(device.create_shader_module(&info, None)?)
    }
    fn get_stage_info(module: &vk::ShaderModule, shader_stage: vk::ShaderStageFlags) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(shader_stage)
            .module(*module)
            .name(b"main\0")
            .build()
    }
}

unsafe fn reflect_and_serialize(filepath: &str, name: &str) -> Result<spirv::Ast<spirv_cross::glsl::Target>, MyError> {
    
    let words = tools::shader_spirv(filepath)?;
    
    let module = spirv::Module::from_words(&words);
    let ast = spirv::Ast::<spirv_cross::glsl::Target>::parse(&module)?;

    let mut shader_node = YamlNode {
        name: name.to_string(),
        node_type: get_shader_stage(&ast)?,
        ..Default::default()
    };
    let ast_resources = ast.get_shader_resources()?;
    
    shader_node.push(serialize_resource("Input", &ast_resources.stage_inputs));
    shader_node.push(serialize_resource("Output", &ast_resources.stage_outputs));
    shader_node.push(serialize_resource("UniformBuffer", &ast_resources.uniform_buffers));
    shader_node.push(serialize_resource("SampledImage", &ast_resources.sampled_images));

    shader_node.serialize("assets/shaders/reflected/", name)?;
    
    Ok(ast)
}
fn deserialize(name: &str) -> Result<YamlNode, MyError>{
    let path = "assets/shaders/reflected/";
    YamlNode::deserialize(path, name)
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
pub fn string_to_shader_stage(name: &str) -> Result<vk::ShaderStageFlags, MyError> {
    match name {
        "Vertex" => Ok(vk::ShaderStageFlags::VERTEX),
        "Fragment" => Ok(vk::ShaderStageFlags::FRAGMENT),
        _ => Err("Invalid Shader Stage".into())
    }
}
fn get_shader_stage(ast: &spirv::Ast<spirv_cross::glsl::Target>) -> Result<String, MyError> {
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