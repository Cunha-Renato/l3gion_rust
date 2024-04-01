use std::{collections::HashSet, ffi::CStr, os::raw::c_void};

use sllog::*;
use vulkanalia:: {
    prelude::v1_2::*, vk::{self, ExtDebugUtilsExtension}, window as vk_window, Entry, Instance, Version
};
use winit::window::Window;

use crate::MyError;

const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

pub struct VkInstance {
    instance: Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}
impl VkInstance {
    pub unsafe fn new(
        entry: &Entry,
        window: &Window,
    ) -> Result<Self, MyError> 
    {
        let application_info = vk::ApplicationInfo::builder()
        .application_version(vk::make_version(1, 2, 0))
        .engine_version(vk::make_version(1, 2, 0))
        .api_version(vk::make_version(1, 2, 0));
    
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
        
        let mut messenger = vk::DebugUtilsMessengerEXT::default();
        if VALIDATION_ENABLED {
            messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
        }

        Ok(Self {
            instance,
            messenger,
        })
    }
    pub fn get_instance(&self) -> &Instance {
        &self.instance
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