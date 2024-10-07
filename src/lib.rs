#![allow(non_camel_case_types)]

pub type StdError = Box<dyn std::error::Error>;
pub mod utils;
pub mod lg_core;

pub use lg_core::timer::LgTimer;
pub use lg_core::lg_types::{self, units_of_time::AsLgTime, reference::Rfc};
pub use lg_core::uuid::UUID;

pub extern crate sllog;
pub extern crate optick;
pub extern crate imgui;
pub extern crate nalgebra_glm as glm;