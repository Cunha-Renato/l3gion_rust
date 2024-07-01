use crate::lg_core::{glm, uuid::UUID};

// --------------------------------------- PROPERTIES ---------------------------------------
pub(crate) const WINDOW_TITTLE_COLOR_FOCUSED: glm::Vec4 = glm::Vec4::new(0.16, 0.29, 0.48, 1.0);
pub(crate) const WINDOW_TITTLE_COLOR_UNFOCUSED: glm::Vec4 = glm::Vec4::new(0.04, 0.04, 0.04, 1.0);
pub(crate) const WINDOW_TITTLE_HEIGHT: f32 = 25.0;

pub(crate) const WINDOW_COLOR_FOCUSED: glm::Vec4 = glm::Vec4::new(0.08, 0.08, 0.08, 1.0);
pub(crate) const WINDOW_COLOR_UNFOCUSED: glm::Vec4 = glm::Vec4::new(0.05, 0.05, 0.05, 1.0);

// --------------------------------------- MATERIALS ---------------------------------------
// WINDOW
pub(crate) const WINDOW_MATERIAL: UUID = UUID::from_u128(4);

// --------------------------------------- MESHES ---------------------------------------
// WINDOW
pub(crate) const WINDOW_MESH: UUID = UUID::from_u128(252411435688744967694609164507863584779);