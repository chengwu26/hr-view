use std::{fmt::Display, num::NonZeroU16};

const HEART_RATE_16BIT: u8 = 1;
const SENSOR_CONTACT_STATUS: u8 = 1 << 1;
const SENSOR_CONTACT_SUPPORT: u8 = 1 << 2;
const ENERGY_EXPENDED_SUPPORT: u8 = 1 << 3;
const RR_INTERVAL_SUPPORT: u8 = 1 << 4;

/// LE Bluetooth _Heart Rate Measurement_ characteristic data represent
#[derive(Debug, Clone, Copy)]
pub struct HeartRateMeasurement {
    /// Unit: bpm
    pub heart_rate: u16,
    pub sensor_contact: Option<bool>,
    /// Unit: kiloJoules
    pub energy_expended: Option<u16>,
    /// Unit: 1/1024 seconds
    pub rr_interval: Option<NonZeroU16>,
}

impl HeartRateMeasurement {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        let mut raw = raw.iter();
        let flags = raw.next()?;

        let check_flag = |flag: u8| flags & flag == flag;
        fn next_u16<'a, I: Iterator<Item = &'a u8>>(iter: &mut I) -> Option<u16> {
            Some(u16::from_le_bytes([*iter.next()?, *iter.next()?]))
        }

        let heart_rate = if check_flag(HEART_RATE_16BIT) {
            next_u16(&mut raw)
        } else {
            raw.next().map(|v| *v as u16)
        }?;
        let sensor_contact =
            check_flag(SENSOR_CONTACT_SUPPORT).then_some(check_flag(SENSOR_CONTACT_STATUS));
        let energy_expended = check_flag(ENERGY_EXPENDED_SUPPORT)
            .then(|| next_u16(&mut raw))
            .flatten();
        let rr_interval = check_flag(RR_INTERVAL_SUPPORT)
            .then(|| next_u16(&mut raw).and_then(NonZeroU16::new))
            .flatten();

        Some(Self {
            heart_rate,
            sensor_contact,
            energy_expended,
            rr_interval,
        })
    }
}

impl Display for HeartRateMeasurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn display_or_na<T: ToString>(opt: Option<T>, unit: &str) -> String {
            opt.map_or("N/A".into(), |v| format!("{}{unit}", v.to_string()))
        }
        writeln!(
            f,
            "Heart rate: {}",
            display_or_na(Some(self.heart_rate), " bpm")
        )?;
        writeln!(
            f,
            "Sensor contact: {}",
            display_or_na(self.sensor_contact, "")
        )?;
        writeln!(
            f,
            "Energy expended: {}",
            display_or_na(self.energy_expended, " kJ")
        )?;
        writeln!(
            f,
            "RR-Interval: {}",
            display_or_na(self.rr_interval.map(|v| v.get() * 1000 / 1024), " ms")
        )
    }
}
