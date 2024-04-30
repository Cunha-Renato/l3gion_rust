use super::{
    application::ApplicationCore, 
    event::LgEvent, 
    lg_types::reference::Rfc, 
    renderer::{
        material::LgMaterial, 
        mesh::LgMesh, 
        shader::LgShader, 
        vertex::Vertex, 
        Renderer
    }
};
use nalgebra_glm as glm;

pub struct TestScene {
    app_core: Rfc<ApplicationCore>,
    triangle: LgMesh<Vertex>,
    red: LgMaterial,
}
impl TestScene {
    pub fn new(core: Rfc<ApplicationCore>) -> Self {
        let triangle = LgMesh::new(
            vec![
                Vertex { position: glm::vec2(-0.5, -0.5) },
                Vertex { position: glm::vec2( 0.5, -0.5) },
                Vertex { position: glm::vec2( 0.0,  0.5) },
            ], 
            vec![]
        );
        let red = LgMaterial::new(vec![
            Rfc::new(LgShader::builder()
                .stage(super::renderer::shader::ShaderStage::VERTEX)
                .src_code(std::path::Path::new("resources/shaders/src/std_v.vert")).unwrap()
                .build()
            ),
            Rfc::new(LgShader::builder()
                .stage(super::renderer::shader::ShaderStage::FRAGMENT)
                .src_code(std::path::Path::new("resources/shaders/src/std_f.frag")).unwrap()
                .build()
            )
        ]);

        Self {
            triangle,
            red,
            app_core: core,
        } 
    }
    pub fn init(&mut self) {
        
    }
    pub fn on_update(&mut self) {
        unsafe {
            self.app_core.borrow_mut().renderer.borrow_mut().draw(
                &self.triangle,
                &self.red
            ).unwrap();
        }
    }
    pub fn on_event(&mut self, event: &LgEvent) {
        
    }
    pub fn destroy(&mut self) {

    }
}