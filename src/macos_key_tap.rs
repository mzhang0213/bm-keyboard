use crate::key_event::KeyEvent;
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField, KeyCode,
};
use core_graphics::sys::CGEventRef;
use foreign_types::ForeignType;
use iced::futures::channel::mpsc::UnboundedSender;
use std::ffi::c_ulong;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventKeyboardGetUnicodeString(
        event: CGEventRef,
        max_string_length: c_ulong,
        actual_string_length: *mut c_ulong,
        unicode_string: *mut u16,
    );
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum TapMode {
    Grab,
    Listen,
}

pub fn spawn(tx: UnboundedSender<KeyEvent>, mode: TapMode) {
    let name = match mode {
        TapMode::Grab => "macos-key-grab",
        TapMode::Listen => "macos-key-listen",
    };

    if let Err(err) = std::thread::Builder::new()
        .name(name.to_string())
        .spawn(move || run(tx, mode))
    {
        eprintln!("[listener] failed to spawn {name}: {err}");
    }
}

fn run(tx: UnboundedSender<KeyEvent>, mode: TapMode) {
    let result = catch_unwind(AssertUnwindSafe(|| run_event_tap(tx, mode)));
    match result {
        Ok(Ok(())) => eprintln!("[listener] macOS event tap returned cleanly"),
        Ok(Err(msg)) => eprintln!("[listener] macOS event tap error: {msg}"),
        Err(payload) => eprintln!(
            "[listener] macOS event tap PANICKED: {}",
            panic_message(&payload)
        ),
    }
}

fn run_event_tap(tx: UnboundedSender<KeyEvent>, mode: TapMode) -> Result<(), &'static str> {
    let options = match mode {
        TapMode::Grab => CGEventTapOptions::Default,
        TapMode::Listen => CGEventTapOptions::ListenOnly,
    };

    let current_loop = CFRunLoop::get_current();
    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        options,
        vec![CGEventType::KeyDown],
        move |_proxy, event_type, event| {
            let should_swallow = catch_unwind(AssertUnwindSafe(|| {
                handle_key_event(event_type, event, &tx, mode)
            }));

            match should_swallow {
                Ok(true) => event.set_type(CGEventType::Null),
                Ok(false) => {}
                Err(payload) => {
                    eprintln!(
                        "[listener] macOS event callback PANICKED: {}",
                        panic_message(&payload)
                    );
                }
            }

            None
        },
    )
    .map_err(|_| "could not create event tap; check Accessibility and Input Monitoring")?;

    let loop_source = tap
        .mach_port
        .create_runloop_source(0)
        .map_err(|_| "could not create event tap run loop source")?;

    current_loop.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });
    tap.enable();
    eprintln!("[listener] macOS event tap started in {mode:?} mode");
    CFRunLoop::run_current();
    Ok(())
}

fn handle_key_event(
    event_type: CGEventType,
    event: &CGEvent,
    tx: &UnboundedSender<KeyEvent>,
    mode: TapMode,
) -> bool {
    if !matches!(event_type, CGEventType::KeyDown) {
        return false;
    }

    if should_pass_through(event.get_flags()) {
        return false;
    }

    let Ok(code) = u16::try_from(event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE))
    else {
        return false;
    };

    let evt = match special_keycode(code) {
        Some(evt) => evt,
        None => match unicode_string_from_event(event) {
            Some(text) => KeyEvent::Char(text),
            None => return false,
        },
    };

    if tx.unbounded_send(evt).is_err() {
        return false;
    }

    matches!(mode, TapMode::Grab)
}

fn unicode_string_from_event(event: &CGEvent) -> Option<String> {
    let mut buf = [0u16; 8];
    let mut actual: c_ulong = 0;
    unsafe {
        CGEventKeyboardGetUnicodeString(
            event.as_ptr(),
            buf.len() as c_ulong,
            &mut actual as *mut c_ulong,
            buf.as_mut_ptr(),
        );
    }
    let len = actual as usize;
    if len == 0 || len > buf.len() {
        return None;
    }
    let s = String::from_utf16(&buf[..len]).ok()?;
    if s.chars().all(|c| c.is_control()) {
        return None;
    }
    Some(s)
}

fn should_pass_through(flags: CGEventFlags) -> bool {
    flags.intersects(
        CGEventFlags::CGEventFlagCommand
            | CGEventFlags::CGEventFlagControl
            | CGEventFlags::CGEventFlagAlternate,
    )
}

fn special_keycode(code: u16) -> Option<KeyEvent> {
    match code {
        KeyCode::DELETE => Some(KeyEvent::Backspace),
        KeyCode::RETURN => Some(KeyEvent::Enter),
        KeyCode::ESCAPE => Some(KeyEvent::Escape),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_control_keys() {
        assert!(matches!(
            special_keycode(KeyCode::DELETE),
            Some(KeyEvent::Backspace)
        ));
        assert!(matches!(
            special_keycode(KeyCode::RETURN),
            Some(KeyEvent::Enter)
        ));
        assert!(matches!(
            special_keycode(KeyCode::ESCAPE),
            Some(KeyEvent::Escape)
        ));
        assert!(special_keycode(0).is_none());
    }
}
