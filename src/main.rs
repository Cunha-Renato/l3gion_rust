use l3gion_rust::{lg_core::{application::{ApplicationCreateInfo, L3gion, PersistentApplicationInfo}, test_layer::TestLayer}, profiler_begin, profiler_end};

fn main() {
    std::env::set_var("LOG", "4");

    let mut legion = L3gion::new(ApplicationCreateInfo {
        persistant_info: PersistentApplicationInfo { v_sync: false },
        renderer_api: lg_renderer::renderer::CreationApiInfo::OPEN_GL,
        window_info: lg_renderer::renderer::CreationWindowInfo::new("L3gion", 1080, 720),
    }).unwrap();
    
    let app = legion.get_app_mut();
    app.push_layer(TestLayer::new()).unwrap();
    
    legion.run().unwrap();
}