use std::process::{Command, exit};
use std::env;
use std::error::Error;

// Use indicatif for styled output or progress indicators (if needed)
// Add to Cargo.toml dependencies:
// indicatif = "0.17"
// sudo     = "0.4"
use sudo::{escalate_if_needed, RunningAs, check};

/// Runs a command with given arguments, returning its stdout as String if successful.
fn run_command(cmd: &str, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new(cmd)
        .args(args)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Command '{}' failed: {}", cmd, stderr).into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Platform detection: allow macOS and Linux (no output yet)
    let platform = env::consts::OS;
    if !matches!(platform, "macos" | "darwin" | "linux") {
        eprintln!("❌ Unsupported platform: {}. This installer only supports macOS and Linux.", platform);
        exit(1);
    }

    // Elevate privileges if needed
    if check() != RunningAs::Root {
        println!("☝🏻 r3install is running as regular user, escalating using sudo");
        escalate_if_needed()?;
    }

    // Now running as root: print once
    println!("✔️ Detected platform: {}", platform);
    println!("✔️ Sudo privileges acquired.");

    // Example: run a command reserved for root
    println!(" Running 'ls /root' to verify root-only access...");
    let root_listing = run_command("ls", &["/root"])?;
    println!("Contents of /root:\n{}", root_listing);

    // Next steps:
    // 1. Install Xcode CLI tools (macOS) or essential build tools (Linux)
    // 2. Install Homebrew (macOS) or Linuxbrew (Linux) if missing
    // 3. Install brew tools, casks (macOS), and MAS apps (macOS)

    Ok(())
}
