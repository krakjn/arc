 
use regex::Regex;
use std::env;
use std::path::PathBuf;
// use std::ffi::OsStr;
use std::fs;
use std::process::{Command, Stdio};
use arc::*;


// This function only gets compiled if the target OS is linux
// #[cfg(target_os = "linux")]
//-----------------------------------------
// And this function only gets compiled if the target OS is *not* linux
// #[cfg(not(target_os = "linux"))]
// ===== for future reference === 
// if cfg!(target_os = "windows") {
//     println!("{}", format!("\u{1b}[{};{}m{}\u{1b}[0m", attr, color, arg));
// } else {

pub fn print_fancy(arg: &str, attr: Attrs, color: Colors) {
        print!("{}", format!("\u{1b}[{};{}m{}\u{1b}[0m", attr, color, arg));
}

pub fn print_fancyln(arg: &str, attr: Attrs, color: Colors) {
        println!("{}", format!("\u{1b}[{};{}m{}\u{1b}[0m", attr, color, arg));
}

fn find_git() -> bool {
    let mut cwd = env::current_dir().unwrap();
    let start_dir = cwd.clone();    // to reset directory
    let up_dir = "..";

    let root_dir = Regex::new(r"[a-zA-Z]:\\$|/$").unwrap();

    loop {
        cwd = env::current_dir().unwrap();

        if root_dir.is_match(&cwd.to_string_lossy()[..]) {
            env::set_current_dir(start_dir).unwrap();
            return false;
        }

        let list_files = fs::read_dir(&cwd).unwrap();


        let names = list_files.filter_map(|entry| {
                    entry.ok().and_then(|e|
                    e.path().file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
                    )
        }).collect::<Vec<String>>();

        for name in names {
            if name == ".git" {
                env::set_current_dir(start_dir).unwrap();
                return true;
            }
        }
        env::set_current_dir(&up_dir).unwrap();
    }
}

// ##############################################################
// ##   / ____/___  ____ ___  ____ ___  ____ _____  ____/ /____##
// ##  / /   / __ \/ __ `__ \/ __ `__ \/ __ `/ __ \/ __  / ___/##
// ## / /___/ /_/ / / / / / / / / / / / /_/ / / / / /_/ (__  ) ##
// ## \____/\____/_/ /_/ /_/_/ /_/ /_/\__,_/_/ /_/\__,_/____/  ##
// ##############################################################

// .___       .__  __   
// |   | ____ |__|/  |_ 
// |   |/    \|  \   __\
// |   |   |  \  ||  |  
// |___|___|  /__||__|  
//          \/          
pub fn init(arc: &Arc) {  // $INIT
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

fn clone_path(name: &str) -> (String, String) {

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
// end of checkout parse

}

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

// new feature
// need to add .bashrc to `gitbash` for nice printouts
/**
 * ⤧ diverged ahead
 * ⤩ diverged behind
 * ↰ detached
 * ☣ dirty
 * ⚠ ⚡±
 * ◒ ♝ ☣ ✇ ☢ ☯ ☸ ✈
 * ⥢ ⥣ ⥤ ⥥ ⥦ ⥧ ⥨ ⥩ ⥪ ⥫ ⥬ ⥭ ⥮ ⥯ ⥰
 * ← → ↑ ↓ ↔ ↕ ↖ ↗ ↘ ↙ ↜ ↝ ↞ ↟ ↠ ↡ ↢ ↣
 */

//   _________ __          __                
//  /   _____//  |______ _/  |_ __ __  ______
//  \_____  \\   __\__  \\   __\  |  \/  ___/
//  /        \|  |  / __ \|  | |  |  /\___ \ 
// /_______  /|__| (____  /__| |____//____  >
//         \/           \/                \/ 
pub fn status (arc: &Arc) { // $STATUS
    let start_dir = env::current_dir().unwrap();    // to reset directory
    // regex expressions    
    let branch_name     = Regex::new(r"(?m)^## (.*?)\..*$").unwrap();
    let untracked_branch = Regex::new(r"(?m)^## (.*?)$").unwrap();
    let ahead           = Regex::new(r"(?m)^##.*\[ahead (\d+)\].*$").unwrap();
    let behind          = Regex::new(r"(?m)^##.*\[behind (\d+)\].*$").unwrap();
    let diverged        = Regex::new(r"(?m)^##.*\[ahead (\d+), behind (\d+)\].*$").unwrap();
    let changed         = Regex::new(r"(?m)^[^#]{2} .*$").unwrap();
    // condition of files
    let staged          = Regex::new(r"(?m)[A-Z]. .*$").unwrap();
    let unstaged        = Regex::new(r"(?m).[A-Z] .*$").unwrap();
    let untracked       = Regex::new(r"(?m)^\?\? (.*)$").unwrap();
    // state of specific files
    let added           = Regex::new(r"(?m)^A  (.*)$").unwrap();
    let modified        = Regex::new(r"(?m)^.M (.*)$").unwrap(); // AM is added and modified
    let modified_staged = Regex::new(r"(?m)^M  (.*)$").unwrap();
    let delete          = Regex::new(r"(?m)^ D (.*)$").unwrap();
    let delete_staged   = Regex::new(r"(?m)^D  (.*)$").unwrap();

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
            print!("{:.>25} ", caps.get(1).unwrap().as_str());
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
            let diverged_str = format!("↑{},↓{} ", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str());
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


        if arc.arc_cmd.opt == Opt::Verbose(true) {

            if !branch_name.is_match(&stdout) {
                print!("{} ", format!("\u{1b}[{};{}m{}\u{1b}[0m", Attrs::UNDERLINE, Colors::BRIGHT_PURPLE, "not pushed"));
            }

            if changed.is_match(&stdout) {

                print_fancyln("changed", Attrs::BOLD, Colors::RED);
               

                // add some verbosity
                if staged.is_match(&stdout) {

                    for caps in delete_staged.captures_iter(&stdout) {
                        print_fancy(format!("{: >15}", "deleted: ").as_str(), Attrs::BOLD, Colors::GREEN);
                        println!("{}", caps.get(1).unwrap().as_str());
                    }

                    for caps in added.captures_iter(&stdout) {
                        print_fancy(format!("{: >15}", "added: ").as_str(), Attrs::BOLD, Colors::GREEN);
                        println!("{}", caps.get(1).unwrap().as_str());
                    }

                    for caps in modified_staged.captures_iter(&stdout) {
                        print_fancy(format!("{: >15}", "modified: ").as_str(), Attrs::BOLD, Colors::GREEN);
                        println!("{}", caps.get(1).unwrap().as_str());
                    }

                } 
                
                if unstaged.is_match(&stdout) {
                    // unstaged
                    for caps in delete.captures_iter(&stdout) {
                        print_fancy(format!("{: >15}", "deleted: ").as_str(), Attrs::BOLD, Colors::RED);
                        println!("{}", caps.get(1).unwrap().as_str());
                    }

                    for caps in modified.captures_iter(&stdout) {
                        print_fancy(format!("{: >15}", "modified: ").as_str(), Attrs::BOLD, Colors::RED);
                        println!("{}", caps.get(1).unwrap().as_str());
                    }

                }

                if untracked.is_match(&stdout) {

                    for caps in untracked.captures_iter(&stdout) {
                        print_fancy(format!("{: >15}", "untracked: ").as_str(), Attrs::BOLD, Colors::YELLOW);
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

//   _________                    
//  /   _____/__.__. ____   ____  
//  \_____  <   |  |/    \_/ ___\ 
//  /        \___  |   |  \  \___ 
// /_______  / ____|___|  /\___  >
//         \/\/         \/     \/ 
pub fn sync(arc: &Arc) { // $SYNC

    let start_dir = env::current_dir().unwrap();    // to reset directory
    let switched = Regex::new(r"Switch.*'(.*?)'").unwrap();
    let already = Regex::new(r".*[A|a]lready.*").unwrap();
    let up_to_date = Regex::new(r".*up.to.date.*").unwrap();
    let updating = Regex::new(r".*Updating.*").unwrap();
    let conflict = Regex::new(r".*conflict.*").unwrap();


    for repo in &arc.repos {

        env::set_current_dir(&arc.path.tld).expect("arc root directory not found");
        let mut path = PathBuf::from(&arc.path.tld);
        path.push(&repo.name);              
        
        let chdir_result = env::set_current_dir(&path);
        match chdir_result {
            Err(_) => { // if added repo to arc.json clone down new additions
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

                print!("{:.<50}", &repo.name); 
                print!("{:.>25} ", &short_name);
                print_fancyln("cloning", Attrs::BOLD, Colors::CYAN);
                
                let fatal = Regex::new(r".*[f|F]+atal.*").unwrap();
                let url = format!("{}/{}.git", &arc.remote.url, &repo.name);
                let output = Command::new("git")
                                    .arg("clone")
                                    .arg(&url)
                                    .stdout(Stdio::inherit())
                                    .stderr(Stdio::inherit())
                                    .output()
                                    .expect("git clone failed");
                // let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if fatal.is_match(&stderr) {
                    print!("{}", &stderr);
                } 
                env::set_current_dir(&arc.path.tld).unwrap();
                continue;
            },
            Ok(_) => (), // repo already exists
        }

        // ###############  C H E C K O U T 
        let output = Command::new("git")
                            .arg("checkout")
                            .arg("master")
                            .output()
                            .expect("Couldn't invoke checkout");

        let checkout_stdout = String::from_utf8_lossy(&output.stdout);
        let checkout_stderr = String::from_utf8_lossy(&output.stderr);

        print!("{:.<50}", &repo.name);

        if switched.is_match(&checkout_stderr) {
            print!("{:.>25} ", "switched-to-master");
        } else if already.is_match(&checkout_stderr) {
            print!("{:.>25} ", "on-master");
        } else {
            print!("{:.>25} ", "unable");
            print_fancyln("dirty", Attrs::BOLD, Colors::RED);
            println!("stdout: {}", &checkout_stdout);
            println!("stderr: {}", &checkout_stderr);
            continue; // soft fail
        }

        // ###############  P U L L 
        let pull_output = Command::new("git")
                                .arg("pull")
                                .output()
                                .expect("couldn't pull");
        let pull_stdout = String::from_utf8_lossy(&pull_output.stdout);
        let pull_stderr = String::from_utf8_lossy(&pull_output.stderr);

        if up_to_date.is_match(&pull_stdout) {
            print_fancyln("up-to-date", Attrs::BOLD, Colors::GREEN);
        } else if updating.is_match(&pull_stdout) {
            print_fancyln("updating", Attrs::INVERSE, Colors::GREEN);
            print!("{}", pull_stdout);
        } else if conflict.is_match(&pull_stdout) {
            print_fancyln("conflict", Attrs::BOLD, Colors::YELLOW);
            print!("{}", pull_stdout);
        } else {
            // unknown condition
            print_fancyln("unable", Attrs::BOLD, Colors::RED);
            print!("{}", pull_stderr);
        }

        
    }
    env::set_current_dir(start_dir).unwrap();

    // ---->>>----- possibly make more threads
    // use std::thread;
    // use std::time::Duration;

    // let original_thread = thread::spawn(|| {
    // let _detached_thread = thread::spawn(|| {
    //     // Here we sleep to make sure that the first thread returns before.
    //     thread::sleep(Duration::from_millis(10));
    //     // This will be called, even though the JoinHandle is dropped.
    //     println!("♫ Still alive ♫");
    // });
    // });

    // original_thread.join().expect("The thread being joined has panicked");
    // println!("Original thread is joined.");

    // // We make sure that the new thread has time to run, before the main
    // // thread returns.

    // thread::sleep(Duration::from_millis(1000));

}

// __________                      __   
// \______   \ ____   ______ _____/  |_ 
//  |       _// __ \ /  ___// __ \   __\
//  |    |   \  ___/ \___ \\  ___/|  |  
//  |____|_  /\___  >____  >\___  >__|  
//         \/     \/     \/     \/     
pub fn reset(arc: &Arc) { // $SYNC
    let start_dir = env::current_dir().unwrap();    // to reset directory
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
        },
        Ok(_) => (), // do nothing
    }

    let mut do_it = false;
    if &input.to_ascii_lowercase().trim_end()[..] == "y" ||
        &input.to_ascii_lowercase().trim_end()[..] == "yes"  {
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
                },
                Ok(_) => (),
            }

            print!("{:.<50}", &repo.name);

            match arc.arc_cmd.opt {
                Opt::Sanitize(yes) => {
                    if yes {
                        Command::new("git")
                                .arg("rm").arg("--cached").arg("-r").arg(".")
                                .output()
                                .expect("couldn't sanitize");

                        print_fancy("sanitized ", Attrs::BOLD, Colors::CYAN);
                        print_fancy("now at-> ", Attrs::NORMAL, Colors::YELLOW);
                        
                        let output = Command::new("git")
                                            .arg("reset").arg("--hard").arg("origin/master")
                                            .output()
                                            .expect("couldn't reset");
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let caps = reset_regex.captures(&stdout).expect("couldn't capture regex");
                        println!("{}", caps.get(1).unwrap().as_str());

                    } else {
                        // regular reset
                        let output = Command::new("git")
                                            .arg("reset").arg("--hard").arg("origin/master")
                                            .output()
                                            .expect("couldn't reset");
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let caps = reset_regex.captures(&stdout).expect("couldn't capture regex");

                        print_fancy("now at-> ", Attrs::NORMAL, Colors::YELLOW);
                        println!("{}", caps.get(1).unwrap().as_str());
                    }
                },
                _ => (), // no other should match
            }

        }
        env::set_current_dir(start_dir).unwrap();

    } else {
        println!("Aborting");
    }


}

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
