use std::env;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::locales::Language;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub hr_window_pos: iced::Point,
    pub hr_window_scale: f32,
    pub hr_window_visible: bool,
    pub hr_window_locked: bool,
    pub lang: Language,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct ConfigSerdeable {
    pub hr_window_pos: (f32, f32),
    pub hr_window_scale: f32,
    pub hr_window_visible: bool,
    pub hr_window_locked: bool,
    pub lang: Language,
}

fn config_path() -> PathBuf {
    let name = "hr_view.json";
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            return PathBuf::from(appdata).join(name);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("myapp_config.json");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home)
                .join(".config")
                .join("myapp_config.json");
        }
    }

    PathBuf::from(name)
}

impl Config {
    pub const DEFAULT_SIZE: iced::Size = iced::Size {
        width: 170.0,
        height: 70.0,
    };
    pub fn load() -> Option<Self> {
        let config = std::fs::read_to_string(config_path()).ok()?;
        let mut config = serde_json::from_str::<ConfigSerdeable>(&config)
            .ok()
            .map(Config::from)?;
        config.hr_window_scale = config.hr_window_scale.clamp(0.5, 5.0);
        Some(config)
    }

    pub fn save(&self) {
        if let Ok(config) = serde_json::to_string(&ConfigSerdeable::from(*self)) {
            let _ = std::fs::write(config_path(), config);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hr_window_pos: iced::Point::ORIGIN,
            hr_window_scale: 0.7,
            hr_window_visible: true,
            hr_window_locked: false,
            lang: sys_locale::get_locale()
                .map(|v| Language::from(v.as_str()))
                .unwrap_or_default(),
        }
    }
}

impl From<ConfigSerdeable> for Config {
    fn from(value: ConfigSerdeable) -> Self {
        Self {
            hr_window_pos: value.hr_window_pos.into(),
            hr_window_scale: value.hr_window_scale,
            hr_window_visible: value.hr_window_visible,
            hr_window_locked: value.hr_window_locked,
            lang: value.lang,
        }
    }
}

impl From<Config> for ConfigSerdeable {
    fn from(value: Config) -> Self {
        Self {
            hr_window_pos: (value.hr_window_pos.x, value.hr_window_pos.y),
            hr_window_scale: value.hr_window_scale,
            hr_window_visible: value.hr_window_visible,
            hr_window_locked: value.hr_window_locked,
            lang: value.lang,
        }
    }
}
