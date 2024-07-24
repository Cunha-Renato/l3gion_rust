use glutin::{
    config::GlConfig, context::NotCurrentGlContext, display::{
        GetGlDisplay, 
        GlDisplay
    } 
};
use glutin_winit::GlWindow;
use raw_window_handle::HasRawWindowHandle;
use crate::{lg_core::renderer::CreationWindowInfo, StdError};
use super::GlSpecs;

pub(crate) fn init_window(window_info: &CreationWindowInfo) ->
Result<(Option<winit::window::Window>, glutin::config::Config), StdError> {
    let template = glutin::config::ConfigTemplateBuilder::new();

    let window_builder = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize{ 
            width: window_info.width, 
            height: window_info.height 
        })
        .with_title(window_info.title.clone());

    let display_builder = glutin_winit::DisplayBuilder::new()
        .with_window_builder(Some(window_builder));
    
    Ok(display_builder.build(
        &window_info.event_loop.unwrap(), 
        template, 
        gl_config_picker
    )?)
}

pub(crate) fn init_opengl(window: Option<winit::window::Window>, gl_config: glutin::config::Config) -> Result<(winit::window::Window, GlSpecs), StdError>
{
    let window = match window {
        Some(window) => window,
        None => return Err("Failed to create a window! (OpenGL)".into())
    };

    let raw_window_handle = window.raw_window_handle();
    
    let gl_display = gl_config.display();

    let contex_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::OpenGl(Some(glutin::context::Version::new(4, 2))))
        .with_debug(true)
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

pub(crate) fn gl_config_picker(configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>) -> glutin::config::Config {
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