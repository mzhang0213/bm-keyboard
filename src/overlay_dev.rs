mod agent;
mod key_event;
mod listener_dev;

use agent::{DisplayOptions, KeyboardAgent, Message};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{stream, Point};
use iced::{window, Subscription, Theme};

fn key_stream() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let (tx, mut rx) = mpsc::unbounded();
        listener_dev::spawn(tx);
        while let Some(evt) = rx.next().await {
            let _ = output.send(Message::Key(evt)).await;
        }
    })
}

fn main() -> iced::Result {
    let opts = DisplayOptions::default();
    iced::application(
        "bm_keyboard overlay (dev)",
        KeyboardAgent::update,
        KeyboardAgent::view,
    )
    .window(window::Settings {
        size: opts.window_size,
        position: window::Position::Specific(Point{x:300.0,y:300.0}),
        decorations: false,
        resizable: false,
        level: window::Level::AlwaysOnTop,
        ..window::Settings::default()
    })
    .subscription(|_| Subscription::run(key_stream))
    .default_font(opts.font)
    .theme(|_| Theme::Dark)
    .run()
}
