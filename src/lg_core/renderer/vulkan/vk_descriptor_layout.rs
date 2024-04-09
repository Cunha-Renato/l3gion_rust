use vulkanalia::{
    prelude::v1_2::*, 
    vk,
};

use crate::MyError;

use super::vk_device::VkDevice;

pub struct VkDescriptorLayout {
    layouts: Vec<vk::DescriptorSetLayout>,
}
impl VkDescriptorLayout {
    pub fn new(device: &VkDevice, bindings: Vec<Vec<vk::DescriptorSetLayoutBinding>>) -> Result<Self, MyError> {
        let mut result = Vec::new();
        for bindings in &bindings {
            let info = vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(bindings.as_slice());
            
            result.push(device.get_device().create)
        }
    }
}