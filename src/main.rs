#[macro_use]
extern crate serde_derive;

// import statements
// implicit "use"
mod arc; // looks for "arc.rs" or "arc/mod.rs"
mod cli;
mod cmds;
mod defs;

#[cfg(target_os = "windows")]
fn enable_ansi_term() {
    use wincolor::Console;
    let result = Console::stdout();
    match result {
        Err(_) => (), // using mingw (git-bash)
        Ok(mut console) => console
            .set_virtual_terminal_processing(true)
            .expect("couldn't set virtual terminal"),
    }
}

/**
    ___
  /     \  _____   |__|  ____
 /  \ /  \ \__  \  |  | /    \
/    Y    \ / __ \_|  ||   |  \
\____|____/(______/|__||___|__/
 */
fn main() {
    #[cfg(target_os = "windows")]
    enable_ansi_term();

    let mut _arc = defs::Arc::new();
    cli::parse(&mut _arc);
}
