# SilentWitness (Rust)

Modernised, single-binary rewrite of the SilentWitness Windows screen-recording
and keystroke-logging suite. Replaces the previous Python 3.12 + portable-Python
+ PyInstaller stack with a native Rust application.

## Why the rewrite?

- **Single self-contained `.exe`** — no portable Python, no `pip install`, no
  PATH fiddling. One file.
- **Faster, smaller, fewer dependencies** — ~5 MB release binary (plus FFmpeg
  sidecar) instead of hundreds of MB of embedded Python.
- **Modern GUI** — `egui`-based status and config windows with live refresh,
  DPI-aware, styled to match Windows 11.
- **Structured config** — TOML with strong types and defaults, instead of INI.
- **Structured logs** — daily-rotated log files via `tracing`.
- **Strong type safety** — the compiler catches whole classes of bugs the
  dynamic Python version had to handle at runtime.

## Visibility / transparency

SilentWitness is intended for **consensual monitoring** (self-auditing,
employee monitoring with notice, documentation, debugging). The rewrite
preserves and strengthens visible indicators:

- Persistent tray icon that changes colour when recording.
- Monitor window with a live process/status tab and recordings list.
- On `silentwitness install`, the installer prints a reminder when keystroke
  logging is enabled.

There are **no stealth, anti-detection, or anti-forensic features.** Do not
use this tool on devices you don't own or administer, and ensure you comply
with local law.

## Build

Requires Rust 1.78+ and the MSVC toolchain on Windows.

```powershell
cd rust
cargo build --release
# result: rust\target\release\silentwitness.exe
```

FFmpeg is **not** embedded in the binary. Point `[paths].ffmpeg` in
`config.toml` at an `ffmpeg.exe`, or drop a copy next to `silentwitness.exe`
under `ffmpeg\bin\ffmpeg.exe` and it will be auto-detected.

## Usage

```
silentwitness                 # full tray + recorder (default)
silentwitness monitor         # open the recordings monitor window
silentwitness config          # open the graphical config editor
silentwitness recorder        # headless recorder, no tray
silentwitness install --autostart   # add Start Menu shortcut + HKCU\Run
silentwitness uninstall       # remove autostart
silentwitness doctor          # print resolved paths / versions
```

## Config (`config.toml`)

Stored at `%LOCALAPPDATA%\SilentWitness\config.toml` by default.
The file is created with sensible defaults the first time the app runs.

```toml
[paths]
ffmpeg = ""     # leave empty for auto-detection
ffprobe = ""
key_overlay = ""

[recording]
recording_dir = "C:/Users/you/AppData/Local/SilentWitness/Recordings"
filename_template = "{username}-{timestamp}.mkv"
record_video = true
record_duration = "01:30:00"
idle_threshold = "01:00:00"
check_interval = 10
framerate = 30
crf = 25
preset = "veryfast"
enable_key_logging = false
enable_key_overlay = false

[tray]
enabled = true
tooltip_idle = "SilentWitness — idle"
tooltip_recording = "SilentWitness — recording"

[logging]
enabled = true
to_file = true
log_dir = "C:/Users/you/AppData/Local/SilentWitness/Logs"
level = "info"

[startup]
enable_startup_menu = false
enable_auto_start = false
start_recorder = true
start_monitor = false
startup_delay_secs = 10
```

## Layout

```
rust/
├── Cargo.toml
├── build.rs                  # embeds app.ico + Win manifest
├── assets/                   # ICO files baked into the exe
├── src/
│   ├── main.rs               # CLI + tray event loop
│   ├── config.rs             # TOML schema + loader
│   ├── logging.rs            # tracing setup
│   ├── idle.rs               # GetLastInputInfo wrapper
│   ├── ffmpeg.rs             # binary discovery + process spawn
│   ├── recorder.rs           # idle→record state machine
│   ├── keylogger.rs          # rdev key capture
│   ├── tray.rs               # system tray icon
│   ├── startup.rs            # Start Menu shortcut + HKCU\Run
│   └── gui/
│       ├── mod.rs
│       ├── monitor.rs        # recordings list window
│       └── config_editor.rs  # TOML-backed config UI
```

## Migrating from the Python version

1. Install the new exe.
2. Run `silentwitness doctor` — it will write a default `config.toml` at
   `%LOCALAPPDATA%\SilentWitness\config.toml`.
3. Copy settings from the old `Scripts\config.ini` into the new TOML (the
   names are mostly the same; `record_duration` and `idle_threshold` still
   accept `HH:MM:SS` strings).
4. Point `[paths].ffmpeg` at your existing `ffmpeg.exe` or move the
   `ffmpeg\` folder next to the new exe.
5. `silentwitness install --autostart` replaces
   `startup_manager.py --enable-auto-start`.

The legacy Python code is preserved under `legacy/` for reference.
