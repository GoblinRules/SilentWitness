//! User input idle detection.
//!
//! On Windows this uses `GetLastInputInfo` (global, across all input devices).
//! On other platforms returns 0, which effectively disables idle-based
//! start/stop (the recorder runs continuously, matching the user's intent).

use std::time::Duration;

#[cfg(windows)]
pub fn idle_duration() -> Duration {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        if GetLastInputInfo(&mut info).as_bool() {
            let now = GetTickCount();
            let millis = now.wrapping_sub(info.dwTime);
            return Duration::from_millis(millis as u64);
        }
    }
    Duration::ZERO
}

#[cfg(not(windows))]
pub fn idle_duration() -> Duration {
    Duration::ZERO
}
