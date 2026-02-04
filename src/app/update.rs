use btleplug::api::CentralState;
use iced::{Task, window};
use log::{debug, warn};

use super::{App, BlockResize, ConnectionState, Message};
use Message::*;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        debug!("Received message: {message:?}");
        match message {
            LanguageChanged(lang) => {
                self.config.lang = lang;
                Task::none()
            }
            MouseEvent(event, id) => {
                use iced::mouse::{Button, Event, ScrollDelta};
                if id == self.main_window {
                    return Task::none();
                }
                match event {
                    Event::ButtonPressed(Button::Left) => window::drag(id),
                    Event::WheelScrolled {
                        delta: ScrollDelta::Lines { x: _, y } | ScrollDelta::Pixels { x: _, y },
                    } => {
                        if y > 0.0 {
                            Task::done(Message::HeartRateWindowResize(BlockResize::Increment))
                        } else {
                            Task::done(Message::HeartRateWindowResize(BlockResize::Decrease))
                        }
                    }
                    _ => unreachable!("[BUG] Received a unexpected mouse event."),
                }
            }
            HeartRateWindowResize(resize) => {
                let id = self.hr_window;
                let new_scale = match resize {
                    BlockResize::Increment => self.config.hr_window_scale() + 0.05,
                    BlockResize::Decrease => self.config.hr_window_scale() - 0.05,
                };
                self.config.set_hr_window_scale(new_scale);
                window::resize(id, self.config.hr_window_size())
            }
            Exit => {
                let mut config = self.config;
                Task::batch([
                    window::position(self.hr_window).then(move |opt| {
                        if let Some(p) = opt {
                            config.hr_window_pos = p;
                        }
                        config.save();
                        Task::none()
                    }),
                    self.connected_device()
                        .map(|device| {
                            let device = device.clone();
                            Task::future(async move { device.disconnect().await })
                                .then(|_| Task::none())
                        })
                        .unwrap_or_else(Task::none),
                ])
                .chain(iced::exit())
            }
            ConnectDevice => {
                use ConnectionState::*;
                Task::done(ConnectionStateUpdated(Connecting)).chain(
                    Task::future(self.connect()).then(|res| match res {
                        Ok(addr) => Task::done(ConnectionStateUpdated(Connected(addr))),
                        Err(e) => Task::done(ConnectionStateUpdated(NotConnected)).chain(
                            Task::done(ErrorOccurred(format!("Failed to connect device: {e}"))),
                        ),
                    }),
                )
            }
            LockHeartRateWindow(enable) => {
                self.config.hr_window_locked = enable;
                if enable {
                    window::enable_mouse_passthrough(self.hr_window)
                } else {
                    window::disable_mouse_passthrough(self.hr_window)
                }
            }
            DisconnectDevice => Task::future(self.disconnect())
                .map(|res| res.err())
                .and_then(|e| {
                    Task::done(ErrorOccurred(format!("Failed to disconnect device: {e}")))
                }),
            AdapterStateUpdated(state) => {
                self.adapter_state = state;
                match (&self.adapter_state, &self.connected_device()) {
                    (CentralState::PoweredOn, _) => Task::done(ScanDevice(true)),
                    (_, Some(_)) => Task::done(DeviceDisconnected),
                    _ => Task::none(),
                }
            }
            ConnectionStateUpdated(state) => {
                self.connection_state = state;
                let ConnectionState::Connected(_) = self.connection_state else {
                    return Task::none();
                };

                Task::future(self.subscribe()).then(|res| match res {
                    Err(e) => {
                        Task::done(ErrorOccurred(format!("Failed to get heart rate data: {e}")))
                            .chain(Task::done(DisconnectDevice))
                    }
                    Ok(s) => Task::done(ScanDevice(false)).chain(Task::run(s, |opt| match opt {
                        None => {
                            warn!("Received invalid heart rate data");
                            ErrorOccurred("Invalid heart rate data".into())
                        }
                        Some(hrm) => HeartRateUpdated(hrm),
                    })),
                })
            }
            ScanDevice(start) => {
                if start {
                    self.selected_device = None;
                    self.discovered_devices.clear();
                    Task::future(self.start_scan())
                        .map(|res| res.err())
                        .and_then(|e| {
                            Task::done(ErrorOccurred(format!("Failed to start scan: {e}")))
                        })
                } else {
                    Task::future(self.stop_scan())
                        .map(|res| res.err())
                        .and_then(|e| {
                            Task::done(ErrorOccurred(format!("Failed to stop scan: {e}")))
                        })
                }
            }
            DeviceDisconnected => {
                self.connection_state = ConnectionState::NotConnected;
                self.heart_rate = None;
                if CentralState::PoweredOn == self.adapter_state {
                    Task::done(ScanDevice(true))
                } else {
                    Task::none()
                }
            }
            ShowHeartRateWindow(show) => {
                self.config.hr_window_visible = show;
                use iced::window::Mode::*;
                let mode = if show { Windowed } else { Hidden };
                window::set_mode(self.hr_window, mode).chain(window::gain_focus(self.main_window))
            }
            DiscoveredDevice(device) => {
                self.discovered_devices.push(device);
                Task::none()
            }
            SelectDevice(id) => {
                self.selected_device = Some(id);
                Task::none()
            }
            HeartRateUpdated(rate) => {
                self.heart_rate = Some(rate);
                Task::none()
            }
            ErrorOccurred(msg) => {
                self.set_error_message(msg);
                window::request_user_attention(
                    self.main_window,
                    Some(window::UserAttention::Informational),
                )
            }
        }
    }
}
