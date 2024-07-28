#[derive(Debug, Clone)]
pub struct VertexInfo {
    pub stride: usize,
    pub gl_info: Vec<(u32, i32, i32)>
}

pub trait LgVertex: GlVertex + Sized {
    fn vertex_info(&self) -> VertexInfo {
        unsafe { VertexInfo {
            stride: self.stride(),
            gl_info: Self::gl_info(),
        }}
    }

    fn stride(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
pub trait GlVertex {
    /// (location, components, offset)
    unsafe fn gl_info() -> Vec<(u32, i32, i32)>;
}

#[macro_export]
macro_rules! lg_vertex {
    ($struct_name:ident, $($fields:tt), *) => {
        impl crate::lg_core::renderer::vertex::GlVertex for $struct_name {
            #[allow(unused_assignments)]
            unsafe fn gl_info() -> Vec<(u32, i32, i32)> {
                const fn size_of_raw<T>(_: *const T) -> usize {
                    core::mem::size_of::<T>()
                }
                let mut result = Vec::new();
                let mut location = 0;
                $(
                    let dummy = core::mem::MaybeUninit::<$struct_name>::uninit();
                    let dummy_ptr = dummy.as_ptr();
                    let member_ptr = core::ptr::addr_of!((*dummy_ptr).$fields);
                    let member_offset = member_ptr as i32 - dummy_ptr as i32;
                    
                    result.push((
                        location,
                        (size_of_raw(member_ptr) / core::mem::size_of::<f32>()) as i32,
                        member_offset
                    ));
                    location += 1;
                )*
                
                result
            }
        }
        impl crate::lg_core::renderer::vertex::LgVertex for $struct_name {}
    };
}

use nalgebra_glm as glm;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub tex_coord: glm::Vec2,
}
lg_vertex!(Vertex, position, normal, tex_coord);