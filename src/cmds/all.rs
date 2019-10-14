use std::env;
use std::path::PathBuf;
// use std::ffi::OsStr;
use std::process::{Command, Stdio};

use defs::*;


//    _____  .__  .__   
//   /  _  \ |  | |  |  
//  /  /_\  \|  | |  |  
// /    |    \  |_|  |__
// \____|__  /____/____/
//         \/           
pub fn all(arc: &Arc) { // $ALL

    let mut cmd_str: Vec<String> = Vec::new();
    let start_dir = env::current_dir().unwrap();    // to reset directory


    if let Opt::All(data) = &arc.arc_cmd.opt {
        println!("data: {:?}", data);
        cmd_str = data.clone();
    }

    for repo in &arc.repos {
        let mut full_path = PathBuf::from(&arc.path.tld);
        full_path.push(&repo.name);
        let result_change = env::set_current_dir(&full_path);
        match result_change {
            Err(e) => println!("error was this {}", e),
            Ok(_) => (),   
        }

        Command::new(&cmd_str[0])
            .args(&cmd_str[1..])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute");
    }

    env::set_current_dir(start_dir).unwrap();
}