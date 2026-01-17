use iced::Subscription;
use iced::window;

use super::{App, Message};

impl App {
    pub fn subscription(&self) -> Subscription<Message> {
        use iced::mouse;
        Subscription::batch([
            window::close_events().map(|_| Message::Exit),
            iced::event::listen_with(|event, status, id| {
                if status == iced::event::Status::Captured {
                    return None;
                }
                let iced::Event::Mouse(event) = event else {
                    return None;
                };
                match event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        Some(Message::MouseEvent(event, id))
                    }
                    mouse::Event::WheelScrolled { .. } => Some(Message::MouseEvent(event, id)),
                    _ => None,
                }
            }),
        ])
    }
}
