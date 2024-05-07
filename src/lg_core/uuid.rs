use rand::Rng;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq)]
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
    pub const fn to_u32_4(&self) -> [u32; 4] {
        [
            (self.0 & 0xFFFF_FFFF_FFFF_FFFF) as u32,
            ((self.0 >> 32) & 0xFFFF_FFFF_FFFF_FFFF) as u32,
            ((self.0 >> 64) & 0xFFFF_FFFF_FFFF_FFFF) as u32,
            ((self.0 >> 96) & 0xFFFF_FFFF_FFFF_FFFF) as u32
        ]
    }
    pub const fn from_u32_4(val: [u32; 4]) -> Self {
        Self(((val[0] as u128) << 96) 
            | ((val[1] as u128) << 64) 
            | ((val[2] as u128) << 32) 
            | (val[3] as u128)
        )
    }
    pub const fn from_u128(val: u128) -> Self {
        Self(val)
    }
}