use vulkanalia:: {
    bytecode::Bytecode, prelude::v1_0::*, vk::{self, ShaderModule}
};
use crate::MyError;
use super::vk_device::VkDevice;

pub struct Shader {
    module: vk::ShaderModule,
    pub info: vk::PipelineShaderStageCreateInfo,
}
impl Shader {
    // Public
    pub unsafe fn new(
        device: &VkDevice,
        stage: vk::ShaderStageFlags,
        bytes: &[u8],
    ) -> Result<Self, MyError> 
    {
        let device = device.get_device();

        let module = Self::create_module(device, bytes)?;
        let info = Self::get_stage_info(&module, stage);

        Ok(Self {
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
    fn get_stage_info(module: &ShaderModule, shader_stage: vk::ShaderStageFlags) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(shader_stage)
            .module(*module)
            .name(b"main\0")
            .build()
    }
}