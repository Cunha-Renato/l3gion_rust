// VULKANALIA
use vulkanalia::{
    prelude::v1_0::*, vk::{
        KhrSurfaceExtension, 
        SurfaceKHR,
    }, 
};

use crate::MyError;

#[derive(Default, Debug, Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}
impl QueueFamilyIndices {
    pub unsafe fn get(
        instance: &Instance,
        surface: &SurfaceKHR,
        physical_device: vk::PhysicalDevice
    ) -> Result<Self, MyError>
    {
        let properties = instance
            .get_physical_device_queue_family_properties(physical_device);
        
        let graphics = properties
            .iter()
            .position(|p| 
                p.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            )
            .map(|i| i as u32);

        let present = properties
            .iter()
            .enumerate()
            .position(|(i, _)| 
                instance.get_physical_device_surface_support_khr(
                    physical_device, 
                    i as u32,
                    *surface
                ).is_ok()
            )
            .map(|i| i as u32);

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self { graphics, present })
        }
        else {
            Err("Missing required queue families!".into())
        } 
    }
}