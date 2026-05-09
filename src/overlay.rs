mod agent;
mod cursor;
mod input_logic;
mod key_event;
mod listener;
mod token;

use agent::{DisplayOptions, InputAgent, Message, SynthesizerAgent};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::widget::text;
use iced::{stream, window, Element, Subscription, Task, Theme};

struct App {
    input: InputAgent,
    synth: SynthesizerAgent,
    input_window: window::Id,
    synth_window: window::Id,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let opts = DisplayOptions::default();

        let input_pos = cursor::snapped(-10.0);
        let synth_pos = iced::Point::new(input_pos.x, input_pos.y - 90.0);

        let (input_id, input_open) = window::open(window::Settings {
            size: opts.window_size,
            position: window::Position::Specific(input_pos),
            decorations: false,
            resizable: false,
            level: window::Level::AlwaysOnTop,
            ..window::Settings::default()
        });

        let (synth_id, synth_open) = window::open(window::Settings {
            size: opts.window_size,
            position: window::Position::Specific(synth_pos),
            decorations: false,
            resizable: false,
            level: window::Level::AlwaysOnTop,
            ..window::Settings::default()
        });

        let mut input = InputAgent::default();
        input.set_window(input_id);

        let mut synth = SynthesizerAgent::default();
        synth.set_window(synth_id);

        (
            Self {
                input,
                synth,
                input_window: input_id,
                synth_window: synth_id,
            },
            Task::batch(vec![
                input_open.map(Message::WindowOpened),
                synth_open.map(Message::WindowOpened),
            ]),
        )
    }

    fn title(&self, _id: window::Id) -> String {
        "병음 Keyboard".into()
    }

    fn view(&self, id: window::Id) -> Element<'_, Message> {
        if id == self.input_window {
            self.input.view()
        } else if id == self.synth_window {
            self.synth.view()
        } else {
            text("").into()
        }
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::WindowOpened(_) => Task::none(),
            other => {
                let input_task = self.input.update(other);
                let synth_task = self.synth.update(self.input.text());
                Task::batch(vec![input_task, synth_task])
            }
        }
    }
}

fn key_stream() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let (tx, mut rx) = mpsc::unbounded();
        listener::spawn(tx);
        while let Some(evt) = rx.next().await {
            let _ = output.send(Message::Key(evt)).await;
        }
    })
}

fn main() -> iced::Result {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("\n=== PANIC ===\n{info}\n=== /PANIC ===");
        prev(info);
    }));

    let opts = DisplayOptions::default();
    iced::daemon(App::title, App::update, App::view)
        .subscription(|_| Subscription::run(key_stream))
        .default_font(opts.font)
        .theme(|_, _| Theme::Dark)
        .run_with(App::new)
}
