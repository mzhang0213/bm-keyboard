pub struct Token {
    text: String,
    ch: char,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            text: String::new(),
            ch: '\0',
        }
    }
}

impl Token {
    pub fn convert(text: &str) -> Option<(&'static str, char)> {
        CANONICAL
            .iter()
            .find(|(p, _)| text.starts_with(p))
            .map(|(k, j)| (*k, *j))
    }

    pub fn new(input: &str) -> Self {
        match Self::convert(input) {
            Some((text, ch)) => Self {
                text: text.to_string(),
                ch,
            },
            None => {
                eprintln!("warning: no canonical match for {input:?}");
                Self {
                    text: input.to_string(),
                    ch: '\0',
                }
            }
        }
    }
}

pub const CANONICAL: &[(&str, char)] = &[
    // 3-letter
    ("yae", 'ㅒ'),
    ("yeo", 'ㅕ'),
    ("oae", 'ㅙ'),
    // 2-letter consonants
    ("ch", 'ㅊ'),
    ("kk", 'ㄲ'),
    ("tt", 'ㄸ'),
    ("pp", 'ㅃ'),
    ("ss", 'ㅆ'),
    ("jj", 'ㅉ'),
    // 2-letter vowels
    ("ae", 'ㅐ'),
    ("eo", 'ㅓ'),
    ("eu", 'ㅡ'),
    ("oe", 'ㅚ'),
    ("ya", 'ㅑ'),
    ("ye", 'ㅖ'),
    ("yo", 'ㅛ'),
    ("yu", 'ㅠ'),
    ("ui", 'ㅢ'),
    ("wa", 'ㅘ'),
    ("we", 'ㅞ'),
    ("wi", 'ㅟ'),
    ("wo", 'ㅝ'),
    // 1-letter
    ("a", 'ㅏ'),
    ("e", 'ㅔ'),
    ("i", 'ㅣ'),
    ("o", 'ㅗ'),
    ("u", 'ㅜ'),
    ("g", 'ㄱ'),
    ("n", 'ㄴ'),
    ("d", 'ㄷ'),
    ("r", 'ㄹ'),
    ("m", 'ㅁ'),
    ("b", 'ㅂ'),
    ("s", 'ㅅ'),
    ("j", 'ㅈ'),
    ("k", 'ㅋ'),
    ("t", 'ㅌ'),
    ("p", 'ㅍ'),
    ("h", 'ㅎ'),
];
