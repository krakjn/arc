use crate::{arc, cmds, defs::*};
use clap::{App, Arg};
use std::env;

fn build_cli() -> App<'static, 'static> {
    App::new("arc")
        .version(env!("CARGO_PKG_VERSION"))
        .about("The bridge across repositories")
        .subcommand(App::new("info").about("Print contact info, and purpose of arc"))
        .subcommand(App::new("init").about("Initialize all repos defined in arc.json"))
        .subcommand(App::new("pull").about("Fetch and merge on current branch"))
        .subcommand(
            App::new("status")
                .about("Show status of all repos in arc.json")
                .arg(
                    Arg::with_name("verbose")
                        .short("v")
                        .long("verbose")
                        // .multiple(true) // for further enhancement
                        .help("Show more info"),
                ),
        )
        .subcommand(
            App::new("checkout")
                .about("Checks out all dependent repos on same branch")
                .arg(
                    Arg::with_name("checkout_new_branch")
                        .short("b")
                        .takes_value(true)
                        .help("Checkout new branch"),
                ),
        )
        .subcommand(App::new("show").about("Display current configuration"))
        .subcommand(App::new("sync").about("Checkout master and pull"))
        .subcommand(
            App::new("reset")
                .about("Reset worktree to origin/master")
                .arg(
                    Arg::with_name("sanitize")
                        .short("s")
                        .long("sanitize")
                        .help("Removed cached changes"),
                ),
        )
        // TODO not supported yet
        // .subcommand(App::new("upgrade").about("Update arc to lastest release (under maintenance)"))
        // .arg(
        //     Arg::with_name("cmd")
        //         .help("Invoke any command across all repos")
        //         .short("c")
        //         .multiple(true)
        //         .takes_value(true)
        //         .value_name("COMMAND"),
        // )
        .arg(Arg::with_name("gcmd")
                .conflicts_with("cmd")
                .help("Invoke any git command across all repos")
                .short("g")
                .multiple(true)
                .takes_value(true)
                .value_name("COMMAND"),
        )
        .arg(
            Arg::with_name("version")
                .help("Displays version of arc")
                .short("v")
                .long("version"),
        )
}
/**
 * brief
 * @param _arc, arc instance
 */
pub fn parse(_arc: &mut Arc) {
    let argv: Vec<String> = env::args().collect();

    if argv.len() > 1 {
        /* Note: clap cannot handle passing flags to another program
         * hance a few custom calls to pull argument vector
         */
        // let argv_str = argv.iter().map(|s| &**s).collect::<Vec<&str>>();

        if &argv[1] == "-c" || &argv[1] == "-g" {
            arc::load_arc(_arc); // need to load arc for context
            if &argv[1] == "-g" {
                cmds::all(_arc, true, &argv);
            } else {
                cmds::all(_arc, false, &argv);
            }
            return; // Not regular parsing to be handled by clap
        } else if &argv[1] == "-v" || &argv[1] == "--version" {
            // don't need to load arc for version
            println!("arc-v{}", env!("CARGO_PKG_VERSION"));
            return; // Not regular parsing to be handled by clap
        }
    }

    let arg_matches = build_cli().get_matches();

    arc::load_arc(_arc);

    match arg_matches.subcommand() {
        ("info", Some(_)) => arc::print_info(),
        ("checkout", Some(_sub)) => {
            let branch_name = _sub.value_of("checkout_branch").unwrap();
            let mut new_branch = false;
            if _sub.is_present("checkout_new_branch") {
                new_branch = true;
            }
            cmds::checkout(_arc, new_branch, branch_name);
        }
        ("init", Some(_)) => cmds::init(_arc),
        ("pull", Some(_)) => cmds::pull(_arc),
        ("reset", Some(_sub)) => {
            let mut sanitize: bool = false;
            if _sub.is_present("sanitize") {
                sanitize = true;
            }
            cmds::reset(_arc, sanitize);
        }
        ("status", Some(_sub)) => {
            let mut verbose: bool = false;
            if _sub.is_present("verbose") {
                verbose = true;
            }
            cmds::status(_arc, verbose);
        }
        ("show", Some(_)) => arc::show_json(_arc),
        ("sync", Some(_)) => cmds::sync(_arc),
        ("upgrade", Some(_)) => println!("...under maintenance"),
        _ => { /* unsupported */ }
    }
}
