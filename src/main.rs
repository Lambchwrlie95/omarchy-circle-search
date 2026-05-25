use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod capture;
mod config;
mod overlay;
mod search;
mod theme;

#[derive(Parser)]
#[command(
    name = "omarchy-circle-search",
    version,
    about = "Select a screen region for visual search"
)]
struct Cli {
    #[arg(
        long,
        help = "Copy screenshot region to clipboard instead of searching"
    )]
    copy: bool,

    #[arg(long, short = 'n', help = "Send desktop notification on completion")]
    notify: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        let _ = notify(&format!("Error: {e}"));
        return Err(e);
    }
    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    check_base_deps()?;
    if cli.copy {
        check_copy_deps()?;
    }

    let theme = theme::load();
    let cfg = config::load();
    let (region, engine) = overlay::show_selector(&theme, &cfg)?;

    if !cli.copy {
        check_search_deps()?;
        if engine != overlay::Engine::AiChat {
            check_visual_search_deps()?;
        }
    }

    let screenshot_path = capture::capture_region(&region)?;

    if cli.copy {
        copy_image_to_clipboard(&screenshot_path)?;
        if cli.notify {
            notify("Screenshot copied to clipboard")?;
        }
        let _ = std::fs::remove_file(&screenshot_path);
    } else {
        search::open_search(&screenshot_path, engine, &cfg)?;
        if engine != overlay::Engine::AiChat {
            notify("Searching…")?;
        }
    }

    Ok(())
}

fn check_base_deps() -> Result<()> {
    check_command("grim", "screenshot capture")
}

fn check_copy_deps() -> Result<()> {
    check_command("wl-copy", "clipboard")
}

fn check_search_deps() -> Result<()> {
    check_command("python3", "search helpers")?;
    check_command("xdg-open", "opening search results")
}

fn check_visual_search_deps() -> Result<()> {
    check_command("curl", "image upload")
}

fn check_command(cmd: &str, purpose: &str) -> Result<()> {
    if !Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        anyhow::bail!("{cmd} not found (needed for {purpose})");
    }
    Ok(())
}

fn copy_image_to_clipboard(path: &PathBuf) -> Result<()> {
    let file = std::fs::File::open(path).context("failed to open image for clipboard")?;
    Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(Stdio::from(file))
        .status()
        .context("failed to run wl-copy")?;
    Ok(())
}

fn notify(msg: &str) -> Result<()> {
    Command::new("omarchy-notification-send")
        .args(["", "Circle Search", msg])
        .status()
        .ok();
    Ok(())
}
