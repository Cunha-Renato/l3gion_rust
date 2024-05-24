use l3gion_rust::L3gion;

fn main() {
    let mut l3gion = L3gion::new(l3gion_rust::RendererAPI::OPEN_GL).unwrap();
    l3gion.init().unwrap();
    l3gion.run();
}