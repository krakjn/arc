use crate::cmds;
use crate::config;
use crate::defs::BANNER;
use crate::error::Result;
use crate::run::RunOpts;
use crate::utils;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::io;
use std::path::PathBuf;

/// Arc - Aggregate Repository Control
///
/// The bridge across repositories
#[derive(Parser, Debug)]
#[command(name = "arc")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "The bridge across repositories", long_about = None)]
#[command(before_help = BANNER)]
struct Cli {
    /// Path to arc.toml or the directory that contains it (skips upward search)
    #[arg(long = "config", value_name = "PATH", global = true)]
    config: Option<PathBuf>,

    /// Only these repo paths (exact `path` from arc.toml)
    #[arg(long = "only", global = true, num_args = 1.., value_name = "PATH")]
    only: Vec<String>,

    /// Select a repo by exact `path` (checkout tree root; validated for all commands)
    #[arg(long = "repo", global = true, value_name = "PATH")]
    repo: Option<String>,

    /// Parallel jobs (default: CPU count). Use 1 if git asks for credentials.
    #[arg(short = 'j', long = "jobs", global = true, value_name = "N")]
    jobs: Option<usize>,

    /// Sort repos alphanumerically by path (default: arc.toml order)
    #[arg(long = "sort", global = true)]
    sort: bool,

    /// Color output
    #[arg(long = "color", global = true, value_enum, default_value_t = clap::ColorChoice::Auto)]
    color: clap::ColorChoice,

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
    /// Write a starter arc.toml to cwd or PATH
    Init {
        /// Directory or file path for arc.toml (default: current directory)
        path: Option<PathBuf>,
    },

    /// Clone all repos defined in arc.toml
    Clone,

    /// Fetch and merge on current branch
    Pull,

    /// Show status of all repos in arc.toml
    Status {
        /// Show more detailed information
        #[arg(short, long)]
        verbose: bool,

        /// Fetch remotes before showing ahead/behind
        #[arg(long)]
        fetch: bool,
    },

    /// Status of CWD/deps, or align that tree onto BRANCH
    Checkout {
        /// Branch to align (omit for scoped status of the tree)
        branch: Option<String>,
    },

    /// Display current configuration
    Show,

    /// Checkout default branch and pull
    Sync,

    /// Reset worktree to remote default branch
    Reset {
        /// Remove cached changes
        #[arg(short, long)]
        sanitize: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },

    /// Fetch from each repo's configured remote
    Fetch,

    /// Generate shell completions
    Completions { shell: Shell },
}

/// Parse CLI and run the selected command.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    utils::apply_color(cli.color);

    if matches!(cli.command, Some(Commands::Completions { .. }))
        && let Some(Commands::Completions { shell }) = cli.command
    {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "arc", &mut io::stdout());
        return Ok(());
    }

    if let Some(Commands::Init { path }) = &cli.command {
        return config::init(path.as_deref());
    }

    let opts = RunOpts::new(cli.only, cli.jobs, cli.sort);
    let ws = config::load(cli.config.as_deref())?;

    if let Some(path) = cli.repo.as_deref()
        && config::repo_by_path(&ws, path).is_none()
    {
        return Err(crate::error::ArcError::ConfigValidation(format!(
            "unknown repo path '{path}' (must match path in arc.toml exactly)"
        )));
    }

    if let Some(args) = cli.gcmd.as_ref() {
        return cmds::all(&ws, true, args, &opts);
    }

    if let Some(args) = cli.cmd.as_ref() {
        return cmds::all(&ws, false, args, &opts);
    }

    match cli.command {
        Some(Commands::Clone) => cmds::clone(&ws, &opts),
        Some(Commands::Pull) => cmds::pull(&ws, &opts),
        Some(Commands::Status { verbose, fetch }) => cmds::status(&ws, verbose, fetch, &opts),
        Some(Commands::Checkout { branch }) => {
            cmds::checkout(&ws, branch.as_deref(), cli.repo.as_deref(), opts.sort)
        }
        Some(Commands::Show) => {
            config::show(&ws);
            Ok(())
        }
        Some(Commands::Sync) => cmds::sync(&ws, &opts),
        Some(Commands::Reset { sanitize, yes }) => cmds::reset(&ws, sanitize, yes, &opts),
        Some(Commands::Fetch) => cmds::fetch(&ws, &opts),
        Some(Commands::Init { .. }) | Some(Commands::Completions { .. }) => unreachable!(),
        None => cmds::status(&ws, false, false, &opts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clap_debug_assert() {
        Cli::command().debug_assert();
    }

    #[test]
    fn checkout_branch_optional() {
        let none = Cli::try_parse_from(["arc", "checkout"]).unwrap();
        assert!(matches!(
            none.command,
            Some(Commands::Checkout { branch: None })
        ));
        let named = Cli::try_parse_from(["arc", "checkout", "feat"]).unwrap();
        assert!(matches!(
            named.command,
            Some(Commands::Checkout {
                branch: Some(ref b)
            }) if b == "feat"
        ));
    }

    #[test]
    fn repo_flag_parses() {
        let cli = Cli::try_parse_from(["arc", "--repo", "path/other", "checkout"]).unwrap();
        assert_eq!(cli.repo.as_deref(), Some("path/other"));
    }

    #[test]
    fn sort_flag_parses() {
        let cli = Cli::try_parse_from(["arc", "--sort", "status"]).unwrap();
        assert!(cli.sort);
        let off = Cli::try_parse_from(["arc", "status"]).unwrap();
        assert!(!off.sort);
    }
}
