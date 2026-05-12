use crate::key_event::KeyEvent;
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField, KeyCode,
};
use iced::futures::channel::mpsc::UnboundedSender;
use std::panic::{AssertUnwindSafe, catch_unwind};

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

    let Some(evt) = translate_keycode(code) else {
        return false;
    };

    if tx.unbounded_send(evt).is_err() {
        return false;
    }

    matches!(mode, TapMode::Grab)
}

fn should_pass_through(flags: CGEventFlags) -> bool {
    flags.intersects(
        CGEventFlags::CGEventFlagCommand
            | CGEventFlags::CGEventFlagControl
            | CGEventFlags::CGEventFlagAlternate,
    )
}

fn translate_keycode(code: u16) -> Option<KeyEvent> {
    match code {
        KeyCode::DELETE => Some(KeyEvent::Backspace),
        KeyCode::RETURN => Some(KeyEvent::Enter),
        KeyCode::ESCAPE => Some(KeyEvent::Escape),
        _ => qwerty_char(code).map(|ch| KeyEvent::Char(ch.to_string())),
    }
}

fn qwerty_char(code: u16) -> Option<char> {
    match code {
        0 => Some('a'),
        1 => Some('s'),
        2 => Some('d'),
        3 => Some('f'),
        4 => Some('h'),
        5 => Some('g'),
        6 => Some('z'),
        7 => Some('x'),
        8 => Some('c'),
        9 => Some('v'),
        11 => Some('b'),
        12 => Some('q'),
        13 => Some('w'),
        14 => Some('e'),
        15 => Some('r'),
        16 => Some('y'),
        17 => Some('t'),
        18 => Some('1'),
        19 => Some('2'),
        20 => Some('3'),
        21 => Some('4'),
        22 => Some('6'),
        23 => Some('5'),
        24 => Some('='),
        25 => Some('9'),
        26 => Some('7'),
        27 => Some('-'),
        28 => Some('8'),
        29 => Some('0'),
        30 => Some(']'),
        31 => Some('o'),
        32 => Some('u'),
        33 => Some('['),
        34 => Some('i'),
        35 => Some('p'),
        37 => Some('l'),
        38 => Some('j'),
        39 => Some('\''),
        40 => Some('k'),
        41 => Some(';'),
        42 => Some('\\'),
        43 => Some(','),
        44 => Some('/'),
        45 => Some('n'),
        46 => Some('m'),
        47 => Some('.'),
        49 => Some(' '),
        50 => Some('`'),
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
    fn translates_letters_from_macos_keycodes() {
        assert!(matches!(translate_keycode(0), Some(KeyEvent::Char(s)) if s == "a"));
        assert!(matches!(translate_keycode(13), Some(KeyEvent::Char(s)) if s == "w"));
        assert!(matches!(translate_keycode(46), Some(KeyEvent::Char(s)) if s == "m"));
    }

    #[test]
    fn translates_control_keys() {
        assert!(matches!(
            translate_keycode(KeyCode::DELETE),
            Some(KeyEvent::Backspace)
        ));
        assert!(matches!(
            translate_keycode(KeyCode::RETURN),
            Some(KeyEvent::Enter)
        ));
        assert!(matches!(
            translate_keycode(KeyCode::ESCAPE),
            Some(KeyEvent::Escape)
        ));
    }
}
