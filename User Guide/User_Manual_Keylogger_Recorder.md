
# 🎛️ User Manual: Keylogger + Recorder Suite

---

## 📦 Overview

This program is a modular suite of scripts designed for **silent monitoring and activity logging** on a Windows PC. It consists of:

1. `ffmpeg_auto_recorder.py` – Automatically starts screen recording via FFmpeg based on activity or idle state.
2. `recorder_status_gui.py` – GUI interface for controlling and monitoring the recorders.
3. `keylogger.py` – Logs all keyboard input with timestamping, new line detection, and clipboard paste tracking.
4. `config.ini` – Central configuration file for paths, filenames, and other settings.
5. `ini_editor.py` – GUI tool to safely edit `config.ini`.

---

## 🧠 Purpose

The system is built to:
- Record screen activity silently for documentation, monitoring, or debugging.
- Log all keyboard input naturally, mimicking how a user types.
- Be persistent, recover gracefully, and require minimal manual intervention.
- Allow granular control (start, stop, monitor, terminate) of each subcomponent.

---

## 🗂️ Script Details

### 1. `ffmpeg_auto_recorder.py`
**Purpose**: Automatically record screen using FFmpeg based on idle time or always-on mode.

**Manual Kill Instructions**:
```cmd
taskkill /f /im python.exe
```
...or in Task Manager:
- Look for `python.exe` or `pythonw.exe`
- Right-click → End Task

### 2. `recorder_status_gui.py`
**Purpose**: Tkinter-based GUI that lets you:
- View active recorders (split by type)
- Start/Stop each independently
- View log console in styled terminal
- Monitor recordings folder

### 3. `keylogger.py`
**Purpose**: Silently logs keystrokes in a **natural** and **readable** way.

**Logged Features**:
- Printable characters, lowercase
- Timestamp per line
- Clipboard paste logging (`<PASTE: ...>`)

**Manual Stop**:
```cmd
taskkill /f /im python.exe
```

### 4. `config.ini`
**Purpose**: Central config for paths, filenames, etc.

### 5. `ini_editor.py`
**Purpose**: Simple GUI to update `config.ini` safely.

---

## 🧪 Troubleshooting

| Problem | Fix |
|--------|-----|
| No recordings | Check FFmpeg path, permissions |
| GUI won’t launch | Ensure Tkinter and GUI support |
| Network path issue | Test path manually in Explorer |
| Clipboard not captured | Check `pyperclip` installed and access allowed |

---

## ✅ Dependencies

Ensure these Python libraries are installed:
```bash
pip install pynput pyperclip psutil
```

---

## 📁 Folder Layout
```
C:\Tools\OBS\
│
├── Scripts\
│   ├── ffmpeg_auto_recorder.py
│   ├── recorder_status_gui.py
│   ├── keylogger.py
│   ├── ini_editor.py
│   └── config.ini
├── ffmpeg\
│   └── bin\ffmpeg.exe
└── Python\
```

---
