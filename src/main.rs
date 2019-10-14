// CRATES
// crates belong at the root of the project i.e. (main.rs)
extern crate regex;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate ftp;

// MODULES (local namespace)
mod arc;
mod defs;
mod cmds;

use std::env;
use defs::*;

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate winapi_util;
#[cfg(windows)]
extern crate wincolor;


#[cfg(target_os = "windows")]
fn enable_ansi_term() {
    use wincolor::Console;
    let result = Console::stdout();
    match result {
        Err(_) => (), // using mingw (git-bash)
        Ok(mut console) => console.set_virtual_terminal_processing(true)
                        .expect("couldn't set virtual terminal"),
    }
}




fn run_cmd(mut arc: &Arc) {
    use cmds::*;

    match arc.arc_cmd.cmd {
        // don't need reinit (sync does this already)
        Cmd::Init =>      init(&mut arc),
        Cmd::Pull =>      pull(&mut arc),
        Cmd::Checkout =>  checkout(&mut arc),
        Cmd::Reset =>     reset(&mut arc),
        Cmd::Status =>    status(&mut arc),
        Cmd::Sync =>      sync(&mut arc),
        Cmd::All =>       all(&mut arc),
        Cmd::Upgrade =>   arc::upgrade(&mut arc),
        _ => (), // all other commands are handled before
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

    let argv: Vec<String> = env::args().collect();
    let cmd = arc::parse(&argv);
    let mut arc: Arc = Arc::new(cmd);
    arc::load_arc(&mut arc);
    run_cmd(&mut arc);
}
