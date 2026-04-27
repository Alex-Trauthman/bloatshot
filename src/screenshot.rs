use anyhow::{Result, anyhow};
use std::path::Path;
use std::process::Command;

/// Captures a selected area using `slurp` and `grim`.
pub fn capture_screenshot(output_path: &Path) -> Result<()> {
    let slurp_output = Command::new("slurp")
        .output()
        .map_err(|e| anyhow!("Failed to execute slurp: {}. Is it installed?", e))?;

    if !slurp_output.status.success() {
        return Err(anyhow!("Selection cancelled or failed."));
    }

    let region = String::from_utf8_lossy(&slurp_output.stdout)
        .trim()
        .to_string();

    let grim_status = Command::new("grim")
        .arg("-g")
        .arg(region)
        .arg(output_path)
        .status()
        .map_err(|e| anyhow!("Failed to execute grim: {}. Is it installed?", e))?;

    if !grim_status.success() {
        return Err(anyhow!("Failed to capture screenshot with grim."));
    }

    Ok(())
}
