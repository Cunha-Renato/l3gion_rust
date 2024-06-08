#![allow(non_camel_case_types)]

pub type StdError = Box<dyn std::error::Error>;
pub mod utils;
pub mod lg_core;