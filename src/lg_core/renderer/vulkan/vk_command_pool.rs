use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};
use crate::StdError;

pub unsafe fn create_command_pool(
    device: &Device,
    queue_indice: u32,
) -> Result<vk::CommandPool, StdError>
{
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT
            | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_indice);
    
    Ok(device.create_command_pool(&info, None)?)
}