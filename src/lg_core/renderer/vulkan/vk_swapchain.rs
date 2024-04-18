use winit::window::Window;
use vulkanalia:: {
    prelude::v1_2::*, 
    vk::{KhrSurfaceExtension, KhrSwapchainExtension, SurfaceKHR},
};
use crate::StdError;

use super::{vk_device::{QueueFamilyIndices, VkDevice}, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice};
#[derive(Clone, Debug)]
pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}
impl SwapchainSupport {
    pub unsafe fn get(
        instance: &Instance,
        surface: &SurfaceKHR,
        physical_device: vk::PhysicalDevice,     
    ) -> Result<Self, StdError> 
    {
        Ok(Self {
            capabilities: instance
                .get_physical_device_surface_capabilities_khr(
                    physical_device,
                    *surface
                )?,
            formats: instance
                .get_physical_device_surface_formats_khr(
                    physical_device,
                    *surface
                )?,
            present_modes: instance
                .get_physical_device_surface_present_modes_khr(
                    physical_device,
                    *surface
                )?
        })
    }
}

#[derive(Default)]
pub struct VkSwapchain {
    pub swapchain: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,   
    pub views: Vec<vk::ImageView>,
}
impl VkSwapchain {
    pub unsafe fn new(
        window: &Window,
        instance: &VkInstance,
        surface: &SurfaceKHR,
        physical_device: &VkPhysicalDevice,
        device: &VkDevice,
    ) -> Result<Self, StdError> 
    {
        let instance = instance.get_instance();
        let device = device.get_device();
        let physical_device = physical_device.get_device();

        let indices = QueueFamilyIndices::get(
            instance,
            surface,
            *physical_device
        )?;
        let support = SwapchainSupport::get(
            instance,
            surface,
            *physical_device
        )?;
        
        let surface_format = Self::get_format(&support.formats);
        let present_mode = Self::get_present_mode(&support.present_modes);
        let extent = Self::get_extent(window, support.capabilities);
        
        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0 && image_count > support.capabilities.max_image_count {
            image_count = support.capabilities.max_image_count;
        }
        
        let mut queue_family_indices = vec![];
        let image_sharing_mode = if indices.graphics != indices.present {
            queue_family_indices.push(indices.graphics);
            queue_family_indices.push(indices.present);
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };
        
        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());
        
        let swapchain = device.create_swapchain_khr(&info, None)?;
        
        let images = device.get_swapchain_images_khr(swapchain)?;
        
        let views = images
        .iter()
        .map(|image| {
            // View
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
        
            let info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::_2D)
                .format(vk::Format::R8G8B8A8_SRGB)
                .subresource_range(subresource_range);
        
            device.create_image_view(&info, None).unwrap()
        })
        .collect();
        
        Ok(Self {
            swapchain,
            format: surface_format.format,
            extent,
            images,
            views,
        })
    }
    
    // Helper
    fn get_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        formats
            .iter()
            .cloned()
            .find(|f| {
                f.format == vk::Format::R8G8B8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
        .unwrap_or_else(|| formats[0])
    }
    fn get_present_mode(
        present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        present_modes
            .iter()
            .cloned()
            .find(|m| *m == vk::PresentModeKHR::IMMEDIATE)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }
    fn get_extent(
        window: &Window,
        capabilities: vk::SurfaceCapabilitiesKHR
    ) -> vk::Extent2D
    {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        }
        else {
            let size = window.inner_size();
            let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));
            
            vk::Extent2D::builder()
                .width(clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                    size.width
                ))
                .height(clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                    size.height
                ))
                .build()
        }
    }
}