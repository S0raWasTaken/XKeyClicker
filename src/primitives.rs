use std::ops::Not;

pub trait NotMut {
    fn not_mut(&mut self);
}

// For inverting a bool without assigning old value to a variable
impl NotMut for bool {
    fn not_mut(&mut self) {
        *self = self.not();
    }
}

#[derive(Debug)]
pub enum KeyType {
    Repeated,
    Keybind,
    None,
}

pub struct SendBox<T>(pub T);

unsafe impl<T> Send for SendBox<T> {}
unsafe impl<T> Sync for SendBox<T> {}
