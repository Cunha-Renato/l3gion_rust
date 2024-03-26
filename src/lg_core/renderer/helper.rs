use std::{collections::HashSet, ffi::CStr, os::raw::c_void};

use sllog::*;
use winit::window::Window;
use vulkanalia:: {
    loader::{LibloadingLoader, LIBRARY}, prelude::v1_0::*, vk, window as vk_window, Entry, Instance, Version
};
use crate::{lg_core::renderer::vulkan::queue_family::QueueFamilyIndices, MyError};

use super::vulkan::{command_buffer::VkCommandPool, image::ImageData, swapchain::VkSwapchain};

// CONSTANTS
const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[
    vk::KHR_SWAPCHAIN_EXTENSION.name
];
const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[derive(Default)]
pub struct RendererData {
    pub physical_device: vk::PhysicalDevice,
    pub surface: vk::SurfaceKHR,
    pub queue_indices: QueueFamilyIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub swapchain: VkSwapchain,
    pub msaa_samples: vk::SampleCountFlags,
    pub render_pass: vk::RenderPass,
    pub command_pool: VkCommandPool,
    pub color_image: ImageData,
    pub depth_image: ImageData,
    pub framebuffers: Vec<vk::Framebuffer>,
}
impl RendererData {
    
}

// Helper
pub unsafe fn create_entry(window: &Window) -> Result<Entry, MyError> {
    let loader = LibloadingLoader::new(LIBRARY)?;
    let entry = Entry::new(loader).map_err(|b| error!("{}", b)).unwrap();

    Ok(entry)
}
pub unsafe fn create_instance(
        window: &Window,
        entry: &Entry,
) -> Result<Instance, MyError> 
{
    let application_info = vk::ApplicationInfo::builder()
        .application_version(vk::make_version(1, 0, 0))
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));
    
    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();
    
    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err("Validation layer requested but not supported!".into());
    }
    
    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    }
    else {
        Vec::new()
    };

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();
    
    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    // Required by Vulkan SDK on macOS since 1.3.216
    let flags = if 
        cfg!(target_os = "macos") &&
        entry.version()? >= PORTABILITY_MACOS_VERSION
    {
        info!("Enabling extensions for macOS portablilty!");
        extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
        extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
        
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } 
    else {
        vk::InstanceCreateFlags::empty()
    };
    
    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .flags(flags);
        
    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));
    
    if VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }
    
    let instance = entry.create_instance(&info, None)?;
    Ok(instance)
}

pub unsafe fn create_logical_device(
    entry: &Entry,
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
    
    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    }
    else {
        vec![]
    };
    
    let mut extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();
    
    // Required by Vulkan SDK on macOS since 1.3.216
    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }
    
    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true);
    
    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
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

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 
{
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();
    
    match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => 
            error!("({:?}) {}", type_, message),

        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => 
            warn!("({:?}) {}", type_, message),

        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => 
            info!("({:?}) {}", type_, message),

        _ => trace!("({:?}) {}", type_, message)
    }
    
    vk::FALSE
}