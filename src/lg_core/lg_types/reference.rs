use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Default, Debug, PartialEq)]
pub struct Rfc<T: ?Sized> {
    data: Rc<RefCell<T>>,
}

impl<T> Rfc<T> {
    pub fn new(value: T) -> Self {
        Rfc {
            data: Rc::new(RefCell::new(value)),
        }
    }
    pub fn downgrade(&self) -> Weak<RefCell<T>> {
        Rc::downgrade(&self.data)
    }
}
impl<T: ?Sized> Rfc<T> {
    pub fn borrow(&self) -> std::cell::Ref<'_, T> {
        self.data.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, T> {
        self.data.borrow_mut()
    }
    pub fn from_rc_refcell(val: &Rc<RefCell<T>>) -> Self {
        Self {
            data: val.clone()
        }
    }
}
impl<T: ?Sized> Clone for Rfc<T> {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}

#[macro_export]
macro_rules! as_dyn {
    ($val:expr, $data_type:ty) => {
        Rfc::from_rc_refcell(&(Rc::new(RefCell::new($val)) as Rc<RefCell<$data_type>>))
    };
}