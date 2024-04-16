#![allow(non_camel_case_types)]

pub mod camera;
pub mod vertex;
pub mod texture;
pub mod vulkan;
pub mod object;
pub mod object_storage;
pub mod helper;
pub mod uniform_buffer_object;

use std::borrow::BorrowMut;
use std::{borrow::Borrow, mem::size_of};
use std::ptr::copy_nonoverlapping as memcpy;

use nalgebra_glm as glm;
use vulkanalia::vk::KhrSurfaceExtension;
use winit::window::Window;
use vulkanalia::{
    vk::{
        self, DeviceV1_0, Handle, HasBuilder, KhrSwapchainExtension
    }, 
    window as vk_window, 
    Entry, 
};
use crate::MyError;
use helper::RendererData;
use self::uniform_buffer_object::ModelUBOId;
use self::{uniform_buffer_object::{ModelUBO, ViewProjUBO}, vulkan::vk_memory_manager::VkMemoryManager};
use self::camera::Camera;
use self::object_storage::ObjectStorage;
use self::vulkan::vk_device::{VkDevice, VkQueueFamily};
use self::vulkan::vk_instance::VkInstance;
use self::vulkan::vk_physical_device::VkPhysicalDevice;
use self::{object::Object, vertex::Vertex, vulkan::{vk_pipeline::*, vk_swapchain::VkSwapchain}};
use super::{lg_types::reference::Rfc, uuid::UUID};

const MAX_PIPELINES: u32 = 21;

enum Pipelines {
    DEFAULT,
    OBJECT_PICKER,
}

pub struct Renderer {
    window: Rfc<Window>,
    _entry: Entry,
    instance: VkInstance,
    device: Rfc<VkDevice>,
    memory_manager: Rfc<VkMemoryManager>,
    data: RendererData,
    pipelines: Vec<VkPipeline>,
    objects: ObjectStorage<Vertex>,
    frame_active_objects: Vec<UUID>,
    frame: usize,
    pub resized: bool,
    camera: Rfc<Camera>,
}
impl Renderer {
    pub unsafe fn init(window: Window) -> Result<(Self, Rfc<Window>), MyError> {
        let mut data = RendererData::default();
        let entry = helper::create_entry()?; 
        let instance = VkInstance::new(&entry, &window)?;

        data.msaa_samples = vk::SampleCountFlags::_8;
        data.surface = vk_window::create_surface(instance.get_instance(), &window, &window)?;
        data.physical_device = VkPhysicalDevice::new(&instance, &data.surface)?;

        let device = Rfc::new(VkDevice::new(
            &instance, 
            &data.physical_device.borrow(), 
            &data.surface
        )?);

        let memory_manager = Rfc::new(VkMemoryManager::new(device.clone(), &instance, &data.physical_device.borrow())?);

        data.swapchain = VkSwapchain::new(
            &window, 
            &instance.borrow(), 
            &data.surface, 
            &data.physical_device.borrow(), 
            &device.borrow()
        )?;

        device.borrow_mut().allocate_command_buffers(VkQueueFamily::GRAPHICS, data.swapchain.images.len() as u32 + MAX_PIPELINES)?;
        device.borrow_mut().allocate_command_buffers(VkQueueFamily::PRESENT, data.swapchain.images.len() as u32 + MAX_PIPELINES)?;
        device.borrow_mut().allocate_command_buffers(VkQueueFamily::TRANSFER, data.swapchain.images.len() as u32 + MAX_PIPELINES)?;
        
        let pp1 = VkPipeline::get_2d(
            device.clone(), 
            &instance, 
            &data.physical_device, 
            &data.swapchain, 
            memory_manager.clone(), 
            data.msaa_samples
        )?;
        let pp2 = VkPipeline::obj_picker(
            device.clone(), 
            &instance, 
            &data.physical_device, 
            &data.swapchain, 
            memory_manager.clone(), 
        )?;
        let pipelines = vec![pp1, pp2];

        let window = Rfc::new(window);

        helper::create_sync_objects(device.borrow().get_device(), &mut data)?;


        Ok((Self {
            window: window.clone(),
            _entry: entry,
            instance,
            device: device.clone(),
            memory_manager: memory_manager.clone(),
            data,
            pipelines,
            objects: ObjectStorage::new(
                device.clone(),
                memory_manager.clone(),
            ),
            frame_active_objects: Vec::new(),
            frame: 0,
            resized: false,
            camera: Rfc::default()
        },
        window.clone()))
    }
    pub fn set_camera(&mut self, camera: Rfc<Camera>) {
        self.camera = camera;        
    }
    
    pub unsafe fn draw(&mut self, object: Rfc<Object<Vertex>>) -> Result<(), MyError> {
        optick::event!();

        self.objects.insert(
            object.clone(), 
            &self.instance, 
            &self.data.physical_device.borrow(),
        );
        self.frame_active_objects.push(object.borrow().uuid());
        
        Ok(())
    }
    pub unsafe fn render(
        &mut self,
    ) -> Result<(), MyError>
    {
        optick::event!();
        // Wait for gpu to finish the rendering
        self.device.borrow().get_device().wait_for_fences(
            &[self.data.sync_objects[self.frame].render_fence],
            true, 
            u64::MAX
        )?;
        self.device.borrow().get_device().reset_fences(&[self.data.sync_objects[self.frame].render_fence])?;
        
        let result = self.device
            .borrow()
            .get_device()
            .acquire_next_image_khr(
                self.data.swapchain.swapchain, 
                u64::MAX, 
                self.data.sync_objects[self.frame].present_semaphore,
                vk::Fence::null()
            );
        
        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                self.recreate_swapchain()?;
                return Err("Out of date KHR".into());
            },
            Err(e) => return Err(e.into()),
        };

        self.objects.destroy_inactive_objects()?;
        self.update_camera_buffer()?;
        self.update_object_uniforms()?;
        self.prepare_cmd_buffer(image_index)?;

        let wait_semaphores = &[self.data.sync_objects[self.frame].present_semaphore];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = &[self.data.sync_objects[self.frame].render_semaphore];

        let mut command_buffers = Vec::new();        
        for pp_index in 0..self.pipelines.len() {
            command_buffers.push(self.device.borrow().get_graphics_queue().command_buffers[image_index * self.pipelines.len() + pp_index]);
        }
        
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);
        
        self.device.borrow().get_device().queue_submit(
            self.device.borrow().get_graphics_queue().queue, 
            &[submit_info], 
            self.data.sync_objects[self.frame].render_fence
        )?;
        
        let swapchains = &[self.data.swapchain.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.device.borrow().get_device().queue_present_khr(self.device.borrow().get_present_queue().queue, &present_info);

        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        
        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain()?;
            self.camera.borrow_mut().set_viewport_size(self.data.swapchain.extent.width as f32, self.data.swapchain.extent.height as f32);
        }
        else if let Err(e) = result {
            return Err(e.into());
        }
        
        self.frame = (self.frame + 1) % helper::MAX_FRAMES_IN_FLIGHT;
        self.frame_active_objects.clear();

        Ok(())
    }
    unsafe fn update_camera_buffer(&mut self) -> Result<(), MyError> {
        let ubo = ViewProjUBO { 
            view: *self.camera.borrow().get_view_matrix(),
            proj: self.camera.borrow().get_projection_matrix(),
        };

        for pipeline in &mut self.pipelines {
            for i in 0..self.frame_active_objects.len() {
                let memory = self.memory_manager.borrow_mut().map_buffer(
                    pipeline.descriptor_data[self.frame].buffers[1].buffer.clone(),
                    0,
                    size_of::<ViewProjUBO>() as u64,
                    vk::MemoryMapFlags::empty(),
                )?;
                memcpy(&ubo, memory.cast(), 1);

                self.memory_manager.borrow_mut().unmap_buffer(pipeline.descriptor_data[self.frame].buffers[1].buffer.clone())?;
                
                pipeline.descriptor_data[self.frame].update_buffer(
                    1,
                    0,
                    0,
                    i
                );
            }
        }
        Ok(())
    }
    unsafe fn update_object_uniforms(&mut self) -> Result<(), MyError>
    {
        for pipeline in &mut self.pipelines {
            let mut offset = 0;
            for (obj_index, fa_object) in self.frame_active_objects.iter().enumerate() {
                let object = self.objects
                    .get_objects()
                    .get(fa_object)
                    .unwrap()
                    .object
                    .borrow();
                
                let texture = object.vk_texture.as_ref().unwrap();
                let transform = &object.object.borrow().transform;
                
                // Def Pipeline
                let data = glm::scaling(&transform.scale)
                    * glm::rotate(&glm::Mat4::identity(), transform.angle, &transform.rotation_axis)
                    * glm::translate(&glm::Mat4::identity(), &transform.position);
                let ubo = ModelUBOId { data, id: glm::UVec4::new(obj_index as u32, 0, 0, 0) };

                // Copy

                let memory = self.memory_manager.borrow_mut().map_buffer(
                    pipeline.descriptor_data[self.frame].buffers[0].buffer.clone(),
                    offset,
                    size_of::<ModelUBO>() as u64,
                    vk::MemoryMapFlags::empty(),
                )?;
                memcpy(&ubo, memory.cast(), 1);
                self.memory_manager.borrow_mut().unmap_buffer(pipeline.descriptor_data[self.frame].buffers[0].buffer.clone())?;

                pipeline.descriptor_data[self.frame].update_buffer(
                    0,
                    2,
                    0,
                    obj_index,
                );
                pipeline.descriptor_data[self.frame].update_sampled_image(
                    &texture.borrow(),
                    1,
                    0,
                    obj_index
                );

                offset += size_of::<ModelUBOId>() as u64;
            }
        }

        Ok(())
    }
    pub unsafe fn destroy(&mut self) -> Result<(), MyError> {
        self.device.borrow().get_device().device_wait_idle().unwrap();
        
        self.destroy_swapchain()?;
    
        // Objects
        self.objects.borrow_mut().destroy()?;
        
        for sync_obj in &self.data.sync_objects {
            self.device.borrow().get_device().destroy_fence(sync_obj.render_fence, None);
            self.device.borrow().get_device().destroy_semaphore(sync_obj.present_semaphore, None);
            self.device.borrow().get_device().destroy_semaphore(sync_obj.render_semaphore, None);
        }
        
        self.device.borrow().destroy_command_pools();
        self.memory_manager.borrow_mut().destroy(&self.device.borrow());
        self.device.borrow().get_device().destroy_device(None);
        self.instance.get_instance().destroy_surface_khr(self.data.surface, None);
        self.instance.destroy();
    
        Ok(())
    }
    
    unsafe fn prepare_cmd_buffer(&mut self, index: usize) -> Result<(), MyError>
    {
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain.extent);

        let info = vk::CommandBufferBeginInfo::builder(); 
        let borrow = self.device.borrow();


        for (pp_index, pipeline) in self.pipelines.iter().enumerate() {
            let command_buffer = &borrow.get_graphics_queue().command_buffers[index * self.pipelines.len() + pp_index];
            
            self.device.borrow().get_device().reset_command_buffer(*command_buffer, vk::CommandBufferResetFlags::empty())?;

            // Prepare to submit commands
            self.device.borrow().get_device().begin_command_buffer(*command_buffer, &info)?;
            
            // Begin render pass
            let color_clear_value = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }; 
            let depth_clear_value = vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
            };
            let clear_values = &[color_clear_value, depth_clear_value];

            let begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(pipeline.render_pass)
                .framebuffer(pipeline.framebuffers[index])
                .render_area(render_area)
                .clear_values(clear_values);

            self.device.borrow().get_device().cmd_begin_render_pass(*command_buffer, &begin_info, vk::SubpassContents::INLINE);
            self.device.borrow().get_device().cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);

            let mut ubo_offset = 0;
            for (obj_index, fa_object) in self.frame_active_objects.iter().enumerate() {
                let object = self.objects
                    .get_objects()
                    .get(fa_object)
                    .unwrap()
                    .object
                    .borrow();
                
                self.device.borrow().get_device().cmd_bind_vertex_buffers(
                    *command_buffer, 
                    0, 
                    &[object.vertex_buffer.as_ref().unwrap().buffer.borrow().buffer], 
                    &[0]
                );
                self.device.borrow().get_device().cmd_bind_index_buffer(
                    *command_buffer, 
                    object.index_buffer.as_ref().unwrap().buffer.borrow().buffer, 
                    0, 
                    vk::IndexType::UINT32
                );
                self.device.borrow().get_device().cmd_bind_descriptor_sets(
                    *command_buffer, 
                    vk::PipelineBindPoint::GRAPHICS, 
                    pipeline.layout, 
                    0, 
                    pipeline.descriptor_data[self.frame].get_sets(obj_index).as_slice(),
                    &[ubo_offset]
                );
                ubo_offset += size_of::<ModelUBOId>() as u32;
                
                // Draw call
                self.device.borrow().get_device().cmd_draw_indexed(
                    *command_buffer, 
                    object.object.borrow().indices.len() as u32,
                    1, 
                    0, 
                    0, 
                    0
                );
            }

            // End renderpass
            self.device.borrow().get_device().cmd_end_render_pass(*command_buffer);
            // End command submit
            self.device.borrow().get_device().end_command_buffer(*command_buffer)?;
        }
        
        Ok(())
    }
    unsafe fn destroy_swapchain(&mut self) -> Result<(), MyError>{
        self.device.borrow().free_command_buffers();

        for p in &mut self.pipelines {
            p.destroy(&self.device.borrow())?;
        }
        self.data.swapchain.views.iter().for_each(|v| self.device.borrow().get_device().destroy_image_view(*v, None));
        
        self.device.borrow().get_device().destroy_swapchain_khr(self.data.swapchain.swapchain, None);
        
        Ok(())
    }
    unsafe fn recreate_swapchain(&mut self) -> Result<(), MyError> {
        self.device.borrow().get_device().device_wait_idle()?;
        
        self.destroy_swapchain()?;
        self.data.swapchain = VkSwapchain::new(
            &self.window.borrow(), 
            &self.instance, 
            &self.data.surface, 
            &self.data.physical_device, 
            &self.device.borrow()
        )?;

        let pp1 = VkPipeline::get_2d(
            self.device.clone(), 
            &self.instance, 
            &self.data.physical_device, 
            &self.data.swapchain, 
            self.memory_manager.clone(), 
            self.data.msaa_samples
        )?;
        let pp2 = VkPipeline::obj_picker(
            self.device.clone(), 
            &self.instance, 
            &self.data.physical_device, 
            &self.data.swapchain, 
            self.memory_manager.clone(), 
        )?;
        self.pipelines = vec![pp1, pp2];

        self.device.borrow_mut().allocate_command_buffers(VkQueueFamily::GRAPHICS, self.data.swapchain.images.len() as u32 + MAX_PIPELINES)?;
        self.device.borrow_mut().allocate_command_buffers(VkQueueFamily::PRESENT, self.data.swapchain.images.len() as u32 + MAX_PIPELINES)?;
        self.device.borrow_mut().allocate_command_buffers(VkQueueFamily::TRANSFER, self.data.swapchain.images.len() as u32 + MAX_PIPELINES)?;

        Ok(())
    }
}
