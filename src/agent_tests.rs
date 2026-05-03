use super::*;
use crate::key_event::KeyEvent;

#[test]
fn default_text_is_empty() {
    let a = InputAgent::default();
    assert_eq!(a.text(), "");
}

#[test]
fn typing_appends_chars() {
    let mut a = InputAgent::default();
    let _ = a.update(Message::Key(KeyEvent::Char("ㅈ".into())));
    let _ = a.update(Message::Key(KeyEvent::Char("ㅏ".into())));
    assert_eq!(a.text(), "ㅈㅏ");
}

#[test]
fn backspace_pops_one_char() {
    let mut a = InputAgent::default();
    a.text = "ab".into();
    let _ = a.update(Message::Key(KeyEvent::Backspace));
    assert_eq!(a.text(), "a");
}

#[test]
fn backspace_on_empty_is_noop() {
    let mut a = InputAgent::default();
    let _ = a.update(Message::Key(KeyEvent::Backspace));
    assert_eq!(a.text(), "");
}

#[test]
fn escape_clears_text() {
    let mut a = InputAgent::default();
    a.text = "anything".into();
    let _ = a.update(Message::Key(KeyEvent::Escape));
    assert_eq!(a.text(), "");
}

#[test]
fn enter_clears_text() {
    let mut a = InputAgent::default();
    a.text = "commit me".into();
    let _ = a.update(Message::Key(KeyEvent::Enter));
    assert_eq!(a.text(), "");
}

#[test]
fn click_appends_jang() {
    let mut a = InputAgent::default();
    let _ = a.update(Message::Clicked);
    assert_eq!(a.text(), "장");
}

#[test]
fn measured_size_grows_with_text() {
    let mut a = InputAgent::default();
    a.text = "a".into();
    let small = a.measured_size();
    a.text = "aaaaaaaa".into();
    let big = a.measured_size();
    assert!(big.width > small.width, "wider text should produce wider window");
    assert_eq!(big.height, small.height, "height should be constant");
}
