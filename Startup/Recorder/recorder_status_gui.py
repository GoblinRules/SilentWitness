import sys
sys.path.append(r"C:\Tools\OBS\Python\Lib\site-packages")

import os
import subprocess
import configparser
from datetime import datetime
import psutil
import tkinter as tk
from tkinter import ttk
import re
import threading
import socket
import time
import shutil
from pathlib import Path

CONFIG_FILE = os.path.join(os.path.dirname(__file__), 'config.ini')
config = configparser.ConfigParser()
config.read(CONFIG_FILE)

FILENAME_TEMPLATE = config.get('Recording', 'filename_template', fallback='{username}-{timestamp}.mkv')
RECORDING_DIR_BASE = config.get('Recording', 'recording_dir', fallback='C:\\Tools\\OBS\\Recordings')
FFPROBE_PATH = config.get('Recording', 'ffprobe_path', fallback='C:\\Tools\\OBS\\ffmpeg\\bin\\ffprobe.exe')

RECYCLE_FOLDER = "_RecycleBin"

log_console = None

def log_to_console(msg):
    if log_console:
        log_console.configure(state='normal')
        log_console.insert(tk.END, f"{datetime.now().strftime('%H:%M:%S')} - {msg}\n")
        log_console.see(tk.END)
        log_console.configure(state='disabled')

def get_process_user(keyword):
    for proc in psutil.process_iter(['username', 'cmdline']):
        try:
            cmdline = proc.info.get("cmdline")
            if cmdline and any(keyword.lower() in part.lower() for part in cmdline):
                return proc.info["username"]
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    return "—"

def is_process_running(keyword, process_name=None):
    for proc in psutil.process_iter(['name', 'cmdline']):
        try:
            name = proc.info.get('name', '').lower()
            if process_name and name != process_name.lower():
                continue
            cmdline = proc.info.get('cmdline') or []
            if any(keyword.lower() in part.lower() for part in cmdline) or keyword.lower() in name:
                return True
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    return False


def terminate_proc(process_name, cmd_match=None):
    found = False
    for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
        try:
            name = proc.info.get('name', '').lower()
            cmdline = proc.info.get('cmdline', [])
            if not cmdline:
                continue
            cmd_str = ' '.join(cmdline).lower()
            if name == process_name.lower() and (cmd_match is None or cmd_match.lower() in cmd_str):
                log_to_console(f"[KILL] {process_name} PID {proc.pid}")
                proc.terminate()
                time.sleep(1)
                if proc.is_running():
                    proc.kill()
                found = True
        except (psutil.NoSuchProcess, psutil.AccessDenied) as e:
            log_to_console(f"[ERROR] {e}")
    if not found:
        log_to_console(f"❌ No matching {process_name} process found.")

def get_pc_name():
    return socket.gethostname()

def get_duration(file_path):
    try:
        # Skip files smaller than 1MB (likely still being written or corrupt)
        if os.path.getsize(file_path) < 1_000_000:
            return "N/A"

        cmd = [FFPROBE_PATH, "-v", "error", "-show_entries", "format=duration",
               "-of", "default=noprint_wrappers=1:nokey=1", str(file_path)]
        output = subprocess.check_output(cmd, stderr=subprocess.STDOUT).decode().strip()
        seconds = float(output)
        hours, remainder = divmod(int(seconds), 3600)
        minutes, secs = divmod(remainder, 60)
        return f"{hours}:{minutes:02d}:{secs:02d}" if hours else f"{minutes}:{secs:02d}"
    except subprocess.CalledProcessError as e:
        log_to_console(f"[Duration Error] {file_path} → ffprobe returned {e.returncode}")
    except Exception as e:
        log_to_console(f"[Duration Exception] {file_path}: {e}")
    return "N/A"


def list_recordings(limit=None, query="", user_filter="All"):
    recordings = []
    root_dir = Path(RECORDING_DIR_BASE)
    if not root_dir.exists():
        return []
    for vid in sorted(root_dir.rglob("*.mkv"), reverse=True):
        if query.lower() not in str(vid.name).lower():
            continue
        stat = vid.stat()
        user = re.match(r"(.+)-\d{8}-\d{6}\.mkv", vid.name)
        user = user.group(1) if user else "unknown"
        if user_filter != "All" and user_filter != user:
            continue
        recordings.append({
            "datetime": datetime.fromtimestamp(stat.st_mtime),
            "path": str(vid),
            "user": user,
            "size": stat.st_size / 1024 / 1024,
            "duration": get_duration(vid)
        })
    if limit and limit.isdigit():
        recordings = recordings[:int(limit)]
    return recordings

def update_status():
    processes = [
        ("Recorder Encoder Status", "ffmpeg", "ffmpeg.exe"),
        ("Recorder Tray Indicator Status", "ffmpeg_auto_recorder.py", None),
        ("Overlay Status", "KeyOverlay", None),
        ("Keylogger Status", "keylogger.py", None)
    ]

    for name, keyword, exe in processes:
        dot, txt, user_lbl = status_elements[name]
        active = is_process_running(keyword, exe)
        dot.config(text="●", foreground="green" if active else "red")
        txt.config(text="Active" if active else "Not Active")
        user_lbl.config(text=f"User: {get_process_user(keyword)}")

def auto_refresh_status():
    while True:
        update_status()
        time.sleep(10)

def on_recording_click(event):
    item = tree.identify_row(event.y)
    col = tree.identify_column(event.x)
    if not item or not col:
        return
    path = tree.item(item, "tags")[0]
    if col == "#6":
        folder = os.path.dirname(path)
        if os.path.exists(folder):
            subprocess.Popen(f'explorer "{folder}"')
    elif col == "#7":
        if os.path.exists(path):
            os.startfile(path)
    elif col == "#8":
        keylog = path.replace(".mkv", ".txt")
        if os.path.exists(keylog):
            os.startfile(keylog)
    elif col == "#9":
        recycle_target = Path(RECORDING_DIR_BASE) / get_pc_name() / RECYCLE_FOLDER
        recycle_target.mkdir(exist_ok=True)
        try:
            shutil.move(path, recycle_target / Path(path).name)
            txt_path = path.replace(".mkv", ".txt")
            if os.path.exists(txt_path):
                shutil.move(txt_path, recycle_target / Path(txt_path).name)
            refresh_recordings()
            log_to_console(f"🗑 Moved {os.path.basename(path)} to recycle")
        except Exception as e:
            log_to_console(f"[Move Error] {e}")

def refresh_recordings():
    tree.delete(*tree.get_children())
    limit = limit_var.get()
    query = search_var.get()
    selected_user = user_filter.get()
    users = set()
    for rec in list_recordings(limit, query, selected_user):
        dt = rec["datetime"]
        users.add(rec['user'])
        tree.insert("", "end", values=[
            dt.strftime("%Y-%m-%d"), dt.strftime("%H:%M:%S"),
            f"{rec['duration']}", f"{rec['size']:.1f}MB", rec["user"],
            "📂", "▶", "📄", "🗑"
        ], tags=(rec["path"],))
    user_filter['values'] = ["All"] + sorted(users)

def build_gui():
    global tree, limit_var, search_var, user_filter, status_elements, log_console

    root = tk.Tk()
    root.title("OBS Recorder Monitor")
    root.geometry("1100x650")

    notebook = ttk.Notebook(root)
    notebook.pack(fill="both", expand=True)

    status_tab = ttk.Frame(notebook)
    notebook.add(status_tab, text="Status")

    status_elements = {}

    def make_status_row(tab, row, name, keyword, process_name, stop_action):
        ttk.Label(tab, text=f"{name}:").grid(row=row, column=0, sticky="w", padx=10, pady=5)
        dot = ttk.Label(tab, text="●")
        dot.grid(row=row, column=1)
        txt = ttk.Label(tab, text="")
        txt.grid(row=row, column=2, sticky="w")
        btn = ttk.Button(tab, text="❌ Stop", command=stop_action)
        btn.grid(row=row, column=3, padx=10)
        user_lbl = ttk.Label(tab, text="User: —")
        user_lbl.grid(row=row, column=4, padx=10)
        status_elements[name] = (dot, txt, user_lbl)

    make_status_row(status_tab, 0, "Recorder Encoder Status", "ffmpeg", "ffmpeg.exe",
                    lambda: terminate_proc("ffmpeg.exe"))
    make_status_row(status_tab, 1, "Recorder Tray Indicator Status", "ffmpeg_auto_recorder.py", None,
                    lambda: (terminate_proc("python.exe", "ffmpeg_auto_recorder.py"),
                             terminate_proc("pythonw.exe", "ffmpeg_auto_recorder.py")))
    make_status_row(status_tab, 2, "Overlay Status", "KeyOverlay", None,
                    lambda: (terminate_proc("KeyOverlay.exe"), terminate_proc("Carnac.exe")))
    make_status_row(status_tab, 3, "Keylogger Status", "keylogger.py", None,
                    lambda: terminate_proc("pythonw.exe", "keylogger.py"))

    log_console = tk.Text(
    status_tab,
    height=10,
    bg="#1e1e1e",        # Medium dark background
    fg="#CCCCCC",        # White text
    insertbackground="#000000",  # Black cursor
    font=("Consolas", 10),
    relief="solid",      # Border style
    bd=1,                # Border width (in pixels)
    state="disabled"
)

    log_console.grid(row=4, column=0, columnspan=5, sticky="nsew", padx=10, pady=10)
    status_tab.grid_rowconfigure(4, weight=1)
    status_tab.grid_columnconfigure(4, weight=1)

    recordings_tab = ttk.Frame(notebook)
    notebook.add(recordings_tab, text="Recordings")

    top_frame = ttk.Frame(recordings_tab)
    top_frame.pack(fill="x", pady=5)

    ttk.Label(top_frame, text="Search:").pack(side="left", padx=5)
    search_var = tk.StringVar()
    search_box = ttk.Entry(top_frame, textvariable=search_var)
    search_box.pack(side="left", padx=5)
    search_box.bind("<Return>", lambda e: refresh_recordings())

    ttk.Label(top_frame, text="Show Last:").pack(side="left", padx=5)
    limit_var = tk.StringVar(value="50")
    limit_menu = ttk.Combobox(top_frame, textvariable=limit_var, values=["30", "50", "100", "All"], width=5)
    limit_menu.pack(side="left", padx=5)

    ttk.Label(top_frame, text="Filter by User:").pack(side="left", padx=5)
    user_filter = ttk.Combobox(top_frame, values=["All"], width=15)
    user_filter.set("All")
    user_filter.pack(side="left", padx=5)
    user_filter.bind("<<ComboboxSelected>>", lambda e: refresh_recordings())

    ttk.Button(top_frame, text="🔄 Refresh", command=refresh_recordings).pack(side="left", padx=5)

    columns = ("Date", "Time", "Duration", "Size", "User", "Open", "Play", "Keylog", "Delete")
    tree = ttk.Treeview(recordings_tab, columns=columns, show="headings")
    for col in columns:
        tree.heading(col, text=col)
        tree.column(col, width=100, anchor="center")
    tree.pack(fill="both", expand=True)
    tree.bind("<ButtonRelease-1>", on_recording_click)

    update_status()
    refresh_recordings()
    threading.Thread(target=auto_refresh_status, daemon=True).start()
    root.mainloop()

if __name__ == "__main__":
    build_gui()
