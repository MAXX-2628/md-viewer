use crate::zoom;
use std::path::PathBuf;

pub struct Settings {
    pub dark: bool,
    pub zoom: f32,
    pub last_file: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark: false,
            zoom: 1.0,
            last_file: None,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let mut settings = Self::default();
        let Ok(text) = std::fs::read_to_string(Self::path()) else {
            return settings;
        };

        for line in text.lines() {
            if let Some(value) = line.strip_prefix("dark=") {
                settings.dark = value == "1";
            } else if let Some(value) = line.strip_prefix("zoom=") {
                if let Ok(value) = value.parse::<f32>() {
                    settings.zoom = zoom::clamp(value);
                }
            } else if let Some(value) = line.strip_prefix("last=")
                && !value.is_empty()
            {
                settings.last_file = Some(PathBuf::from(value));
            }
        }

        settings
    }

    pub fn save(&self) {
        let mut text = format!("dark={}\nzoom={}\n", u8::from(self.dark), self.zoom);
        if let Some(path) = &self.last_file {
            text.push_str("last=");
            text.push_str(&path.to_string_lossy());
            text.push('\n');
        }

        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, text);
    }

    fn path() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("LOCALAPPDATA").map(PathBuf::from))
            .unwrap_or_else(std::env::temp_dir);
        base.join("MD Viewer").join("settings.txt")
    }
}
