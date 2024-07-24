use crate::lg_core::renderer::texture::{TextureFilter, TextureSpecs};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTarget {
    pub framebuffer: gl::types::GLuint,
    pub color_texture: gl::types::GLuint,
    pub depth_texture: Option<gl::types::GLuint>,
    
    pub specs: RenderTargetSpecs
}
impl RenderTarget {
    pub fn new(specs: RenderTargetSpecs) -> Self {
        let mut fb = 0;
        let mut color_tex = 0;
        let mut depth_tex = 0;
        
        unsafe {
            // Framebuffer
            gl::GenFramebuffers(1, &mut fb); 
            gl::BindFramebuffer(gl::FRAMEBUFFER, fb);
        
            // Color Texture
            gl::GenTextures(1, &mut color_tex);
            gl::BindTexture(gl::TEXTURE_2D, color_tex);
            gl::TexImage2D(
                gl::TEXTURE_2D, 
                0, 
                specs.color_texture_specs.tex_format.to_opengl() as i32, 
                specs.viewport.2 as i32, 
                specs.viewport.3 as i32, 
                0, 
                specs.color_texture_specs.tex_format.to_opengl(),
                specs.color_texture_specs.tex_type.to_opengl(), 
                std::ptr::null()
            );
            gl::TexParameteri(
                gl::TEXTURE_2D, 
                gl::TEXTURE_MIN_FILTER, 
                specs.color_texture_specs.tex_filter.to_opengl() as i32
            );
            gl::TexParameteri(
                gl::TEXTURE_2D, 
                gl::TEXTURE_MAG_FILTER, 
               specs.color_texture_specs.tex_filter.to_opengl() as i32
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER, 
                gl::COLOR_ATTACHMENT0, 
                gl::TEXTURE_2D, 
                color_tex, 
                0
            );
        
            // Depth Texture
            gl::GenTextures(1, &mut depth_tex);
            gl::BindTexture(gl::TEXTURE_2D, depth_tex);
            gl::TexImage2D(
                gl::TEXTURE_2D, 
                0, 
                gl::DEPTH_COMPONENT as i32, 
                specs.viewport.2 as i32, 
                specs.viewport.3 as i32, 
                0, 
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null()
            );
            gl::TexParameteri(
                gl::TEXTURE_2D, 
                gl::TEXTURE_MIN_FILTER, 
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D, 
                gl::TEXTURE_MAG_FILTER, 
                gl::LINEAR as i32,
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER, 
                gl::DEPTH_ATTACHMENT, 
                gl::TEXTURE_2D, 
                depth_tex, 
                0
            );

            let fb_complete = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if fb_complete != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer is not complete!");
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        
        let depht_tex = if specs.depth_test {
            Some(depth_tex)
        } else { None };

        Self {
            framebuffer: fb,
            color_texture: color_tex,
            depth_texture: depht_tex,
            
            specs,
        }
    }
}
impl Drop for RenderTarget {
    fn drop(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::DeleteFramebuffers(1, &self.framebuffer);
            gl::DeleteTextures(1, &self.color_texture);
            
            if let Some(tex) = self.depth_texture {
                gl::DeleteTextures(1, &tex);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTargetSpecs {
    pub viewport: (i32, i32, i32, i32),
    pub depth_test: bool,
    pub depth_filter: TextureFilter,
    pub color_texture_specs: TextureSpecs,
}