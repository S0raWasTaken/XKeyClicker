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

    let time_millis: Entry = builder.object("time_millis").unwrap();

    let xkc_handle_clone = xkc_handle.clone();

    time_millis.connect_changed(move |entry| {
        if let Ok(cooldown) = entry.buffer().text().parse::<u64>() {
            xkc_handle_clone.cooldown.lock().unwrap().millis = cooldown;
        } else if !entry.buffer().text().is_empty() {
            entry.set_text("100");
            xkc_handle_clone.cooldown.lock().unwrap().millis = 100;
        }
    });

    let time_micros: Entry = builder.object("time_micros").unwrap();
    let xkc_handle_clone = xkc_handle.clone();

    time_micros.connect_changed(move |entry| {
        if let Ok(cooldown) = entry.buffer().text().parse::<u64>() {
            xkc_handle_clone.cooldown.lock().unwrap().micros = cooldown;
        } else if !entry.buffer().text().is_empty() {
            entry.set_text("0");
            xkc_handle_clone.cooldown.lock().unwrap().micros = 0;
        }
    });

    let time_nanos: Entry = builder.object("time_nanos").unwrap();
    let xkc_handle_clone = xkc_handle.clone();

    time_nanos.connect_changed(move |entry| {
        if let Ok(cooldown) = entry.buffer().text().parse::<u64>() {
            xkc_handle_clone.cooldown.lock().unwrap().nanos = cooldown;
        } else if !entry.buffer().text().is_empty() {
            entry.set_text("0");
            xkc_handle_clone.cooldown.lock().unwrap().nanos = 0;
        }
    });

    let start_keybind_button: Button = builder.object("start_keybind").unwrap();
    let keybind_entry: Entry = builder.object("keybind_entry").unwrap();

    let entry_sender_copy = entry_sender.clone();
    let xkc_handle_clone = xkc_handle.clone();

    start_keybind_button.connect_clicked(move |_| {
        set_start_keybind(
            &entry_sender.clone(),
            &keybind_entry,
            &xkc_handle_clone.clone(),
        );
    });

    let key_selector_button: Button = builder.object("key_selector").unwrap();
    let repeated_key_entry: Entry = builder.object("repeated_key_entry").unwrap();

    key_selector_button.connect_clicked(move |_| {
        set_repeated_key(
            &entry_sender_copy.clone(),
            &repeated_key_entry,
            &xkc_handle.clone(),
        );
    });

    window.show_all();
}

fn set_repeated_key(
    entry_sender: &Sender<Entry>,
    repeated_key_entry: &Entry,
    xkc_handle: &ArcXKeyClicker,
) {
    *xkc_handle.should_recv.lock().unwrap() = KeyType::Repeated;
    repeated_key_entry.set_text("Press a key to bind");
    entry_sender.send(repeated_key_entry.clone()).unwrap();
}

fn set_start_keybind(
    entry_sender: &Sender<Entry>,
    keybind_entry: &Entry,
    xkc_handle: &ArcXKeyClicker,
) {
    *xkc_handle.should_recv.lock().unwrap() = KeyType::Keybind;
    keybind_entry.set_text("Press a key to bind");
    entry_sender.send(keybind_entry.clone()).unwrap();
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
