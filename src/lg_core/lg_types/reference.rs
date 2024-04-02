use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct Ref<T: ?Sized> {
    data: Rc<RefCell<T>>,
}

impl<T> Ref<T> {
    pub fn new(value: T) -> Self {
        Ref {
            data: Rc::new(RefCell::new(value)),
        }
    }
    pub fn downgrade(&self) -> Weak<RefCell<T>> {
        Rc::downgrade(&self.data)
    }
}
impl<T: ?Sized> Ref<T> {
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

impl<T: ?Sized> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}