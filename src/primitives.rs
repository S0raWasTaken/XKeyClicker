use std::{
    ops::Not,
    sync::{Arc, Mutex},
    time::Duration,
};

use rdev::Key;

pub trait NotMut {
    fn not_mut(&mut self);
}

// For inverting a bool without assigning old value to a variable
impl NotMut for bool {
    fn not_mut(&mut self) {
        *self = self.not();
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum KeyType {
    Repeated,
    Keybind,
    #[default]
    None,
}

#[derive(Debug)]
pub struct XKeyClicker {
    pub keybind: Mutex<Key>,
    pub should_recv: Mutex<KeyType>,
    pub state: Mutex<bool>,
    pub cooldown: Mutex<Cooldown>,
    pub repeated_key: Mutex<Option<Key>>,
}

impl Default for XKeyClicker {
    fn default() -> Self {
        Self {
            keybind: Mutex::new(Key::F7),
            should_recv: Mutex::default(),
            state: Mutex::default(),
            cooldown: Mutex::default(),
            repeated_key: Mutex::default(),
        }
    }
}

impl XKeyClicker {
    pub fn new() -> Arc<XKeyClicker> {
        Arc::default()
    }
}

#[derive(Debug)]
pub struct Cooldown {
    pub mins: u64,
    pub secs: u64,
    pub millis: u64,
    pub micros: u64,
}

impl Default for Cooldown {
    fn default() -> Self {
        Self {
            mins: 0,
            secs: 0,
            millis: 100,
            micros: 0,
        }
    }
}

impl Cooldown {
    pub fn as_duration(&self) -> Duration {
        Duration::from_secs(self.mins * 60) // There's no Duration::from_mins() ¯\_(ツ)_/¯
            + Duration::from_secs(self.secs)
            + Duration::from_millis(self.millis)
            + Duration::from_micros(self.micros)
    }
}

pub struct SendBox<T>(pub T);

unsafe impl<T> Send for SendBox<T> {}
unsafe impl<T> Sync for SendBox<T> {}
