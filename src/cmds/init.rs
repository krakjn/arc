use std::env;
use std::fs;
use std::process::{Command, Stdio};

use crate::arc::*;
use crate::defs::*;
// .___       .__  __
// |   | ____ |__|/  |_
// |   |/    \|  \   __\
// |   |   |  \  ||  |
// |___|___|  /__||__|
//          \/
pub fn init(arc: &Arc) {
    // $INIT
    // clone down all the repos
    for repo in &arc.repos {
        let (relative_path, short_name) = clone_path(&repo.name);
        // always create path from top level directory
        env::set_current_dir(&arc.path.tld).unwrap();
        if relative_path != "shallow" {
            let result_create = fs::create_dir_all(&relative_path);
            match result_create {
                Err(e) => println!("couldn't create directory {}", e),
                Ok(_) => (),
            }
            let result_change = env::set_current_dir(&relative_path);
            match result_change {
                Err(e) => println!("couldn't find directory? {}", e),
                Ok(_) => (),
            }
        }

        let repo_url_name = format!("{}/{}.git", &arc.remote.url, &repo.name);

        let fmt_str = format!("loading >>> {:_>50}", short_name);
        print_fancyln(&fmt_str, Attrs::BOLD, Colors::CYAN);

        Command::new("git")
            .arg("clone")
            .arg("--progress")
            .arg(repo_url_name)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute git clone");

        env::set_current_dir(&arc.path.tld).unwrap();

        // println!("stdout {}", String::from_utf8_lossy(&output.stdout));
        // println!("stderr {}", String::from_utf8_lossy(&output.stderr));
    }
    print_fancy(">>> ", Attrs::BOLD, Colors::CYAN);
    print_fancyln("arc initialized", Attrs::BOLD, Colors::GREEN);
}

pub fn clone_path(name: &str) -> (String, String) {
    let create_vec: Vec<&str> = name.clone().split(|c| c == '/' || c == '\\').collect();
    let end_of_vec = create_vec.len() - 1;
    // repo_short_name is name of git project
    let short_name = create_vec[end_of_vec];
    // relative_path is relative path given in arc.json
    let mut relative_path = String::new();

    if create_vec.len() == 1 {
        relative_path = String::from("shallow");
        return (relative_path.clone(), short_name.to_string());
    }

    for i in 0..end_of_vec {
        // println!("vec is = {}, i = {}", create_vec[i], i);
        if create_vec.len() - i == 2 {
            //reached end of path (before repo name)
            relative_path.push_str(create_vec[i]);
        } else {
            let add_dir = String::from(create_vec[i].to_owned() + "/");
            relative_path.push_str(&add_dir);
        }
    }
    return (relative_path.clone(), short_name.to_string());
}
