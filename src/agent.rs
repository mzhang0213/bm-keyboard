use crate::key_event::KeyEvent;
use crate::input_logic::InputLogic;
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

pub struct KeyboardAgent {
    text: String,
    options: DisplayOptions,
    synthesizer: SynthesizerAgent
}

impl Default for KeyboardAgent {
    fn default() -> Self {
        Self {
            text: "testing!!".to_string(),
            options: DisplayOptions::default(),
            synthesizer: SynthesizerAgent::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Key(KeyEvent),
    Clicked,
}

impl KeyboardAgent {
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
        window::get_latest().and_then(move |id| window::resize(id, size))
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

The synthesizer should read the text on each keystroke,
and run a conversion computation automatically to determine
the displayed output in the box.
*/
pub struct SynthesizerAgent {
    text: String,
    display: bool,
    mode: InputLogic
}

impl Default for SynthesizerAgent {
    fn default() -> Self {
        Self {
            text: "".to_string(),
            display: false,
            mode: InputLogic::Converting
        }
    }
}

impl SynthesizerAgent {
    /**
    Call on keyboard input -
    */
    pub fn update(&mut self) {

    }
}

