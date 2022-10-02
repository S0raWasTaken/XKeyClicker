#![warn(clippy::pedantic)]
#![windows_subsystem = "windows"]

use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::{sleep, spawn},
};

use gtk::{
    gio::ApplicationFlags,
    prelude::{ApplicationExt, ApplicationExtManual, BuilderExtManual},
    traits::{ButtonExt, EntryExt, GtkWindowExt, WidgetExt},
    Application, ApplicationWindow, Builder, Button, EditableSignals, Entry,
};
use primitives::{KeyType, NotMut, SendBox, XKeyClicker};
use rdev::{listen, simulate, Event, EventType};

mod primitives;

type ArcXKeyClicker = Arc<XKeyClicker>;

fn main() {
    let xkeyclicker = XKeyClicker::new();

    let (entry_sender, entry_receiver) = channel();
    let entry_receiver = Arc::new(SendBox(entry_receiver));

    let xkc_handle = xkeyclicker.clone();
    // Spawn keybind listener
    spawn(move || {
        listen(move |e| {
            keybind(&e, &entry_receiver.clone(), &xkc_handle.clone());
        })
        .unwrap();
    });

    let xkc_handle = xkeyclicker.clone();
    // Spawn auto clicker
    spawn(move || auto_clicker(&xkc_handle));

    let app = Application::new(Some("com.s0ra.xkeyclicker"), ApplicationFlags::default());
    app.connect_activate(move |app| build_ui(app, entry_sender.clone(), xkeyclicker.clone()));
    app.run();
}

fn auto_clicker(xkc_handle: &ArcXKeyClicker) {
    loop {
        if *xkc_handle.state.lock().unwrap() {
            if let Some(key) = *xkc_handle.repeated_key.lock().unwrap() {
                let delay = &*xkc_handle.cooldown.lock().unwrap();
                simulate(&EventType::KeyPress(key)).unwrap();
                simulate(&EventType::KeyRelease(key)).unwrap();
                sleep(delay.as_duration());
            }
        }
    }
}

fn build_ui(app: &Application, entry_sender: Sender<Entry>, xkc_handle: ArcXKeyClicker) {
    let builder = Builder::from_string(include_str!("xkeyclicker.ui"));
    let window: ApplicationWindow = builder.object("window").unwrap();

    window.set_application(Some(app));

    macro_rules! time_entry {
        ($time_type:tt, $default_cooldown:tt) => {
            let $time_type: Entry = builder
                .object(&format!("time_{}", stringify!($time_type)))
                .unwrap();
            let xkc_handle_clone = xkc_handle.clone();

            $time_type.connect_changed(move |entry| {
                if let Ok(cooldown) = entry.buffer().text().parse::<u64>() {
                    xkc_handle_clone.cooldown.lock().unwrap().$time_type = cooldown;
                } else if !entry.buffer().text().is_empty() {
                    entry.set_text("0");
                    xkc_handle_clone.cooldown.lock().unwrap().$time_type = 0;
                }
            });
        };
    }

    time_entry!(mins, 0);
    time_entry!(secs, 0);
    time_entry!(millis, 100);
    time_entry!(micros, 0);

    let start_keybind_button: Button = builder.object("start_keybind").unwrap();
    let keybind_entry: Entry = builder.object("keybind_entry").unwrap();

    let entry_sender_copy = entry_sender.clone();
    let xkc_handle_clone = xkc_handle.clone();

    start_keybind_button.connect_clicked(move |_| {
        set_keybind(
            &entry_sender.clone(),
            &keybind_entry,
            &xkc_handle_clone.clone(),
            KeyType::Keybind,
        );
    });

    let key_selector_button: Button = builder.object("key_selector").unwrap();
    let repeated_key_entry: Entry = builder.object("repeated_key_entry").unwrap();

    key_selector_button.connect_clicked(move |_| {
        set_keybind(
            &entry_sender_copy.clone(),
            &repeated_key_entry,
            &xkc_handle.clone(),
            KeyType::Repeated,
        );
    });

    window.show_all();
}

fn set_keybind(
    entry_sender: &Sender<Entry>,
    key_entry: &Entry,
    xkc_handle: &ArcXKeyClicker,
    key_type: KeyType,
) {
    *xkc_handle.should_recv.lock().unwrap() = key_type;
    key_entry.set_text("Press a key to bind");
    entry_sender.send(key_entry.clone()).unwrap();
}

fn keybind(event: &Event, receiver: &Arc<SendBox<Receiver<Entry>>>, xkc_handle: &ArcXKeyClicker) {
    if let Event {
        time: _,
        name: _,
        event_type: EventType::KeyPress(key),
    } = event
    {
        let mut should_recv = xkc_handle.should_recv.lock().unwrap();
        if let KeyType::Repeated = *should_recv {
            *xkc_handle.repeated_key.lock().unwrap() = Some(*key);
            *should_recv = KeyType::None;

            let entry = receiver.0.try_recv().unwrap();
            entry.set_text(&format!("{key:?}"));
        } else if let KeyType::Keybind = *should_recv {
            *xkc_handle.keybind.lock().unwrap() = *key;
            *should_recv = KeyType::None;

            let entry = receiver.0.try_recv().unwrap();
            entry.set_text(&format!("{key:?}"));
        } else if *key == *xkc_handle.keybind.lock().unwrap() {
            xkc_handle.state.lock().unwrap().not_mut();
        }
    }
}
