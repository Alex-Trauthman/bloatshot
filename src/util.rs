use anyhow::{Result, anyhow};
use chrono::{Datelike, Local, Timelike};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolves paths starting with `~/` to absolute paths.
pub fn resolve_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            let mut resolved = PathBuf::from(home);
            resolved.push(&path[2..]);
            return resolved;
        }
    }
    PathBuf::from(path)
}

/// Generates a timestamped save path within the bloatshots directory.
pub fn get_auto_save_path(base_dir: Option<&str>) -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow!("HOME env var not set"))?;
    let base = match base_dir {
        Some(d) => resolve_path(d),
        None => PathBuf::from(home).join("bloatshots"),
    };

    let now = Local::now();
    let date_dir = base.join(format!(
        "{}-{:02}-{:02}",
        now.year(),
        now.month(),
        now.day()
    ));

    std::fs::create_dir_all(&date_dir)
        .map_err(|e| anyhow!("Failed to create directory {}: {}", date_dir.display(), e))?;

    let filename = format!(
        "{:02}-{:02}-{:02}.png",
        now.hour(),
        now.minute(),
        now.second()
    );
    Ok(date_dir.join(filename))
}

/// Sends a system notification with an optional image icon.
pub fn send_notification(title: &str, body: &str, image_path: Option<&Path>) {
    let mut cmd = Command::new("notify-send");
    cmd.arg(title).arg(body).arg("-a").arg("Bloatshot");
    if let Some(path) = image_path {
        cmd.arg("-i").arg(path);
    }
    let _ = cmd.spawn();
}

/// Copies text to the Wayland clipboard using `wl-copy`.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            anyhow!(
                "Failed to execute wl-copy: {}. Is wl-clipboard installed?",
                e
            )
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }

    child.wait()?;
    Ok(())
}

/// Copies image data to the Wayland clipboard using `wl-copy`.
pub fn copy_image_to_clipboard(path: &Path) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .arg("-t")
        .arg("image/png")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            anyhow!(
                "Failed to execute wl-copy: {}. Is wl-clipboard installed?",
                e
            )
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        let mut file = std::fs::File::open(path)?;
        std::io::copy(&mut file, &mut stdin)?;
    }

    child.wait()?;
    Ok(())
}

/// Opens the provided path in the default system editor.
pub fn open_in_editor(path: &Path) -> Result<()> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|e| anyhow!("Failed to open editor: {}", e))?;
    Ok(())
}
