use std::{
    sync::{mpsc, Mutex},
    thread,
};

use eframe::egui::Ui;
use tokio::sync::watch;

use rdev::{listen, Event, EventType, Key};

pub(crate) struct Keyboard {
    should_receive: watch::Sender<bool>,
    receiver: mpsc::Receiver<Key>,
}

impl Keyboard {
    pub(crate) fn start_listening() -> Self {
        let (tx, rx) = mpsc::channel();
        let (should_receive, should_receive_rx) = watch::channel(false);

        thread::spawn(move || listen(move |e| Self::listen_thread(&tx, &should_receive_rx, &e)));

        Self {
            should_receive,
            receiver: rx,
        }
    }

    fn listen_thread(tx: &mpsc::Sender<Key>, send: &watch::Receiver<bool>, event: &Event) {
        if let Event {
            event_type: EventType::KeyPress(key),
            ..
        } = event
        {
            if *send.borrow() {
                if let Err(e) = tx.send(*key) {
                    eprintln!(
                        "error:Keyboard::listen_thread% Failed to send value to main thread: {e}({e:?})"
                    );
                }
            }
        }
    }

    pub fn read(&self) -> Option<Key> {
        self.stop();
        self.should_receive.send_replace(true);
        self.receiver.try_recv().ok()
    }

    pub fn stop(&self) {
        self.should_receive.send_replace(false);
    }
}

pub(crate) fn key_button(
    ui: &mut Ui,
    keyboard: &Mutex<Keyboard>,
    prefix: &str,
    changing: &mut bool,
    target: &mut Option<Key>,
) {
    let keyboard = keyboard.lock().unwrap();
    let mut label = prefix.to_string();

    if *changing {
        label = "Press any key...".to_string();
    } else if let Some(key) = target {
        label.push_str(&format!(" (Current: {key:?})"));
    } else {
        label.push_str(" (Current: None)");
    }
    if ui.button(label).clicked() {
        *changing = true;
    }
    if *changing {
        if let Some(key) = keyboard.read() {
            *changing = false;
            *target = Some(key);
        }
    }
}
