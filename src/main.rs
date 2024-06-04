use l3gion_rust::{lg_core::test_layer::TestLayer, profile_scope, profiler_begin, profiler_end, L3gion};

fn main() {
    std::env::set_var("LOG", "4");
    let mut l3gion = L3gion::new(l3gion_rust::RendererAPI::OPEN_GL).unwrap();
    l3gion.add_layer(TestLayer::new());
    l3gion.init().unwrap();
    l3gion.run();
}