use l3gion_rust::lg_core::{application::{ApplicationCreateInfo, L3gion, PersistentApplicationInfo}, renderer::CreationWindowInfo};

fn main() {
    std::env::set_var("LOG", "4");

    let legion = L3gion::new(ApplicationCreateInfo {
        persistant_info: PersistentApplicationInfo { v_sync: false },
        window_info: CreationWindowInfo {
            event_loop: None,
            title: "L3gion".to_string(),
            width: 1080,
            height: 720,
        },
    }).unwrap();
    
    legion.run().unwrap();
}