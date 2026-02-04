mod bluetooth;
mod boot;
mod subscription;
mod update;
mod view;

use btleplug::api::{BDAddr, CentralState};
use btleplug::platform::Adapter;
use iced::time::Instant;
use iced::window;

use crate::config::Config;
use crate::hrm::HeartRateMeasurement;
use crate::hrs_device::HrsDevice;
use crate::locales::Language;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    NotConnected,
    Connecting,
    Connected(BDAddr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockResize {
    Increment,
    Decrease,
}

#[derive(Debug, Clone)]
pub enum Message {
    Exit,
    SelectDevice(BDAddr),
    ConnectDevice,
    DisconnectDevice,
    ShowHeartRateWindow(bool),
    LockHeartRateWindow(bool),
    MouseEvent(iced::mouse::Event, window::Id),
    LanguageChanged(Language),

    HeartRateWindowResize(BlockResize),
    ScanDevice(bool),
    AdapterStateUpdated(CentralState),
    ConnectionStateUpdated(ConnectionState),
    DiscoveredDevice(HrsDevice),
    DeviceDisconnected,
    HeartRateUpdated(HeartRateMeasurement),
    ErrorOccurred(String),
}

#[derive(Debug)]
pub struct App {
    adapter: Adapter,
    adapter_state: CentralState,
    connection_state: ConnectionState,
    discovered_devices: Vec<HrsDevice>,

    main_window: window::Id,
    hr_window: window::Id,
    selected_device: Option<BDAddr>,
    heart_rate: Option<HeartRateMeasurement>,
    last_error: (String, Instant),

    config: Config,
}

impl App {
    pub fn theme(&self, id: window::Id) -> Option<iced::Theme> {
        (id == self.main_window).then_some(iced::Theme::CatppuccinMacchiato)
    }

    /// Get last error message, if the last error is more than 5 seconds now, this message will be
    /// considered expired and this function will return `None`.
    fn error_message(&self) -> Option<&str> {
        (self.last_error.1.elapsed() < iced::time::seconds(5)).then_some(&self.last_error.0)
    }

    fn set_error_message(&mut self, msg: String) {
        self.last_error = (msg, Instant::now())
    }
}
