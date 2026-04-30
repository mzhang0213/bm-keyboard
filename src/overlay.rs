mod agent;
mod key_event;
mod listener;
mod input_logic;

use agent::{DisplayOptions, KeyboardAgent, Message};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{stream, Point};
use iced::{window, Subscription, Theme};

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
    let opts = DisplayOptions::default();
    iced::application(
        "bm_keyboard overlay",
        KeyboardAgent::update,
        KeyboardAgent::view,
    )
    .window(window::Settings {
        size: opts.window_size,
        position: window::Position::Specific(Point::new(300.0,300.0)),
        decorations: false,
        resizable: false,
        level: window::Level::AlwaysOnTop,
        ..window::Settings::default()
    })
    // .window(window::Settings {
    //     size: opts.window_size,
    //     position: window::Position::Specific(Point::new(300.0,50.0)),
    //     decorations: false,
    //     resizable: false,
    //     level: window::Level::AlwaysOnTop,
    //     ..window::Settings::default()
    // })
    .subscription(|_| Subscription::run(key_stream))
    .default_font(opts.font)
    .theme(|_| Theme::Dark)
    .run()
}
