use std::sync::Arc;

use btleplug::api::{Central, CentralEvent, CentralState, Manager as _};
use btleplug::platform::{Adapter, Manager};
use iced::futures::{Stream, StreamExt, executor};
use iced::time::Instant;
use iced::window::gain_focus;
use iced::{Task, window};

use super::{App, HrsDevice, Message};
use crate::config::Config;

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let (adapter, adapter_state, adapter_events) = executor::block_on(init_bluetooth());

        let adapter2 = Arc::new(adapter.clone());
        let adapter_message = adapter_events.filter_map(move |event| {
            let adapter2 = adapter2.clone();
            async move {
                match event {
                    CentralEvent::DeviceDiscovered(id) => HrsDevice::from_id(&adapter2, &id)
                        .await
                        .map(Message::DiscoveredDevice),
                    CentralEvent::DeviceDisconnected(_) => Some(Message::DeviceDisconnected),
                    CentralEvent::StateUpdate(state) => Some(Message::AdapterStateUpdated(state)),
                    _ => None,
                }
            }
        });
        let adapter_events = Task::done(Message::AdapterStateUpdated(adapter_state.clone()))
            .chain(Task::stream(adapter_message));

        let config = Config::load().unwrap_or_default();
        let (main_window, open_main_window) = create_main_window();
        let (hr_window, open_hr_window) = create_hr_window(&config);

        (
            Self {
                adapter,
                adapter_state,
                connection_state: Default::default(),
                discovered_devices: Vec::new(),

                main_window,
                hr_window,
                selected_device: None,
                heart_rate: None,
                last_error: (String::new(), Instant::now() - iced::time::seconds(5)),

                config,
            },
            Task::batch([
                adapter_events,
                open_main_window,
                open_hr_window,
                gain_focus(main_window),
                Task::done(Message::LockHeartRateWindow(config.hr_window_locked)),
            ]),
        )
    }
}

async fn init_bluetooth() -> (
    Adapter,
    CentralState,
    impl Stream<Item = CentralEvent> + Send,
) {
    let adapter = Manager::new()
        .await
        .expect("Failed to get Bluetooth adapter.")
        .adapters()
        .await
        .expect("Failed to get Bluetooth adapter.")
        .into_iter()
        .next()
        .expect("Bluetooth adapter not found.");

    let adapter_state = adapter
        .adapter_state()
        .await
        .expect("Failed to get Bluetooth adapter state.");

    let adapter_events = adapter
        .events()
        .await
        .expect("Failed to listen Bluetooth adapter events.");

    (adapter, adapter_state, adapter_events)
}

fn create_main_window() -> (window::Id, Task<Message>) {
    let (id, open) = window::open(window::Settings {
        size: (550, 300).into(),
        min_size: Some((550, 220).into()),
        position: window::Position::Centered,
        ..Default::default()
    });
    (id, open.then(|_| Task::none()))
}

fn create_hr_window(config: &Config) -> (window::Id, Task<Message>) {
    #[cfg(target_os = "windows")]
    let platform = iced::window::settings::PlatformSpecific {
        skip_taskbar: true,
        ..Default::default()
    };
    let (id, open) = window::open(window::Settings {
        size: config.hr_window_size(),
        position: window::Position::Specific(config.hr_window_pos),
        visible: config.hr_window_visible,
        transparent: true,
        decorations: false,
        level: window::Level::AlwaysOnTop,
        #[cfg(target_os = "windows")]
        platform_specific: platform,
        ..Default::default()
    });
    (id, open.then(|_| Task::none()))
}
