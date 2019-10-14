pub static ARC_VERSION: &'static str = "0.1.6";
/*
 * ^ version number to stay on top line (for shipping)
 */
use std::env;
use std::fs;
use std::process::exit as exit;
use regex::Regex;
use std::path::{Path, PathBuf};

use defs::*;

impl Arc {
    pub fn new(arg: ArcCmd) -> Arc {
        Arc {
            path: ArcPath{tld: PathBuf::new(), arc: PathBuf::new()},
            remote: Remote{name: String::new(), url: String::new()},
            repos: Vec::new(),
            arc_cmd: arg,
        }
    }
}



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

pub fn find_git() -> bool {
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
    let start_dir = cwd.clone();    // to reset directory
    let up_dir = Path::new("..");

    let root_dir = Regex::new(r"^[a-zA-Z]:\\+$|/$").unwrap();

    loop {
        cwd = env::current_dir().unwrap();
        
        if root_dir.is_match(&cwd.to_string_lossy()) {
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
        },
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
    arc.remote.url  = arcj.remote.url;

    for (k, v) in arcj.projects {
        // println!("..... --> {}", k);
        let r = Repo{name: k, deps: v};
        // println!("loaded arc repo....{}", &r.name);
        arc.repos.push(r);
    }
}


pub fn print_version() {
    println!("arc-{}", ARC_VERSION);
}

pub fn print_info() {
    println!("   _____ ___________________
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
    http://dev.sandel.local/sandel-development/arc

The simplest solution, is the best solution - Tony Bright")
}


pub fn print_help() {
    println!("\
arc [command] [option]
    -h, --help              display this help message
    info                    print contact info, and purpose of arc
    -v, --version           print out version number of arc
[global]
    init                    initialize arc, cloning down all projects in arc.json
    status -v               display status of current + dependents (verbose)
    sync                    checkout master, and pull down updates 
    reset -s,--sanitize     CAUTION: reset working tree to remote
    -c, cmd                 invoke any shell command across arc
    upgrade                 upgrade arc to current released version
[dependent]
    pull                    fetch and merge on given and dependent branch
    checkout -b             checkout <branch_name> (-b) to create");

}

pub fn print_usage() {
    println!("\
usage: arc [-h | --help] [-v | --version] [init] [sync] [info]
           [status] [pull] [checkout <branch_name>] [-c | cmd] [reset]");
}


pub fn parse(argv: &Vec<String>) -> ArcCmd {

    if argv.len() == 1 {
        print_usage();
        exit(0);
    }

    let command: &str = &argv[1].to_ascii_lowercase();
    let options = &argv[2..];


    let mut this_arc = ArcCmd::new();

    match command {
        "-v" =>         { print_version(); std::process::exit(0) },
        "--version" =>  { print_version(); std::process::exit(0) },
        "-h" =>         { print_help(); std::process::exit(0) },
        "--help" =>     { print_help(); std::process::exit(0) },
        "info" =>       { print_info(); std::process::exit(0) },
        "init" =>       this_arc.cmd = Cmd::Init,
        "pull" =>       this_arc.cmd = Cmd::Pull, 
        "checkout" =>   this_arc.cmd = Cmd::Checkout,
        "reset" =>{
                        this_arc.cmd = Cmd::Reset;
                        this_arc.opt = Opt::Sanitize(false);
        }   
        "sync" =>       this_arc.cmd = Cmd::Sync,
        "status" =>     this_arc.cmd = Cmd::Status,
        "-c" =>         this_arc.cmd = Cmd::All,
        "cmd" =>        this_arc.cmd = Cmd::All,
        "upgrade" =>    this_arc.cmd = Cmd::Upgrade,
        _ =>            { print_usage(); std::process::exit(0)},
    }

    let mut new_branch = false;
    let mut all_vec: Vec<String> = Vec::new();

    for opt in options {

        if this_arc.cmd == Cmd::Checkout {
            match opt.as_str() {
                "-b" => new_branch = true,
                _ => {
                    if new_branch {
                        this_arc.opt = Opt::NewBranch(opt.to_string());
                    } else {
                        this_arc.opt = Opt::Branch(opt.to_string());
                    }
                },
            }

        } else if this_arc.cmd == Cmd::Status {
            match opt.as_str() {
                "-v" => this_arc.opt = Opt::Verbose(true),
                _ => this_arc.opt = Opt::Verbose(false),
            }

        } else if this_arc.cmd == Cmd::Reset {
            match opt.as_str() {
                "-s" =>         this_arc.opt = Opt::Sanitize(true),
                "--sanitize" => this_arc.opt = Opt::Sanitize(true),
                _ => (),
            }

        } else if this_arc.cmd == Cmd::All {
            all_vec.push(opt.to_string());

        } else {
            println!("invalid option");
        }
    }

    if this_arc.cmd == Cmd::All {
        // println!("all command opt -> {:?}", &all_vec);
        this_arc.opt = Opt::All(all_vec);
    }

    return this_arc;

}

pub fn upgrade(arc: &Arc) { //$UPGRADE

    let start_dir = env::current_dir().unwrap();

    // install location is known, either "C:\bin\arc.exe" or "/usr/bin/"
    let version = Regex::new(r"(\d+).(\d+).(\d+)").unwrap();

    let caps = version.captures(&ARC_VERSION).expect("couldn't capture arc-version");

    let curr_major: u32 = caps.get(1).unwrap().as_str().parse().unwrap();
    let curr_feature: u32 = caps.get(2).unwrap().as_str().parse().unwrap();
    let curr_minor: u32 = caps.get(3).unwrap().as_str().parse().unwrap();


    // File Transfer Protocal
    use ftp::FtpStream;

    let mut ftp_stream = FtpStream::connect(("10.0.0.45", 21)).unwrap();
    ftp_stream.login("anonymous", "").unwrap();


    #[cfg(target_os = "linux")]
    ftp_stream.cwd("arc-releases/linux").unwrap();
    #[cfg(target_os = "macos")]
    ftp_stream.cwd("arc-releases/mac").unwrap();
    #[cfg(target_os = "windows")]
    ftp_stream.cwd("arc-releases/windows").unwrap();

    let list_vec = ftp_stream.nlst(Some(".")).unwrap();

    let mut already_latest = true; // to show user they have latest
    let mut latest_major: u32 = 0;
    let mut latest_feature: u32 = 0;
    let mut latest_minor: u32 = 0;
    let mut latest_arc = String::new();
    
    for new_arc in list_vec {
        let caps = version.captures(&new_arc).expect("couldn't capture arc-version");
        let new_major: u32 = caps.get(1).unwrap().as_str().parse().unwrap();
        let new_feature: u32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let new_minor: u32 = caps.get(3).unwrap().as_str().parse().unwrap();

        if  new_major > latest_major || 
            new_feature > latest_feature ||
            new_minor > latest_minor {

                latest_major = new_major;
                latest_feature = new_feature;
                latest_minor = new_minor;
                latest_arc = new_arc.clone();
        }
    }

    if  latest_major > curr_major || 
        latest_feature > curr_feature ||
        latest_minor > curr_minor {

        already_latest = false;

        let cursor = ftp_stream.simple_retr(&latest_arc).unwrap();
        let contents = cursor.into_inner();
        // let text = str::from_utf8(&vec).unwrap();
        // println!("got data: {}", text);
        use std::process::Command;

        if cfg!(target_os = "windows") {
            let mut script_loc = PathBuf::from(&arc.path.tld);
            script_loc.push("sandel-development");
            script_loc.push("arc");
            script_loc.push("scripts");
            env::set_current_dir(script_loc).expect("chdir sandel-development/arc/scripts fail");
            fs::write("arc.exe", &contents).unwrap();
            Command::new("windows_upgrade.cmd")
                    .output()
                    .expect("couldn't run windows_upgrade.cmd");
            fs::remove_file("arc.exe").expect("couldn't remove arc.exe");
        } else {
            let mut script_loc = PathBuf::from(&arc.path.tld);
            script_loc.push("sandel-development/arc/scripts");
            env::set_current_dir(script_loc).expect("chdir sandel-development/arc/scripts fail");
            fs::write("arc", &contents).unwrap();
            Command::new("./unix_install.sh")
                    .output()
                    .expect("couldn't run unix_install.sh");
            fs::remove_file("arc").expect("couldn't remove arc");
        }

    }


    if already_latest {
        println!("arc is already latest version");
    } else {
        println!("arc upgraded {}.{}.{} -> {}.{}.{}", 
                curr_major, curr_feature, curr_minor, 
                latest_major, latest_feature, latest_minor);
    }
    // reset directory
    env::set_current_dir(&start_dir).unwrap();
    ftp_stream.quit().expect("error quiting ftp server");
}
