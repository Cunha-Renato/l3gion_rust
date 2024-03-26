use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};
use crate::MyError;

pub unsafe fn get_memory_type_index(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32, MyError>
{
    let memory = instance. get_physical_device_memory_properties(*physical_device);
    
    if let Some(result) = (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            
            suitable && memory_type.property_flags.contains(properties)
        })
    {
        return Ok(result);
    }

    Err("Failed to find suitable memory type!".into())
}