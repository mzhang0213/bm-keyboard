#[derive(Debug, Clone)]
pub enum KeyEvent {
    Char(String),
    Backspace,
    Enter,
    Escape,
}
