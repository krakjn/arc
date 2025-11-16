use crate::defs::Arc;
use crate::error::Result;
use crate::{arc, cmds};
use clap::{Parser, Subcommand};
use std::env;

/// Arc - Aggregate Repository Control
/// 
/// The bridge across repositories
#[derive(Parser, Debug)]
#[command(name = "arc")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "The bridge across repositories", long_about = None)]
struct Cli {
    /// Invoke any command across all repos
    #[arg(short = 'c', num_args = 1.., value_name = "COMMAND")]
    cmd: Option<Vec<String>>,

    /// Invoke any git command across all repos
    #[arg(short = 'g', num_args = 1.., value_name = "COMMAND", conflicts_with = "cmd")]
    gcmd: Option<Vec<String>>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print contact info and purpose of arc
    Info,
    
    /// Initialize all repos defined in arc.json
    Init,
    
    /// Fetch and merge on current branch
    Pull,
    
    /// Show status of all repos in arc.json
    Status {
        /// Show more detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Checks out all dependent repos on same branch
    Checkout {
        /// Branch name to checkout
        branch: String,
        
        /// Create a new branch
        #[arg(short = 'b')]
        new: bool,
    },
    
    /// Display current configuration
    Show,
    
    /// Checkout default branch and pull
    Sync,
    
    /// Reset worktree to origin default branch
    Reset {
        /// Remove cached changes
        #[arg(short, long)]
        sanitize: bool,
    },
}

/// Parse command line arguments and execute appropriate command
pub fn parse(arc_instance: &mut Arc) -> Result<()> {
    let argv: Vec<String> = env::args().collect();

    // Handle special cases where we pass through args to git/commands
    if argv.len() > 1 {
        if &argv[1] == "-c" || &argv[1] == "-g" {
            arc::load_arc(arc_instance)?;
            if &argv[1] == "-g" {
                return cmds::all(arc_instance, true, &argv);
            } else {
                return cmds::all(arc_instance, false, &argv);
            }
        }
    }

    let cli = Cli::parse();

    // Handle -c and -g flags (for command execution across repos)
    if cli.cmd.is_some() || cli.gcmd.is_some() {
        arc::load_arc(arc_instance)?;
        let is_git = cli.gcmd.is_some();
        let cmd_vec = if is_git {
            cli.gcmd.as_ref().unwrap()
        } else {
            cli.cmd.as_ref().unwrap()
        };
        
        // Build argv-like vec for compatibility with cmds::all
        let mut argv_like = vec![String::new(), if is_git { "-g".to_string() } else { "-c".to_string() }];
        argv_like.extend(cmd_vec.clone());
        return cmds::all(arc_instance, is_git, &argv_like);
    }

    match cli.command {
        Some(Commands::Info) => {
            arc::print_info();
            Ok(())
        }
        Some(Commands::Init) => {
            arc::load_arc(arc_instance)?;
            cmds::init(arc_instance)
        }
        Some(Commands::Pull) => {
            arc::load_arc(arc_instance)?;
            cmds::pull(arc_instance)
        }
        Some(Commands::Status { verbose }) => {
            arc::load_arc(arc_instance)?;
            cmds::status(arc_instance, verbose)
        }
        Some(Commands::Checkout { branch, new }) => {
            arc::load_arc(arc_instance)?;
            cmds::checkout(arc_instance, new, &branch)
        }
        Some(Commands::Show) => {
            arc::load_arc(arc_instance)?;
            arc::show_json(arc_instance);
            Ok(())
        }
        Some(Commands::Sync) => {
            arc::load_arc(arc_instance)?;
            cmds::sync(arc_instance)
        }
        Some(Commands::Reset { sanitize }) => {
            arc::load_arc(arc_instance)?;
            cmds::reset(arc_instance, sanitize)
        }
        None => {
            // No subcommand provided, print help
            println!("Run 'arc --help' for usage information");
            Ok(())
        }
    }
}
