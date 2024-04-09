extern crate spirv_cross;
use std::{fs::File, io::Read, path::Path};
use spirv_cross::spirv;
use vulkanalia:: {
    bytecode::Bytecode, prelude::v1_2::*, vk
};
use crate::{lg_core::serializer::YamlNode, utils::tools, MyError};
use super::vk_device::VkDevice;

pub struct Shader {
    pub name: String,
    module: vk::ShaderModule,
    pub info: vk::PipelineShaderStageCreateInfo,
}
impl Shader {
    // Public
    pub unsafe fn new(
        device: &VkDevice,
        stage: vk::ShaderStageFlags,
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

        let module = Self::create_module(device, &bytes)?;
        let info = Self::get_stage_info(&module, stage);
        reflect_and_serialize(path, &name)?;

        Ok(Self {
            name,
            module,
            info,
        })
    }
    pub unsafe fn destroy_module(&mut self, device: &Device) {
        device.destroy_shader_module(self.module, None);
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

unsafe fn reflect_and_serialize(filepath: &str, name: &str) -> Result<(), MyError> {
    
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
    
    Ok(())
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