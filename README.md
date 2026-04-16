# SilentWitness

[![License: Non-Commercial](https://img.shields.io/badge/License-Non--Commercial-red.svg)](LICENSE.md)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-blue.svg)](https://www.microsoft.com/windows)
[![Built with: Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

A Windows screen-recording and keystroke-logging suite for consensual
monitoring (self-auditing, documented testing, supervised devices).

> **v0.2 rewrite — Rust, single self-contained `.exe`.** The previous Python
> 3.12 + portable-Python + PyInstaller stack has been replaced with a native
> Rust application. The legacy Python code is preserved under [`legacy/`](legacy/)
> for reference; all new work lives in [`rust/`](rust/).

---

## Quick start

1. Download `silentwitness.exe` from the
   [Releases](https://github.com/goblinrules/silentwitness/releases) page,
   **or** build it yourself (see below).
2. Place `ffmpeg.exe` and `ffprobe.exe` on your PATH, or drop them at
   `ffmpeg\bin\ffmpeg.exe` next to `silentwitness.exe`.
3. Run it:
   ```powershell
   .\silentwitness.exe            # tray + recorder
   .\silentwitness.exe monitor    # open the recordings/status window
   .\silentwitness.exe config     # graphical config editor
   .\silentwitness.exe doctor     # verify install
   ```

The first run writes a default `config.toml` to
`%LOCALAPPDATA%\SilentWitness\config.toml` — edit it via `silentwitness config`
or by hand.

## Features

- **Single self-contained `.exe`** — no Python, no installer, no virtualenv.
- **Always-visible tray icon** that changes colour while recording.
- **Modern `egui` GUI** — DPI-aware status monitor with a live recordings
  list, and a schema-validated config editor.
- **Automatic start / stop** based on `GetLastInputInfo` idle time.
- **Optional keystroke logging** alongside video (Ctrl+V paste content is
  captured; printable/control keys are serialised with timestamps).
- **Daily-rotated structured logs** via `tracing`.
- **Start Menu shortcut + HKCU\Run autostart** installer
  (`silentwitness install --autostart`).

## Screenshots

Legacy UI screenshots (still representative — same layout, new widgets):

- ![Monitor status](User%20Guide/Screens/Recorder%20Monitor%20Status%20Page.png)
- ![Config recording tab](User%20Guide/Screens/Recorder%20Config_Recording.png)
- ![Recordings list](User%20Guide/Screens/Recorder%20Monitor%20Recordings%20Page.png)

## Build from source

Requires **Rust 1.78+** and the **MSVC toolchain** on Windows.

```powershell
cd rust
cargo build --release
# or, for a packaged release including LICENSE and README:
.\build.ps1 -Zip
```

The resulting `rust\target\release\silentwitness.exe` is fully self-contained
(minus the FFmpeg sidecar). CI builds are produced by
[`.github/workflows/release.yml`](.github/workflows/release.yml) on every tag
starting with `v`.

## Configuration

See [`rust/README.md`](rust/README.md#config-configtoml) for the full TOML
schema. Anything you could set in the old `config.ini` has an equivalent key.

## Ethics & legal

This tool is **not stealth software** and must not be used as such. Before
deploying:

- Only install on devices you own or administer.
- Notify every user whose keystrokes or screen will be recorded.
- Comply with local wiretap / privacy / workplace monitoring law.
- Store recordings on encrypted volumes.

Contributions that add anti-detection, process-hiding, or silent-install
features **will be rejected**.

## License

Non-commercial use only — see [LICENSE.md](LICENSE.md).

## Legacy Python version

The original Python suite (portable Python 3.12 + pyinstaller + PowerShell
setup scripts) lives in [`legacy/`](legacy/). It is no longer maintained but
remains runnable for anyone mid-migration.
