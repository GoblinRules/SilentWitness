# 🚀 SilentWitness GitHub Setup Guide

This guide explains how to set up SilentWitness from the GitHub repository.

## 📋 Prerequisites

- **Windows 10/11** (64-bit)
- **PowerShell 5.1+** (usually pre-installed)
- **Internet connection** for downloading dependencies
- **Administrator privileges** (recommended for best results)

## 🔧 Installation Options

### **Option 1: One-Click Setup (Easiest)**

1. **Clone the repository:**
   ```bash
   git clone https://github.com/GoblinRules/silentwitness.git
   cd silentwitness
   ```

2. **Run the setup script:**
   - **Double-click** `setup_silentwitness.bat`
   - **OR** right-click → "Run as administrator"

3. **Wait for completion:**
   - FFmpeg download (~50-100MB)
   - Python download (~30-50MB)
   - Package installation (~100-200MB)
   - Total time: 5-15 minutes depending on internet speed

### **Option 2: PowerShell Setup**

1. **Open PowerShell as Administrator:**
   - Press `Win + X`
   - Select "Windows PowerShell (Admin)"

2. **Navigate to the repository:**
   ```powershell
   cd C:\path\to\silentwitness
   ```

3. **Run the setup script:**
   ```powershell
   .\setup_silentwitness.ps1
   ```

### **Option 3: Manual Setup (Advanced)**

1. **Install Python 3.12+:**
   - Download from [python.org](https://www.python.org/downloads/)
   - Add to PATH during installation

2. **Install FFmpeg:**
   - Download from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/)
   - Extract to `C:\Tools\SilentWitness\ffmpeg\`

3. **Install Python packages:**
   ```bash
   pip install -r requirements.txt
   ```

4. **Copy files to target directory:**
   ```bash
   xcopy /E /I Scripts C:\Tools\SilentWitness\Scripts
   ```

## 🔍 What the Setup Script Does

### **Directory Structure Created:**
```
C:\Tools\SilentWitness\
├── Scripts\                 # Your Python scripts
├── ffmpeg\                  # FFmpeg binaries
├── Python\                  # Portable Python environment
├── Recordings\              # Video output directory
├── Logs\                    # Application logs
└── Startup\                 # Windows shortcuts
```

### **Downloads & Installations:**
- ✅ **FFmpeg** from gyan.dev (official builds)
- ✅ **Python 3.12** portable from python.org
- ✅ **pip** package manager
- ✅ **All required packages** from requirements.txt
- ✅ **Python path configuration**
- ✅ **Directory structure creation**

## 🚨 Troubleshooting

### **Common Issues:**

#### **"Execution Policy" Error:**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

#### **"Access Denied" Error:**
- Run PowerShell as Administrator
- Check antivirus software
- Ensure you have write permissions to `C:\Tools\`

#### **Download Failures:**
- Check internet connection
- Try running as Administrator
- Check firewall/antivirus settings

#### **Python Installation Issues:**
- Ensure you have 64-bit Windows
- Try manual Python installation first
- Check PATH environment variable

### **Verification Commands:**

```powershell
# Check if Python is working
C:\Tools\SilentWitness\Python\python.exe --version

# Check if FFmpeg is working
C:\Tools\SilentWitness\ffmpeg\bin\ffmpeg.exe -version

# Check installed packages
C:\Tools\SilentWitness\Python\Scripts\pip.exe list
```

## 📚 Next Steps

After successful setup:

1. **Test the installation:**
   ```cmd
   cd C:\Tools\SilentWitness\Scripts
   python ffmpeg_auto_recorder.py
   ```

2. **Read the documentation:**
   - [User Manual](User%20Guide/User_Manual_Keylogger_Recorder.md)
   - [README.md](README.md)

3. **Configure settings:**
   - Edit `Scripts\config.ini`
   - Use `ini_editor.py` for GUI configuration

## 🤝 Contributing

### **For Developers:**
- Fork the repository
- Create a feature branch
- Make your changes
- Test thoroughly
- Submit a pull request

### **Repository Structure:**
```
silentwitness/
├── Scripts/                  # Core Python code (~50KB)
├── User Guide/               # Documentation + Screenshots (~100KB)
│   ├── User_Manual_Keylogger_Recorder.md
│   └── Screens/              # Application screenshots
├── setup_silentwitness.ps1   # Main setup script (~10KB)
├── install_silentwitness.ps1 # One-liner installer (~1KB)
├── setup_silentwitness.bat   # Setup launcher (~1KB)
├── requirements.txt           # Python dependencies (~1KB)
├── .gitignore                # Git exclusions (~1KB)
├── GITHUB_SETUP.md           # Setup guide (~5KB)
└── README.md                 # Project overview (~4KB)
```

## 📞 Support

- **Issues:** Create a GitHub issue
- **Discussions:** Use GitHub Discussions
- **Wiki:** Check the repository wiki
- **Releases:** Check for latest versions

---

**Note:** This setup script downloads official binaries from trusted sources. Always verify the integrity of downloaded files and ensure compliance with local laws and regulations.
