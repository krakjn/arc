use crate::cmds;
use crate::config;
use crate::error::Result;
use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

/// Arc - Aggregate Repository Control
///
/// The bridge across repositories
#[derive(Parser, Debug)]
#[command(name = "arc")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "The bridge across repositories", long_about = None)]
struct Cli {
    /// Path to arc.toml (skips upward search)
    #[arg(long = "config", value_name = "PATH", global = true)]
    config: Option<PathBuf>,

    /// Invoke any command across all repos
    #[arg(short = 'c', num_args = 1.., value_name = "COMMAND")]
    cmd: Option<Vec<String>>,

    /// Invoke any git command across all repos
    #[arg(
        short = 'g',
        num_args = 1..,
        value_name = "COMMAND",
        conflicts_with = "cmd"
    )]
    gcmd: Option<Vec<String>>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print contact info and purpose of arc
    Info,

    /// Initialize all repos defined in arc.toml
    Init,

    /// Write a starter arc.toml to cwd or PATH
    InitConfig {
        /// Directory or file path for arc.toml (default: current directory)
        path: Option<PathBuf>,
    },

    /// Fetch and merge on current branch
    Pull,

    /// Show status of all repos in arc.toml
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

/// Parse CLI and run the selected command.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config.as_deref();

    if let Some(args) = cli.gcmd.as_ref() {
        let ws = config::load(config_path)?;
        return cmds::all(&ws, true, args);
    }

    if let Some(args) = cli.cmd.as_ref() {
        let ws = config::load(config_path)?;
        return cmds::all(&ws, false, args);
    }

    match cli.command {
        Some(Commands::Info) => {
            config::print_info();
            Ok(())
        }
        Some(Commands::Init) => {
            let ws = config::load(config_path)?;
            cmds::init(&ws)
        }
        Some(Commands::InitConfig { path }) => config::init_config(path.as_deref()),
        Some(Commands::Pull) => {
            let ws = config::load(config_path)?;
            cmds::pull(&ws)
        }
        Some(Commands::Status { verbose }) => {
            let ws = config::load(config_path)?;
            cmds::status(&ws, verbose)
        }
        Some(Commands::Checkout { branch, new }) => {
            let ws = config::load(config_path)?;
            cmds::checkout(&ws, new, &branch)
        }
        Some(Commands::Show) => {
            let ws = config::load(config_path)?;
            config::show(&ws);
            Ok(())
        }
        Some(Commands::Sync) => {
            let ws = config::load(config_path)?;
            cmds::sync(&ws)
        }
        Some(Commands::Reset { sanitize }) => {
            let ws = config::load(config_path)?;
            cmds::reset(&ws, sanitize)
        }
        None => {
            let mut cmd = Cli::command();
            cmd.print_help()?;
            println!();
            Ok(())
        }
    }
}
