# 🎥 SilentWitness Screen Recording & Keylogging Suite

[![License: Non-Commercial](https://img.shields.io/badge/License-Non--Commercial-red.svg)](LICENSE.md)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-blue.svg)](https://www.microsoft.com/windows)
[![Python: 3.12+](https://img.shields.io/badge/Python-3.12+-green.svg)](https://www.python.org/)

A comprehensive Windows-based suite for automated screen recording and keystroke logging using FFmpeg and Python.

## 🚀 Quick Start

1. **Run the main recorder:**
   ```cmd
   cd Scripts
   python ffmpeg_auto_recorder.py
   ```

2. **Open the status GUI:**
   ```cmd
   python recorder_status_gui.py
   ```

3. **Start keylogging:**
   ```cmd
   python keylogger.py
   ```

## 📁 Project Structure

```
C:\Tools\SilentWitness\
├── Scripts\                 # Core Python scripts
│   ├── ffmpeg_auto_recorder.py
│   ├── recorder_status_gui.py
│   ├── keylogger.py
│   ├── ini_editor.py
│   └── config.ini
├── ffmpeg\                  # FFmpeg distribution (downloaded by setup)
├── Python\                  # Portable Python environment (downloaded by setup)
├── Recordings\              # Output video files (created by setup)
├── Logs\                    # Application logs (created by setup)
├── Startup\                 # Windows startup shortcuts
│   └── Recorder\
└── User Guide\              # Documentation
    ├── User_Manual_Keylogger_Recorder.md
    └── Screens\             # Application screenshots
```

## ⚙️ Configuration

Edit `Scripts\config.ini` to customize:
- Recording paths and formats
- Idle detection thresholds
- Logging preferences
- Tray icon settings

## 📚 Documentation

- **[User Manual](User%20Guide/User_Manual_Keylogger_Recorder.md)** - Complete setup and usage guide
- **[Build Instructions](User%20Guide/User_Manual_Keylogger_Recorder.md#-build-instructions)** - How to set up the portable environment

## 🔧 Dependencies

- Python 3.12+
- FFmpeg
- See `requirements.txt` for Python packages

## 🚀 Quick Setup

### **Option 1: One-Liner Install (Easiest - No Cloning!)**
```powershell
# Run this single command in PowerShell:
irm https://raw.githubusercontent.com/GoblinRules/silentwitness/main/install_silentwitness.ps1 | iex
```

**What happens:**
- Downloads setup script directly from GitHub
- Executes it automatically
- Sets up everything in `C:\Tools\SilentWitness\`
- No need to clone or download anything manually!

### **Option 2: Traditional Setup (Clone First)**
1. **Download the repository:**
   ```bash
   git clone https://github.com/yourusername/silentwitness.git
   cd silentwitness
   ```

2. **Run the setup script:**
   ```cmd
   # Double-click setup_silentwitness.bat
   # OR run from PowerShell:
   .\setup_silentwitness.ps1
   ```

3. **The script will automatically:**
   - Download FFmpeg from official sources
   - Download Python 3.12 portable
   - Install all required Python packages
   - Configure paths and environment
   - Create necessary directories

### **Option 2: Manual Setup**
1. **Install Python 3.12+**
2. **Install FFmpeg** from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/)
3. **Install Python packages:**
   ```bash
   pip install -r requirements.txt
   ```
4. **Copy files to `C:\Tools\SilentWitness\`**
5. **Update paths in `Scripts\config.ini`**

## 🎯 Features

- **Automated Screen Recording** - Start/stop based on user activity
- **Keystroke Logging** - Natural text capture with timestamps
- **GUI Management** - Visual control and monitoring interface
- **System Tray Integration** - Background operation with status indicators
- **Configurable Settings** - Easy customization via INI files

## 📄 License

This project is licensed under a **Custom Non-Commercial License**. 

- ✅ **Personal & Educational Use**: Allowed freely
- ❌ **Commercial Use**: Requires permission and licensing
- 📧 **Contact**: [your-email@domain.com] for commercial inquiries

See [LICENSE.md](LICENSE.md) for full terms and conditions.

## ⚠️ Note

This tool is designed for legitimate monitoring purposes (documentation, debugging, etc.). Ensure compliance with local laws and privacy regulations.
