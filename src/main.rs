mod util;

use eframe::{egui, App as EApp, NativeOptions};
use rdev::Key;
use util::{key_button, Keyboard};

fn main() {
    let opts = NativeOptions::default();
    eframe::run_native("XKeyClicker", opts, Box::new(|_| Box::new(App::default())))
}

struct App {
    raw_click_interval: String,
    click_interval: usize,
    keyboard: Keyboard,
    is_changing_keybind: bool,
    current_keybind: Option<Key>,
    is_changing_repeated_key: bool,
    repeated_key: Option<Key>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            keyboard: Keyboard::start_listening(),
            is_changing_keybind: false,
            click_interval: 0,
            current_keybind: None,
            raw_click_interval: String::new(),
            is_changing_repeated_key: false,
            repeated_key: None,
        }
    }
}

impl EApp for App {
    fn update(&mut self, cx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(cx, |panel| {
            panel.horizontal(|h| {
                h.label("Click Interval (In Miliseconds)");
                let old = self.raw_click_interval.clone();
                h.text_edit_singleline(&mut self.raw_click_interval);
                if !self.raw_click_interval.is_empty() {
                    if let Ok(click_interval) = self.raw_click_interval.parse() {
                        self.click_interval = click_interval;
                    } else {
                        self.raw_click_interval = old;
                    }
                }
            });

            panel.horizontal(|h| {
                key_button(
                    h,
                    &self.keyboard,
                    "Change Keybind",
                    &mut self.is_changing_keybind,
                    &mut self.current_keybind,
                );
            });

            panel.horizontal(|h| {
                key_button(
                    h,
                    &self.keyboard,
                    "Change Repeated key",
                    &mut self.is_changing_repeated_key,
                    &mut self.repeated_key,
                );
            })
        });
    }
}
