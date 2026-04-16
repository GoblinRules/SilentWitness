import sys
sys.path.append(r"C:\Tools\SilentWitness\Python\Lib\site-packages")

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
RECORDING_DIR_BASE = config.get('Recording', 'recording_dir', fallback='C:\\Tools\\SilentWitness\\Recordings')
FFPROBE_PATH = config.get('Recording', 'ffprobe_path', fallback='C:\\Tools\\SilentWitness\\ffmpeg\\bin\\ffprobe.exe')

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
        log_to_console(f"[Duration Error] {file_path} → {e}")
    return "N/A"

def get_file_size(file_path):
    try:
        size_bytes = os.path.getsize(file_path)
        return size_bytes / (1024 * 1024)  # Convert to MB
    except Exception:
        return 0

def get_recordings():
    recordings = []
    if not os.path.exists(RECORDING_DIR_BASE):
        return recordings
    
    for root, dirs, files in os.walk(RECORDING_DIR_BASE):
        for file in files:
            if file.endswith('.mkv'):
                file_path = os.path.join(root, file)
                try:
                    # Extract date and time from filename or use file modification time
                    filename = os.path.basename(file)
                    dt = datetime.fromtimestamp(os.path.getmtime(file_path))
                    
                    # Try to extract username from filename
                    username_match = re.search(r'^([^-]+)-', filename)
                    username = username_match.group(1) if username_match else "Unknown"
                    
                    recordings.append({
                        'path': file_path,
                        'filename': filename,
                        'date': dt.strftime('%Y-%m-%d'),
                        'time': dt.strftime('%H:%M:%S'),
                        'duration': get_duration(file_path),
                        'size': get_file_size(file_path),
                        'user': username
                    })
                except Exception as e:
                    log_to_console(f"Error processing {file}: {e}")
    
    # Sort by date/time, newest first
    recordings.sort(key=lambda x: (x['date'], x['time']), reverse=True)
    return recordings

def open_file(file_path):
    try:
        os.startfile(file_path)
        log_to_console(f"Opened: {os.path.basename(file_path)}")
    except Exception as e:
        log_to_console(f"Error opening {file_path}: {e}")

def open_folder(file_path):
    try:
        folder = os.path.dirname(file_path)
        os.startfile(folder)
        log_to_console(f"Opened folder: {folder}")
    except Exception as e:
        log_to_console(f"Error opening folder: {e}")

def delete_file(file_path):
    try:
        # Move to recycle folder instead of permanent deletion
        recycle_dir = os.path.join(os.path.dirname(file_path), RECYCLE_FOLDER)
        os.makedirs(recycle_dir, exist_ok=True)
        
        filename = os.path.basename(file_path)
        recycle_path = os.path.join(recycle_dir, filename)
        
        # Add timestamp if file already exists
        counter = 1
        while os.path.exists(recycle_path):
            name, ext = os.path.splitext(filename)
            recycle_path = os.path.join(recycle_dir, f"{name}_{counter}{ext}")
            counter += 1
        
        shutil.move(file_path, recycle_path)
        log_to_console(f"Moved to recycle: {filename}")
        refresh_recordings()
    except Exception as e:
        log_to_console(f"Error deleting {file_path}: {e}")

def on_recording_click(event):
    item = tree.selection()[0]
    file_path = tree.item(item, "tags")[0]
    col = tree.identify_column(event.x)
    
    if col == "#6":  # Open folder
        open_folder(file_path)
    elif col == "#7":  # Play
        open_file(file_path)
    elif col == "#8":  # Keylog
        keylog_path = file_path.replace(".mkv", ".txt")
        if os.path.exists(keylog_path):
            open_file(keylog_path)
        else:
            log_to_console("No keylog file found")
    elif col == "#9":  # Delete
        if tk.messagebox.askyesno("Confirm Delete", "Move to recycle bin?"):
            delete_file(file_path)

def refresh_recordings():
    global tree
    tree.delete(*tree.get_children())
    
    recordings = get_recordings()
    limit = limit_var.get()
    if limit != "All":
        recordings = recordings[:int(limit)]
    
    search_term = search_var.get().lower()
    user_filter_val = user_filter.get()
    
    users = set()
    for rec in recordings:
        # Apply search filter
        if search_term and search_term not in rec['filename'].lower():
            continue
        
        # Apply user filter
        if user_filter_val != "All" and rec['user'] != user_filter_val:
            continue
        
        users.add(rec['user'])
        
        tree.insert("", "end", values=[
            rec['date'], rec['time'], rec['duration'], 
            f"{rec['size']:.1f}MB", rec["user"],
            "📂", "▶", "📄", "🗑"
        ], tags=(rec["path"],))
    user_filter['values'] = ["All"] + sorted(users)

def build_gui():
    global tree, limit_var, search_var, user_filter, status_elements, log_console

    root = tk.Tk()
    root.title("SilentWitness Recorder Monitor")
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
    user_filter = ttk.Combobox(values=["All"], width=15)
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

def update_status():
    for name, (dot, txt, user_lbl) in status_elements.items():
        if name == "Recorder Encoder Status":
            is_running = is_process_running("ffmpeg", "ffmpeg.exe")
            dot.config(text="🟢" if is_running else "🔴")
            txt.config(text="Running" if is_running else "Stopped")
            user_lbl.config(text=f"User: {get_process_user('ffmpeg')}")
        elif name == "Recorder Tray Indicator Status":
            is_running = is_process_running("ffmpeg_auto_recorder.py")
            dot.config(text="🟢" if is_running else "🔴")
            txt.config(text="Running" if is_running else "Stopped")
            user_lbl.config(text=f"User: {get_process_user('ffmpeg_auto_recorder.py')}")
        elif name == "Overlay Status":
            is_running = is_process_running("KeyOverlay")
            dot.config(text="🟢" if is_running else "🔴")
            txt.config(text="Running" if is_running else "Stopped")
            user_lbl.config(text=f"User: {get_process_user('KeyOverlay')}")
        elif name == "Keylogger Status":
            is_running = is_process_running("keylogger.py")
            dot.config(text="🟢" if is_running else "🔴")
            txt.config(text="Running" if is_running else "Stopped")
            user_lbl.config(text=f"User: {get_process_user('keylogger.py')}")

def auto_refresh_status():
    while True:
        time.sleep(5)
        try:
            root = tk.Tk()
            root.after(0, update_status)
            root.destroy()
        except:
            pass

if __name__ == "__main__":
    build_gui()
