use crate::input_logic::InputLogic;
use crate::key_event::KeyEvent;
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

fn convert(input: &str) -> String {
    //conversion script
    input.to_string()
}

#[cfg(test)]
#[path = "agent_tests.rs"]
mod tests;
