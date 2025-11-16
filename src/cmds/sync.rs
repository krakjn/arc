use crate::defs::*;
use crate::error::{ArcError, Result};
use crate::utils;
use owo_colors::OwoColorize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

//   _________
//  /   _____/__.__. ____   ____
//  \_____  <   |  |/    \_/ ___\
//  /        \___  |   |  \  \___
// /_______  / ____|___|  /\___  >
//         \/\/         \/     \/
/// Sync all repositories to default branch and pull
pub fn sync(arc: &Arc) -> Result<()> {
    let start_dir = env::current_dir()?;

    for repo in &arc.repos {
        env::set_current_dir(&arc.path.tld)?;
        let mut path = PathBuf::from(&arc.path.tld);
        path.push(&repo.name);

        // If repo doesn't exist, clone it
        if env::set_current_dir(&path).is_err() {
            let (relative_path, short_name) = utils::clone_path(&repo.name);
            env::set_current_dir(&arc.path.tld)?;

            if relative_path != "shallow" {
                fs::create_dir_all(&relative_path)?;
                env::set_current_dir(&relative_path)?;
            }

            print!("{:.<50}", &repo.name);
            print!("{:.>25} ", &short_name);
            println!("{}", "cloning".cyan().bold());

            let url = format!("{}/{}.git", &arc.remote.url, &repo.name);
            let output = Command::new("git")
                .args(["clone", &url])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .map_err(|e| ArcError::CloneFailed {
                    repo: repo.name.clone(),
                    message: e.to_string(),
                })?;

            let stderr = String::from_utf8_lossy(&output.stderr);
            if utils::FATAL.is_match(&stderr) {
                eprintln!("{}", stderr.red());
            }
            
            env::set_current_dir(&arc.path.tld)?;
            continue;
        }

        // Detect default branch for this repo
        let default_branch = utils::detect_default_branch()
            .unwrap_or_else(|_| "master".to_string());

        // Checkout default branch
        let output = Command::new("git")
            .args(["checkout", &default_branch])
            .output()?;

        let checkout_stdout = String::from_utf8_lossy(&output.stdout);
        let checkout_stderr = String::from_utf8_lossy(&output.stderr);

        print!("{:.<50}", &repo.name);

        if utils::SWITCHED.is_match(&checkout_stderr) {
            print!("{:.>25} ", format!("switched-to-{}", default_branch));
        } else if utils::ALREADY.is_match(&checkout_stderr) {
            print!("{:.>25} ", format!("on-{}", default_branch));
        } else {
            print!("{:.>25} ", "unable");
            println!("{}", "dirty".red().bold());
            if !checkout_stdout.is_empty() {
                println!("stdout: {}", checkout_stdout);
            }
            if !checkout_stderr.is_empty() {
                println!("stderr: {}", checkout_stderr);
            }
            continue;
        }

        // Pull changes
        let pull_output = Command::new("git")
            .arg("pull")
            .output()?;
        let pull_stdout = String::from_utf8_lossy(&pull_output.stdout);
        let pull_stderr = String::from_utf8_lossy(&pull_output.stderr);

        if utils::UP_TO_DATE.is_match(&pull_stdout) {
            println!("{}", "up-to-date".green().bold());
        } else if utils::UPDATING.is_match(&pull_stdout) {
            println!("{}", "updating".green().bold());
            print!("{}", pull_stdout);
        } else if utils::CONFLICT.is_match(&pull_stdout) {
            println!("{}", "conflict".yellow().bold());
            print!("{}", pull_stdout);
        } else if !pull_stderr.is_empty() {
            println!("{}", "unable".red().bold());
            print!("{}", pull_stderr);
        } else {
            println!("{}", "ok".green());
        }
    }
    
    env::set_current_dir(start_dir)?;
    Ok(())

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
