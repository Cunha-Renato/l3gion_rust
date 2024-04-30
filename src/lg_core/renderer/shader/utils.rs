use std::{fs::File, io::{Read, Write}};

use crate::StdError;

use super::ShaderStage;

// Compilation
pub(crate) fn compilation_get_shader_stage(extension: &str) -> Result<ShaderStage, StdError> {
    Ok(match extension {
        "vert" => ShaderStage::VERTEX,
        "frag" => ShaderStage::FRAGMENT,
        _ => return Err("Shader extension not suported! try (.vert/.frag)".into())
    })
}
pub(crate) fn read_folder(src_folder: &str, dst_folder: &str) -> Result<(), StdError> {
    let files: std::fs::ReadDir = std::fs::read_dir(src_folder)?;
    for file in files {
        let file = file?.path();

        if !file.is_file() { continue }

        let file_name = file.file_stem().unwrap().to_str().unwrap();
        let file_extension = match file.extension() {
            Some(ext) => ext.to_str().unwrap(),
            None => continue
        };
        let dst_path = format!("{}/{}.spv", dst_folder, file_name);

        if let Ok(shader_stage) = compilation_get_shader_stage(file_extension) {
            compile(file.to_str().unwrap(), &dst_path, shader_stage)?;
        }
    }
    
    Ok(())
}
fn compile(src_path: &str, dst_path: &str, shader_stage: ShaderStage) -> Result<(), StdError> {
    // Reading
    let mut src_file = File::open(src_path)?;
    let mut src_code = String::new();
    src_file.read_to_string(&mut src_code)?;
    
    // Compiling
    let compiler = match shaderc::Compiler::new() {
        Some(c) => c,
        None => return Err("Failed to create shader compiler! (shader)".into()),
    };    
    let binary = compiler.compile_into_spirv(
        &src_code,
        shader_stage.to_shaderc_stage()?,
        src_path, 
        "main", 
        None
    )?;

    // Writing
    let mut dst_file = File::create(dst_path)?;
    dst_file.write_all(binary.as_binary_u8())?;

    Ok(())
}