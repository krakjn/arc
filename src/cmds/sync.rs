use regex::Regex;
use std::env;
use std::path::PathBuf;
// use std::ffi::OsStr;
use std::fs;
use std::process::{Command, Stdio};

use crate::arc::*;
use crate::defs::*;

//   _________
//  /   _____/__.__. ____   ____
//  \_____  <   |  |/    \_/ ___\
//  /        \___  |   |  \  \___
// /_______  / ____|___|  /\___  >
//         \/\/         \/     \/
pub fn sync(arc: &Arc) {
    // $SYNC

    let start_dir = env::current_dir().unwrap(); // to reset directory
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
            Err(_) => {
                // if added repo to arc.json clone down new additions
                let (relative_path, short_name) = super::init::clone_path(&repo.name);
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
            }
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
