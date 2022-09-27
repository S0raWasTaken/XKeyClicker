#![warn(clippy::pedantic)]
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{sleep, spawn},
    time::Duration,
};

use gtk::{
    gio::ApplicationFlags,
    prelude::{ApplicationExt, ApplicationExtManual, BuilderExtManual},
    traits::{ButtonExt, EntryExt, GtkWindowExt, WidgetExt},
    Application, ApplicationWindow, Builder, Button, EditableSignals, Entry,
};
use primitives::{KeyType, NotMut, SendBox};
use rdev::{listen, simulate, Event, EventType, Key};

static KEYBIND: Mutex<Key> = Mutex::new(Key::F7);
static STATE: Mutex<bool> = Mutex::new(false);
static SHOULD_RECV: Mutex<KeyType> = Mutex::new(KeyType::None);
static COOLDOWN: Mutex<u64> = Mutex::new(100);
static REPEATED_KEY: Mutex<Option<Key>> = Mutex::new(None);

mod primitives;

fn main() {
    let (entry_sender, entry_receiver) = channel();
    let entry_receiver = Arc::new(SendBox(entry_receiver));

    // Spawn keybind listener
    spawn(move || listen(move |e| keybind(&e, &entry_receiver.clone())).unwrap());

    // Spawn auto clicker
    spawn(auto_clicker);

    let app = Application::new(Some("com.s0ra.xkeyclicker"), ApplicationFlags::default());
    app.connect_activate(move |app| build_ui(app, entry_sender.clone()));
    app.run();
}

fn auto_clicker() {
    loop {
        if *STATE.lock().unwrap() {
            if let Some(key) = *REPEATED_KEY.lock().unwrap() {
                let delay = Duration::from_millis(*COOLDOWN.lock().unwrap());
                simulate(&EventType::KeyPress(key)).unwrap();
                simulate(&EventType::KeyRelease(key)).unwrap();
                sleep(delay);
            }
        }
    }
}

fn build_ui(app: &Application, entry_sender: Sender<Entry>) {
    let builder = Builder::from_string(include_str!("xkeyclicker.ui"));
    let window: ApplicationWindow = builder.object("window").unwrap();

    window.set_application(Some(app));

    let time_millis: Entry = builder.object("time_millis").unwrap();

    time_millis.connect_changed(|entry| {
        if let Ok(cooldown) = entry.buffer().text().parse::<u64>() {
            *COOLDOWN.lock().unwrap() = cooldown;
        } else if !entry.buffer().text().is_empty() {
            entry.set_text("100");
            *COOLDOWN.lock().unwrap() = 100;
        }
    });

    let start_keybind_button: Button = builder.object("start_keybind").unwrap();
    let keybind_entry: Entry = builder.object("keybind_entry").unwrap();

    let entry_sender_copy = entry_sender.clone();

    start_keybind_button
        .connect_clicked(move |_| set_start_keybind(&entry_sender.clone(), &keybind_entry));

    let key_selector_button: Button = builder.object("key_selector").unwrap();
    let repeated_key_entry: Entry = builder.object("repeated_key_entry").unwrap();

    key_selector_button.connect_clicked(move |_| {
        set_repeated_key(&entry_sender_copy.clone(), &repeated_key_entry);
    });

    window.show_all();
}

fn set_repeated_key(entry_sender: &Sender<Entry>, repeated_key_entry: &Entry) {
    *SHOULD_RECV.lock().unwrap() = KeyType::Repeated;
    repeated_key_entry.set_text("Press a key to bind");
    entry_sender.send(repeated_key_entry.clone()).unwrap();
}

fn set_start_keybind(entry_sender: &Sender<Entry>, keybind_entry: &Entry) {
    *SHOULD_RECV.lock().unwrap() = KeyType::Keybind;
    keybind_entry.set_text("Press a key to bind");
    entry_sender.send(keybind_entry.clone()).unwrap();
}

fn keybind(event: &Event, receiver: &Arc<SendBox<Receiver<Entry>>>) {
    if let Event {
        time: _,
        name: _,
        event_type: EventType::KeyPress(key),
    } = event
    {
        let mut should_recv = SHOULD_RECV.lock().unwrap();
        if let KeyType::Repeated = *should_recv {
            *REPEATED_KEY.lock().unwrap() = Some(*key);
            *should_recv = KeyType::None;

            let entry = receiver.0.try_recv().unwrap();
            entry.set_text(&format!("{key:?}"));
        } else if let KeyType::Keybind = *should_recv {
            *KEYBIND.lock().unwrap() = *key;
            *should_recv = KeyType::None;

            let entry = receiver.0.try_recv().unwrap();
            entry.set_text(&format!("{key:?}"));
        } else if *key == *KEYBIND.lock().unwrap() {
            STATE.lock().unwrap().not_mut();
        }
    }
}
