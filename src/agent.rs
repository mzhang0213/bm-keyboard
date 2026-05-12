use std::iter::Peekable;
use std::str::Chars;
use crate::input_logic::InputLogic;
use crate::key_event::KeyEvent;
use crate::token::Token;
use iced::widget::{container, mouse_area, text};
use iced::{window, Element, Font, Length, Size, Task};

pub struct DisplayOptions {
    pub window_size: Size,
    pub font_size: f32,
    pub font: Font,
    pub padding: f32,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            window_size: Size::new(80.0, 80.0),
            font_size: 32.0,
            font: Font::with_name("Apple SD Gothic Neo"),
            padding: 12.0,
        }
    }
}


pub struct InputAgent {
    text: String,
    options: DisplayOptions,
    window_id: Option<window::Id>,
}

impl Default for InputAgent {
    fn default() -> Self {
        Self {
            text: String::new(),
            options: DisplayOptions::default(),
            window_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Key(KeyEvent),
    Clicked,
    WindowOpened(window::Id),
}

impl InputAgent {
    pub fn set_window(&mut self, id: window::Id) {
        self.window_id = Some(id);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Key(KeyEvent::Char(s)) => self.text.push_str(&s),
            Message::Key(KeyEvent::Backspace) => {
                self.text.pop();
            }
            Message::Key(KeyEvent::Enter) => {
                println!("commit: {}", self.text);
                self.text.clear();
            }
            Message::Key(KeyEvent::Escape) => self.text.clear(),
            Message::Clicked => self.text.push('장'),
            Message::WindowOpened(_) => return Task::none(),
        }
        self.resize_task()
    }

    pub fn view(&self) -> Element<'_, Message> {
        mouse_area(
            container(text(self.text.as_str()).size(self.options.font_size))
                .padding(self.options.padding)
                .width(Length::Shrink)
                .height(Length::Shrink),
        )
        .on_press(Message::Clicked)
        .into()
    }

    fn resize_task(&self) -> Task<Message> {
        let size = self.measured_size();
        match self.window_id {
            Some(id) => window::resize(id, size),
            None => Task::none(),
        }
    }

    fn measured_size(&self) -> Size {
        let char_count = self.text.chars().count().max(1) as f32;
        let char_width = self.options.font_size * 1.1;
        let w = char_count * char_width + self.options.padding * 2.0;
        let h = self.options.font_size * 1.6 + self.options.padding * 2.0;
        Size::new(w, h)
    }
}

/**
Represents the converted text from the input box.

The synthesizer reads the InputAgent's text on each keystroke
and runs a conversion computation to determine what to display.
*/
pub struct SynthesizerAgent {
    text: String,
    display: bool,
    mode: InputLogic,
    options: DisplayOptions,
    window_id: Option<window::Id>,
}

impl Default for SynthesizerAgent {
    fn default() -> Self {
        Self {
            text: String::new(),
            display: false,
            mode: InputLogic::Converting,
            options: DisplayOptions::default(),
            window_id: None,
        }
    }
}

impl SynthesizerAgent {
    pub fn set_window(&mut self, id: window::Id) {
        self.window_id = Some(id);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn update(&mut self, input: &str) -> Task<Message> {
        self.text = match self.mode {
            InputLogic::Converting => convert(input),
            InputLogic::Synthesizing => input.to_string(),
        };
        self.resize_task()
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(text(self.text.as_str()).size(self.options.font_size))
            .padding(self.options.padding)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into()
    }

    fn resize_task(&self) -> Task<Message> {
        let size = self.measured_size();
        match self.window_id {
            Some(id) => window::resize(id, size),
            None => Task::none(),
        }
    }

    fn measured_size(&self) -> Size {
        let char_count = self.text.chars().count().max(1) as f32;
        let char_width = self.options.font_size * 1.1;
        let w = char_count * char_width + self.options.padding * 2.0;
        let h = self.options.font_size * 1.6 + self.options.padding * 2.0;
        Size::new(w, h)
    }
}




/**
A struct for a multiple choice char combo
- Two letter consonants
- Vowel romanization ambiguity - oe vs wae vs woe, ui eui, etc.
*/
pub struct CharChoice {
    text: String,
    choices: Vec<String>,
}

impl Default for CharChoice {
    fn default() -> Self {
        Self {
            text: String::new(),
            choices: Vec::new(),
        }
    }
}

impl CharChoice {
    pub fn new(&mut self, input: &str) {
        self.text = input.to_string();
        self.tokenize();
    }

    pub fn choices(&self) -> &[String] {
        &self.choices
    }

    fn tokenize(&mut self) {
        self.choices.clear();
        match self.text.as_str() {
            "oi" => self.choices.push("oe".to_string()),
            "wae" => self.choices.push("wae".to_string()),
            _ => {}
        }
    }
}


pub struct CharScanner {
    tokens: Vec<String>,
    digraph_count: usize,
}

impl Default for CharScanner {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            digraph_count: 0,
        }
    }
}

impl CharScanner {
    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }

    pub fn scan(&mut self, input: &str) {
        let mut rest = input.chars().peekable();
        while let Some(c) = rest.next() {
            let token = self.classify(c, &mut rest);
            self.tokens.push(token);
        }
    }

    fn classify(&mut self, c: char, rest: &mut Peekable<Chars<'_>>) -> String {
        if let Some(&next) = rest.peek() {
            match (c, next) {
                ('c', 'h') | ('e', 'o') | ('n', 'g') | ('g', 'g') => {
                    rest.next();
                    self.digraph_count += 1;
                    return format!("[{c}{next}]");
                }
                _ => {}
            }
        }

        match c {
            'a' | 'e' | 'i' | 'o' | 'u' => format!("vowel({c})"),
            '0'..='9' => {
                let value = c.to_digit(10).unwrap();
                format!("digit(value={value})")
            }
            'A'..='Z' | 'a'..='z' => {
                let code = c as u32;
                format!("alpha({c}, code=0x{code:02X})")
            }
            c if c.is_whitespace() => "space".into(),
            c if c.is_ascii_punctuation() => format!("punct({c})"),
            other => format!("other({other:?}, U+{:04X})", other as u32),
        }
    }
}


const INITIALS: [&str; 19] = [
    "g", "gg", "n", "d", "dd", "r", "m", "b", "bb",
    "s", "ss", "", "j", "jj", "ch", "k", "t", "p", "h",
];
const VOWELS: [&str; 21] = [
    "a", "ae", "ya", "yae", "eo", "e", "yeo", "ye",
    "o", "wa", "wae", "oe", "yo", "u", "wo", "we",
    "wi", "yu", "eu", "ui", "i",
];
const FINALS: [&str; 28] = [
    "", "g", "kk", "gs", "n", "nj", "nh", "d", "l",
    "lg", "lm", "lb", "ls", "lt", "lp", "lh", "m",
    "b", "bs", "s", "ss", "ng", "j", "ch", "k", "t", "p", "h",
];

fn compose(initial: u32, vowel: u32, final_: u32) -> char {
    let codepoint = 0xAC00 + initial * 588 + vowel * 28 + final_;
    char::from_u32(codepoint).unwrap()
}

fn decompose(syllable: char) -> (u32, u32, u32) {
    let s = syllable as u32 - 0xAC00;
    let initial = s / 588;
    let vowel = (s % 588) / 28;
    let final_ = s % 28;
    (initial, vowel, final_)
}


/*
flow:
classify -> tokenize input, custom detect special combos
synth -> custom algo to make valid korean words: greedy match tokens to words, edge cases matching on tokens, etc -> make korean based on confidence guesses
    -somehow make some tokens have "confidence" where they require multiple choice

detecting kr vs world:
-see how many valid kr chars we synthesized vs how many chars inputted
-compute some "fits korean" conversion ratio -> if str matches that, render default after input to be

multiple choice
-> how do we prevent computing and displaying all permutations of a "promblematic" multiple choice str?
-> sol: use what jpy keyboard does: tab menu for quick accepts, space menu and segments to control what each segment should output
    ex: <input> ggggg
    ggg gg or gg ggg, expand
    ggg=ㄱㄱㄱ or ㄱㄲ or ㄲㄱ
    gg=ㄱㄱ or ㄲ
    g=ㄱ


rules:
-w -> look for vowel
-y -> look for vowel
-pq
 */


fn convert(input: &str) -> String {
    let mut output = String::new();
    let mut i = 0;
    while i < input.len() {
        match Token::convert(&input[i..]) {
            Some((canonical, jamo)) => {
                output.push(jamo);
                i += canonical.len();
            }
            None => i += 1,
        }
    }
    output
}

#[cfg(test)]
#[path = "agent_tests.rs"]
mod tests;
