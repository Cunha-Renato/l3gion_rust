use l3gion_rust::lg_core::{application::{ApplicationCreateInfo, L3gion}, renderer::CreationWindowInfo};

fn main() {
    std::env::set_var("LOG", "4");

    let legion = L3gion::new(ApplicationCreateInfo {
        window_info: CreationWindowInfo {
            event_loop: None,
            title: "L3gion".to_string(),
            width: 1080,
            height: 720,
            vsync: true,
        },
    }).unwrap();
    
    legion.run().unwrap();
}