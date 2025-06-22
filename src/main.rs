use std::process::{Command, exit};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;

// Dependencies in Cargo.toml:
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// sudo     = "0.4"
use serde::Deserialize;
use serde_json;
use sudo::{check, escalate_if_needed, RunningAs};

/// Configuration for the installer, loaded from JSON
#[derive(Deserialize, Debug)]
struct Config {
    install_build_tools: bool,
    brew_packages: Vec<String>,
    brew_casks: Vec<String>,
    mas_apps: Vec<String>,
    downloads: Vec<Download>,
}

#[derive(Deserialize, Debug)]
struct Download {
    source: String,
    destination: String,
    clone_repo: bool,
}

/// Runs or simulates a command based on dry_run flag, returning stdout on success.
fn run_command(cmd: &str, args: &[&str], dry_run: bool) -> Result<String, Box<dyn Error>> {
    let cmd_str = format!("{} {}", cmd, args.join(" "));
    if dry_run {
        println!("⚠️ Dry-run: would execute: {}", cmd_str);
        return Ok(String::new());
    }

    let output = Command::new(cmd)
        .args(args)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Command '{}' failed: {}", cmd_str, stderr).into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse -d/--dry-run and config path
    let mut args = env::args().skip(1);
    let mut dry_run = false;
    let mut config_path = "config.json".to_string();
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "-d" | "--dry-run" => {
                dry_run = true;
                if let Some(path) = args.next() {
                    config_path = path;
                }
            }
            other => config_path = other.to_string(),
        }
    }
    if dry_run {
        println!("⚠️ Dry-run mode activated");
    }

    // Load config
    let mut file = File::open(&config_path)
        .map_err(|e| format!("Failed to open config {}: {}", config_path, e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Platform detection
    let platform = env::consts::OS;
    if !matches!(platform, "macos" | "darwin" | "linux") {
        eprintln!("❌ Unsupported platform: {}. Only macOS and Linux supported.", platform);
        exit(1);
    }

    // Elevate privileges if needed
    if check() != RunningAs::Root {
        println!("☝🏻 Escalating privileges via sudo...");
        escalate_if_needed()?;
    }
    println!("✔️ Running as root on {}", platform);

    // Build tools
    if config.install_build_tools {
        if platform == "linux" {
            println!("🛠 Installing build-essential...");
            run_command("apt-get", &["update"], dry_run)?;
            run_command("apt-get", &["install", "-y", "build-essential"], dry_run)?;
        } else {
            println!("🛠 Installing Xcode CLI tools...");
            run_command("xcode-select", &["--install"], dry_run)?;
        }
    }

    // Brew packages
    for pkg in &config.brew_packages {
        println!("🍺 Installing brew package {}...", pkg);
        run_command("brew", &["install", pkg], dry_run)?;
    }

    // Brew casks & MAS apps (macOS)
    if matches!(platform, "macos" | "darwin") {
        for cask in &config.brew_casks {
            println!("🍺 Installing brew cask {}...", cask);
            run_command("brew", &["install", "--cask", cask], dry_run)?;
        }
        for app_id in &config.mas_apps {
            println!("📲 Installing MAS app {}...", app_id);
            run_command("mas", &["install", app_id], dry_run)?;
        }
    }

    // Downloads & clones
    for dl in &config.downloads {
        if dl.clone_repo {
            println!("🔄 Cloning {} into {}", dl.source, dl.destination);
            run_command("git", &["clone", &dl.source, &dl.destination], dry_run)?;
        } else {
            println!("⬇️ Downloading {} to {}", dl.source, dl.destination);
            run_command("curl", &["-L", &dl.source, "-o", &dl.destination], dry_run)?;
        }
    }

    println!("🎉 All tasks completed.");
    Ok(())
}
