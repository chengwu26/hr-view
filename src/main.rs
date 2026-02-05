#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use hr_view::App;

fn main() -> iced::Result {
    #[cfg(debug_assertions)]
    env_logger::init();

    iced::daemon(App::boot, App::update, App::view)
        .title("Heart Rate View")
        .subscription(App::subscription)
        .theme(App::theme)
        .style(|_, theme| iced::theme::Style {
            background_color: iced::Color::TRANSPARENT,
            text_color: theme.palette().text,
        })
        .run()
}
