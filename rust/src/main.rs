// When built as a release Windows binary we want the windowed subsystem so no
// console pops up. Debug builds still get a console for `eprintln!`.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod config;
mod ffmpeg;
mod gui;
mod idle;
mod keylogger;
mod logging;
mod recorder;
mod startup;
mod tray;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
#[cfg(windows)]
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::config::Config;
use crate::recorder::Recorder;

/// SilentWitness — Windows screen recorder & keystroke logger.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Override path to config.toml
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the full tray + recorder loop (default if no subcommand given).
    Run,
    /// Open the monitor (recordings list + status) window.
    Monitor,
    /// Open the graphical config editor.
    Config,
    /// Run only the headless recorder (no tray, no window).
    Recorder,
    /// Worker subprocess that does key-logging to the given file.
    /// Not intended for direct user invocation.
    #[command(hide = true)]
    KeylogWorker {
        #[arg(long)]
        output: PathBuf,
    },
    /// Install Start Menu shortcut and (optionally) HKCU\Run autostart.
    Install {
        #[arg(long)]
        autostart: bool,
    },
    /// Remove autostart registration.
    Uninstall,
    /// Print version, resolved config path, and detected ffmpeg.
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let (cfg, cfg_path) = Config::load_or_create(cli.config.as_deref())?;

    match cli.command.unwrap_or(Command::Run) {
        Command::Run => cmd_run(cfg),
        Command::Monitor => {
            let _g = logging::init(&cfg.logging, "monitor")?;
            gui::monitor::run(cfg)
        }
        Command::Config => {
            let _g = logging::init(&cfg.logging, "config")?;
            gui::config_editor::run(cfg, cfg_path)
        }
        Command::Recorder => cmd_recorder_only(cfg),
        Command::KeylogWorker { output } => cmd_keylog_worker(output),
        Command::Install { autostart } => cmd_install(cfg, autostart),
        Command::Uninstall => cmd_uninstall(),
        Command::Doctor => cmd_doctor(cfg, cfg_path),
    }
}

fn cmd_run(cfg: Config) -> Result<()> {
    let _guard = logging::init(&cfg.logging, "run")?;
    tracing::info!("SilentWitness starting (full mode)");

    let recorder = Arc::new(Recorder::new());
    let status_rx = recorder.subscribe();

    // Spawn the recorder loop on a dedicated thread.
    let cfg_clone = cfg.clone();
    let recorder_clone = Arc::clone(&recorder);
    std::thread::Builder::new()
        .name("recorder".into())
        .spawn(move || {
            if let Err(e) = recorder_clone.run(cfg_clone) {
                tracing::error!("recorder loop exited with error: {e:#}");
            }
        })?;

    if !cfg.tray.enabled {
        tracing::warn!("tray disabled in config — running headless");
        wait_for_ctrl_c();
        recorder.request_stop();
        return Ok(());
    }

    // Tray must run on the main thread on Windows/macOS.
    let handles = tray::build(
        cfg.tray.tooltip_idle.clone(),
        cfg.tray.tooltip_recording.clone(),
    )?;

    run_tray_event_loop(handles, recorder, status_rx)
}

#[cfg(windows)]
fn run_tray_event_loop(
    handles: tray::TrayHandles,
    recorder: Arc<Recorder>,
    mut status_rx: tokio::sync::watch::Receiver<recorder::RecorderStatus>,
) -> Result<()> {
    // On Windows, tray-icon needs a message pump.
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };

    let stop = Arc::new(AtomicBool::new(false));
    let exe = std::env::current_exe()?;
    // Null HWND tells PeekMessageW to pull messages for any window on the
    // current thread — exactly what tray-icon needs.
    let null_hwnd: HWND = HWND(std::ptr::null_mut());

    loop {
        if stop.load(Ordering::Relaxed) {
            recorder.request_stop();
            break;
        }

        // Pump Windows messages for the tray.
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, null_hwnd, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        // Sync tray icon/tooltip with recorder status.
        handles.poll_status(&mut status_rx);

        // Handle tray menu events.
        while let Some(ev) = tray::try_recv_menu_event() {
            if ev.id == handles.item_quit.id() {
                stop.store(true, Ordering::Relaxed);
            } else if ev.id == handles.item_open_monitor.id() {
                spawn_self(&exe, &["monitor"]);
            } else if ev.id == handles.item_open_config.id() {
                spawn_self(&exe, &["config"]);
            } else if ev.id == handles.item_toggle.id() {
                tracing::info!("toggle requested — not implemented yet");
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

#[cfg(not(windows))]
fn run_tray_event_loop(
    handles: tray::TrayHandles,
    recorder: Arc<Recorder>,
    mut status_rx: tokio::sync::watch::Receiver<recorder::RecorderStatus>,
) -> Result<()> {
    let exe = std::env::current_exe()?;
    loop {
        handles.poll_status(&mut status_rx);
        while let Some(ev) = tray::try_recv_menu_event() {
            if ev.id == handles.item_quit.id() {
                recorder.request_stop();
                return Ok(());
            } else if ev.id == handles.item_open_monitor.id() {
                spawn_self(&exe, &["monitor"]);
            } else if ev.id == handles.item_open_config.id() {
                spawn_self(&exe, &["config"]);
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn spawn_self(exe: &std::path::Path, args: &[&str]) {
    let _ = std::process::Command::new(exe).args(args).spawn();
}

fn cmd_recorder_only(cfg: Config) -> Result<()> {
    let _g = logging::init(&cfg.logging, "recorder")?;
    tracing::info!("SilentWitness headless recorder");
    let rec = Arc::new(Recorder::new());
    rec.run(cfg)
}

fn cmd_keylog_worker(output: PathBuf) -> Result<()> {
    // Minimal logging — this subprocess exits when its parent kills it.
    let stop = Arc::new(AtomicBool::new(false));
    keylogger::run(output, stop)
}

fn cmd_install(cfg: Config, autostart: bool) -> Result<()> {
    let exe = std::env::current_exe()?;
    startup::install_startup_shortcut(&exe, &["run"], "SilentWitness", autostart)?;
    println!("Installed Start Menu shortcut (autostart={autostart})");
    if cfg.recording.enable_key_logging {
        eprintln!(
            "NOTE: keystroke logging is enabled in config. Ensure you have \
             consent from everyone who uses this device."
        );
    }
    Ok(())
}

fn cmd_uninstall() -> Result<()> {
    startup::remove_startup("SilentWitness")?;
    println!("Removed autostart entry");
    Ok(())
}

fn cmd_doctor(cfg: Config, cfg_path: PathBuf) -> Result<()> {
    println!("SilentWitness v{}", env!("CARGO_PKG_VERSION"));
    println!("Config:         {}", cfg_path.display());
    println!(
        "ffmpeg:         {}",
        ffmpeg::ffmpeg_path(&cfg)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<NOT FOUND>".into())
    );
    println!(
        "ffprobe:        {}",
        ffmpeg::ffprobe_path(&cfg)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<NOT FOUND>".into())
    );
    println!("Recording dir:  {}", cfg.recording.recording_dir);
    println!("Log dir:        {}", cfg.logging.log_dir);
    println!("Tray enabled:   {}", cfg.tray.enabled);
    Ok(())
}

fn wait_for_ctrl_c() {
    // Simple busy-wait fallback without a ctrlc dependency: console Ctrl+C
    // will terminate the process anyway on Windows.
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
