//! TOML configuration for SilentWitness.
//!
//! Config is resolved in the following precedence order:
//!   1. Path provided via `--config <path>`
//!   2. `%APPDATA%\SilentWitness\config.toml`
//!   3. `<exe_dir>/config.toml`
//!   4. Default values (materialised to the APPDATA location on first run)

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_check_interval() -> u64 {
    10
}

fn default_framerate() -> u32 {
    30
}

fn default_record_duration() -> String {
    "01:30:00".to_string()
}

fn default_idle_threshold() -> String {
    "01:00:00".to_string()
}

fn default_filename_template() -> String {
    "{username}-{timestamp}.mkv".to_string()
}

fn default_crf() -> u32 {
    25
}

fn default_preset() -> String {
    "veryfast".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    /// Path to the ffmpeg.exe binary. Leave empty to auto-detect via PATH or
    /// the bundled `<exe_dir>/ffmpeg/bin/ffmpeg.exe` location.
    pub ffmpeg: String,
    /// Path to ffprobe (used by the monitor GUI for durations).
    pub ffprobe: String,
    /// Optional external overlay executable (e.g. Carnac / KeyOverlay).
    pub key_overlay: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            ffmpeg: String::new(),
            ffprobe: String::new(),
            key_overlay: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RecordingConfig {
    pub recording_dir: String,
    #[serde(default = "default_filename_template")]
    pub filename_template: String,
    #[serde(default = "default_true")]
    pub record_video: bool,
    #[serde(default = "default_record_duration")]
    pub record_duration: String,
    #[serde(default = "default_idle_threshold")]
    pub idle_threshold: String,
    #[serde(default = "default_check_interval")]
    pub check_interval: u64,
    #[serde(default = "default_framerate")]
    pub framerate: u32,
    #[serde(default = "default_crf")]
    pub crf: u32,
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(default)]
    pub enable_key_logging: bool,
    #[serde(default)]
    pub enable_key_overlay: bool,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            recording_dir: default_recording_dir(),
            filename_template: default_filename_template(),
            record_video: true,
            record_duration: default_record_duration(),
            idle_threshold: default_idle_threshold(),
            check_interval: default_check_interval(),
            framerate: default_framerate(),
            crf: default_crf(),
            preset: default_preset(),
            enable_key_logging: false,
            enable_key_overlay: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrayConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_tooltip_idle")]
    pub tooltip_idle: String,
    #[serde(default = "default_tooltip_recording")]
    pub tooltip_recording: String,
}

fn default_tooltip_idle() -> String {
    "SilentWitness — idle".into()
}
fn default_tooltip_recording() -> String {
    "SilentWitness — recording".into()
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tooltip_idle: default_tooltip_idle(),
            tooltip_recording: default_tooltip_recording(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub to_file: bool,
    pub log_dir: String,
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            to_file: true,
            log_dir: default_log_dir(),
            level: default_log_level(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StartupConfig {
    pub enable_startup_menu: bool,
    pub enable_auto_start: bool,
    pub start_recorder: bool,
    pub start_monitor: bool,
    pub startup_delay_secs: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub paths: PathsConfig,
    pub recording: RecordingConfig,
    pub tray: TrayConfig,
    pub logging: LoggingConfig,
    pub startup: StartupConfig,
}

fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("SilentWitness")
}

fn default_recording_dir() -> String {
    data_dir()
        .join("Recordings")
        .to_string_lossy()
        .into_owned()
}

fn default_log_dir() -> String {
    data_dir().join("Logs").to_string_lossy().into_owned()
}

/// Default on-disk path for the config file.
pub fn default_config_path() -> PathBuf {
    data_dir().join("config.toml")
}

impl Config {
    /// Load config, writing defaults to disk if no file exists yet.
    pub fn load_or_create(explicit: Option<&Path>) -> Result<(Self, PathBuf)> {
        let path = Self::resolve_path(explicit);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating config dir {:?}", parent))?;
            }
            let cfg = Self::default();
            let toml_str = toml::to_string_pretty(&cfg)?;
            std::fs::write(&path, toml_str)
                .with_context(|| format!("writing default config to {:?}", path))?;
            return Ok((cfg, path));
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading config at {:?}", path))?;
        let cfg: Config = toml::from_str(&content)
            .with_context(|| format!("parsing config at {:?}", path))?;
        Ok((cfg, path))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let toml_str = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_str).with_context(|| format!("writing config to {:?}", path))?;
        Ok(())
    }

    fn resolve_path(explicit: Option<&Path>) -> PathBuf {
        if let Some(p) = explicit {
            return p.to_path_buf();
        }
        let appdata = default_config_path();
        if appdata.exists() {
            return appdata;
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let local = dir.join("config.toml");
                if local.exists() {
                    return local;
                }
            }
        }
        appdata
    }
}

/// Parse an HH:MM:SS (or MM:SS) duration into seconds.
pub fn parse_hms(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    let (h, m, sec) = match parts.as_slice() {
        [h, m, s] => (h.parse::<u64>().ok()?, m.parse::<u64>().ok()?, s.parse::<u64>().ok()?),
        [m, s] => (0u64, m.parse::<u64>().ok()?, s.parse::<u64>().ok()?),
        [s] => (0u64, 0u64, s.parse::<u64>().ok()?),
        _ => return None,
    };
    Some(h * 3600 + m * 60 + sec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hms() {
        assert_eq!(parse_hms("01:30:00"), Some(5400));
        assert_eq!(parse_hms("10:00"), Some(600));
        assert_eq!(parse_hms("45"), Some(45));
        assert_eq!(parse_hms("bad"), None);
    }

    #[test]
    fn roundtrip_defaults() {
        let cfg = Config::default();
        let text = toml::to_string_pretty(&cfg).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.recording.framerate, cfg.recording.framerate);
    }
}
