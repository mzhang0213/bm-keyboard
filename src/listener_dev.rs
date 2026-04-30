use crate::key_event::KeyEvent;
use iced::futures::channel::mpsc::UnboundedSender;
use rdev::{listen, Event, EventType, Key};

pub fn spawn(tx: UnboundedSender<KeyEvent>) {
    std::thread::spawn(move || {
        if let Err(e) = listen(move |event: Event| {
            if let EventType::KeyPress(k) = event.event_type {
                if let Some(evt) = translate(k, event.name) {
                    let _ = tx.unbounded_send(evt);
                }
            }
        }) {
            eprintln!("rdev listen error: {:?}", e);
        }
    });
}

fn translate(key: Key, name: Option<String>) -> Option<KeyEvent> {
    match key {
        Key::Backspace => Some(KeyEvent::Backspace),
        Key::Return => Some(KeyEvent::Enter),
        Key::Escape => Some(KeyEvent::Escape),
        _ => name.filter(|s| !s.is_empty()).map(KeyEvent::Char),
    }
}
