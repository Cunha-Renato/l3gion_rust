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
    /// If the value is too big and the unit is too small, losses may occur, the oposite is also true.
    pub fn new(unit: TIME_UNIT, value: f64) -> Self {
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

    pub fn unit(&self) -> TIME_UNIT {
        self.unit
    }

    /// Returns the seconds regardles of TIME_UNIT. No conversions or overhead.
    pub fn get_seconds(&self) -> f64 {
        self.value
    }

    /// Converts the value (SEC) into the specified TIME_UNIT, this function can overflow the f64 type, for example if you are working with a very high number of hours and try to convert to nanos.
    pub fn get_unit_value(&self) -> f64 {
        match self.unit {
            TIME_UNIT::HOUR => self.value * HOUR_IN_SECOND,
            TIME_UNIT::MIN => self.value * MIN_IN_SECOND,
            TIME_UNIT::SEC => self.value,
            TIME_UNIT::MILLIS => self.value * MILLIS_IN_SECOND,
            TIME_UNIT::MICRO => self.value * MICRO_IN_SECOND,
            TIME_UNIT::NANO => self.value * NANO_IN_SECOND,
        }
    }

    /// Simply returns a new LgTime with the same value but the unit changed to HOUR.
    pub fn h(&self) -> Self {
        Self {
            unit: TIME_UNIT::HOUR,
            value: self.value
        }
    }
    
    /// Simply returns a new LgTime with the same value but the unit changed to MIN.
    pub fn mi(&self) -> Self {
        Self {
            unit: TIME_UNIT::MIN,
            value: self.value
        }
    }
   
    /// Simply returns a new LgTime with the same value but the unit changed to SEC.
    pub fn s(&self) -> Self {
        *self
    }
    
    /// Simply returns a new LgTime with the same value but the unit changed to MILLIS.
    pub fn ms(&self) -> Self {
        Self {
            unit: TIME_UNIT::MILLIS,
            value: self.value
        }
    }
    
    /// Simply returns a new LgTime with the same value but the unit changed to MICRO.
    pub fn us(&self) -> Self {
        Self {
            unit: TIME_UNIT::MICRO,
            value: self.value
        }
    }
    
    /// Simply returns a new LgTime with the same value but the unit changed to NANO.
    pub fn ns(&self) -> Self {
        Self {
            unit: TIME_UNIT::NANO,
            value: self.value
        }
    }
}
impl Debug for LgTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LgTime")
            .field("unit", &self.unit)
            .field("seconds", &self.value)
            .field("converted", &self.get_unit_value())
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
        (self.value - other.value).abs() < f64::EPSILON
    }
}
impl PartialOrd for LgTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl From<std::time::Duration> for LgTime {
    fn from(value: std::time::Duration) -> Self {
        todo!()
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
impl Mul<f64> for LgTime {
    type Output = LgTime;

    fn mul(self, rhs: f64) -> Self::Output {
        let value = self.value * rhs;
        
        Self::Output {
            unit: self.unit,
            value,
        }
    }
}
impl Div<f64> for LgTime {
    type Output = LgTime;

    fn div(self, rhs: f64) -> Self::Output {
        let value = self.value / rhs;
        
        Self::Output {
            unit: self.unit,
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
                LgTime::new(TIME_UNIT::HOUR, *self as f64)
            }
            fn mi(&self) -> LgTime {
                LgTime::new(TIME_UNIT::MIN, *self as f64)
            }
            fn s(&self) -> LgTime {
                LgTime::new(TIME_UNIT::SEC, *self as f64)
            }
            fn ms(&self) -> LgTime {
                LgTime::new(TIME_UNIT::MILLIS, *self as f64)
            }
            fn us(&self) -> LgTime {
                LgTime::new(TIME_UNIT::MICRO, *self as f64)
            }
            fn ns(&self) -> LgTime {
                LgTime::new(TIME_UNIT::NANO, *self as f64)
            }
        }
    };
}

macro_rules! impl_mul_div_lgtime{
    ($p_type:tt) => {
        impl Mul<LgTime> for $p_type {
            type Output = $p_type;
            
            fn mul(self, rhs: LgTime) -> Self::Output {
                self * rhs.get_seconds() as $p_type
            }
        }
        
        impl Div<LgTime> for $p_type {
            type Output = $p_type;
            
            fn div(self, rhs: LgTime) -> Self::Output {
                self / rhs.get_seconds() as $p_type
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

impl_mul_div_lgtime!(f32);
impl_mul_div_lgtime!(f64);
impl_mul_div_lgtime!(u8);
impl_mul_div_lgtime!(u16);
impl_mul_div_lgtime!(u32);
impl_mul_div_lgtime!(u64);
impl_mul_div_lgtime!(u128);
impl_mul_div_lgtime!(i8);
impl_mul_div_lgtime!(i16);
impl_mul_div_lgtime!(i32);
impl_mul_div_lgtime!(i64);
impl_mul_div_lgtime!(i128);