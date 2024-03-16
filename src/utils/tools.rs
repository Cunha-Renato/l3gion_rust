use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::os::raw::c_char;

/// Helper function to convert [c_char; SIZE] to string
pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    // Implementation 1
    //    let end = '\0' as u8;
    //
    //    let mut content: Vec<u8> = vec![];
    //
    //    for ch in raw_string_array.iter() {
    //        let ch = (*ch) as u8;
    //
    //        if ch != end {
    //            content.push(ch);
    //        } else {
    //            break
    //        }
    //    }
    //
    //    String::from_utf8(content)
    //        .expect("Failed to convert vulkan raw string")

    // Implementation 2
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}

pub fn str_array_to_c_char_array(origin: &[&str]) -> Vec<*const c_char>
{
    let mut result = Vec::with_capacity(origin.len());    
    
    // Possible overflow
    for val in origin
    {
        result.push(val.as_ptr() as *const c_char);
    }
    
    result
}

pub fn shader_spirv(path: &str) -> Result<Vec<u32>, io::Error> {
    let spv_file = File::open(path)?;
    let mut buf_reader = BufReader::new(spv_file);
    let mut spv_data = Vec::new();

    loop {
        let mut buffer = [0; 4];  // Read 4 bytes at a time (size of u32)
        match buf_reader.read_exact(&mut buffer) {
            Ok(()) => {
                let value = u32::from_le_bytes(buffer);
                spv_data.push(value);
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }

    Ok(spv_data)
}