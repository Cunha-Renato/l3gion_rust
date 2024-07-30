use egui_winit::pixels_per_point;
use sllog::{info, warn};
use super::renderer::{command::SendRendererCommand, Renderer};

pub struct GUI {
    context: egui::Context,
    egui_winit: egui_winit::State,
    
    viewport_info: egui::ViewportInfo,
    pixels_per_point: f32,
    
    shapes: Vec<egui::epaint::ClippedShape>,
    textures_delta: egui::TexturesDelta,
}
impl GUI {
    pub(crate) fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        pixels_per_point: Option<f32>,
    ) -> Self {
        let context = egui::Context::default();
        let egui_winit = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            pixels_per_point,
            None
        );

        Self { 
            context,
            egui_winit,
            
            viewport_info: Default::default(),
            pixels_per_point: pixels_per_point.unwrap_or(1.0),

            shapes: Vec::default(),
            textures_delta: Default::default(),
        }
    }

    pub(crate) fn on_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.egui_winit.on_window_event(window, event)
    }

    pub(crate) fn run(
        &mut self, 
        window: &winit::window::Window,
        run_ui: impl FnMut(&egui::Context)
    ) {
        let raw_input = self.egui_winit.take_egui_input(window);

        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output,
        } = self.context.run(raw_input, run_ui);

        if viewport_output.len() > 1 {
            warn!("Multiple viewports not yet supported by EguiGlow");
        }
        for (_, egui::ViewportOutput { commands, .. }) in viewport_output {
            let mut actions_requested: ahash::HashSet<egui_winit::ActionRequested> = Default::default();
            egui_winit::process_viewport_commands(
                &self.context,
                &mut self.viewport_info,
                commands,
                window,
                &mut actions_requested,
            );
            for action in actions_requested {
                warn!("{:?} not yet supported by EguiGlow", action);
            }
        }

        self.egui_winit
            .handle_platform_output(window, platform_output);

        self.shapes = shapes;
        self.pixels_per_point = pixels_per_point;
        self.textures_delta.append(textures_delta);
    }

    pub(crate) fn paint(
        &mut self, 
        renderer: &Renderer
    ) {
        let shapes = std::mem::take(&mut self.shapes);
        let textures_delta = std::mem::take(&mut self.textures_delta);

        let pixels_per_point = self.pixels_per_point;
        let clipped_primitives = self.context.tessellate(shapes, pixels_per_point);

        renderer.send(SendRendererCommand::DRAW_GUI(textures_delta, clipped_primitives));
    }
}