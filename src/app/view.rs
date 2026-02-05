use btleplug::api::CentralState;
use iced::border::rounded;
use iced::widget::container::rounded_box;
use iced::widget::{
    Column, Container, button, center, column, container, pick_list, responsive, right_center, row,
    rule, slider, space, text, toggler, value,
};
use iced::{Element, Length, window};
use iced_aw::widget::{labeled_frame, selection_list_with};

use super::{App, ConnectionState, Message};
use crate::locales::{Language, TranslateItem};

fn themed_container<'a, E: Into<iced::Element<'a, Message>>>(content: E) -> Container<'a, Message> {
    center(content).style(|theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme.palette().background)),
        ..Default::default()
    })
}

fn adapter_message<'a>(item: TranslateItem, lang: Language) -> Container<'a, Message> {
    themed_container(text(item.translate(lang)).size(30))
}

impl App {
    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        match (self.adapter_state.clone(), id == self.main_window) {
            (CentralState::Unknown, true) => {
                adapter_message(TranslateItem::UnknownAdapterState, self.config.lang).into()
            }
            (CentralState::PoweredOff, true) => {
                adapter_message(TranslateItem::AdapterPowereddOff, self.config.lang).into()
            }
            (CentralState::Unknown | CentralState::PoweredOff, false) => center("N/A")
                .style(|theme| {
                    let mut style = rounded_box(theme);
                    style.border = rounded(self.config.hr_window_size().height / 2.0);
                    style.background = Some(
                        iced::Color {
                            a: 0.5,
                            ..iced::Color::BLACK
                        }
                        .into(),
                    );
                    style
                })
                .into(),
            (CentralState::PoweredOn, true) => themed_container(self.main_window_view()).into(),
            (CentralState::PoweredOn, false) => self.heart_rate_window_view(),
        }
    }

    fn main_window_view(&self) -> Container<'_, Message> {
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

        labeled_frame::LabeledFrame::new(
            TranslateItem::ScanTitle.translate(self.config.lang),
            devices,
        )
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
                "{} {}",
                TranslateItem::ConnectedTitle.translate(self.config.lang),
                self.connected_device()
                    .expect("[BUG] No device connected, but attempt display heart rate infomation")
            ))
            .push(rule::horizontal(1))
            .push(hrm_info)
            .into()
    }

    fn toggle_connect_btn_view(&self) -> Element<'_, Message> {
        let btn = match self.connection_state {
            ConnectionState::NotConnected => {
                button(TranslateItem::ConnectButton.translate(self.config.lang)).on_press_maybe(
                    self.selected_device
                        .as_ref()
                        .and(Some(Message::ConnectDevice)),
                )
            }
            ConnectionState::Connecting => {
                button(TranslateItem::ConnectingButton.translate(self.config.lang))
            }
            ConnectionState::Connected(_) => {
                button(TranslateItem::DisconnectButton.translate(self.config.lang))
                    .on_press(Message::DisconnectDevice)
            }
        };
        right_center(row![btn, space().width(Length::Fixed(4.0))])
            .height(Length::Shrink)
            .into()
    }

    fn settings_view(&self) -> Element<'_, Message> {
        let hear_rate_window = toggler(self.config.hr_window_visible)
            .label(TranslateItem::ShowHeartRateWindowSetting.translate(self.config.lang))
            .text_size(15)
            .on_toggle(Message::ShowHeartRateWindow);
        let lock_heart_rate_window = toggler(self.config.hr_window_locked)
            .label(TranslateItem::LockHeartRateWindowSetting.translate(self.config.lang))
            .text_size(15)
            .on_toggle(Message::LockHeartRateWindow);
        let hr_window_opaque = slider(
            0.0..=1.0,
            self.config.hr_window_opaque,
            Message::HeartRateWindowOpaqueChanged,
        )
        .step(0.01);
        let hr_window_opaque = column![
            text(TranslateItem::HeartRateWindowOpaqueSetting.translate(self.config.lang)).size(15),
            hr_window_opaque
        ];
        let language = pick_list(
            crate::locales::Language::ALL,
            Some(self.config.lang),
            Message::LanguageChanged,
        )
        .text_size(15);
        let settings = Column::new()
            .spacing(4)
            .padding(4)
            .push(language)
            .push(space().height(5))
            .push(hear_rate_window)
            .push(lock_heart_rate_window)
            .push(hr_window_opaque);

        labeled_frame::LabeledFrame::new(
            TranslateItem::SettingsTitle.translate(self.config.lang),
            settings,
        )
        .height(Length::Fill)
        .width(Length::FillPortion(3))
        .stroke_width(1)
        .into()
    }

    fn heart_rate_window_view(&self) -> Element<'_, Message> {
        responsive(move |size| {
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
                            a: self.config.hr_window_opaque,
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
