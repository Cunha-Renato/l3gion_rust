use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};
use crate::MyError;

pub unsafe fn create_command_pool(
    device: &Device,
    queue_indice: u32,
) -> Result<vk::CommandPool, MyError>
{
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT
            | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_indice);
    
    Ok(device.create_command_pool(&info, None)?)
}