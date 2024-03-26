use std::collections::HashSet;

use crate::{lg_core::renderer::vulkan::{queue_family::QueueFamilyIndices, swapchain::SwapchainSupport}, MyError};
use sllog::*;
use vulkanalia::{
    vk::{self, InstanceV1_0},
    Instance,
};

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[
    vk::KHR_SWAPCHAIN_EXTENSION.name
];

pub unsafe fn pick_physical_device(
    instance: &Instance, 
    surface: &vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, QueueFamilyIndices), MyError> 
{
    if let Some((indices, physical_device)) = instance.enumerate_physical_devices()?
        .iter()
        .map(|pd| 
            (check_physical_device(
                instance,
                surface,
                *pd
            ), pd)
        )
        .filter(|(val, _)| 
            val.is_ok()
        )
        .min_by_key(|(val, _)| {
            val.as_ref().unwrap().0
        })
    {
        let properties = instance.get_physical_device_properties(*physical_device);
        warn!("Physical Selected:\n  Name: {}\n  Type: {:?}", properties.device_name, properties.device_type);
        let physical_device = *physical_device;
        // data.msaa_samples = get_max_msaa_samples(instance, data);

        return Ok((physical_device, indices.unwrap().1));
    }

    Err("Failed to find any suitable Physical Device!".into())
}

unsafe fn check_physical_device(
    instance: &Instance,
    surface: &vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<(usize, QueueFamilyIndices), MyError>
{
    let indices = QueueFamilyIndices::get(
        instance,
        surface,
        physical_device
    )?;
    check_physical_device_extensions(
        instance,
        physical_device
    )?;

    let properties = instance
        .get_physical_device_properties(physical_device);
    
    let features = instance
        .get_physical_device_features(physical_device);
    

    let result = match properties.device_type {
        vk::PhysicalDeviceType::DISCRETE_GPU => 0,
        vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
        vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
        vk::PhysicalDeviceType::CPU => 3,
        _ => 4,
    };
    
    if features.geometry_shader != vk::TRUE {
        return Err("Physical Device not suitable! (Missing Geometry Shader Support)".into());
    }
    else if features.sampler_anisotropy != vk::TRUE {
        return Err("No sampler anisotropy!".into());
    }

    let support = SwapchainSupport::get(instance, surface, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err("Insufficient swapchain support!".into());
    }
    
    info!("Checking Physical Device:\n  Name: {}\n  Type: {:?}", properties.device_name, properties.device_type);

    if result < 4 { return Ok((result, indices)); }
    
    Err("Could not find suitable device!".into())
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<(), MyError>
{
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>(); 
    
    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        return Ok(());
    }

    Err("Missing required device extensions!".into())
}