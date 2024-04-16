use std::collections::HashSet;
use sllog::*;
use vulkanalia:: {
    loader::{LibloadingLoader, LIBRARY}, prelude::v1_2::*, vk, Entry, Instance
};
use crate::MyError;
use super::vulkan::{vk_device::QueueFamilyIndices, vk_physical_device::VkPhysicalDevice, vk_swapchain::VkSwapchain};

// CONSTANTS
const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[
    vk::KHR_SWAPCHAIN_EXTENSION.name
];
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[derive(Default)]
pub struct SyncObjects {
    pub present_semaphore: vk::Semaphore,
    pub render_semaphore: vk::Semaphore,
    pub render_fence: vk::Fence,
}

#[derive(Default)]
pub struct RendererData {
    pub physical_device: VkPhysicalDevice,
    pub surface: vk::SurfaceKHR,
    pub swapchain: VkSwapchain,
    pub msaa_samples: vk::SampleCountFlags,
    pub sync_objects: Vec<SyncObjects>,
}

// Helper
pub unsafe fn create_entry() -> Result<Entry, MyError> {
    let loader = LibloadingLoader::new(LIBRARY)?;
    let entry = Entry::new(loader).map_err(|b| error!("{}", b)).unwrap();

    Ok(entry)
}
pub unsafe fn create_logical_device(
    _entry: &Entry,
    physical_device: &vk::PhysicalDevice,
    indices: &QueueFamilyIndices,
    instance: &Instance,
) -> Result<(Device, (vk::Queue, vk::Queue)), MyError>
{
    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);
    
    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();
    
    let extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();
    
    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true);
    
    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&[])
        .enabled_extension_names(&extensions)
        .enabled_features(&features);
    
    let device = instance.create_device(*physical_device, &info, None)?;
    
    let graphics_queue = device.get_device_queue(
        indices.graphics, 
        0,
    );
    let present_queue = device.get_device_queue(
        indices.present,
        0
    );
    
    Ok((device, (graphics_queue, present_queue)))
}

pub unsafe fn get_depth_format(instance: &Instance, physical_device: &vk::PhysicalDevice) -> Result<vk::Format, MyError>
{
    let canditates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];
    
    get_supported_format(
        instance, 
        physical_device, 
        canditates, 
        vk::ImageTiling::OPTIMAL, 
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
    )
}

unsafe fn get_supported_format(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    canditates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags
) -> Result<vk::Format, MyError>
{
    match canditates
        .iter()
        .cloned()
        .find(|f| {
            let properties = instance.get_physical_device_format_properties(
                *physical_device, 
                *f
            );
            
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        })
    {
        Some(result) => Ok(result),
        None => Err("Failed to find supported format!".into())
    }
}

pub unsafe fn create_sync_objects(
    device: &Device, 
    data: &mut RendererData,
) -> Result<(), MyError>
{
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);
    
    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        data.sync_objects.push(SyncObjects {
            render_fence: device.create_fence(&fence_info, None)?,
            present_semaphore: device.create_semaphore(&semaphore_info, None)?,
            render_semaphore: device.create_semaphore(&semaphore_info, None)?,
        });
    }

    Ok(())
}