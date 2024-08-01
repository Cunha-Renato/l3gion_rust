use crate::lg_core::renderer::Renderer;

pub(super) fn config_imgui(renderer: &Renderer) {
    renderer.send(crate::lg_core::renderer::command::SendRendererCommand::SET_FONT(include_bytes!("resources/fonts/roboto/Roboto-Regular.ttf").to_vec(), 17.0));

    let mut core = renderer.core();
    let imgui_context = core.imgui().context();

    imgui_context.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE
        | imgui::ConfigFlags::VIEWPORTS_ENABLE;

    if imgui_context.io().config_flags.contains(imgui::ConfigFlags::VIEWPORTS_ENABLE) {
        imgui_context.style_mut().window_rounding = 0.0;
    }
    
    set_l3gion_theme(imgui_context);
}

fn set_l3gion_theme(context: &mut imgui::Context) {
    let orange = [0.8, 0.4, 0.21, 1.0];
    let faded_orange = [0.8, 0.5, 0.31, 1.0];
    let dark = [0.09, 0.07, 0.07, 1.0];
    let bright = [0.2, 0.18, 0.18, 1.0];

    let colors = &mut context.style_mut().colors;
    colors[imgui::StyleColor::WindowBg as usize] = dark;
    colors[imgui::StyleColor::ChildBg as usize] = dark;
    colors[imgui::StyleColor::MenuBarBg as usize] = dark;

    // Headers
    colors[imgui::StyleColor::Header as usize] = bright;
    colors[imgui::StyleColor::HeaderHovered as usize] = orange;
    colors[imgui::StyleColor::HeaderActive as usize] = faded_orange;

    // Buttons
    colors[imgui::StyleColor::Button as usize] = bright;
    colors[imgui::StyleColor::ButtonHovered as usize] = orange;
    colors[imgui::StyleColor::ButtonActive as usize] = faded_orange;

    // Frame BG
    colors[imgui::StyleColor::FrameBg as usize] = bright;
    colors[imgui::StyleColor::FrameBgHovered as usize] = bright;
    colors[imgui::StyleColor::FrameBgActive as usize] = orange;

    // Tabs
    colors[imgui::StyleColor::Tab as usize] = dark;
    colors[imgui::StyleColor::TabHovered as usize] = faded_orange;
    colors[imgui::StyleColor::TabActive as usize] = orange;
    colors[imgui::StyleColor::TabUnfocused as usize] = dark;
    colors[imgui::StyleColor::TabUnfocusedActive as usize] = dark;

    // Title
    colors[imgui::StyleColor::TitleBg as usize] = dark;
    colors[imgui::StyleColor::TitleBgActive as usize] = dark;
    colors[imgui::StyleColor::TitleBgCollapsed as usize] = dark;

    // Resize
    colors[imgui::StyleColor::ResizeGrip as usize] = bright;
    colors[imgui::StyleColor::ResizeGripHovered as usize] = orange;
    colors[imgui::StyleColor::ResizeGripActive as usize] = faded_orange;

    colors[imgui::StyleColor::Separator as usize] = bright;
    colors[imgui::StyleColor::SeparatorHovered as usize] = orange;
    colors[imgui::StyleColor::SeparatorActive as usize] = faded_orange;

    // Navigation
    colors[imgui::StyleColor::NavWindowingHighlight as usize] = orange;
    colors[imgui::StyleColor::ScrollbarGrabActive as usize] = orange;
    colors[imgui::StyleColor::NavHighlight as usize] = orange;

    // Tools
    colors[imgui::StyleColor::PopupBg as usize] = dark;
    colors[imgui::StyleColor::DockingPreview as usize] = bright;
    colors[imgui::StyleColor::DockingEmptyBg as usize] = dark;
    colors[imgui::StyleColor::CheckMark as usize] = orange;
}