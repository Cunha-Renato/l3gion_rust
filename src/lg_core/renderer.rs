pub mod camera;
pub mod renderer_core;
pub mod model;
pub mod vertex;
pub mod texture;
pub mod vulkan;
pub mod object;
pub mod helper;
pub mod uniform_buffer_object;

use std::collections::HashMap;

use nalgebra_glm as glm;
use winit::window::Window;
use vulkanalia::{
    vk::{
        self, DeviceV1_0, Handle, HasBuilder, KhrSwapchainExtension
    }, 
    window as vk_window, 
    Device, 
    Entry, 
    Instance
};
use crate::MyError;
use texture::Texture;
use helper::RendererData;

use self::{object::Object, uniform_buffer_object::UniformBufferObject, vertex::Vertex, vulkan::{command_buffer::VkCommandPool, descriptor::DescriptorData, framebuffer, image::ImageData, physical_device, pipeline::VkPipeline, render_pass, shader::Shader, swapchain::VkSwapchain, uniform_buffer::UniformBuffer, vk_texture::VkTexture}};

use super::uuid::UUID;

pub struct Renderer {
    entry: Entry,
    instance: Instance,
    data: RendererData,
    device: Device,
    test_pipeline: VkPipeline, // One Pipeline for each kind of rendering (ie. Batch, Circle, Rect, Normal)
    texture: VkTexture, // The textures also need to be in a hash map (probably)
    objects: HashMap<UUID, Object<Vertex>>,
    frame: usize,
    resized: bool,
}
impl Renderer {
    pub unsafe fn init(window: &Window) -> Result<Self, MyError> {
        let mut data = RendererData::default();
        let entry = helper::create_entry(window)?; 
        let instance = helper::create_instance(window, &entry)?;
        
        data.surface = vk_window::create_surface(&instance, &window, &window)?;
        let (physical_device, indices) = physical_device::pick_physical_device(
            &instance, 
            &data.surface
        )?;
        data.physical_device = physical_device;
        data.queue_indices = indices;

        let (device, queues) = helper::create_logical_device(
            &entry, 
            &data.physical_device, 
            &data.queue_indices, 
            &instance
        )?;
        data.graphics_queue = queues.0;
        data.present_queue = queues.1;
        data.swapchain = VkSwapchain::new(
            window, 
            &instance, 
            &data.surface, 
            &physical_device, 
            &device
        )?;
        data.msaa_samples = vk::SampleCountFlags::_8;
        data.render_pass = render_pass::create_render_pass(
            &instance, 
            &device, 
            &physical_device, 
            data.swapchain.format, 
            data.msaa_samples
        )?;
        
        // Shaders
        let vert_shader = Shader::new(
            &device, 
            vk::ShaderStageFlags::VERTEX, 
            include_bytes!("../../assets/shaders/compiled/vertex.spv")
        )?;
        let frag_shader = Shader::new(
            &device, 
            vk::ShaderStageFlags::FRAGMENT, 
            include_bytes!("../../assets/shaders/compiled/fragment.spv")
        )?;
        
        // Viewport and Scissor
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(data.swapchain.extent.width as f32)
            .height(data.swapchain.extent.width as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();
        
        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(data.swapchain.extent)
            .build();

        let ubo = UniformBuffer::new::<UniformBufferObject>(
            &instance, 
            &device, 
            &physical_device, 
            &data.swapchain
        )?;

        let test_pipeline = VkPipeline::new(
            &device, 
            vert_shader, 
            frag_shader, 
            ubo,
            &[Vertex::binding_description()],
            &Vertex::attribute_descritptions(), 
            DescriptorData::new_default(
                &device, 
                &data.swapchain, 
                &ubo, 
            )?,// Setup for normal rendering, change later
            vec![viewport], 
            vec![scissor], 
            data.msaa_samples, 
            data.render_pass
        )?;
        
        data.command_pool = VkCommandPool::new(
            &instance, 
            &device, 
            &data.queue_indices
        )?;

        data.color_image = ImageData::new_with_memory(
            &instance, 
            &physical_device, 
            &device, 
            data.swapchain.format, 
            vk::ImageType::_2D, 
            vk::ImageViewType::_2D, 
            vk::ImageAspectFlags::COLOR, 
            data.swapchain.extent.width, 
            data.swapchain.extent.height, 
            1, 
            data.msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL
        )?;
        data.depth_image = ImageData::new_with_memory(
            &instance, 
            &physical_device,
            &device, 
            helper::get_depth_format(&instance, &data.physical_device)?,
            vk::ImageType::_2D, 
            vk::ImageViewType::_2D, 
            vk::ImageAspectFlags::DEPTH, 
            data.swapchain.extent.width, 
            data.swapchain.extent.height, 
            1, 
            data.msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL
        )?;

        data.framebuffers = framebuffer::create_framebuffers(
            &device, 
            &data.render_pass, 
            &data.swapchain.image_data, 
            &data.color_image, 
            &data.depth_image, 
            data.swapchain.extent.width, 
            data.swapchain.extent.height
        )?;

        // TODO: Make this similar to objects, better yet maybe tie this to the object itself.
        let texture = VkTexture::new(
            &instance, 
            &device, 
            &physical_device, 
            &data.command_pool, 
            &data.graphics_queue, 
            Texture::new("assets/textures/grid.png")?,
        )?;
        
        data.command_pool.create_buffers(&device, data.framebuffers.len() as u32);

        Ok(Self {
            entry,
            instance,
            data,
            device,
            test_pipeline,
            texture,
            objects: HashMap::default(),
            frame: 0,
            resized: false,
        })
    }
    
    pub fn vsync(&mut self, option: bool) {
        todo!()
    }
    pub fn msaa(&mut self, value: u32) {
        todo!()
    }
    
    // TODO: Review all of this shit below!!!!
    // I belive that, as an argument this function should have a single struct with data for:
    //  object's position, rotation, scale, and maybe other things relevant to drawing
    //  It is ok to send those informations as an uniform buffer because it is not batched, on the contrary the transform should be aplied in the cpu side??
    pub unsafe fn draw(&mut self, mut object: Object<Vertex>) -> Result<(), MyError> {
        // Vertex and Index buffers are inside the Object, should I put the uniforms to??
        // Update the uniform buffer????
        // In case of updating the vertices or indices recreate the buffers, (Object's function)

        if !self.objects.contains_key(&object.uuid()) {
            object.create_vertex_buffer(
                &self.instance, 
                &self.device, 
                &self.data.physical_device, 
                &self.data.command_pool, 
                &self.data.graphics_queue
            )?;
            object.create_index_buffer(
                &self.instance, 
                &self.device, 
                &self.data.physical_device, 
                &self.data.command_pool, 
                &self.data.graphics_queue
            )?;

            self.objects.insert(object.uuid(), object);
        }
        
        // Send commands to bind the vertex, index and uniform buffers
        // Send command do draw indexed
        for (i, command_buffer) in self.data.command_pool.buffers.iter().enumerate() {
            let info = vk::CommandBufferBeginInfo::builder(); 
            
            // Prepare to submit commands
            self.device.begin_command_buffer(*command_buffer, &info)?;
            
            // Begin render pass
            let begin_info = self.data.command_pool.get_render_pass_begin_info(
                &self.data.swapchain, 
                &self.data.render_pass, 
                &self.data.framebuffers[i]
            );
            self.device.cmd_begin_render_pass(*command_buffer, &begin_info, vk::SubpassContents::INLINE);

            self.device.cmd_bind_vertex_buffers(
                *command_buffer, 
                0, 
                &[object.vertex_buffer()?.buffer], 
                &[0]
            );
            self.device.cmd_bind_index_buffer(
                *command_buffer, 
                object.index_buffer()?.buffer, 
                0, 
                vk::IndexType::UINT32
            );
            self.device.cmd_bind_descriptor_sets(
                *command_buffer, 
                vk::PipelineBindPoint::GRAPHICS, 
                self.test_pipeline.layout, 
                0, 
                &[self.test_pipeline.descriptor_data.sets[i]], 
                &[]
            );
            
            // Draw call
            self.device.cmd_draw_indexed(
                *command_buffer, 
                object.indices().len() as u32, 
                1, 
                0, 
                0, 
                0
            );
            
            // End renderpass
            self.device.cmd_end_render_pass(*command_buffer);
            
            // End command submit
            self.device.end_command_buffer(*command_buffer)?;
        }

        Ok(())
    }
    pub unsafe fn render(
        &mut self,
    ) -> Result<(), MyError>
    {
        let in_flight_fence = self.data.in_flight_fences[self.frame];
        
        self.device.wait_for_fences(&[in_flight_fence], true, u64::MAX)?;
        
        let result = self.device.acquire_next_image_khr(
            self.data.swapchain.swapchain, 
            u64::MAX, 
            self.data.image_available_semaphores[self.frame], 
            vk::Fence::null()
        );
        
        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                // TODO: Recreate swapchain
                return Err("Out of date KHR".into());
            },
            Err(e) => return Err(e.into()),
        };
        
        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device.wait_for_fences(&[image_in_flight], true, u64::MAX)?;
        }
        
        self.data.images_in_flight[image_index] = in_flight_fence;
        
        // Update uniform buffer

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_pool.buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);
        
        self.device.reset_fences(&[in_flight_fence])?;
        
        self.device.queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence)?;
        
        let swapchains = &[self.data.swapchain.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);
        
        let result = self.device.queue_present_khr(self.data.present_queue, &present_info);

        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        
        if self.resized || changed {
            self.resized = false;
            // TODO: Recreate swapchain
        }
        else if let Err(e) = result {
            return Err(e.into());
        }
        
        self.frame = (self.frame + 1) % helper::MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }
    
    pub fn destroy(&mut self) -> Result<(), MyError> {
        todo!()
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