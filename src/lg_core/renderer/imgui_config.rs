pub(super) fn imgui_init(window: &winit::window::Window) -> (imgui::Context, imgui_winit_support::WinitPlatform) {
    let mut imgui_context = imgui::Context::create();

    // .ini file
    imgui_context.set_ini_filename(None);
    
    if let Ok(ini_contents) = std::fs::read_to_string("imgui.ini") {
        imgui_context.load_ini_settings(&ini_contents);
    }

    let mut imgui_winit = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
    imgui_winit.attach_window(
        imgui_context.io_mut(), 
        &window, 
        imgui_winit_support::HiDpiMode::Rounded
    );
    imgui_context.io_mut().font_global_scale = (1.0 / imgui_winit.hidpi_factor()) as f32;
    
    (imgui_context, imgui_winit)
}