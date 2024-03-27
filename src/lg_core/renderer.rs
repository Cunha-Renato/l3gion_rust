pub mod camera;
pub mod renderer_core;
pub mod model;
pub mod vertex;
pub mod texture;
pub mod vulkan;
pub mod object;
pub mod helper;
pub mod uniform_buffer_object;

use nalgebra_glm as glm;
use winit::window::Window;
use vulkanalia::{
    vk::{
        self, DeviceV1_0, ExtDebugUtilsExtension, Handle, HasBuilder, InstanceV1_0, KhrSurfaceExtension, KhrSwapchainExtension
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

pub struct Renderer {
    window: Window,
    entry: Entry,
    instance: Instance,
    data: RendererData,
    device: Device,
    test_pipeline: VkPipeline, // One Pipeline for each kind of rendering (ie. Batch, Circle, Rect, Normal)
    texture: VkTexture, // The textures also need to be in a hash map (probably)
    objects: Vec<Object<Vertex>>,
    frame: usize,
    resized: bool,
}
impl Renderer {
    pub unsafe fn init(window: Window) -> Result<Self, MyError> {
        let mut data = RendererData::default();
        let entry = helper::create_entry(&window)?; 
        let instance = helper::create_instance(&window, &entry, &mut data.messenger)?;
        
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
            &window, 
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
            window,
            entry,
            instance,
            data,
            device,
            test_pipeline,
            texture,
            objects: Vec::default(),
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
    //  Talking about uniform buffer, for every draw call I need to bind a different uniform buffer, (YAY I love this api (i don't))
    pub unsafe fn draw(&mut self, mut object: Object<Vertex>) -> Result<(), MyError> {
        // Vertex and Index buffers are inside the Object, should I put the uniforms to??
        // Update the uniform buffer????
        // In case of updating the vertices or indices recreate the buffers, (Object's function)
        // So, change of plans, instead of sending the commands themselfs, I will store the object's key so they can be draw later. (need to free the keys array after drawing). Question more expensive???? IDK
        // I will iterate the keys and search the Object HashMap, and for every object I will send those commands,
        // it is also necessary to create a seperate function to record those commands
        // I may lose control over the object itself, I don't like the idea to send a reference back, I think I should reset the objects queue every frame

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

        self.objects.push(object);
        
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
        
        self.data.command_pool.reset_command_buffer(&self.device, image_index)?;
        self.prepare_cmd_buffer(image_index)?;

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
            self.recreate_swapchain()?;
        }
        else if let Err(e) = result {
            return Err(e.into());
        }
        
        self.frame = (self.frame + 1) % helper::MAX_FRAMES_IN_FLIGHT;
        self.clear_objects()?;

        Ok(())
    }
    
    pub unsafe fn destroy(&mut self) -> Result<(), MyError> {
        self.device.device_wait_idle().unwrap();
        
        self.destroy_swapchain();
        
        // Texture
        self.device.destroy_sampler(self.texture.sampler, None);
        self.device.destroy_image_view(self.texture.image_data.views[0], None);
        self.device.destroy_image(self.texture.image_data.images[0], None);
        self.device.free_memory(self.texture.image_data.memories.unwrap()[0], None);

        self.device.destroy_descriptor_set_layout(self.test_pipeline.descriptor_data.layout, None);

        // Vertices
        self.clear_objects(); 
        
        for i in 0..helper::MAX_FRAMES_IN_FLIGHT {
            self.device.destroy_fence(self.data.in_flight_fences[i], None);
            
            self.device.destroy_semaphore(self.data.image_available_semaphores[i], None);
            
            self.device.destroy_semaphore(self.data.render_finished_semaphores[i], None);
        }

        self.data.command_pool.destroy(&self.device);
        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);

        if helper::VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }
        
        self.instance.destroy_instance(None);

        Ok(())
    }
    
    // Private
    unsafe fn clear_objects(&mut self) -> Result<(), MyError> {
        for object in self.objects {
            // Clearing Vertices
            self.device.destroy_buffer(object.vertex_buffer()?.buffer, None);
            self.device.free_memory(object.vertex_buffer()?.memory, None);
            
            // Clearing Indices
            self.device.destroy_buffer(object.index_buffer()?.buffer, None);
            self.device.free_memory(object.index_buffer()?.memory, None);
        }
        
        self.objects.clear();

        Ok(())
    }
    unsafe fn prepare_cmd_buffers(&mut self) -> Result<(), MyError>
    {
        for (i, _) in self.data.command_pool.buffers.iter().enumerate() {
            self.prepare_cmd_buffer(i)?;
        }

        Ok(())
    }
    unsafe fn prepare_cmd_buffer(&mut self, index: usize) -> Result<(), MyError>
    {
        let info = vk::CommandBufferBeginInfo::builder(); 
        let command_buffer = &self.data.command_pool.buffers[index];
        
        // Prepare to submit commands
        self.device.begin_command_buffer(*command_buffer, &info)?;
        
        // Begin render pass
        let begin_info = self.data.command_pool.get_render_pass_begin_info(
            &self.data.swapchain, 
            &self.data.render_pass, 
            &self.data.framebuffers[index]
        );

        self.device.cmd_begin_render_pass(*command_buffer, &begin_info, vk::SubpassContents::INLINE);
        
        self.device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, self.test_pipeline.pipeline);

        for object in &self.objects {
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
                &[self.test_pipeline.descriptor_data.sets[index]], 
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
        }

        // End renderpass
        self.device.cmd_end_render_pass(*command_buffer);
        
        // End command submit
        self.device.end_command_buffer(*command_buffer)?;
        
        Ok(())
    }
    unsafe fn destroy_swapchain(&mut self) {
        self.data.command_pool.free_buffers(&self.device);
        self.test_pipeline.descriptor_data.destroy_pool(&self.device);
        
        // Uniform buffer
        self.test_pipeline.uniform_buffer.memories.iter().for_each(|m| self.device.free_memory(*m, None));
        self.test_pipeline.uniform_buffer.buffers.iter().for_each(|u| self.device.destroy_buffer(*u, None));

        // Depth images        
        self.device.destroy_image_view(self.data.depth_image.views[0], None);
        self.device.free_memory(self.data.depth_image.memories.unwrap()[0], None);
        self.device.destroy_image(self.data.depth_image.images[0], None);
        
        // Color images
        self.device.destroy_image_view(self.data.color_image.views[0], None);
        self.device.free_memory(self.data.color_image.memories.unwrap()[0], None);
        self.device.destroy_image(self.data.color_image.images[0], None);
        
        self.data.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None));
        
        self.device.destroy_pipeline(self.test_pipeline.pipeline, None);
    
        self.device.destroy_render_pass(self.data.render_pass, None);
        
        self.data.swapchain.image_data.views.iter().for_each(|v| self.device.destroy_image_view(*v, None));
        
        self.device.destroy_swapchain_khr(self.data.swapchain.swapchain, None);
    }
    unsafe fn recreate_swapchain(&mut self) -> Result<(), MyError> {
        self.device.device_wait_idle()?;
        
        self.destroy_swapchain();
        self.data.swapchain = VkSwapchain::new(
            &self.window, 
            &self.instance, 
            &self.data.surface, 
            &self.data.physical_device, 
            &self.device
        )?;
        self.data.render_pass = render_pass::create_render_pass(
            &self.instance, 
            &self.device, 
            &self.data.physical_device, 
            self.data.swapchain.format, 
            self.data.msaa_samples
        )?;

        // Shaders
        let vert_shader = Shader::new(
            &self.device, 
            vk::ShaderStageFlags::VERTEX, 
            include_bytes!("../../assets/shaders/compiled/vertex.spv")
        )?;
        let frag_shader = Shader::new(
            &self.device, 
            vk::ShaderStageFlags::FRAGMENT, 
            include_bytes!("../../assets/shaders/compiled/fragment.spv")
        )?;
        let ubo = UniformBuffer::new::<UniformBufferObject>(
            &self.instance, 
            &self.device, 
            &self.data.physical_device, 
            &self.data.swapchain
        )?;
        // Viewport and Scissor
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(self.data.swapchain.extent.width as f32)
            .height(self.data.swapchain.extent.width as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();
        
        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.data.swapchain.extent)
            .build();



        self.test_pipeline = VkPipeline::new(
            &self.device, 
            vert_shader, 
            frag_shader, 
            ubo,
            &[Vertex::binding_description()],
            &Vertex::attribute_descritptions(), 
            DescriptorData::new_default(
                &self.device, 
                &self.data.swapchain, 
                &ubo, 
            )?,// Setup for normal rendering, change later
            vec![viewport], 
            vec![scissor], 
            self.data.msaa_samples, 
            self.data.render_pass
        )?;

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