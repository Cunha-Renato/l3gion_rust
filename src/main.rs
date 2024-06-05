use l3gion_rust::{lg_core::test_layer::TestLayer, L3gion};

fn main() {
    std::env::set_var("LOG", "4");
    let mut l3gion = L3gion::new(l3gion_rust::RendererAPI::OPEN_GL).unwrap();
    l3gion.push_layer(TestLayer::new()).unwrap();
    l3gion.run();
}