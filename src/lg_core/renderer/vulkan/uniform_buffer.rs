use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};

use crate::MyError;
pub struct UniformBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}
impl UniformBuffer {
    pub unsafe fn new() -> Result<Self, MyError> {
        
    }
}