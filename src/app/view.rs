use btleplug::api::CentralState;
use iced::border::rounded;
use iced::widget::container::rounded_box;
use iced::widget::{
    Column, button, center, container, responsive, right_center, row, rule, space, text, toggler,
    value,
};
use iced::{Element, Length, window};
use iced_aw::widget::{labeled_frame, selection_list_with};

use super::{App, ConnectionState, Message};

impl App {
    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        match (self.adapter_state.clone(), id == self.main_window) {
            (CentralState::Unknown, true) => {
                center(text("Unknown Bluetooth adapter state\nI CANNOT WORK!").size(30)).into()
            }
            (CentralState::PoweredOff, true) => {
                center(text("Please turn on your Bluetooth adapter!").size(30)).into()
            }
            (CentralState::Unknown | CentralState::PoweredOff, false) => center("N/A").into(),
            (CentralState::PoweredOn, true) => self.main_window_view(),
            (CentralState::PoweredOn, false) => self.heart_rate_window_view(),
        }
    }

    fn main_window_view(&self) -> Element<'_, Message> {
        let left_pane = Column::new()
            .width(Length::FillPortion(3))
            .spacing(4)
            .push(match &self.connected_device() {
                None => self.devices_view(),
                Some(_) => self.hrm_info_view(),
            })
            .push(self.toggle_connect_btn_view());

        let mut right_pane = Column::new()
            .height(Length::Fill)
            .width(Length::Fixed(250.0))
            .spacing(4)
            .push(self.settings_view());

        if let Some(msg) = self.error_message() {
            right_pane = right_pane.push(text(msg).style(text::warning));
        }

        container(row![left_pane, rule::vertical(1), right_pane].padding(8))
            .style(|theme: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme.palette().background)),
                ..Default::default()
            })
            .into()
    }

    fn devices_view(&self) -> Element<'_, Message> {
        let devices = selection_list_with(
            &self.discovered_devices[..],
            |_, d| Message::SelectDevice(d.address()),
            14.0,
            4,
            |theme, status| {
                let mut style = iced_aw::style::selection_list::primary(theme, status);
                style.border_width = 0.0;
                style
            },
            None,
            Default::default(),
        );

        labeled_frame::LabeledFrame::new("Discovered devices", devices)
            .height(Length::Fill)
            .stroke_width(1)
            .into()
    }

    fn hrm_info_view(&self) -> Element<'_, Message> {
        let hrm_info = center(
            match self.heart_rate {
                None => text("--"),
                Some(hrm) => value(hrm),
            }
            .wrapping(text::Wrapping::None),
        );
        Column::new()
            .spacing(4)
            .push(text!(
                "Connected: {}",
                self.connected_device()
                    .expect("[BUG] No device connected, but attempt display heart rate infomation")
            ))
            .push(rule::horizontal(1))
            .push(hrm_info)
            .into()
    }

    fn toggle_connect_btn_view(&self) -> Element<'_, Message> {
        let btn = match self.connection_state {
            ConnectionState::NotConnected => button("Connect").on_press_maybe(
                self.selected_device
                    .as_ref()
                    .and(Some(Message::ConnectDevice)),
            ),
            ConnectionState::Connecting => button("Connecting"),
            ConnectionState::Connected(_) => {
                button("Disconnect").on_press(Message::DisconnectDevice)
            }
        };
        right_center(row![btn, space().width(Length::Fixed(4.0))])
            .height(Length::Shrink)
            .into()
    }

    fn settings_view(&self) -> Element<'_, Message> {
        let hear_rate_window = toggler(self.config.hr_window_visible)
            .label("Heart rate window")
            .on_toggle(Message::ShowHeartRateWindow);
        // let auto_reconnect = toggler(self.config.automatic_reconnection)
        //     .label("Automatic reconnection")
        //     .on_toggle(Message::ToggleAutoReconnect);
        let lock_heart_rate_window = toggler(self.config.hr_window_locked)
            .label("Lock heart rate window")
            .on_toggle(Message::LockHeartRateWindow);

        let settings = Column::new()
            .spacing(4)
            .push(hear_rate_window)
            // .push(auto_reconnect)
            .push(lock_heart_rate_window);
        labeled_frame::LabeledFrame::new("Settings", settings)
            .height(Length::Fill)
            .width(Length::FillPortion(3))
            .stroke_width(1)
            .into()
    }

    fn heart_rate_window_view(&self) -> Element<'_, Message> {
        responsive(|size| {
            let font_size = size.height / 1.6;
            let icon = text("❤ ").size(font_size);
            let rate = self
                .heart_rate
                .map(|v| v.heart_rate.to_string())
                .unwrap_or_else(|| "--".into());
            let rate = text(rate).size(font_size).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            });
            let content = row![icon, rate, space().width(5)];
            center(content)
                .padding(5)
                .style(move |theme| {
                    let mut style = rounded_box(theme);
                    style.border = rounded(size.height / 2.0);
                    style.background = Some(
                        iced::Color {
                            a: 0.5,
                            ..iced::Color::BLACK
                        }
                        .into(),
                    );
                    style
                })
                .into()
        })
        .into()
    }
}
