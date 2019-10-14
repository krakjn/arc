use regex::Regex;
use std::env;
use std::path::PathBuf;
// use std::ffi::OsStr;
use std::fs;
use std::process::{Command, Stdio};

use defs::*;
use arc::*;

// __________      .__  .__   
// \______   \__ __|  | |  |  
//  |     ___/  |  \  | |  |  
//  |    |   |  |  /  |_|  |__
//  |____|   |____/|____/____/

pub fn pull(arc: &Arc) { // $PULL
    let start_dir = env::current_dir().unwrap();    // to reset directory
    // regex expressions  
    let branch_name = Regex::new(r"(?m)^## (.*?)\..*$").unwrap();
    let untracked_branch = Regex::new(r"(?m)^## (.*?)$").unwrap();
    let up_to_date  = Regex::new(r".*up.to.date.*").unwrap();
    let updating    = Regex::new(r".*Updating.*").unwrap();
    let conflict    = Regex::new(r".*conflict.*").unwrap();

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
            },
            Ok(_) => (),
        }

        let status = Command::new("git")
                            .arg("status")
                            .arg("-bs") // branch short version
                            .output()
                            .expect("couldn't status");
        let branchout = String::from_utf8_lossy(&status.stdout);

        print!("{:.<50}", &repo.name);
        if branch_name.is_match(&branchout) {
            let caps = branch_name.captures(&branchout).unwrap();
            print!("{:.>25} ", caps.get(1).unwrap().as_str());
        } else if untracked_branch.is_match(&branchout) {
            let caps = untracked_branch.captures(&branchout).unwrap();
            print!("{:.>25} ", caps.get(1).unwrap().as_str());
            print_fancyln("untracked", Attrs::BOLD, Colors::PURPLE);
            continue;
        } else {
            println!("{:.>25} ", "not on a branch?");
            continue;
        }

        let output = Command::new("git")
                                .arg("pull")
                                .output()
                                .expect("couldn't pull");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);


        if up_to_date.is_match(&stdout) {
            print_fancyln("up-to-date", Attrs::BOLD, Colors::GREEN);
        } else if updating.is_match(&stdout) {
            print_fancyln("updating", Attrs::INVERSE, Colors::GREEN);
            print!("{}", stdout);
        } else if conflict.is_match(&stdout) {
            print_fancyln("conflict", Attrs::BOLD, Colors::YELLOW);
            print!("{}", stdout);
        } else {
            // unknown condition
            print_fancyln("unable", Attrs::BOLD, Colors::RED);
            print!("{}", stderr);
        }

    }
    // reset directory to what it was before
    env::set_current_dir(&start_dir).unwrap();
    
}
