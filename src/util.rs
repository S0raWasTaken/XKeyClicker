use std::{sync::mpsc, thread};

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

        thread::spawn(move || listen(move |e| Self::listen_thread(&tx, &should_receive_rx, e)));

        Self {
            should_receive,
            receiver: rx,
        }
    }

    fn listen_thread(tx: &mpsc::Sender<Key>, send: &watch::Receiver<bool>, event: Event) {
        if let Event {
            event_type: EventType::KeyPress(key),
            ..
        } = event
        {
            if *send.borrow() {
                if let Err(e) = tx.send(key) {
                    eprintln!(
                        "error:Keyboard::listen_thread% Failed to send value to main thread: {e}({e:?})"
                    )
                }
            }
        }
    }

    pub fn read(&self) -> Option<Key> {
        self.should_receive.send(true).ok()?;
        let key = self.receiver.try_recv().ok()?;
        self.should_receive.send(false).ok()?;

        Some(key)
    }
}
