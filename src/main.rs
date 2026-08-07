use arc::cli;
use owo_colors::OwoColorize;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("{} {e}", "Error:".red().bold());
        std::process::exit(1);
    }
}
