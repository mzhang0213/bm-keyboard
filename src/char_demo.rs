//! Standalone demo: table-driven romaja → jamo scanner with ranked choices.
//! Two-layer lookup: variant input → ranked canonical romaja → jamo char.
//! Each scanned position yields a Vec of (canonical, jamo) ranked by preference.
//! See guides/CHAR_DEMO.md for the walkthrough.
//!
//! Run with: cargo run --example char_demo

mod token;

use token::CANONICAL;


pub struct CharScanner {
    tokens: Vec<Vec<(&'static str, char)>>,
}

impl Default for CharScanner {
    fn default() -> Self {
        Self { tokens: Vec::new() }
    }
}


// Variant input → ranked list of canonical interpretations.
// First entry is the default; later entries are alternatives the app can offer.
// Each canonical must appear as a key in CANONICAL.
// Sorted longest-first so prefix-match returns the longest input.
const VARIANTS: &[(&str, &[&str])] = &[
    ("yo", &["yo", "yeo"]),
    ("gg", &["kk", "g"]),
    ("oi", &["oe"]),
    ("o",  &["o", "eo"]),
];


impl CharScanner {
    pub fn tokens(&self) -> &[Vec<(&'static str, char)>] {
        &self.tokens
    }

    pub fn scan(&mut self, input: &str) {
        let mut i = 0;
        while i < input.len() {
            let rest = &input[i..];
            match Self::lookup(rest) {
                Some((consumed, choices)) => {
                    self.tokens.push(choices);
                    i += consumed;
                }
                None => {
                    i += 1; // unknown — skip one byte (romaja is ASCII)
                }
            }
        }
    }

    /// Find the longest prefix match across both tables.
    /// Returns bytes consumed and a ranked list of (canonical, jamo) choices.
    /// First choice is the default; later ones are alternatives the app can offer.
    fn lookup(rest: &str) -> Option<(usize, Vec<(&'static str, char)>)> {
        let v = VARIANTS.iter().find(|(p, _)| rest.starts_with(p));
        let c = CANONICAL.iter().find(|(p, _)| rest.starts_with(p));

        let v_len = v.map_or(0, |(s, _)| s.len());
        let c_len = c.map_or(0, |(s, _)| s.len());

        if v_len == 0 && c_len == 0 {
            return None;
        }

        if v_len >= c_len {
            let (input, canonicals) = v?;
            let choices: Vec<(&'static str, char)> = canonicals
                .iter()
                .filter_map(|canon| {
                    CANONICAL
                        .iter()
                        .find(|(k, _)| k == canon)
                        .map(|(k, j)| (*k, *j))
                })
                .collect();
            if choices.is_empty() {
                return None;
            }
            Some((input.len(), choices))
        } else {
            let (canonical, jamo) = c?;
            Some((canonical.len(), vec![(*canonical, *jamo)]))
        }
    }
}


#[allow(dead_code)]
const INITIALS: [&str; 19] = [
    "g", "kk", "n", "d", "tt", "r", "m", "b", "pp",
    "s", "ss", "", "j", "jj", "ch", "k", "t", "p", "h",
];
#[allow(dead_code)]
const VOWELS: [&str; 21] = [
    "a", "ae", "ya", "yae", "eo", "e", "yeo", "ye",
    "o", "wa", "wae", "oe", "yo", "u", "wo", "we",
    "wi", "yu", "eu", "ui", "i",
];
#[allow(dead_code)]
const FINALS: [&str; 28] = [
    "", "g", "kk", "gs", "n", "nj", "nh", "d", "l",
    "lg", "lm", "lb", "ls", "lt", "lp", "lh", "m",
    "b", "bs", "s", "ss", "ng", "j", "ch", "k", "t", "p", "h",
];

#[allow(dead_code)]
fn compose(initial: u32, vowel: u32, final_: u32) -> char {
    let codepoint = 0xAC00 + initial * 588 + vowel * 28 + final_;
    char::from_u32(codepoint).unwrap()
}

#[allow(dead_code)]
fn decompose(syllable: char) -> (u32, u32, u32) {
    let s = syllable as u32 - 0xAC00;
    let initial = s / 588;
    let vowel = (s % 588) / 28;
    let final_ = s % 28;
    (initial, vowel, final_)
}


fn main() {
    println!("=== romaja scan demo (ranked choices) ===");
    for input in ["babo", "choi", "yo", "gg", "o", "annyeong"] {
        let mut scanner = CharScanner::default();
        scanner.scan(input);
        println!("input: {input:?}");
        for (pos, choices) in scanner.tokens().iter().enumerate() {
            let rendered: Vec<String> = choices
                .iter()
                .map(|(canon, jamo)| format!("{canon}→{jamo}"))
                .collect();
            println!("  [{pos}] {}", rendered.join(" | "));
        }
        println!();
    }
}
