#![allow(non_camel_case_types)]

pub type StdError = Box<dyn std::error::Error>;
pub mod utils;
pub mod lg_core;

pub use optick;
pub use imgui;
pub use sllog;
pub use nalgebra_glm as glm;
pub use lg_core::timer::LgTimer;
pub use lg_core::lg_types::{self, units_of_time::AsLgTime};