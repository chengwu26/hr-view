use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh")]
    Chinese,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranslateItem {
    UnknownAdapterState,
    AdapterPowereddOff,
    ScanTitle,
    ConnectedTitle,
    ConnectButton,
    ConnectingButton,
    DisconnectButton,
    SettingsTitle,
    ShowHeartRateWindowSetting,
    LockHeartRateWindowSetting,
}

impl Language {
    pub const ALL: &[Self] = &[Language::English, Language::Chinese];
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::English => "English",
            Self::Chinese => "中文",
        };
        f.write_str(string)
    }
}

impl From<&str> for Language {
    fn from(value: &str) -> Self {
        if value.starts_with("zh") {
            Language::Chinese
        } else {
            Language::default()
        }
    }
}

impl TranslateItem {
    pub fn translate(&self, lang: Language) -> &'static str {
        translate(lang, *self)
    }
}

fn translate(lang: Language, key: TranslateItem) -> &'static str {
    use Language::*;
    use TranslateItem::*;
    match (lang, key) {
        (English, UnknownAdapterState) => "Unknown Bluetooth adapter state\nI CANNOT WORK!",
        (English, AdapterPowereddOff) => "Please turn on your Bluetooth adapter!",
        (English, ScanTitle) => "Discoverd devices",
        (English, ConnectedTitle) => "Connected:",
        (English, ConnectButton) => "Connect",
        (English, ConnectingButton) => "Connecting",
        (English, DisconnectButton) => "Disconnect",
        (English, SettingsTitle) => "Settings",
        (English, ShowHeartRateWindowSetting) => "Show heart rate window",
        (English, LockHeartRateWindowSetting) => "Lock heart rate window",

        (Chinese, UnknownAdapterState) => "蓝牙状态未知，无法继续",
        (Chinese, AdapterPowereddOff) => "不是，哥们儿！把蓝牙给开开！",
        (Chinese, ScanTitle) => "扫描到的设备",
        (Chinese, ConnectedTitle) => "连接到：",
        (Chinese, ConnectButton) => "连接",
        (Chinese, ConnectingButton) => "正在连接",
        (Chinese, DisconnectButton) => "断开设备",
        (Chinese, SettingsTitle) => "设置",
        (Chinese, ShowHeartRateWindowSetting) => "显示心率窗口",
        (Chinese, LockHeartRateWindowSetting) => "锁定心率窗口",
    }
}
