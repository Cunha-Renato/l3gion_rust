use std::{fmt::Debug, ops::{Add, Div, Mul, Sub}};

pub const HOUR_IN_SECOND:   f64 = MIN_IN_SECOND / 60.0;
pub const MIN_IN_SECOND:    f64 = 1.0 / 60.0;
pub const MILLIS_IN_SECOND: f64 = 1_000.0;
pub const MICRO_IN_SECOND:  f64 = 1_000_000.0;
pub const NANO_IN_SECOND:   f64 = 1_000_000_000.0;

#[derive(Debug, Clone, Copy)]
pub enum TIME_UNIT {
    HOUR,
    MIN,
    SEC,
    MILLIS,
    MICRO,
    NANO,
}

/// The standard unit of time is SECONDS.
/// So the value is always stored as SECONDS.
#[derive(Clone, Copy)]
pub struct LgTime {
    unit: TIME_UNIT,
    value: f64,
}
impl LgTime {
    pub fn unit(&self) -> TIME_UNIT {
        self.unit
    }

    /// Returns the seconds regardles of TIME_UNIT
    pub fn seconds(&self) -> f64 {
        self.value
    }

    /// Converts the value (SEC) into the specified TIME_UNIT
    pub fn value(&self) -> f64 {
        match self.unit {
            TIME_UNIT::HOUR => self.value * HOUR_IN_SECOND,
            TIME_UNIT::MIN => self.value * MIN_IN_SECOND,
            TIME_UNIT::SEC => self.value,
            TIME_UNIT::MILLIS => self.value * MILLIS_IN_SECOND,
            TIME_UNIT::MICRO => self.value * MICRO_IN_SECOND,
            TIME_UNIT::NANO => self.value * NANO_IN_SECOND,
        }
    }

    /// If the value is too big and the unit is too small, losses may occur
    pub fn from(unit: TIME_UNIT, value: f64) -> Self {
        let value = match unit {
            TIME_UNIT::HOUR => value / HOUR_IN_SECOND,
            TIME_UNIT::MIN => value / MIN_IN_SECOND,
            TIME_UNIT::SEC => value,
            TIME_UNIT::MILLIS => value / MILLIS_IN_SECOND,
            TIME_UNIT::MICRO => value / MICRO_IN_SECOND,
            TIME_UNIT::NANO => value / NANO_IN_SECOND,
        };
        
        Self {
            unit,
            value,
        }
    }
    
    pub fn as_hour(&mut self) {
        self.unit = TIME_UNIT::HOUR;
    }

    pub fn as_minutes(&mut self) {
        self.unit = TIME_UNIT::MIN;
    }
    
    pub fn as_sec(&mut self) {
        self.unit = TIME_UNIT::SEC;
    }
    
    pub fn as_milis(&mut self) {
        self.unit = TIME_UNIT::MILLIS;
    }
    
    pub fn as_micro(&mut self) {
        self.unit = TIME_UNIT::MICRO;
    }
    
    pub fn as_nano(&mut self) {
        self.unit = TIME_UNIT::MICRO;
    }

    pub fn h(&self) -> Self {
        Self {
            unit: TIME_UNIT::HOUR,
            value: self.value
        }
    }
    
    pub fn mi(&self) -> Self {
        Self {
            unit: TIME_UNIT::MIN,
            value: self.value
        }
    }
   
    pub fn s(&self) -> Self {
        Self {
            unit: TIME_UNIT::SEC,
            value: self.value,
        }
    }
    
    pub fn ms(&self) -> Self {
        Self {
            unit: TIME_UNIT::MILLIS,
            value: self.value,
        }
    }
    
    pub fn us(&self) -> Self {
        Self {
            unit: TIME_UNIT::MICRO,
            value: self.value,
        }
    }
    
    pub fn ns(&self) -> Self {
        Self {
            unit: TIME_UNIT::NANO,
            value: self.value,
        }
    }
}
impl Debug for LgTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LgTime")
            .field("unit", &self.unit)
            .field("in seconds", &self.value)
            .finish()
    }
}
impl Default for LgTime {
    fn default() -> Self {
        Self { 
            unit: TIME_UNIT::SEC, 
            value: 0.0 
        }
    }
}
impl PartialEq for LgTime {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Add for LgTime {
    type Output = LgTime;

    fn add(self, rhs: Self) -> Self::Output {
        let value = self.value + rhs.value;
        
        Self::Output {
            unit: TIME_UNIT::SEC,
            value,
        }
    }
}
impl Sub for LgTime {
    type Output = LgTime;

    fn sub(self, rhs: Self) -> Self::Output {
        let value = self.value - rhs.value;
        
        Self::Output {
            unit: TIME_UNIT::SEC,
            value,
        }
    }
}
impl Mul for LgTime {
    type Output = LgTime;

    fn mul(self, rhs: Self) -> Self::Output {
        let value = self.value * rhs.value;
        
        Self::Output {
            unit: TIME_UNIT::SEC,
            value,
        }
    }
}
impl Div for LgTime {
    type Output = LgTime;

    fn div(self, rhs: Self) -> Self::Output {
        let value = self.value / rhs.value;
        
        Self::Output {
            unit: TIME_UNIT::SEC,
            value,
        }
    }
}

pub trait AsLgTime {
    fn h(&self) -> LgTime;
    fn mi(&self) -> LgTime;
    fn s(&self) -> LgTime;
    fn ms(&self) -> LgTime;
    fn us(&self) -> LgTime;
    fn ns(&self) -> LgTime;
}

macro_rules! impl_aslgtime {
    ($p_type:tt) => {
        impl AsLgTime for $p_type {
            fn h(&self) -> LgTime {
                LgTime::from(TIME_UNIT::HOUR, *self as f64)
            }
            fn mi(&self) -> LgTime {
                LgTime::from(TIME_UNIT::MIN, *self as f64)
            }
            fn s(&self) -> LgTime {
                LgTime::from(TIME_UNIT::SEC, *self as f64)
            }
            fn ms(&self) -> LgTime {
                LgTime::from(TIME_UNIT::MILLIS, *self as f64)
            }
            fn us(&self) -> LgTime {
                LgTime::from(TIME_UNIT::MICRO, *self as f64)
            }
            fn ns(&self) -> LgTime {
                LgTime::from(TIME_UNIT::NANO, *self as f64)
            }
        }
    };
}

impl_aslgtime!(f32);
impl_aslgtime!(f64);
impl_aslgtime!(u8);
impl_aslgtime!(u16);
impl_aslgtime!(u32);
impl_aslgtime!(u64);
impl_aslgtime!(u128);
impl_aslgtime!(i8);
impl_aslgtime!(i16);
impl_aslgtime!(i32);
impl_aslgtime!(i64);
impl_aslgtime!(i128);