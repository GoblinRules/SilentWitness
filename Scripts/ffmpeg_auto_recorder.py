import sys
sys.path.append(r"C:\Tools\SilentWitness\Python\Lib\site-packages")
import os
import subprocess
import threading
import time
import logging
import configparser
import ctypes
from datetime import datetime
from pystray import Icon, Menu as TrayMenu, MenuItem as TrayMenuItem
from PIL import Image
import getpass
import psutil

# === Load Config ===
CONFIG_FILE = os.path.join(os.path.dirname(__file__), 'config.ini')
config = configparser.ConfigParser()
config.read(CONFIG_FILE)

# === Time Conversion ===
def parse_hms(hms_str):
    try:
        parts = hms_str.strip().split(':')
        if len(parts) != 3:
            raise ValueError("Expected HH:MM:SS")
        h, m, s = map(int, parts)
        return h * 3600 + m * 60 + s
    except Exception as e:
        logging.warning(f"Failed to parse time '{hms_str}': {e}")
        return 0

# === Parse Config ===
FFMPEG_PATH = config.get('Paths', 'ffmpeg_path')
KEY_OVERLAY_EXE = config.get('Paths', 'key_overlay_exe', fallback='')
ENABLE_KEY_OVERLAY = config.getboolean('Recording', 'enable_key_overlay', fallback=False)
ENABLE_KEY_LOGGING = config.getboolean('Recording', 'enable_key_logging', fallback=False)
RECORD_VIDEO = config.getboolean('Recording', 'record_video', fallback=True)
RECORD_DURATION_STR = config.get('Recording', 'record_duration', fallback='01:00:00')
IDLE_THRESHOLD_STR = config.get('Recording', 'idle_threshold', fallback='00:10:00')
CHECK_INTERVAL = config.getint('Recording', 'check_interval', fallback=10)
FILENAME_TEMPLATE = config.get('Recording', 'filename_template', fallback='{username}-{timestamp}.mkv')
RECORDING_DIR_BASE = config.get('Recording', 'recording_dir', fallback='C:\\Tools\\SilentWitness\\Recordings')

RECORD_DURATION = parse_hms(RECORD_DURATION_STR)
IDLE_THRESHOLD_SECONDS = parse_hms(IDLE_THRESHOLD_STR)

ENABLE_TRAY_ICON = config.getboolean('Tray', 'enable_tray_icon', fallback=True)
ICON_IDLE_PATH = config.get('Tray', 'icon_idle', fallback='')
ICON_RECORDING_PATH = config.get('Tray', 'icon_recording', fallback='')
TOOLTIP_IDLE = config.get('Tray', 'tooltip_idle', fallback='Idle')
TOOLTIP_RECORDING = config.get('Tray', 'tooltip_recording', fallback='Recording')

LOG_DIR = config.get('Logging', 'log_dir', fallback='./logs')
ENABLE_LOGGING = config.getboolean('Logging', 'enable_logging', fallback=True)
LOG_TO_FILE = config.getboolean('Logging', 'log_to_file', fallback=True)
LOG_LEVEL = getattr(logging, config.get('Logging', 'log_level', fallback='INFO').upper(), logging.INFO)

# === Setup Logging ===
os.makedirs(LOG_DIR, exist_ok=True)
log_path = os.path.join(LOG_DIR, 'ffmpeg_auto_recorder.log')
if ENABLE_LOGGING and LOG_TO_FILE:
    logging.basicConfig(filename=log_path, level=LOG_LEVEL, format='%(asctime)s %(levelname)s: %(message)s')
else:
    logging.basicConfig(level=LOG_LEVEL, format='%(asctime)s %(levelname)s: %(message)s')

# === Globals ===
record_proc = None
overlay_proc = None
keylogger_proc = None
is_recording = False
tray_icon = None

# === Overlay ===
def is_overlay_running():
    for proc in psutil.process_iter(['name']):
        if proc.info['name'] and 'KeyOverlay.exe' in proc.info['name']:
            return True
    return False

def start_overlay():
    global overlay_proc
    if ENABLE_KEY_OVERLAY and os.path.exists(KEY_OVERLAY_EXE):
        if is_overlay_running():
            logging.info("KeyOverlay.exe already running. Skipping launch.")
            return
        try:
            overlay_proc = subprocess.Popen([KEY_OVERLAY_EXE], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            logging.info(f"KeyOverlay.exe started with PID {overlay_proc.pid}")
        except Exception as e:
            logging.error(f"Failed to start KeyOverlay.exe: {e}")

# === Keylogger ===
def stop_keylogger():
    global keylogger_proc
    if keylogger_proc:
        try:
            keylogger_proc.terminate()
            keylogger_proc.wait(timeout=5)
            logging.info(f"Keylogger stopped (PID {keylogger_proc.pid})")
        except Exception as e:
            logging.warning(f"Failed to stop keylogger: {e}")
        keylogger_proc = None

def start_keylogger(log_path):
    global keylogger_proc
    stop_keylogger()
    if ENABLE_KEY_LOGGING:
        keylogger_script = os.path.join(os.path.dirname(__file__), 'keylogger.py')
        if not os.path.exists(keylogger_script):
            logging.error(f"Keylogger script not found: {keylogger_script}")
            return

        os.makedirs(os.path.dirname(log_path), exist_ok=True)

        try:
            keylogger_proc = subprocess.Popen(
                [sys.executable, keylogger_script, log_path],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                creationflags=subprocess.CREATE_NO_WINDOW
            )
            logging.info(f"Keylogger started (PID {keylogger_proc.pid}) logging to {log_path}")
        except Exception as e:
            logging.error(f"Failed to launch keylogger: {e}")

# === Recording ===
def generate_filename():
    username = getpass.getuser()
    timestamp = datetime.now().strftime('%d%m%Y-%H%M%S')
    return FILENAME_TEMPLATE.format(username=username, timestamp=timestamp)

def get_recording_path():
    pc_name = os.environ.get('COMPUTERNAME', 'PC')
    date_str = datetime.now().strftime('%Y%m%d')
    folder = os.path.join(RECORDING_DIR_BASE, pc_name, date_str)
    os.makedirs(folder, exist_ok=True)
    return os.path.join(folder, generate_filename())

def start_recording():
    global record_proc, is_recording
    filepath = get_recording_path()
    cmd = [FFMPEG_PATH, '-f', 'gdigrab', '-framerate', '30', '-i', 'desktop']
    cmd += ['-c:v', 'libx264', '-preset', 'veryfast', '-crf', '25', '-pix_fmt', 'yuv420p']
    if RECORD_DURATION > 0:
        cmd += ['-t', str(RECORD_DURATION)]
    cmd.append(filepath)

    try:
        startupinfo = subprocess.STARTUPINFO()
        startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW

        keylog_path = filepath.replace(".mkv", ".txt")
        start_keylogger(keylog_path)

        record_proc = subprocess.Popen(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            startupinfo=startupinfo,
            creationflags=subprocess.CREATE_NO_WINDOW
        )
        is_recording = True
        logging.info("Recording started")
        update_tray()
    except Exception as e:
        logging.error(f"Failed to start recording: {e}")

def stop_recording():
    global record_proc, is_recording
    if record_proc:
        record_proc.terminate()
        record_proc.wait()
        record_proc = None
    stop_keylogger()
    is_recording = False
    logging.info("Recording stopped")
    update_tray()

def restart_recording():
    stop_recording()
    time.sleep(1)
    start_recording()

def is_user_idle():
    class LASTINPUTINFO(ctypes.Structure):
        _fields_ = [('cbSize', ctypes.c_uint), ('dwTime', ctypes.c_uint)]
    lii = LASTINPUTINFO()
    lii.cbSize = ctypes.sizeof(lii)
    if ctypes.windll.user32.GetLastInputInfo(ctypes.byref(lii)):
        millis = ctypes.windll.kernel32.GetTickCount() - lii.dwTime
        return (millis / 1000.0) > IDLE_THRESHOLD_SECONDS
    return False

def idle_monitor():
    while True:
        time.sleep(CHECK_INTERVAL)
        if is_user_idle():
            if is_recording:
                stop_recording()
        else:
            if not is_recording:
                start_recording()

def update_tray():
    if tray_icon:
        try:
            icon_path = ICON_RECORDING_PATH if is_recording else ICON_IDLE_PATH
            tray_icon.icon = Image.open(icon_path)
            tray_icon.title = TOOLTIP_RECORDING if is_recording else TOOLTIP_IDLE
        except Exception as e:
            logging.warning(f"Failed to update tray icon: {e}")

def tray_setup():
    global tray_icon
    if not ENABLE_TRAY_ICON:
        return
    menu = TrayMenu(TrayMenuItem('Break & Restart', lambda icon, item: restart_recording()))
    try:
        tray_icon = Icon("FFmpegAutoRecorder", Image.open(ICON_IDLE_PATH), TOOLTIP_IDLE, menu)
        threading.Thread(target=tray_icon.run, daemon=True).start()
    except Exception as e:
        logging.error(f"Tray icon setup failed: {e}")

def main():
    logging.info("FFmpeg Auto Recorder starting up")

    if ENABLE_KEY_OVERLAY:
        start_overlay()

    tray_setup()
    idle_monitor()

if __name__ == '__main__':
    main()
