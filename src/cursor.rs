use iced::Point;

#[cfg(target_os = "macos")]
pub fn screen_pos() -> Option<Point> {
    use core_graphics::event::CGEvent;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    let evt = CGEvent::new(src).ok()?;
    let p = evt.location();
    Some(Point::new(p.x as f32, p.y as f32))
}

#[cfg(not(target_os = "macos"))]
pub fn screen_pos() -> Option<Point> {
    None
}

pub fn snapped(offset_y: f32) -> Point {
    let p = screen_pos().unwrap_or(Point::new(300.0, 300.0));
    Point::new(p.x, p.y + offset_y)
}
