use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::pin::Pin;

use btleplug::api::{AddressType, BDAddr, Central, Peripheral as _, bleuuid};
use btleplug::platform::{Adapter, Peripheral, PeripheralId};
use iced::futures::{Stream, StreamExt};
use uuid::Uuid;

use crate::hrm::HeartRateMeasurement;

pub(crate) const HRS_UUID: Uuid = bleuuid::uuid_from_u16(0x180D);
pub(crate) const HRM_UUID: Uuid = bleuuid::uuid_from_u16(0x2A37);

/// A Bluetooth device that provided _Heart Rate Service_
#[derive(Clone, Debug)]
pub struct HrsDevice {
    name: Option<String>,
    address_type: Option<AddressType>,
    peripheral: Peripheral,
}

impl HrsDevice {
    /// If the device is not discoverd by `adapter` or failed to get the properties associated with
    /// the device, the `None` returned.
    pub async fn from_id(adapter: &Adapter, id: &PeripheralId) -> Option<Self> {
        let peripheral = adapter.peripheral(id).await.ok()?;
        // (I think)Due to the `peripheral` is discoverd by the `adapter`, the properties always is `Some`
        let properties = peripheral.properties().await.ok().flatten()?;
        properties.services.contains(&HRS_UUID).then(|| HrsDevice {
            name: properties.local_name,
            address_type: properties.address_type,
            peripheral,
        })
    }

    pub fn address(&self) -> BDAddr {
        self.peripheral.address()
    }

    pub fn address_type(&self) -> Option<AddressType> {
        self.address_type
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_ref())
    }

    pub async fn connect(&self) -> btleplug::Result<()> {
        self.peripheral.connect().await
    }

    pub async fn disconnect(&self) -> btleplug::Result<()> {
        self.peripheral.disconnect().await
    }

    /// Subscribe the _Heart Rate Measurement_ characteristics and return a stream of heart rate
    /// for heart rate updates. This stream will be closed when the device disconnected.
    ///
    /// If the stream yield a `None`, it indicate the client received a new but invalid data.
    pub async fn subscribe(
        &self,
    ) -> btleplug::Result<Pin<Box<dyn Stream<Item = Option<HeartRateMeasurement>> + Send + 'static>>>
    {
        self.peripheral.discover_services().await?;
        let hrm_cahr = self
            .peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == HRM_UUID)
            .unwrap();
        self.peripheral.subscribe(&hrm_cahr).await?;
        Ok(self
            .peripheral
            .notifications()
            .await?
            .filter_map(async |v| {
                (v.uuid == HRM_UUID).then(|| HeartRateMeasurement::parse(&v.value[..]))
            })
            .boxed())
    }

    pub fn is_connected(&self) -> impl Future<Output = btleplug::Result<bool>> + Send + 'static {
        let device = self.peripheral.clone();
        async move { device.is_connected().await }
    }
}

impl Display for HrsDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{name}"),
            None => write!(f, "{}", self.address()),
        }
    }
}

impl Hash for HrsDevice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.address(), state)
    }
}

impl PartialEq for HrsDevice {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for HrsDevice {}
