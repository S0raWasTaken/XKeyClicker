mod util;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use eframe::{
    egui::{self, DragValue},
    epaint::Vec2,
    App as EApp, NativeOptions,
};
use rdev::{simulate, EventType, Key};
use tokio::sync::watch;
use util::{key_button, Keyboard};

fn main() {
    let mut opts = NativeOptions::default();

    opts.max_window_size = Some(Vec2::new(420., 52.));
    opts.resizable = false;

    eframe::run_native("XKeyClicker", opts, Box::new(|_| Box::new(App::default())))
}

#[derive(Default, Clone)]
struct Config {
    is_changing_keybind: bool,
    current_keybind: Option<Key>,
    is_changing_repeated_key: bool,
    repeated_key: Option<Key>,
    click_interval: u64,
}

struct App {
    config: Config,
    keyboard: Arc<Mutex<Keyboard>>,
    handle_tx: watch::Sender<Config>,
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = watch::channel(Config::default());
        let keyboard = Arc::new(Mutex::new(Keyboard::start_listening()));
        let handle_kb = keyboard.clone();

        thread::spawn(move || loop {
            let _ = handle(&handle_kb, &rx);
        });

        Self {
            keyboard,
            config: Config::default(),
            handle_tx: tx,
        }
    }
}

impl EApp for App {
    fn update(&mut self, cx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { config, .. } = self;

        egui::CentralPanel::default().show(cx, |panel| {
            panel.horizontal(|h| {
                h.horizontal(|h| {
                    h.label("Click Interval");

                    h.add(DragValue::new(&mut config.click_interval).suffix("ms"));
                });

                h.vertical_centered_justified(|v| {
                    key_button(
                        v,
                        &self.keyboard,
                        "Change Keybind",
                        &mut config.is_changing_keybind,
                        &mut config.current_keybind,
                    );
                    key_button(
                        v,
                        &self.keyboard,
                        "Change Repeated key",
                        &mut config.is_changing_repeated_key,
                        &mut config.repeated_key,
                    );
                })
            });
        });

        self.handle_tx.send_replace(config.clone());
    }
}

fn handle(keyboard: &Mutex<Keyboard>, app: &watch::Receiver<Config>) -> Option<()> {
    let app = app.borrow();
    if app.is_changing_repeated_key || app.is_changing_keybind {
        return None;
    };

    let repeated_key = app.repeated_key?;
    let current_key = app.current_keybind?;
    let key = keyboard.lock().unwrap().read();

    if let Some(keybind) = key {
        if keybind == current_key {
            simulate(&EventType::KeyPress(repeated_key)).ok();
            simulate(&EventType::KeyRelease(repeated_key)).ok();
            std::thread::sleep(Duration::from_millis(app.click_interval));
        }
    }

    Some(())
}
