use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn geometry_string(&self) -> String {
        format!("{},{} {}x{}", self.x, self.y, self.width, self.height)
    }
}

pub fn capture_region(region: &Region) -> Result<PathBuf> {
    let path = std::env::temp_dir().join(format!("circle-search-{}.png", std::process::id()));
    let status = Command::new("grim")
        .arg("-g")
        .arg(region.geometry_string())
        .arg(&path)
        .status()
        .context("failed to run grim")?;
    if !status.success() {
        anyhow::bail!("grim capture failed");
    }
    Ok(path)
}

/// Spawns grim without waiting so the caller can do other work (GTK init) while
/// the screenshot is being captured.
pub fn spawn_fullscreen_capture() -> Result<(std::process::Child, PathBuf)> {
    let path = std::env::temp_dir().join(format!("circle-search-full-{}.png", std::process::id()));
    let child = Command::new("grim")
        .arg(&path)
        .spawn()
        .context("failed to spawn grim for fullscreen capture")?;
    Ok((child, path))
}
