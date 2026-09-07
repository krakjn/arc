use arc::cli;
use owo_colors::OwoColorize;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!(
            "{} {e}",
            arc::utils::paint("Error:", |s| s.red().bold().to_string())
        );
        std::process::exit(1);
    }
}
