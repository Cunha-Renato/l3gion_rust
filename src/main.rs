use l3gion_rust::lg_core::{application::{ApplicationCreateInfo, L3gion, PersistentApplicationInfo}, renderer::CreationWindowInfo, editor_layer::EditorLayer};

fn main() {
    std::env::set_var("LOG", "4");

    let mut legion = L3gion::new(ApplicationCreateInfo {
        persistant_info: PersistentApplicationInfo { v_sync: false },
        window_info: CreationWindowInfo {
            event_loop: None,
            title: "L3gion".to_string(),
            width: 1080,
            height: 720,
        },
    }).unwrap();
    
    let app = legion.get_app_mut();
    app.push_layer(EditorLayer::new()).unwrap();
    
    legion.run().unwrap();
}