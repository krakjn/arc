use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::arc::*;
use crate::defs::*;

//    _____  .__  .__
//   /  _  \ |  | |  |
//  /  /_\  \|  | |  |
// /    |    \  |_|  |__
// \____|__  /____/____/
//         \/
pub fn all(arc: &Arc, gcmd: bool, cmd_str: &Vec<String>) {
    // $ALL

    let start_dir = env::current_dir().unwrap(); // to reset directory

    for repo in &arc.repos {
        let mut full_path = PathBuf::from(&arc.path.tld);
        full_path.push(&repo.name);
        let result_change = env::set_current_dir(&full_path);
        match result_change {
            Err(e) => println!("error was this {}", e),
            Ok(_) => (),
        }
        // let repo_str = format!(" {} ", repo.name);
        // let header = format!("{:#^75}", repo_str.as_str());
        // print_fancyln(header.as_str(), Attrs::NORMAL, Colors::YELLOW);

        print_fancy(repo.name.as_str(), Attrs::NORMAL, Colors::CYAN);
        let header = format!(" {:#>width$}", "#", width = 75 - repo.name.len());
        print_fancyln(header.as_str(), Attrs::NORMAL, Colors::YELLOW);

        if gcmd {
            // position 1 == binary name
            // position 2 == "-c" || "-g"  flag

            let output = Command::new("git")
                .args(&cmd_str[2..])
                // .stdout(Stdio::inherit())
                // .stderr(Stdio::inherit())
                .output()
                .expect("failed to execute");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            print!("{}", stdout);
            print!("{}", stderr);
        } else {
            let output = Command::new(&cmd_str[2])
                .args(&cmd_str[3..])
                // .stdout(Stdio::inherit())
                // .stderr(Stdio::inherit())
                .output()
                .expect("failed to execute");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            print!("{}", stdout);
            print!("{}", stderr);
        }
    }
    env::set_current_dir(start_dir).unwrap();
}
