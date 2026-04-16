//! Windows startup integration (Start Menu shortcut + HKCU\Run registration).
//!
//! Uses a tiny WScript.Shell helper via PowerShell rather than pulling in a
//! COM crate; this keeps the dependency surface small and mirrors the old
//! behaviour of `startup_manager.py`.
//!
//! On non-Windows these functions are no-ops.

use std::path::Path;

use anyhow::Result;

#[cfg(windows)]
pub fn install_startup_shortcut(exe: &Path, args: &[&str], name: &str, autostart: bool) -> Result<()> {
    use std::io::Write;
    use std::process::Command;

    let start_menu = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot locate AppData"))?
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("SilentWitness");
    std::fs::create_dir_all(&start_menu)?;
    let lnk = start_menu.join(format!("{name}.lnk"));

    // Build a PowerShell script to create the .lnk
    let ps = format!(
        r#"$s = New-Object -ComObject WScript.Shell
$sc = $s.CreateShortcut('{lnk}')
$sc.TargetPath = '{exe}'
$sc.Arguments = '{args}'
$sc.WorkingDirectory = '{cwd}'
$sc.WindowStyle = 7
$sc.IconLocation = '{exe}'
$sc.Save()"#,
        lnk = lnk.display(),
        exe = exe.display(),
        args = args.join(" "),
        cwd = exe.parent().map(|p| p.display().to_string()).unwrap_or_default()
    );

    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", "-"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(ps.as_bytes())?;
    }
    child.wait()?;

    if autostart {
        // Register HKCU\Software\Microsoft\Windows\CurrentVersion\Run
        let value = format!("\"{}\" {}", exe.display(), args.join(" "));
        Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                name,
                "/t",
                "REG_SZ",
                "/d",
                &value,
                "/f",
            ])
            .status()?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn remove_startup(name: &str) -> Result<()> {
    use std::process::Command;
    Command::new("reg")
        .args([
            "delete",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            name,
            "/f",
        ])
        .status()
        .ok();
    Ok(())
}

#[cfg(not(windows))]
pub fn install_startup_shortcut(_exe: &Path, _args: &[&str], _name: &str, _autostart: bool) -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn remove_startup(_name: &str) -> Result<()> {
    Ok(())
}
