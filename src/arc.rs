use crate::defs::*;
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

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

pub fn format_fancy(arg: &str, attr: Attrs, color: Colors) -> String {
    format!("\u{1b}[{};{}m{}\u{1b}[0m", attr, color, arg)
}

pub fn find_git() -> bool {
    let mut cwd = env::current_dir().unwrap();
    let start_dir = cwd.clone(); // to reset directory
    let up_dir = "..";

    let root_dir = Regex::new(r"[a-zA-Z]:\\$|/$").unwrap();

    loop {
        cwd = env::current_dir().unwrap();

        if root_dir.is_match(&cwd.to_string_lossy()[..]) {
            env::set_current_dir(start_dir).unwrap();
            return false;
        }

        let list_files = fs::read_dir(&cwd).unwrap();

        let names = list_files
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str().map(|s| String::from(s)))
                })
            })
            .collect::<Vec<String>>();

        for name in names {
            // could add performance by only searching ".hidden" folders
            if name == ".git" {
                env::set_current_dir(start_dir).unwrap();
                return true;
            }
        }
        env::set_current_dir(&up_dir).unwrap();
    }
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
fn find_arc(arc: &mut Arc) -> bool {
    let mut cwd = env::current_dir().unwrap();
    let start_dir = cwd.clone(); // to reset directory
    let up_dir = Path::new("..");

    let root_dir = Regex::new(r"^[a-zA-Z]:\\+$|/$").unwrap();

    loop {
        cwd = env::current_dir().unwrap();

        if root_dir.is_match(&cwd.to_string_lossy()) {
            env::set_current_dir(start_dir).unwrap();
            return false;
        }

        let list_files = fs::read_dir(&cwd).unwrap();
        let names = list_files
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str().map(|s| String::from(s)))
                })
            })
            .collect::<Vec<String>>();
        // println!("print names = {:?}", names);

        for name in names {
            if name == "arc.json" {
                // println!("MADE IT = {}", cwd.display());
                arc.path.tld = cwd.clone();
                cwd.push("arc.json");
                arc.path.arc = cwd.clone();
                env::set_current_dir(start_dir).unwrap();
                return true;
            }
        }
        env::set_current_dir(&up_dir).unwrap();
    }
}

pub fn load_arc(arc: &mut Arc) {
    /*
     * the location of arc.json will determine the root directory of arc
     */
    if !find_arc(arc) {
        println!("couldn't find 'arc.json' in this directory or any parent directory");
        exit(1);
    }

    let arcj: ArcJson;
    let contents: String;

    let read_result = fs::read_to_string(&arc.path.arc);
    match read_result {
        Err(_) => {
            println!("Couldn't read 'arc.json'");
            exit(1);
        }
        Ok(c) => contents = c,
    }

    let result = serde_json::from_str(&contents[..]);
    match result {
        Err(_) => {
            println!("couldn't parse arc.json, is it valid json?");
            exit(1);
        }
        Ok(json) => arcj = json,
    }

    arc.remote.name = arcj.remote.name;
    arc.remote.url = arcj.remote.url;

    for (k, v) in arcj.projects {
        // println!("..... --> {}", k);
        let r = Repo { name: k, deps: v };
        // println!("loaded arc repo....{}", &r.name);
        arc.repos.push(r);
    }
}

pub fn print_info() {
    println!(
        "   _____ ___________________
  /  _  \\\\______   \\_   ____\\
 /  /_\\  \\|       _/    \\       A_ggregate
/    |    \\    |   \\     \\____  R_epository
\\____|____/____|___/\\________/  C_ontrol

-- The bridge across repositories --
The purpose of this tool is to help larger projects based on git have
a smarter way of checking out projects especially if those projects
have dependencies. Once those dependencies are known to arc, then arc
will handle the rest. Dependent projects will all be checked out to
the same branch, that way you don't have to worry about unrelated
histories ever again. 

Submit issues through GitLab to:
    http://domain/path/arc

The simplest solution, is the best solution - Tony Bright"
    )
}

pub fn show_json(arc: &Arc) {
    for repo in &arc.repos {
        print_fancy(arc.remote.url.as_str(), Attrs::NORMAL, Colors::CYAN);
        let repo_str = format!(" {}", repo.name);
        print_fancy(repo_str.as_str(), Attrs::NORMAL, Colors::YELLOW);
        print!("{:.>width$}", ".", width = 50 - repo.name.len());
        print_fancy("deps:", Attrs::NORMAL, Colors::PURPLE);
        print!(" [ ");
        for dep in &repo.deps {
            if dep == &repo.deps[&repo.deps.len() - 1] {
                print_fancy(
                    format!("{} ", dep).as_str(),
                    Attrs::NORMAL,
                    Colors::BRIGHT_PURPLE,
                );
            } else {
                print_fancy(
                    format!("{}, ", dep).as_str(),
                    Attrs::NORMAL,
                    Colors::BRIGHT_PURPLE,
                );
            }
        }
        println!("]")
    }
}

// TODO: will rewrite using reqwest library
// #[allow(dead_code)]
// pub fn upgrade(_arc: &Arc) -> Result<(), reqwest::Error> {
//     //$UPGRADE

//     let body =
//         reqwest::blocking::get("http://domain/path/arc/-/releases")?.text()?;

//     println!("body = {:?}", body);

//     Ok(())
    // let start_dir = env::current_dir().unwrap();

    // // install location is known, either "C:\bin\arc.exe" or "/usr/bin/"
    // let version = Regex::new(r"(\d+).(\d+).(\d+)").unwrap();

    // let caps = version.captures(&VERSION).expect("couldn't capture arc-version");

    // let curr_major: u32 = caps.get(1).unwrap().as_str().parse().unwrap();
    // let curr_feature: u32 = caps.get(2).unwrap().as_str().parse().unwrap();
    // let curr_minor: u32 = caps.get(3).unwrap().as_str().parse().unwrap();

    // // File Transfer Protocal
    // use ftp::FtpStream;

    // let mut ftp_stream = FtpStream::connect(("10.0.0.45", 21)).unwrap();
    // ftp_stream.login("anonymous", "").unwrap();

    // #[cfg(target_os = "linux")]
    // ftp_stream.cwd("arc-releases/linux").unwrap();
    // #[cfg(target_os = "macos")]
    // ftp_stream.cwd("arc-releases/mac").unwrap();
    // #[cfg(target_os = "windows")]
    // ftp_stream.cwd("arc-releases/windows").unwrap();

    // let list_vec = ftp_stream.nlst(Some(".")).unwrap();

    // let mut already_latest = true; // to show user they have latest
    // let mut latest_major: u32 = 0;
    // let mut latest_feature: u32 = 0;
    // let mut latest_minor: u32 = 0;
    // let mut latest_arc = String::new();

    // for new_arc in list_vec {
    //     let caps = version.captures(&new_arc).expect("couldn't capture arc-version");
    //     let new_major: u32 = caps.get(1).unwrap().as_str().parse().unwrap();
    //     let new_feature: u32 = caps.get(2).unwrap().as_str().parse().unwrap();
    //     let new_minor: u32 = caps.get(3).unwrap().as_str().parse().unwrap();

    //     if  new_major > latest_major ||
    //         new_feature > latest_feature ||
    //         new_minor > latest_minor {

    //             latest_major = new_major;
    //             latest_feature = new_feature;
    //             latest_minor = new_minor;
    //             latest_arc = new_arc.clone();
    //     }
    // }

    // if  latest_major > curr_major ||
    //     latest_feature > curr_feature ||
    //     latest_minor > curr_minor {

    //     already_latest = false;

    //     let cursor = ftp_stream.simple_retr(&latest_arc).unwrap();
    //     let contents = cursor.into_inner();
    //     // let text = str::from_utf8(&vec).unwrap();
    //     // println!("got data: {}", text);
    //     use std::process::Command;

    //     if cfg!(target_os = "windows") {
    //         let mut script_loc = PathBuf::from(&arc.path.tld);
    //         script_loc.push("sandel-development");
    //         script_loc.push("arc");
    //         script_loc.push("scripts");
    //         env::set_current_dir(script_loc).expect("chdir sandel-development/arc/scripts fail");
    //         fs::write("arc.exe", &contents).unwrap();
    //         Command::new("windows_upgrade.cmd")
    //                 .output()
    //                 .expect("couldn't run windows_upgrade.cmd");
    //         fs::remove_file("arc.exe").expect("couldn't remove arc.exe");
    //     } else {
    //         let mut script_loc = PathBuf::from(&arc.path.tld);
    //         script_loc.push("sandel-development/arc/scripts");
    //         env::set_current_dir(script_loc).expect("chdir sandel-development/arc/scripts fail");
    //         fs::write("arc", &contents).unwrap();
    //         Command::new("./unix_install.sh")
    //                 .output()
    //                 .expect("couldn't run unix_install.sh");
    //         fs::remove_file("arc").expect("couldn't remove arc");
    //     }

    // }

    // if already_latest {
    //     println!("arc is already latest version");
    // } else {
    //     println!("arc upgraded {}.{}.{} -> {}.{}.{}",
    //             curr_major, curr_feature, curr_minor,
    //             latest_major, latest_feature, latest_minor);
    // }
    // // reset directory
    // env::set_current_dir(&start_dir).unwrap();
    // ftp_stream.quit().expect("error quiting ftp server");
// }
