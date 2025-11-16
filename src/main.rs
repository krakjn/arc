mod arc;
mod cli;
mod cmds;
mod defs;
mod error;
mod utils;

use error::Result;
use owo_colors::OwoColorize;

#[cfg(target_os = "windows")]
fn enable_ansi_support() -> Result<()> {
    use std::io::Write;
    // Enable ANSI support on Windows 10+
    let _ = std::io::stdout().flush();
    Ok(())
}

fn main() -> Result<()> {
    #[cfg(target_os = "windows")]
    enable_ansi_support()?;

    let mut arc_instance = defs::Arc::new();
    
    if let Err(e) = cli::parse(&mut arc_instance) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
    
    Ok(())
}
