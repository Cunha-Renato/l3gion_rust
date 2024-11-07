use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::StdError;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct UUID(u128);
impl UUID {
    pub fn generate() -> Self {
        let uuid = rand::thread_rng().gen::<u128>();

        // 0 is reserved / invalid
        if uuid == 0u128 { Self::generate() }
        else { Self(uuid) }
    }
    pub const fn get_value(&self) -> u128 {
        self.0
    }
    pub const fn is_valid(&self) -> bool {
        self.0 != 0u128
    }
    // TODO: UNTESTED!
    /* pub const fn to_u32_4(&self) -> [u32; 4] {
        [
            (self.0 & 0xFFFF_FFFF_FFFF_FFFF) as u32,
            ((self.0 >> 32) & 0xFFFF_FFFF_FFFF_FFFF) as u32,
            ((self.0 >> 64) & 0xFFFF_FFFF_FFFF_FFFF) as u32,
            ((self.0 >> 96) & 0xFFFF_FFFF_FFFF_FFFF) as u32
        ]
    } */
    // TODO: UNTESTED!
    /* pub const fn from_u32_4(val: [u32; 4]) -> Self {
        Self(((val[0] as u128) << 96) 
            | ((val[1] as u128) << 64) 
            | ((val[2] as u128) << 32) 
            | (val[3] as u128)
        )
    } */
    pub const fn from_u128(val: u128) -> Self {
        Self(val)
    }
    pub fn from_string(str: &str) -> Result<Self, StdError> {
        if str.is_empty() {
            return Ok(Self(0));
        }

        let mut hasher = sha2::Sha256::new();
        hasher.update(str);
        let bytes = hasher.finalize()[0..16].try_into()?;
        
        Ok(Self(u128::from_be_bytes(bytes)))
    }
}
impl Default for UUID {
    fn default() -> Self {
        Self(0)
    }
}
impl std::fmt::Display for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}