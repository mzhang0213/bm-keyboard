use crate::key_event::KeyEvent;
use iced::futures::channel::mpsc::UnboundedSender;

#[cfg(target_os = "macos")]
pub fn spawn(tx: UnboundedSender<KeyEvent>) {
    crate::macos_key_tap::spawn(tx, crate::macos_key_tap::TapMode::Listen);
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::*;
    use rdev::{Event, EventType, Key, listen};

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
}

#[cfg(not(target_os = "macos"))]
pub use platform::spawn;
