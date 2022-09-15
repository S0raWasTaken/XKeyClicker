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
