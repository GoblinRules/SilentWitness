# SilentWitness User Manual - Screen Recording & Keylogging Suite

## 🎯 Overview

SilentWitness is a comprehensive Windows-based suite for automated screen recording and keystroke logging. It's designed for legitimate monitoring purposes such as:

- **Documentation & Training** - Record software demonstrations
- **Debugging & Support** - Capture user interactions for troubleshooting
- **Compliance & Audit** - Maintain records of system usage
- **Research & Analysis** - Study user behavior patterns

## ⚠️ Important Notice

**This tool is designed for legitimate monitoring purposes only. Users must ensure compliance with:**
- Local privacy laws and regulations
- Company policies and consent requirements
- Ethical guidelines for monitoring
- Data protection standards

## 🚀 Quick Start

### 1. **Run the main recorder:**
```cmd
cd C:\Tools\SilentWitness\Scripts
python ffmpeg_auto_recorder.py
```

### 2. **Open the status GUI:**
```cmd
python recorder_status_gui.py
```

### 3. **Start keylogging:**
```cmd
python keylogger.py
```

## 📋 Component Overview

### 1. `ffmpeg_auto_recorder.py`
**Purpose**: Main automation script that:
- Starts/stops recording based on user activity
- Manages FFmpeg processes
- Handles idle detection
- Provides system tray integration

**Manual Stop**:
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
|---------|-----|
| No recordings | Check FFmpeg path, permissions |
| GUI won't launch | Ensure Tkinter and GUI support |
| Network path issue | Test path manually in Explorer |
| Clipboard not captured | Check `pyperclip` installed and access allowed |

---

## ✅ Dependencies

Ensure these Python libraries are installed:
```bash
pip install pynput pyperclip psutil
```

---

## 🏗️ Build Instructions

### Setting up the Portable Python Environment:

1. **Upgrade pip and install core dependencies:**
   ```cmd
   cd C:\Tools\SilentWitness\Python
   python -m pip install --upgrade pip setuptools wheel
   python -m pip install pyautogui pymsgbox pygetwindow pytweening pyscreeze pyrect pywin32
   python -m pip install pystray
   ```

2. **Configure Python path file:**
   Navigate to `C:\Tools\SilentWitness\Python` and edit `python312._pth`:
   ```
   python312.zip
   .
   
   # Uncomment to run site.main() automatically
   import site
   Lib\site-packages
   ```

3. **Build executable (optional):**
   ```cmd
   cd C:\Tools\SilentWitness\Scripts
   C:\Tools\SilentWitness\Python\Scripts\pyinstaller.exe ffmpeg_auto_recorder.spec
   ```

---

## 📁 Folder Layout
```
C:\Tools\SilentWitness\
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

## 🔧 Configuration

### Key Settings in `config.ini`:

**Paths Section:**
- `ffmpeg_path` - Path to FFmpeg executable
- `key_overlay_exe` - Path to key overlay application
- `log_dir` - Directory for log files
- `recording_dir` - Directory for video recordings

**Recording Section:**
- `record_duration` - Maximum recording length (HH:MM:SS)
- `idle_threshold` - Time before stopping due to inactivity
- `check_interval` - Seconds between idle checks
- `enable_key_overlay` - Show key presses on screen
- `enable_key_logging` - Log keystrokes to text files

**Tray Section:**
- `enable_tray_icon` - Show system tray icon
- `icon_idle` - Icon when not recording
- `icon_recording` - Icon when recording

**Logging Section:**
- `enable_logging` - Enable/disable logging
- `log_to_file` - Write logs to files
- `log_level` - Logging verbosity (DEBUG, INFO, WARNING, ERROR)

---

## 📊 Usage Examples

### Basic Recording:
```cmd
cd C:\Tools\SilentWitness\Scripts
python ffmpeg_auto_recorder.py
```

### Monitor Status:
```cmd
python recorder_status_gui.py
```

### Manual Keylogging:
```cmd
python keylogger.py output.txt
```

### Edit Configuration:
```cmd
python ini_editor.py
```

---

## 🚨 Security Considerations

1. **Access Control**: Ensure only authorized users can access the system
2. **Data Protection**: Encrypt sensitive recordings and logs
3. **Retention Policy**: Implement automatic cleanup of old files
4. **Audit Trail**: Log all access and configuration changes
5. **Consent**: Obtain proper consent before monitoring users

---

## 📞 Support

For issues or questions:
1. Check the troubleshooting section above
2. Review log files in the Logs directory
3. Verify all dependencies are installed
4. Ensure proper file permissions

---

**Remember**: SilentWitness is a powerful tool. Use it responsibly and in accordance with all applicable laws and regulations.
