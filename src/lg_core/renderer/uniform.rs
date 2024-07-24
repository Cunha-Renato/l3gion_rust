use super::buffer::Buffer;

#[derive(Clone, Copy, Debug)]
pub enum LgUniformType {
    STRUCT,
    STORAGE_BUFFER,
    COMBINED_IMAGE_SAMPLER
}
impl LgUniformType {
    pub(crate) fn to_opengl(&self) -> gl::types::GLenum {
        match &self {
            LgUniformType::STRUCT => gl::UNIFORM_BUFFER,
            LgUniformType::STORAGE_BUFFER => gl::SHADER_STORAGE_BUFFER,
            LgUniformType::COMBINED_IMAGE_SAMPLER => gl::SAMPLER_2D,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Uniform {
    name: String,
    u_type: LgUniformType,
    binding: usize,
    set: usize,
    update_data: bool,
    pub buffer: Buffer,
}
impl Uniform {
    pub fn new(
        name: &str,
        u_type: LgUniformType,
        binding: usize,
        set: usize,
        update_data: bool,
    ) -> Self 
    {
        Self {
            name: name.to_string(),
            u_type,
            binding,
            set,
            update_data,
            buffer: Buffer::default()
        }
    }
    
    pub unsafe fn new_with_data<D>(
        name: &str,
        u_type: LgUniformType,
        binding: usize,
        set: usize,
        update_data: bool,
        data: D
    ) -> Self 
    {
        Self {
            name: name.to_string(),
            u_type,
            binding,
            set, 
            update_data,
            buffer: Buffer::new(&data)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn u_type(&self) -> LgUniformType {
        self.u_type
    }

    pub fn binding(&self) -> usize {
        self.binding
    }

    pub fn set(&self) -> usize {
        self.set
    }

    pub fn update_data(&self) -> bool {
        self.update_data
    }
    
    pub fn data_size(&self) -> usize {
        self.buffer.data_size()
    }
    
    pub fn get_raw_data(&self) -> *const std::ffi::c_void {
        self.buffer.get_raw_data()
    }
    
    pub fn set_data<D>(&mut self, data: &D) {
        unsafe { self.buffer.set_data(data) };
    }
}