use std::collections::HashSet;
// VULKANALIA
use vulkanalia::{
    prelude::v1_2::*, vk::{
        KhrSurfaceExtension, 
        SurfaceKHR,
    }, 
};
use crate::MyError;
use super::{vk_physical_device::VkPhysicalDevice, vk_instance::VkInstance, vk_queue::VkQueue};

pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[
    vk::KHR_SWAPCHAIN_EXTENSION.name
];

#[derive(Default, Debug, Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
    pub compute: u32,
    pub transfer: u32,
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
        
        let compute = properties
            .iter()
            .position(|p|
                p.queue_flags.contains(vk::QueueFlags::COMPUTE)
            )
            .map(|i| i as u32);
        
        let transfer = properties
            .iter()
            .position(|p| 
                p.queue_flags.contains(vk::QueueFlags::TRANSFER)
            )
            .map(|i| i as u32);

        if let (Some(graphics), Some(present), Some(compute), Some(transfer)) = (graphics, present, compute, transfer) {
            Ok(Self { graphics, present, compute, transfer })
        }
        else {
            Err("Missing required queue families!".into())
        } 
    }
}

pub enum VkQueueFamily {
    GRAPHICS,
    PRESENT,
    COMPUTE,
    TRANSFER,
}
pub struct Queues {
    graphics: VkQueue,
    present: VkQueue,
    compute: VkQueue,
    transfer: VkQueue,
}

pub struct VkDevice {
    device: Device,
    pub queues: Queues,    
}
impl VkDevice {
    pub unsafe fn new(
        entry: &Entry,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        surface: &SurfaceKHR,
    ) -> Result<Self, MyError>
    {
        let device  = create_logical_device(
            entry, 
            instance, 
            physical_device, 
            surface
        )?;
        
        let queue_indices = QueueFamilyIndices::get(instance.get_instance(), surface, *physical_device.get_device())?;
        let queues = Queues {
            graphics: VkQueue::new(&device, queue_indices.graphics)?,
            present: VkQueue::new(&device, queue_indices.compute)?,
            compute: VkQueue::new(&device, queue_indices.compute)?,
            transfer: VkQueue::new(&device, queue_indices.transfer)?,
        };
        
        Ok(Self {
            device,
            queues   
        })
    }
    pub fn get_device(&self) -> &Device {
        &self.device
    }
    pub fn get_graphics_queue(&self) -> &VkQueue {
        &self.queues.graphics
    }
    pub fn get_present_queue(&self) -> &VkQueue {
        &self.queues.present
    }
    pub fn get_compute_queue(&self) -> &VkQueue {
        &self.queues.compute
    }
    pub fn get_transfer_queue(&self) -> &VkQueue {
        &self.queues.transfer
    }
    
    pub unsafe fn destroy_command_pools(&self) {
        self.queues.graphics.destroy_pool(self);
        self.queues.present.destroy_pool(self);
        self.queues.compute.destroy_pool(self);
        self.queues.transfer.destroy_pool(self);
    }
    pub unsafe fn free_command_buffers(&self) {
        self.queues.graphics.free_command_buffers(&self);
        self.queues.present.free_command_buffers(&self);
        self.queues.compute.free_command_buffers(&self);
        self.queues.transfer.free_command_buffers(&self);
    }
    
    pub unsafe fn allocate_command_buffers(&mut self, queue_family: VkQueueFamily, lenght: u32) -> Result<(), MyError> {
        match queue_family {
            VkQueueFamily::GRAPHICS => self.queues.graphics.allocate_command_buffers(&self.device, lenght),
            VkQueueFamily::PRESENT => self.queues.present.allocate_command_buffers(&self.device, lenght),
            VkQueueFamily::COMPUTE => self.queues.compute.allocate_command_buffers(&self.device, lenght),
            VkQueueFamily::TRANSFER => self.queues.transfer.allocate_command_buffers(&self.device, lenght),
        }
    }
}

unsafe fn create_logical_device(
    entry: &Entry,
    instance: &VkInstance,
    physical_device: &VkPhysicalDevice,
    surface: &SurfaceKHR
) -> Result<Device, MyError>
{
    let indices = QueueFamilyIndices::get(instance.get_instance(), surface, *physical_device.get_device())?;
    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);
    unique_indices.insert(indices.transfer);
    
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
    
    let extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();
    
    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true);
    
    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);
    
    let device = instance.get_instance().create_device(*physical_device.get_device(), &info, None)?;
    
    Ok(device)
}

