use super::buffer::Buffer;

#[derive(Clone, Debug)]
pub struct Uniform {
    name: String,
    u_type: lg_renderer::renderer::lg_uniform::LgUniformType,
    binding: usize,
    set: usize,
    update_data: bool,
    pub buffer: Buffer,
}
impl Uniform {
    pub fn new(
        name: &str,
        u_type: lg_renderer::renderer::lg_uniform::LgUniformType,
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
        u_type: lg_renderer::renderer::lg_uniform::LgUniformType,
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
}
impl lg_renderer::renderer::lg_uniform::LgUniform for Uniform {
    fn name(&self) -> &str {
        &self.name
    }

    fn u_type(&self) -> lg_renderer::renderer::lg_uniform::LgUniformType {
        self.u_type
    }

    fn binding(&self) -> usize {
        self.binding
    }

    fn set(&self) -> usize {
        self.set
    }

    fn update_data(&self) -> bool {
        self.update_data
    }
    
    fn data_size(&self) -> usize {
        self.buffer.data_size()
    }
    
    fn get_raw_data(&self) -> *const std::ffi::c_void {
        self.buffer.get_raw_data()
    }
    
    fn set_data<D>(&mut self, data: &D) {
        unsafe { self.buffer.set_data(data) };
    }
}