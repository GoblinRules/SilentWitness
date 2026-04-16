//! System tray icon with two visual states (idle / recording).
//!
//! The tray is always shown while the background recorder is running so users
//! have a persistent, visible indicator of what the app is doing.
//!
//! `TrayIcon` contains `Rc`s internally and therefore isn't `Send`; to avoid
//! dragging it across threads we update the tray synchronously from the
//! thread that owns the Windows message pump via [`TrayHandles::poll_status`].

use anyhow::{Context, Result};
use tokio::sync::watch;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

use crate::recorder::RecorderStatus;

const ICON_IDLE_BYTES: &[u8] = include_bytes!("../assets/icon_idle.ico");
const ICON_RECORDING_BYTES: &[u8] = include_bytes!("../assets/icon_recording.ico");

fn load_icon(bytes: &[u8]) -> Result<Icon> {
    let img = image::load_from_memory(bytes)
        .context("decoding tray icon image")?
        .to_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).context("building tray icon")
}

pub struct TrayHandles {
    pub tray: TrayIcon,
    pub item_open_monitor: MenuItem,
    pub item_open_config: MenuItem,
    #[allow(dead_code)]
    pub item_toggle: MenuItem,
    pub item_quit: MenuItem,
    pub icon_idle: Icon,
    pub icon_recording: Icon,
    pub tooltip_idle: String,
    pub tooltip_recording: String,
}

pub fn build(tooltip_idle: String, tooltip_recording: String) -> Result<TrayHandles> {
    let icon_idle = load_icon(ICON_IDLE_BYTES)?;
    let icon_recording = load_icon(ICON_RECORDING_BYTES)?;

    let menu = Menu::new();
    let item_open_monitor = MenuItem::new("Open Monitor…", true, None);
    let item_open_config = MenuItem::new("Open Config…", true, None);
    let item_toggle = MenuItem::new("Pause / Resume Recording", true, None);
    let item_quit = MenuItem::new("Quit SilentWitness", true, None);

    menu.append(&item_open_monitor)?;
    menu.append(&item_open_config)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&item_toggle)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&item_quit)?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(&tooltip_idle)
        .with_icon(icon_idle.clone())
        .build()
        .context("building tray icon")?;

    Ok(TrayHandles {
        tray,
        item_open_monitor,
        item_open_config,
        item_toggle,
        item_quit,
        icon_idle,
        icon_recording,
        tooltip_idle,
        tooltip_recording,
    })
}

impl TrayHandles {
    /// Check the recorder status channel for a change and update the tray
    /// icon + tooltip if something new has arrived. Safe to call frequently.
    pub fn poll_status(&self, status_rx: &mut watch::Receiver<RecorderStatus>) {
        if status_rx.has_changed().unwrap_or(false) {
            let status = status_rx.borrow_and_update().clone();
            let (icon, tip) = match status {
                RecorderStatus::Recording { .. } => (
                    self.icon_recording.clone(),
                    self.tooltip_recording.clone(),
                ),
                RecorderStatus::Idle => (self.icon_idle.clone(), self.tooltip_idle.clone()),
                RecorderStatus::Error(e) => {
                    (self.icon_idle.clone(), format!("SilentWitness error: {e}"))
                }
            };
            let _ = self.tray.set_icon(Some(icon));
            let _ = self.tray.set_tooltip(Some(&tip));
        }
    }
}

/// Poll the menu event channel; returns the next `MenuEvent` if one is
/// available without blocking.
pub fn try_recv_menu_event() -> Option<MenuEvent> {
    MenuEvent::receiver().try_recv().ok()
}
