mod util;

use eframe::{egui, App as EApp, NativeOptions};
use rdev::Key;
use util::Keyboard;

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
}

impl Default for App {
    fn default() -> Self {
        Self {
            keyboard: Keyboard::start_listening(),
            is_changing_keybind: false,
            click_interval: 0,
            current_keybind: None,
            raw_click_interval: String::new(),
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
                if h.button("Change keybind").clicked() {
                    self.is_changing_keybind = true;
                }

                if let Some(key) = self.keyboard.read() {
                    self.is_changing_keybind = false;
                    self.current_keybind = Some(key);
                }

                if self.is_changing_keybind {
                    h.label("Press any key...");
                } else {
                    if let Some(key) = self.current_keybind {
                        h.label(format!("Current: {key:?}"));
                    } else {
                        h.label("Current: None");
                    }
                }
            })
        });
    }
}
