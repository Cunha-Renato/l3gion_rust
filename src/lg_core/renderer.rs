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
use std::cell::RefCell;
use std::rc::Rc;
use std::{borrow::Borrow, mem::size_of};
use std::ptr::copy_nonoverlapping as memcpy;

use nalgebra_glm as glm;
use vulkanalia::vk::{ExtDebugUtilsExtension, InstanceV1_0, KhrSurfaceExtension};
use winit::window::Window;
use vulkanalia::{
    vk::{
        self, DeviceV1_0, Handle, HasBuilder, KhrSwapchainExtension
    }, 
    window as vk_window, 
    Entry, 
};
use crate::MyError;
use texture::Texture;
use helper::RendererData;
use self::{uniform_buffer_object::{ModelUBO, ViewProjUBO}, vulkan::vk_memory_allocator::VkMemoryManager};
use self::vulkan::vk_descriptor::BufferCategory;
use self::camera::Camera;
use self::object_storage::ObjectStorage;
use self::vulkan::vk_device::{VkDevice, VkQueueFamily};
use self::vulkan::vk_image::VkImage;
use self::vulkan::vk_instance::VkInstance;
use self::vulkan::vk_physical_device::VkPhysicalDevice;
use self::vulkan::vk_renderpass::VkRenderPass;
use self::{object::Object, uniform_buffer_object::UniformBufferObject, vertex::Vertex, vulkan::{framebuffer, vk_pipeline::*, shader::Shader, vk_swapchain::VkSwapchain}};
use super::{lg_types::reference::Rfc, uuid::UUID};

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
    test_pipeline: DefaultPipeline,
    // pipelines: Vec<Rfc<dyn VulkanPipeline>>,
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

        device.borrow_mut().allocate_command_buffers(VkQueueFamily::GRAPHICS, data.swapchain.images.len() as u32)?;
        device.borrow_mut().allocate_command_buffers(VkQueueFamily::PRESENT, data.swapchain.images.len() as u32)?;
        device.borrow_mut().allocate_command_buffers(VkQueueFamily::TRANSFER, data.swapchain.images.len() as u32)?;

        let render_pass = VkRenderPass::get_default(
            &instance, 
            &device.borrow(), 
            &data.physical_device.borrow(), 
            data.swapchain.format, 
            data.msaa_samples
        )?;
        
        let default_pipeline = DefaultPipeline::new(
            device.clone(), 
            &instance,
            &data.physical_device,
            &mut memory_manager.borrow_mut(),
            &[Vertex::binding_description()],
            &Vertex::attribute_descritptions(), 
            &data.swapchain, 
            data.msaa_samples, 
            render_pass
        )?;
        
        /* let render_pass = VkRenderPass::get_object_picker(
            &instance, 
            &device.borrow(), 
            &data.physical_device
        )?; */

        /* let obj_picker = ObjectPickerPipeline::new(
            device.clone(),
            &instance, 
            &data.physical_device, 
            &mut memory_manager.borrow_mut(),
            &[Vertex::binding_description()],
            &Vertex::attribute_descritptions(), 
            &data.swapchain, 
            &render_pass
        )?; */

        let window = Rfc::new(window);

        helper::create_sync_objects(device.borrow().get_device(), &mut data)?;

        // let pipelines = vec![Rc::new(RefCell::new(default_pipeline))];
        // let pipelines: Vec<Rfc<dyn VulkanPipeline>> = pipelines
        //     .iter()
        //     .map(|pp| Rfc::from_rc_refcell(&(pp.clone() as Rc<RefCell<dyn VulkanPipeline>>)))
        //     .collect();


        Ok((Self {
            window: window.clone(),
            _entry: entry,
            instance,
            device: device.clone(),
            memory_manager: memory_manager.clone(),
            data,
            test_pipeline: default_pipeline,
            // pipelines,
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
    
    pub fn vsync(&mut self, option: bool) {
        todo!()
    }
    pub fn msaa(&mut self, value: u32) {
        todo!()
    }
    pub fn set_camera(&mut self, camera: Rfc<Camera>) {
        self.camera = camera;        
    }
    
    // TODO: Create a gigantic uniform buffer, use pushconstants to address them in the shader
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
        let command_buffers = &[self.device.borrow().get_graphics_queue().command_buffers[image_index]];
        let signal_semaphores = &[self.data.sync_objects[self.frame].render_semaphore];
        
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
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
        
        // Copy

        for i in 0..self.frame_active_objects.len() {
            let memory = self.memory_manager.borrow_mut().map_buffer(
                self.test_pipeline.descriptor_data[self.frame].buffers[BufferCategory::VIEW_PROJ as usize].region.clone(),
                0,
                size_of::<ViewProjUBO>() as u64,
                vk::MemoryMapFlags::empty(),
            )?;
            memcpy(&ubo, memory.cast(), 1);

            self.memory_manager.borrow_mut().unmap_buffer(self.test_pipeline.descriptor_data[self.frame].buffers[BufferCategory::VIEW_PROJ as usize].region.clone())?;
            
            self.test_pipeline.descriptor_data[self.frame].update_vp(i);
        }
        Ok(())
    }
    unsafe fn update_object_uniforms(&mut self) -> Result<(), MyError>
    {
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
            let data = glm::scaling(&transform.scale)
                * glm::rotate(&glm::Mat4::identity(), transform.angle, &transform.rotation_axis)
                * glm::translate(&glm::Mat4::identity(), &transform.position);
            let ubo = ModelUBO { data };

            // Copy

            let memory = self.memory_manager.borrow_mut().map_buffer(
                self.test_pipeline.descriptor_data[self.frame].buffers[BufferCategory::MODEL as usize].region.clone(),
                offset,
                size_of::<ModelUBO>() as u64,
                vk::MemoryMapFlags::empty(),
            )?;
            memcpy(&ubo, memory.cast(), 1);
            self.memory_manager.borrow_mut().unmap_buffer(self.test_pipeline.descriptor_data[self.frame].buffers[BufferCategory::MODEL as usize].region.clone())?;

            self.test_pipeline.descriptor_data[self.frame].update_model(
                obj_index,
            );
            self.test_pipeline.descriptor_data[self.frame].update_image(
                texture,
                obj_index
            );

            offset += size_of::<UniformBufferObject>() as u64;
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
    
        if helper::VALIDATION_ENABLED {
            self.instance.get_instance().destroy_debug_utils_messenger_ext(self.instance.messenger.unwrap(), None);
        }
        
        self.device.borrow().destroy_command_pools();
        self.memory_manager.borrow_mut().destroy(&self.device.borrow());
        self.device.borrow().get_device().destroy_device(None);
        self.instance.get_instance().destroy_surface_khr(self.data.surface, None);
        self.instance.get_instance().destroy_instance(None);
    
        Ok(())
    }
    
    unsafe fn prepare_cmd_buffer(&mut self, index: usize) -> Result<(), MyError>
    {
        let info = vk::CommandBufferBeginInfo::builder(); 
        let borrow = self.device.borrow();
        let command_buffer = &borrow.get_graphics_queue().command_buffers[index];
        
        self.device.borrow().get_device().reset_command_buffer(*command_buffer, vk::CommandBufferResetFlags::empty())?;

        // Prepare to submit commands
        self.device.borrow().get_device().begin_command_buffer(*command_buffer, &info)?;
        
        // Begin render pass
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain.extent);

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
            .render_pass(*self.test_pipeline.render_pass.get_render_pass())
            .framebuffer(self.test_pipeline.framebuffers[index])
            .render_area(render_area)
            .clear_values(clear_values);

        self.device.borrow().get_device().cmd_begin_render_pass(*command_buffer, &begin_info, vk::SubpassContents::INLINE);
        self.device.borrow().get_device().cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, self.test_pipeline.pipeline);

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
                &[object.vertex_buffer.as_ref().unwrap().buffer], 
                &[0]
            );
            self.device.borrow().get_device().cmd_bind_index_buffer(
                *command_buffer, 
                object.index_buffer.as_ref().unwrap().buffer, 
                0, 
                vk::IndexType::UINT32
            );
            self.device.borrow().get_device().cmd_bind_descriptor_sets(
                *command_buffer, 
                vk::PipelineBindPoint::GRAPHICS, 
                self.test_pipeline.layout, 
                0, 
                self.test_pipeline.descriptor_data[self.frame].get_sets(obj_index).as_slice(),
                &[ubo_offset]
            );
            ubo_offset += size_of::<UniformBufferObject>() as u32;
            
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
        
        Ok(())
    }
    unsafe fn destroy_swapchain(&mut self) -> Result<(), MyError>{
        self.device.borrow().free_command_buffers();

        self.test_pipeline.destroy(&self.device.borrow(), &mut self.memory_manager.borrow_mut())?;

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

        let render_pass = VkRenderPass::get_default(
            &self.instance, 
            &self.device.borrow(), 
            &self.data.physical_device.borrow(), 
            self.data.swapchain.format, 
            self.data.msaa_samples
        )?;
        self.test_pipeline = DefaultPipeline::new(
            self.device.clone(), 
            &self.instance,
            &self.data.physical_device,
            &mut self.memory_manager.borrow_mut(),
            &[Vertex::binding_description()],
            &Vertex::attribute_descritptions(), 
            &self.data.swapchain, 
            self. data.msaa_samples, 
            render_pass
        )?;

        self.device.borrow_mut().allocate_command_buffers(VkQueueFamily::GRAPHICS, self.data.swapchain.images.len() as u32)?;
        self.device.borrow_mut().allocate_command_buffers(VkQueueFamily::PRESENT, self.data.swapchain.images.len() as u32)?;
        self.device.borrow_mut().allocate_command_buffers(VkQueueFamily::TRANSFER, self.data.swapchain.images.len() as u32)?;

        Ok(())
    }
}
pub struct DrawInfo {
    position: glm::Vec3,
    scale: glm::Vec3,
    rotation: f32,
    color: glm::Vec4,
    texture: Option<Texture>,
    tiling: Option<i32>,
}
pub struct CircleInfo {
    draw_info: DrawInfo,
    radius: f32,
    thickness: Option<f32>,
}

struct VerticesInfo {
    vertices: [Vertex; 4],
    indices: [u16; 5],
}
pub struct Renderer2D {
    vertices_info: VerticesInfo
}
impl Renderer2D {
    fn init() -> Self {
        let vertices = [
            Vertex::new(glm::vec3(-0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3(0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3(0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let indices = [0, 1, 2, 3, 4];
        
        Self {
            vertices_info: VerticesInfo {
                vertices,
                indices
            }
        }
    }
    pub fn shutdown(&mut self) {
        todo!()
    }

    pub fn begin_batch(&mut self) {
        todo!()
    }
    pub fn end_batch(&mut self) {
        
    }
    
    pub fn submit_quad(&mut self, info: &DrawInfo) {
        todo!()
    }
    pub fn submit_circle(&mut self, info: &CircleInfo) {
        todo!()
    }
}