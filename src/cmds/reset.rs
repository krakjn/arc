use regex::Regex;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::arc::*;
use crate::defs::*;

// __________                      __
// \______   \ ____   ______ _____/  |_
//  |       _// __ \ /  ___// __ \   __\
//  |    |   \  ___/ \___ \\  ___/|  |
//  |____|_  /\___  >____  >\___  >__|
//         \/     \/     \/     \/
pub fn reset(arc: &Arc, sanitize: bool) {
    // $SYNC
    let start_dir = env::current_dir().unwrap(); // to reset directory
    let reset_regex = Regex::new(r"at [[:alnum:]]+ (.*)").unwrap();

    print_fancyln("!!!!! WARNING !!!!!", Attrs::INVERSE, Colors::YELLOW);
    println!("This will reset all project folders to remote's HEAD on master branch");
    print!("ALL edits will be ");
    print_fancyln("DISCARDED", Attrs::INVERSE, Colors::RED);
    println!("(yes|NO): ");

    let mut input = String::new();
    let result = std::io::stdin().read_line(&mut input);
    match result {
        Err(e) => {
            println!("invalid character? {}", e);
            std::process::exit(1);
        }
        Ok(_) => (), // do nothing
    }

    let mut do_it = false;
    if &input.to_ascii_lowercase().trim_end()[..] == "y"
        || &input.to_ascii_lowercase().trim_end()[..] == "yes"
    {
        do_it = true;
    }

    if do_it {
        for repo in &arc.repos {
            env::set_current_dir(&arc.path.tld).expect("arc root directory not found");
            let mut path = PathBuf::from(&arc.path.tld);
            path.push(&repo.name);

            let chdir_result = env::set_current_dir(&path);
            match chdir_result {
                Err(_) => {
                    let fmt_str = format!("cannot find directory {}", &repo.name);
                    print_fancyln(&fmt_str, Attrs::UNDERLINE, Colors::RED);
                    env::set_current_dir(&arc.path.tld).unwrap();
                    continue;
                }
                Ok(_) => (),
            }

            print!("{:.<50}", &repo.name);

            if sanitize {
                Command::new("git")
                    .arg("rm")
                    .arg("--cached")
                    .arg("-r")
                    .arg(".")
                    .output()
                    .expect("couldn't sanitize");

                print_fancy("sanitized ", Attrs::BOLD, Colors::CYAN);
                print_fancy("now at-> ", Attrs::NORMAL, Colors::YELLOW);

                let output = Command::new("git")
                    .arg("reset")
                    .arg("--hard")
                    .arg("origin/master")
                    .output()
                    .expect("couldn't reset");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let caps = reset_regex
                    .captures(&stdout)
                    .expect("couldn't capture regex");
                println!("{}", caps.get(1).unwrap().as_str());
            } else {
                // regular reset
                let output = Command::new("git")
                    .arg("reset")
                    .arg("--hard")
                    .arg("origin/master")
                    .output()
                    .expect("couldn't reset");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let caps = reset_regex
                    .captures(&stdout)
                    .expect("couldn't capture regex");

                print_fancy("now at-> ", Attrs::NORMAL, Colors::YELLOW);
                println!("{}", caps.get(1).unwrap().as_str());
            }
        }
        env::set_current_dir(start_dir).unwrap();
    } else {
        println!("Aborting");
    }
}
