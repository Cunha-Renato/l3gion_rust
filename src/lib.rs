#![allow(non_camel_case_types)]

pub type StdError = Box<dyn std::error::Error>;
pub mod utils;
pub mod lg_core;

pub use optick;
pub use imgui;
pub use lg_core::lg_types::{
    no_check_option::*,
    reference::*,
    no_check_option::*
};