use std::ops::{Deref, DerefMut};

/// This type behaves as an Option<T>, with the twist that you don't need to unwrap(), but it will panic if is unitialized.
/// 
/// This type was created because I don't like calling .as_ref().unwrap() or as_mut().unwrap() on an Option.
/// I know it goes against the Rust mindset, but honestly I will leave it here.
#[derive(Debug, Default)]
pub enum NCOption<T> {
    Some(T),
    #[default]
    None
}
impl<T> NCOption<T> {
    /// Sets the data as Some(val).
    pub fn set(&mut self, val: T) {
        *self = Self::Some(val);
    }
    
    pub fn is_some(&self) -> bool {
        matches!(*self, Self::Some(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(*self, Self::None)
    }

    /// Will consume NCOption and return the data. 
    /// 
    /// Panics if unitialized.
    pub fn extract(self) -> T {
        match self {
            NCOption::Some(val) => val,
            NCOption::None => panic!("NCOption wasn't initialized!"),
        }
    }
}

impl<T> Into<Option<T>> for NCOption<T> {
    fn into(self) -> Option<T> {
        match self {
            NCOption::Some(val) => Some(val),
            NCOption::None => None,
        }
    }
}

impl<T> From<Option<T>> for NCOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(val) => Self::Some(val),
            None => Self::None,
        }
    }
}

impl<T> Deref for NCOption<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            NCOption::Some(val) => val,
            NCOption::None => panic!("NCOption wasn't initialized!"),
        }
    }
}

impl<T> DerefMut for NCOption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            NCOption::Some(val) => val,
            NCOption::None => panic!("NCOption wasn't initialized!"),
        }
    }
}