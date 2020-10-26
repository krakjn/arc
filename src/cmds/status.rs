use regex::Regex;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::arc::*;
use crate::defs::*;

//   _________ __          __
//  /   _____//  |______ _/  |_ __ __  ______
//  \_____  \\   __\__  \\   __\  |  \/  ___/
//  /        \|  |  / __ \|  | |  |  /\___ \
// /_______  /|__| (____  /__| |____//____  >
//         \/           \/                \/
pub fn status(arc: &Arc, verbose: bool) {
    // $STATUS
    let start_dir = env::current_dir().unwrap(); // to reset directory
                                                 // regex expressions
    let branch_name = Regex::new(r"(?m)^## (.*?)\..*$").unwrap();
    let untracked_branch = Regex::new(r"(?m)^## (.*?)$").unwrap();
    let ahead = Regex::new(r"(?m)^##.*\[ahead (\d+)\].*$").unwrap();
    let behind = Regex::new(r"(?m)^##.*\[behind (\d+)\].*$").unwrap();
    let diverged = Regex::new(r"(?m)^##.*\[ahead (\d+), behind (\d+)\].*$").unwrap();
    let changed = Regex::new(r"(?m)^[^#]{2} .*$").unwrap();
    // condition of files
    let staged = Regex::new(r"(?m)[A-Z]. .*$").unwrap();
    let unstaged = Regex::new(r"(?m).[A-Z] .*$").unwrap();
    let untracked = Regex::new(r"(?m)^\?\? (.*)$").unwrap();
    // state of specific files
    let added = Regex::new(r"(?m)^A  (.*)$").unwrap();
    let modified = Regex::new(r"(?m)^.M (.*)$").unwrap(); // AM is added and modified
    let modified_staged = Regex::new(r"(?m)^M  (.*)$").unwrap();
    let delete = Regex::new(r"(?m)^ D (.*)$").unwrap();
    let delete_staged = Regex::new(r"(?m)^D  (.*)$").unwrap();

    for repo in &arc.repos {
        env::set_current_dir(&arc.path.tld).expect("arc root directory not found");
        let mut os_path = PathBuf::from(&arc.path.tld);
        os_path.push(&repo.name);
        // println!("{:?}", os_path);
        let chdir_result = env::set_current_dir(&os_path);

        match chdir_result {
            Err(_) => {
                let fmt_str = format!("cannot find directory {}", &repo.name);
                print_fancyln(&fmt_str, Attrs::UNDERLINE, Colors::RED);
                env::set_current_dir(&arc.path.tld).unwrap();
                continue;
            }
            Ok(_) => (),
        }

        let output = Command::new("git")
            .arg("status")
            .arg("-b")
            .arg("--porcelain")
            .output()
            .expect("Couldn't invoke git status");
        let stdout = String::from_utf8_lossy(&output.stdout);

        if branch_name.is_match(&stdout) {
            let caps = branch_name.captures(&stdout).unwrap();
            print!("{:.<50}", &repo.name);
            let curr_branch_name = caps.get(1).unwrap().as_str();
            if curr_branch_name == "master" {
                print!("{:.>25} ", curr_branch_name);
            } else {
                print!("{:.>width$}", ".", width = 25 - curr_branch_name.len());
                let fancy_branch = format_fancy(curr_branch_name, Attrs::NORMAL, Colors::YELLOW);
                print!("{} ", fancy_branch);
            }
        } else if untracked_branch.is_match(&stdout) {
            let caps = untracked_branch.captures(&stdout).unwrap();
            print!("{:.<50}", &repo.name);
            print!("{:.>25} ", caps.get(1).unwrap().as_str());
        } else {
            print!("{:.<50}", &repo.name);
            print!("{:.>25} ", "unhandled state");
            continue;
        }

        if diverged.is_match(&stdout) {
            let caps = diverged.captures(&stdout).unwrap();
            let diverged_str = format!(
                "↑{},↓{} ",
                caps.get(1).unwrap().as_str(),
                caps.get(2).unwrap().as_str()
            );
            print_fancy(&diverged_str, Attrs::BOLD, Colors::BRIGHT_PURPLE);
        } else {
            // print out ahead or behind
            if ahead.is_match(&stdout) {
                let caps = ahead.captures(&stdout).unwrap();
                let ahead_str = format!("↑{} ", caps.get(1).unwrap().as_str());
                print_fancy(&ahead_str, Attrs::BOLD, Colors::CYAN);
            }

            if behind.is_match(&stdout) {
                let caps = behind.captures(&stdout).unwrap();
                let behind_str = format!("↓{} ", caps.get(1).unwrap().as_str());
                print_fancy(&behind_str, Attrs::BOLD, Colors::CYAN);
            }
        }

        if verbose {
            if !branch_name.is_match(&stdout) {
                print_fancy("not pushed", Attrs::UNDERLINE, Colors::BRIGHT_PURPLE);
            }

            if changed.is_match(&stdout) {
                print_fancyln("changed", Attrs::BOLD, Colors::RED);

                // add some verbosity
                if staged.is_match(&stdout) {
                    for caps in delete_staged.captures_iter(&stdout) {
                        print_fancy(
                            format!("{: >15}", "deleted: ").as_str(),
                            Attrs::BOLD,
                            Colors::GREEN,
                        );
                        println!("{}", caps.get(1).unwrap().as_str());
                    }

                    for caps in added.captures_iter(&stdout) {
                        print_fancy(
                            format!("{: >15}", "added: ").as_str(),
                            Attrs::BOLD,
                            Colors::GREEN,
                        );
                        println!("{}", caps.get(1).unwrap().as_str());
                    }

                    for caps in modified_staged.captures_iter(&stdout) {
                        print_fancy(
                            format!("{: >15}", "modified: ").as_str(),
                            Attrs::BOLD,
                            Colors::GREEN,
                        );
                        println!("{}", caps.get(1).unwrap().as_str());
                    }
                }

                if unstaged.is_match(&stdout) {
                    // unstaged
                    for caps in delete.captures_iter(&stdout) {
                        print_fancy(
                            format!("{: >15}", "deleted: ").as_str(),
                            Attrs::BOLD,
                            Colors::RED,
                        );
                        println!("{}", caps.get(1).unwrap().as_str());
                    }

                    for caps in modified.captures_iter(&stdout) {
                        print_fancy(
                            format!("{: >15}", "modified: ").as_str(),
                            Attrs::BOLD,
                            Colors::RED,
                        );
                        println!("{}", caps.get(1).unwrap().as_str());
                    }
                }

                if untracked.is_match(&stdout) {
                    for caps in untracked.captures_iter(&stdout) {
                        print_fancy(
                            format!("{: >15}", "untracked: ").as_str(),
                            Attrs::BOLD,
                            Colors::YELLOW,
                        );
                        println!("{}", caps.get(1).unwrap().as_str());
                    }
                }
            } else {
                print_fancyln("clean", Attrs::BOLD, Colors::GREEN);
            }
        } else {
            // short and sweet version

            if !branch_name.is_match(&stdout) {
                print_fancy("!", Attrs::BOLD, Colors::BRIGHT_PURPLE);
            }

            if changed.is_match(&stdout) {
                let mut partial_clean: bool = true;
                if untracked.is_match(&stdout) {
                    print_fancy("?", Attrs::BOLD, Colors::YELLOW);
                }
                if staged.is_match(&stdout) {
                    print_fancy("✚", Attrs::BOLD, Colors::GREEN);
                    partial_clean = false;
                }
                if unstaged.is_match(&stdout) {
                    print_fancy("✘", Attrs::BOLD, Colors::RED);
                    partial_clean = false;
                }
                if partial_clean {
                    print_fancyln("✔ ", Attrs::BOLD, Colors::GREEN);
                } else {
                    println!(""); // new line
                }
            } else {
                print_fancyln("✔ ", Attrs::BOLD, Colors::GREEN);
            }
        }

        // reset directory to top level
        env::set_current_dir(&arc.path.tld).expect("arc root directory not found");
    }
    // reset directory to what it was before
    env::set_current_dir(&start_dir).unwrap();
}
