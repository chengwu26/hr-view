//! Bluetooth operation
//!
//! Due to the requirements of [iced::Task](iced::Task) on futures, all returned `Future` have
//! `static` lifetime.

use std::pin::Pin;

use btleplug::api::{BDAddr, Central, ScanFilter};
use iced::futures::Stream;

use super::{App, ConnectionState};
use crate::hrm::HeartRateMeasurement;
use crate::hrs_device::{HRS_UUID, HrsDevice};

impl App {
    /// Connect selected device, return `Ok` that contain the connected device's address if
    /// operation successful.
    ///
    /// # Panic:
    /// - `selected_device` is `None` or it not in the `discoverd_devices`
    /// - `connection_state` is not `NotConnected` variant
    pub(crate) fn connect(
        &self,
    ) -> impl Future<Output = Result<BDAddr, btleplug::Error>> + 'static {
        if self.connection_state != ConnectionState::NotConnected {
            unreachable!(
                "[BUG] Received a 'ConnectDevice' message, but current connection state is '{:?}'.",
                self.connection_state
            );
        }
        let address = *self
            .selected_device
            .as_ref()
            .expect("[BUG] Attempt connect a device, but not select any device yet.");
        let device = self
            .discovered_devices
            .iter()
            .find(|d| d.address() == address)
            .expect("[BUG] Attempt connect a undiscoverd device.")
            .clone();
        async move {
            device.connect().await?;
            Ok(address)
        }
    }

    /// Disconnect connected device. If no connected device, return Ok.
    pub(crate) fn disconnect(&self) -> impl Future<Output = Result<(), btleplug::Error>> + 'static {
        let device = self.connected_device().cloned();
        async move {
            match device {
                None => Ok(()),
                Some(device) => device.disconnect().await,
            }
        }
    }

    /// Subscribe the _Heart Rate Measurement_ characteristics and return a stream of heart rate
    /// for heart rate updates. Yield `None` whe received a invalid heart rate data from device.
    /// This stream will be closed when the device disconnected.
    ///
    /// # Panic
    /// - no connected device
    pub(crate) fn subscribe(
        &self,
    ) -> impl Future<
        Output = Result<
            Pin<Box<dyn Stream<Item = Option<HeartRateMeasurement>> + Send + 'static>>,
            btleplug::Error,
        >,
    > + 'static {
        let device = self
            .connected_device()
            .expect("[BUG] Attempt subscribe heart rate, but no device connected.")
            .clone();
        async move { device.subscribe().await }
    }

    /// Start scan _Heart Rate Service_ device
    pub(crate) fn start_scan(&self) -> impl Future<Output = Result<(), btleplug::Error>> + 'static {
        let adapter = self.adapter.clone();
        async move {
            adapter
                .start_scan(ScanFilter {
                    services: vec![HRS_UUID],
                })
                .await
        }
    }

    pub(crate) fn stop_scan(&self) -> impl Future<Output = Result<(), btleplug::Error>> + 'static {
        let adapter = self.adapter.clone();
        async move { adapter.stop_scan().await }
    }

    pub(crate) fn connected_device(&self) -> Option<&HrsDevice> {
        let ConnectionState::Connected(addr) = self.connection_state else {
            return None;
        };
        self.discovered_devices.iter().find(|d| d.address() == addr)
    }
}
