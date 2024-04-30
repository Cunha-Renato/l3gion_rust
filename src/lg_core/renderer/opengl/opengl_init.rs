use glutin::{config::GlConfig, context::NotCurrentGlContext, display::{GetGlDisplay, GlDisplay}};
use glutin_winit::GlWindow;
use sllog::info;
use raw_window_handle::HasRawWindowHandle;

use crate::StdError;

use super::opengl_renderer::GlSpecs;

pub fn init_opengl(event_loop: &winit::event_loop::EventLoop<()>, window_builder: winit::window::WindowBuilder) -> Result<(winit::window::Window, GlSpecs), StdError>{
    let template = glutin::config::ConfigTemplateBuilder::new();

    let display_builder = glutin_winit::DisplayBuilder::new()
        .with_window_builder(Some(window_builder));
    
    let (window, gl_config) = display_builder.build(
        event_loop, 
        template, 
        gl_config_picker
    )?;
    
    info!("Picked a config with {} samples. (OpenGL)", gl_config.num_samples());
    
    let window = match window {
        Some(window) => window,
        None => return Err("Failed to create a window! (OpenGL)".into())
    };

    let raw_window_handle = window.raw_window_handle();
    
    let gl_display = gl_config.display();

    let contex_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3))))
        .build(Some(raw_window_handle));

    let (gl_context, gl_surface) = unsafe { 
        let attrs = window.build_surface_attributes(Default::default());

        let gl_surface = gl_config.display().create_window_surface(&gl_config, &attrs)?;

        (gl_display.create_context(&gl_config, &contex_attributes)?.make_current(&gl_surface)?, gl_surface)
    };
    
    Ok((window, GlSpecs{
        gl_context,
        gl_surface,
        gl_display,
    }))
}

pub fn gl_config_picker(configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>) -> glutin::config::Config {
    configs
        .reduce(|accum, config| {
            if config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}