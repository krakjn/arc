use regex::Regex;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use defs::*;
use arc::*;

// _________ .__                   __                 __   
// \_   ___ \|  |__   ____   ____ |  | ______  __ ___/  |_ 
// /    \  \/|  |  \_/ __ \_/ ___\|  |/ /  _ \|  |  \   __\
// \     \___|   Y  \  ___/\  \___|    <  <_> )  |  /|  |  
//  \______  /___|  /\___  >\___  >__|_ \____/|____/ |__|  
//         \/     \/     \/     \/     \/                  
pub fn checkout(arc: &Arc) { // $CHECKOUT

    if !find_git() {
        println!("not within git repo");
        std::process::exit(1);
    }

    let mut checkout_cmd: Vec<&str> = Vec::new();
    match &arc.arc_cmd.opt {
        Opt::NewBranch(nb) => {
            checkout_cmd = vec!["checkout", "-b", nb];
        },
        Opt::Branch(b) => {
            checkout_cmd = vec!["checkout", b];
        },
        Opt::NoOp => {
            println!("invalid");
            std::process::exit(1);
        },
        _ => () // nothing, no other option should be allowed in here
    }

    let start_dir = env::current_dir().unwrap();    // to reset directory
    let path_str = start_dir.to_string_lossy();
    let this_proj: Vec<&str> = path_str.split(|c| c == '/' || c == '\\').collect();
    let cwd: &str = this_proj[this_proj.len() - 1];


    for repo in &arc.repos {
        // println!("repo name == {}", &repo.name);

        let proj_name: Vec<&str> = repo.name.split(|c| c == '/' || c == '\\').collect();
        let short_name = proj_name[proj_name.len() - 1];

        if short_name == cwd {
            // println!("yay! cwd ==> {}", &cwd);

            for dep in &repo.deps {

                let mut this_path = PathBuf::from(&arc.path.tld);
                this_path.push(&dep);
                let result = env::set_current_dir(&this_path);

                match result {
                    Err(_) => {
                        let fmt_str = format!("cannot find directory {}", dep);
                        print_fancyln(&fmt_str, Attrs::UNDERLINE, Colors::RED);
                        env::set_current_dir(&arc.path.tld).unwrap();
                        continue;
                    },
                    Ok(_) => (),
                }
                checkout_parse(&arc, &checkout_cmd, &dep, true);

            } // end of dep for loop

            break; // no need to keep going (only operates on 'one main' repo)

        }
    }
    // do I need to check if name was set?
    // now on current directory
    env::set_current_dir(&start_dir).expect("couldn't set to current directory");
    checkout_parse(&arc, &checkout_cmd, &cwd, false);

}


fn checkout_parse(arc: &Arc, checkout_cmd: &Vec<&str>, name: &str, is_dep: bool) {
    // check out parse

    let branch_name = Regex::new(r"'(.*?)'").unwrap();
    let switched = Regex::new(r"Switch.*'(.*?)'").unwrap();
    let fetched = Regex::new(r"remote branch '(.*?)'").unwrap();
    let already = Regex::new(r".*[A|a]lready.*").unwrap();


    let output = Command::new("git")
                        .args(checkout_cmd)
                        .output()
                        .expect("Couldn't check out");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if branch_name.is_match(&stdout) {
        let caps = branch_name.captures(&stdout).expect("couldn't capture branch name");

        if is_dep {
            print_fancy(" dep ", Attrs::BOLD, Colors::PURPLE);
        } else {
            print_fancy(" >>> ", Attrs::BOLD, Colors::CYAN);
        }
        print!("{:.<50}", name);

        if fetched.is_match(&stdout) {
            print_fancy("fetched-> ", Attrs::BOLD, Colors::GREEN);
            println!("{}", caps.get(1).unwrap().as_str());   // print fetched branch

        } else if switched.is_match(&stdout) {

            let caps = switched.captures(&stdout).expect("couldn't capture switch");
            print_fancy("switched-> ", Attrs::BOLD, Colors::CYAN);
            println!("{}", caps.get(1).unwrap().as_str());   // print branch name
        
        } else {

            match &arc.arc_cmd.opt {
                Opt::NewBranch(b) => {
                    print_fancy("created-> ", Attrs::UNDERLINE, Colors::CYAN);
                    println!("{}", b);
                },
                Opt::Branch(b) => {
                    print_fancy("on-> ", Attrs::BOLD, Colors::YELLOW);
                    println!("{}", b);
                    // print!("stdout => {}", &stdout);
                },
                _ => (), // should never get here
            }
        }

    } else {    // stderr
        // when creating new branch git dumps to stderr
        // if branch is not tracked seems to be on stderr
        if is_dep {
            print_fancy(" dep ", Attrs::BOLD, Colors::PURPLE);
        } else {
            print_fancy(" >>> ", Attrs::BOLD, Colors::CYAN);
        }
        print!("{:.<50}", name);

        if switched.is_match(&stderr) {

            let caps = switched.captures(&stderr).expect("couldn't capture switched");
            print_fancy("switched-> ", Attrs::BOLD, Colors::CYAN);
            println!("{}", caps.get(1).unwrap().as_str());   // print branch name

        } else if already.is_match(&stderr) {
            print_fancy(&stderr, Attrs::NORMAL, Colors::YELLOW);
        } else {
            // True error
            print_fancyln("unable", Attrs::BOLD, Colors::RED);
            // println!("{}", &checkout_cmd[checkout_cmd.len() - 1]);   // print incorrect branch name
            print_fancy(&stderr, Attrs::NORMAL, Colors::RED);
        }
    }
}
