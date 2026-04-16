//! FFmpeg discovery and child-process management.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::config::Config;

#[cfg(windows)]
const EXE_SUFFIX: &str = ".exe";
#[cfg(not(windows))]
const EXE_SUFFIX: &str = "";

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf))
}

/// Resolve an ffmpeg/ffprobe binary, preferring:
/// 1. explicit config path
/// 2. `<exe_dir>/ffmpeg/bin/<name>`
/// 3. `<exe_dir>/<name>`
/// 4. PATH lookup via `which`
fn find_bin(explicit: &str, name: &str) -> Option<PathBuf> {
    if !explicit.trim().is_empty() {
        let p = PathBuf::from(explicit);
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(dir) = exe_dir() {
        let candidates = [
            dir.join("ffmpeg").join("bin").join(format!("{name}{EXE_SUFFIX}")),
            dir.join(format!("{name}{EXE_SUFFIX}")),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    which::which(name).ok()
}

pub fn ffmpeg_path(cfg: &Config) -> Option<PathBuf> {
    find_bin(&cfg.paths.ffmpeg, "ffmpeg")
}

pub fn ffprobe_path(cfg: &Config) -> Option<PathBuf> {
    find_bin(&cfg.paths.ffprobe, "ffprobe")
}

/// Build an ffmpeg command for desktop capture on Windows (gdigrab).
#[cfg(windows)]
pub fn build_record_cmd(
    ffmpeg: &Path,
    output: &Path,
    framerate: u32,
    duration_secs: u64,
    crf: u32,
    preset: &str,
) -> Command {
    let mut cmd = Command::new(ffmpeg);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "warning",
        "-f",
        "gdigrab",
        "-framerate",
        &framerate.to_string(),
        "-i",
        "desktop",
        "-c:v",
        "libx264",
        "-preset",
        preset,
        "-crf",
        &crf.to_string(),
        "-pix_fmt",
        "yuv420p",
    ]);
    if duration_secs > 0 {
        cmd.args(["-t", &duration_secs.to_string()]);
    }
    cmd.arg(output);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // CREATE_NO_WINDOW keeps ffmpeg headless under windowed builds.
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000);
    cmd
}

/// On non-Windows, we still want the code to compile (for tests/dev).
#[cfg(not(windows))]
pub fn build_record_cmd(
    ffmpeg: &Path,
    output: &Path,
    framerate: u32,
    duration_secs: u64,
    crf: u32,
    preset: &str,
) -> Command {
    let mut cmd = Command::new(ffmpeg);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "warning",
        "-f",
        "x11grab",
        "-framerate",
        &framerate.to_string(),
        "-i",
        ":0.0",
        "-c:v",
        "libx264",
        "-preset",
        preset,
        "-crf",
        &crf.to_string(),
        "-pix_fmt",
        "yuv420p",
    ]);
    if duration_secs > 0 {
        cmd.args(["-t", &duration_secs.to_string()]);
    }
    cmd.arg(output);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd
}

/// Ask ffmpeg to terminate gracefully by writing `q` to stdin… but since we
/// discard stdin, the simpler portable approach is to kill the process. ffmpeg
/// closes the output file cleanly on SIGTERM / TerminateProcess for most muxers
/// including Matroska.
pub fn stop_recording(child: &mut Child) -> Result<()> {
    child.kill().context("killing ffmpeg")?;
    child.wait().context("waiting for ffmpeg")?;
    Ok(())
}

/// Probe the duration of a media file in seconds using ffprobe.
pub fn probe_duration(ffprobe: &Path, file: &Path) -> Result<f64> {
    let out = Command::new(ffprobe)
        .args([
            "-v", "error", "-show_entries", "format=duration", "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(file)
        .output()
        .context("running ffprobe")?;
    if !out.status.success() {
        bail!("ffprobe exited with {}", out.status);
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let dur: f64 = text.trim().parse().context("parsing ffprobe duration")?;
    Ok(dur)
}
