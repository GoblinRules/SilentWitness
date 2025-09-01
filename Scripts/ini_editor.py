import sys
sys.path.append(r"C:\Tools\OBS\Python\Lib\site-packages")
import tkinter as tk
from tkinter import filedialog, ttk, messagebox
import configparser
import os

INI_FILE = "C:/Tools/OBS/Scripts/config.ini"

DEFAULTS = {
    "Paths": {
        "ffmpeg_path": "",
        "key_overlay_exe": ""
    },
    "Logging": {
        "enable_logging": "true",
        "log_dir": "",
        "log_to_file": "true",
        "log_level": "INFO"
    },
    "Recording": {
        "recording_dir": "",
        "filename_template": "{username}-{timestamp}.mkv",
        "record_format": "mkv",
        "record_video": "true",
        "record_duration": "01:00:00",
        "enable_key_overlay": "false",
        "enable_key_logging": "false",
        "idle_threshold": "00:10:00",
        "check_interval": "10"
    },
    "Tray": {
        "enable_tray_icon": "true",
        "icon_idle": "",
        "icon_recording": "",
        "tooltip_idle": "Not Recording",
        "tooltip_recording": "Recording..."
    }
}

TOOLTIPS = {
    "ffmpeg_path": "Path to ffmpeg executable",
    "key_overlay_exe": "Path to KeyOverlay.exe (optional)",
    "enable_logging": "Enable or disable logging",
    "log_dir": "Folder where logs will be stored",
    "log_to_file": "Whether to write logs to a file",
    "log_level": "Logging level (DEBUG, INFO, WARNING, etc)",
    "recording_dir": "Where recordings will be saved (will be organized by PC and date)",
    "filename_template": "Template using placeholders like {username}, {timestamp}",
    "record_format": "Choose between mkv or mp4",
    "record_video": "Enable or disable video recording",
    "record_duration": "Maximum duration (HH:MM:SS, 00:00:00 for no limit)",
    "enable_key_overlay": "Enable launching KeyOverlay.exe",
    "enable_key_logging": "Enable key input logging to a .txt file alongside recording",
    "idle_threshold": "Idle timeout (HH:MM:SS) before pausing recording",
    "check_interval": "Seconds between idle checks",
    "enable_tray_icon": "Enable tray icon indicator",
    "icon_idle": "Icon path when idle (not recording)",
    "icon_recording": "Icon path when recording",
    "tooltip_idle": "Tooltip text when idle",
    "tooltip_recording": "Tooltip text when recording"
}

class INIEditorApp(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("INI Configuration Editor")
        self.geometry("600x600")
        self.config_parser = configparser.ConfigParser(interpolation=None)
        self.entries = {}
        self.tabs = {}
        self.build_ui()
        self.load_ini()

    def build_ui(self):
        notebook = ttk.Notebook(self)
        notebook.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)

        for section in DEFAULTS:
            frame = ttk.Frame(notebook)
            notebook.add(frame, text=section)
            self.tabs[section] = frame
            desc = ttk.Label(frame, text=f"{section} settings", foreground="gray")
            desc.pack(anchor="w", padx=5, pady=3)
            for key in DEFAULTS[section]:
                row = ttk.Frame(frame)
                row.pack(fill="x", padx=5, pady=2)
                ttk.Label(row, text=key, width=25).pack(side="left")

                default_val = DEFAULTS[section][key]
                tooltip = TOOLTIPS.get(key, "")

                if key in ["enable_logging", "log_to_file", "record_video", "enable_key_overlay", "enable_tray_icon", "enable_key_logging"]:
                    combo = ttk.Combobox(row, values=["true", "false"], width=25)
                    combo.set(default_val)
                    combo.pack(side="left")
                    self.entries[(section, key)] = combo
                    self._add_tooltip(combo, tooltip)
                elif key == "record_format":
                    combo = ttk.Combobox(row, values=["mkv", "mp4"], width=25)
                    combo.set(default_val)
                    combo.pack(side="left")
                    self.entries[(section, key)] = combo
                    self._add_tooltip(combo, tooltip)
                elif key == "log_level":
                    combo = ttk.Combobox(row, values=["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"], width=25)
                    combo.set(default_val)
                    combo.pack(side="left")
                    self.entries[(section, key)] = combo
                    self._add_tooltip(combo, tooltip)
                else:
                    entry = ttk.Entry(row, width=40)
                    entry.insert(0, default_val)
                    entry.pack(side="left", fill="x", expand=True)
                    self.entries[(section, key)] = entry
                    self._add_tooltip(entry, tooltip)

                    if "path" in key or "dir" in key or "icon" in key or "exe" in key:
                        btn = ttk.Button(row, text="Browse", width=8, command=lambda e=entry: self.browse_file(e))
                        btn.pack(side="right")

        bottom = ttk.Frame(self)
        bottom.pack(pady=5)
        ttk.Button(bottom, text="Save Changes", command=self.save_ini).pack(side="left", padx=5)
        ttk.Button(bottom, text="Reset to Defaults", command=self.reset_defaults).pack(side="left", padx=5)

    def _add_tooltip(self, widget, text):
        def on_enter(event):
            self.tooltip = tk.Toplevel(widget)
            self.tooltip.overrideredirect(True)
            self.tooltip.geometry(f"+{event.x_root+10}+{event.y_root+5}")
            tk.Label(self.tooltip, text=text, background="#ffffcc", relief="solid", borderwidth=1).pack()
        def on_leave(event):
            if hasattr(self, 'tooltip'):
                self.tooltip.destroy()
        widget.bind("<Enter>", on_enter)
        widget.bind("<Leave>", on_leave)

    def browse_file(self, entry):
        path = filedialog.askopenfilename()
        if path:
            entry.delete(0, tk.END)
            entry.insert(0, path)

    def reset_defaults(self):
        for (section, key), widget in self.entries.items():
            widget_type = widget.__class__.__name__
            default = DEFAULTS[section][key]
            if widget_type == "Combobox":
                widget.set(default)
            else:
                widget.delete(0, tk.END)
                widget.insert(0, default)

    def load_ini(self):
        self.config_parser.read(INI_FILE)
        for (section, key), widget in self.entries.items():
            try:
                val = self.config_parser.get(section, key)
            except:
                val = DEFAULTS[section][key]
            if isinstance(widget, ttk.Combobox):
                widget.set(val)
            else:
                widget.delete(0, tk.END)
                widget.insert(0, val)

    def save_ini(self):
        for section, keys in DEFAULTS.items():
            if not self.config_parser.has_section(section):
                self.config_parser.add_section(section)
            for key in keys:
                val = self.entries[(section, key)].get()
                self.config_parser.set(section, key, val)
        with open(INI_FILE, "w") as f:
            self.config_parser.write(f)
        messagebox.showinfo("INI Saved", "Configuration saved successfully.")

if __name__ == "__main__":
    app = INIEditorApp()
    app.mainloop()
