use crate::key_event::KeyEvent;
use iced::futures::channel::mpsc::UnboundedSender;
use rdev::{grab, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};

static CMD_HELD: AtomicBool = AtomicBool::new(false);

pub fn spawn(tx: UnboundedSender<KeyEvent>) {
    std::thread::spawn(move || {
        eprintln!("[listener] starting rdev::grab (needs Accessibility + Input Monitoring perms)");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            grab(move |event: Event| match &event.event_type {
                EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => {
                    CMD_HELD.store(true, Ordering::SeqCst);
                    Some(event)
                }
                EventType::KeyRelease(Key::MetaLeft) | EventType::KeyRelease(Key::MetaRight) => {
                    CMD_HELD.store(false, Ordering::SeqCst);
                    Some(event)
                }
                EventType::KeyPress(k) => {
                    if CMD_HELD.load(Ordering::SeqCst) {
                        return Some(event);
                    }
                    if let Some(evt) = translate(*k, event.name.clone()) {
                        let _ = tx.unbounded_send(evt);
                    }
                    None
                }
                _ => Some(event),
            })
        }));
        match result {
            Ok(Ok(())) => eprintln!("[listener] rdev::grab returned cleanly"),
            Ok(Err(e)) => eprintln!("[listener] rdev grab error: {e:?} — check System Settings → Privacy & Security → Accessibility + Input Monitoring"),
            Err(payload) => {
                let msg = panic_message(&payload);
                eprintln!("[listener] rdev::grab PANICKED: {msg}");
            }
        }
    });
}

fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

fn translate(key: Key, name: Option<String>) -> Option<KeyEvent> {
    match key {
        Key::Backspace => Some(KeyEvent::Backspace),
        Key::Return => Some(KeyEvent::Enter),
        Key::Escape => Some(KeyEvent::Escape),
        _ => name.filter(|s| !s.is_empty()).map(KeyEvent::Char),
    }
}
