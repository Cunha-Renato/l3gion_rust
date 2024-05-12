use std::{borrow::Borrow, mem::size_of};

use crate::{lg_core::input::LgInput, StdError};

use super::{
    application::ApplicationCore, entity::LgEntity, event::{LgEvent, MouseEvent}, lg_types::reference::Rfc, renderer::{
        material::LgMaterial, mesh::LgMesh, shader::LgShader, texture::LgTexture, vertex::Vertex
    }
};
use lg_renderer::renderer::lg_uniform::{GlUniform, LgUniform};
use nalgebra_glm as glm;

struct TexStorage {
    grid: Rfc<LgTexture>,
    viking: Rfc<LgTexture>,
}
impl TexStorage {
    fn new() -> Result<Self, StdError> {
        Ok(Self {
            grid: Rfc::new(LgTexture::new("resources/textures/grid.png")?),
            viking: Rfc::new(LgTexture::new("resources/textures/viking.png")?),
        })
    }
}

struct ShaderStorage {
    std_v: Rfc<LgShader>,
    std_f: Rfc<LgShader>,
    uniform_v: Rfc<LgShader>,
    uniform_f: Rfc<LgShader>,
    obj_picker_v: Rfc<LgShader>,
    obj_picker_f: Rfc<LgShader>,
}
impl ShaderStorage {
    fn new() -> Self {
        Self {
            std_v: Rfc::new(LgShader::builder()
                .stage(lg_renderer::renderer::lg_shader::ShaderStage::VERTEX)
                .src_code(std::path::Path::new("resources/shaders/src/std_v.vert")).unwrap()
                .build()
            ),
            std_f: Rfc::new(LgShader::builder()
                .stage(lg_renderer::renderer::lg_shader::ShaderStage::FRAGMENT)
                .src_code(std::path::Path::new("resources/shaders/src/std_f.frag")).unwrap()
                .build()
            ),
            uniform_v: Rfc::new(LgShader::builder()
                .stage(lg_renderer::renderer::lg_shader::ShaderStage::VERTEX)
                .src_code(std::path::Path::new("resources/shaders/src/uniform_v.vert")).unwrap()
                .build()
            ),
            uniform_f: Rfc::new(LgShader::builder()
                .stage(lg_renderer::renderer::lg_shader::ShaderStage::FRAGMENT)
                .src_code(std::path::Path::new("resources/shaders/src/uniform_f.frag")).unwrap()
                .build()
            ),
            obj_picker_v: Rfc::new(LgShader::builder()
                .stage(lg_renderer::renderer::lg_shader::ShaderStage::VERTEX)
                .src_code(std::path::Path::new("resources/shaders/src/obj_picker_v.vert")).unwrap()
                .build()
            ),
            obj_picker_f: Rfc::new(LgShader::builder()
                .stage(lg_renderer::renderer::lg_shader::ShaderStage::FRAGMENT)
                .src_code(std::path::Path::new("resources/shaders/src/obj_picker_f.frag")).unwrap()
                .build()
            )
        }
    }
}

struct MaterialStorage {
    grid: Rfc<LgMaterial>,
    viking: Rfc<LgMaterial>,
    red: Rfc<LgMaterial>,
    uniform_color: Rfc<LgMaterial>,
    obj_picker: Rfc<LgMaterial>,
}
impl MaterialStorage {
    fn new(shader_storage: &ShaderStorage, tex_storage: &TexStorage) -> Self {
        let grid = Rfc::new(LgMaterial::new(vec![
                shader_storage.std_v.clone(),
                shader_storage.std_f.clone(),
            ],
            Some(tex_storage.grid.clone()),
            Vec::new()
        ));
        let viking = Rfc::new(LgMaterial::new(vec![
                shader_storage.std_v.clone(),
                shader_storage.std_f.clone(),
            ],
            Some(tex_storage.viking.clone()),
            Vec::new(),
        ));
        let red = Rfc::new(LgMaterial::new(vec![
                shader_storage.std_v.clone(),
                shader_storage.std_f.clone(),
            ],
            Some(tex_storage.viking.clone()),
            Vec::new(),
        ));
        let uniform_color = Rfc::new(LgMaterial::new(vec![
                shader_storage.uniform_v.clone(),
                shader_storage.uniform_f.clone(),
            ],
            None,
            Vec::new(),
        ));
        let obj_picker = Rfc::new(LgMaterial::new(vec![
                shader_storage.obj_picker_v.clone(),
                shader_storage.obj_picker_f.clone(),
            ],
            None,
            Vec::new(),
        ));
        
        Self {
            grid,
            viking,
            red,
            uniform_color,
            obj_picker,
        }
    }
}

struct MeshStorage {
    big_quad: Rfc<LgMesh>,
    med_quad: Rfc<LgMesh>,
    smol_quad: Rfc<LgMesh>,
}
impl MeshStorage {
    fn new() -> Self {
        let big_quad = Rfc::new(LgMesh::new(
            vec![
                Vertex { position: glm::vec2(-0.5, -0.5), tex_coord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec2( 0.5, -0.5), tex_coord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec2( 0.5,  0.5), tex_coord: glm::vec2(1.0, 0.0) },
                Vertex { position: glm::vec2(-0.5,  0.5), tex_coord: glm::vec2(0.0, 0.0) },
            ], 
            vec![
                0, 1, 2,
                2, 3, 0
            ]
        ));
        let med_quad = Rfc::new(LgMesh::new(
            vec![
                Vertex { position: glm::vec2(-0.3, -0.3), tex_coord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec2( 0.3, -0.3), tex_coord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec2( 0.3,  0.3), tex_coord: glm::vec2(1.0, 0.0) },
                Vertex { position: glm::vec2(-0.3,  0.3), tex_coord: glm::vec2(0.0, 0.0) },
            ], 
            vec![
                0, 1, 2,
                2, 3, 0
            ]
        ));
        let smol_quad = Rfc::new(LgMesh::new(
            vec![
                Vertex { position: glm::vec2(-0.15, -0.15), tex_coord: glm::vec2(0.0, 1.0) },
                Vertex { position: glm::vec2( 0.15, -0.15), tex_coord: glm::vec2(1.0, 1.0) },
                Vertex { position: glm::vec2( 0.15,  0.15), tex_coord: glm::vec2(1.0, 0.0) },
                Vertex { position: glm::vec2(-0.15,  0.15), tex_coord: glm::vec2(0.0, 0.0) },
            ], 
            vec![
                0, 1, 2,
                2, 3, 0
            ]
        ));
        
        Self {
            big_quad,
            med_quad,
            smol_quad,
        }
    }
}

#[repr(C)]
struct UBO {
    data: glm::Vec4,
}
impl GlUniform for UBO {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone)]
struct SSBO {
    data: glm::Vec4,
}
impl GlUniform for SSBO {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct Data {
    mouse_position: glm::Vec2,    
    uuid: u32,
}
impl GlUniform for Data {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct TestScene {
    app_core: Rfc<ApplicationCore>,
    meshes: MeshStorage,
    materials: MaterialStorage,
    textures: TexStorage,
    shaders: ShaderStorage,
    
    big: LgEntity,
    smol: LgEntity,
}
impl TestScene {
    pub fn new(app_core: Rfc<ApplicationCore>) -> Self {
        let textures = TexStorage::new().unwrap();
        let shaders = ShaderStorage::new();
        let materials = MaterialStorage::new(&shaders, &textures);
        let meshes = MeshStorage::new();

        let big = LgEntity::new(meshes.big_quad.clone(), materials.grid.clone()).unwrap();
        let mut smol = LgEntity::new(meshes.med_quad.clone(), materials.viking.clone()).unwrap();
        
        // Setting the uniform for smol
        let mut data = Data {
            mouse_position: glm::vec2(0.0, 0.0),
            uuid: 10,
        };
        smol.uniforms.push(LgUniform::new(
            "data", 
            lg_renderer::renderer::lg_uniform::LgUniformType::STRUCT, 
            0, 
            0, 
            data.clone()
        ));
        data.uuid = 20;
        
        // Setting the uniform for obj_picker material
        let ssbo = SSBO {
            data: glm::Vec4::new(0.0, 0.0, 0.0, 0.0),
        };
        materials.obj_picker
            .borrow_mut()
            .uniforms = vec![LgUniform::new(
                "ssbo", 
                lg_renderer::renderer::lg_uniform::LgUniformType::STORAGE_BUFFER, 
                2, 
                0, 
                ssbo
            ),
        ];

        Self {
            app_core,
            meshes,
            materials,
            textures,
            shaders,
            
            big,
            smol,
        }
    }
    pub fn init(&mut self) {
    }
    fn update_entity(&mut self) {
        let mut data = Data {
            mouse_position: LgInput::get().unwrap().get_mouse_position(),
            uuid: 10,
        };
        data.mouse_position.y = self.app_core.borrow().window.borrow().size().1 as f32 - data.mouse_position.y;
        self.smol.uniforms[0] = LgUniform::new(
            "data", 
            lg_renderer::renderer::lg_uniform::LgUniformType::STRUCT, 
            0, 
            0, 
            data.clone()
        );
        data.uuid = 20;
    }
    pub fn on_update(&mut self) {
        self.update_entity();
    }
    pub fn on_event(&mut self, event: &LgEvent) {
        match event {
            LgEvent::WindowEvent(_) => (),
                LgEvent::KeyEvent(_) => (),
                LgEvent::MouseEvent(event) => {
                    if let MouseEvent::ButtonEvent(_) = event {
                        /* let ssbo = unsafe { self.app_core.borrow()
                            .renderer.borrow_mut()
                            .read_buffer::<SSBO>(&self.materials.obj_picker.borrow(), 0)
                            .unwrap()
                        };
                        
                        println!("{:?}", ssbo.data); */
                    }
                },
        }
    }
    pub fn destroy(&mut self) {

    }
}