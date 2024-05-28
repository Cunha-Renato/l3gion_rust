use l3gion_rust::{lg_core::test_layer::TestLayer, L3gion};

fn main() {
    let mut l3gion = L3gion::new(l3gion_rust::RendererAPI::OPEN_GL).unwrap();
    l3gion.add_layer(TestLayer::new());
    l3gion.init().unwrap();
    l3gion.run();
}