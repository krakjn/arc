use crate::cmds::init::clone_repo;
use crate::cmds::pull::print_pull_result;
use crate::defs::Workspace;
use crate::error::Result;
use crate::git;
use crate::parse;
use crate::utils;
use owo_colors::OwoColorize;

/// Sync all repositories to default branch and pull
pub fn sync(ws: &Workspace) -> Result<()> {
    let (path_col, branch_col) = utils::line_layout(ws.repos.iter().map(|r| r.path.as_str()));

    for repo in &ws.repos {
        let dir = ws.repo_dir(repo);

        if !dir.is_dir() {
            let short_name = utils::short_name(&repo.path);
            print!("{}", utils::format_path_column(&repo.path, path_col));
            print!("{}", utils::format_branch_column(short_name, branch_col));
            println!("{}", "cloning".cyan().bold());

            if let Err(e) = clone_repo(ws, repo) {
                eprintln!("{}", e.to_string().red());
            }
            continue;
        }

        let default_branch = git::detect_default_branch(&dir);
        let output = git::run(&dir, &["checkout", &default_branch])?;
        let checkout_stdout = git::stdout(&output);
        let checkout_stderr = git::stderr(&output);

        print!("{}", utils::format_path_column(&repo.path, path_col));

        if parse::extract_switched(&checkout_stderr).is_some()
            || parse::extract_switched(&checkout_stdout).is_some()
        {
            print!(
                "{}",
                utils::format_branch_column(&format!("switched-to-{default_branch}"), branch_col)
            );
        } else if parse::already_on_branch(&checkout_stderr)
            || parse::already_on_branch(&checkout_stdout)
            || output.status.success()
        {
            print!(
                "{}",
                utils::format_branch_column(&format!("on-{default_branch}"), branch_col)
            );
        } else {
            print!("{}", utils::format_branch_column("unable", branch_col));
            println!("{}", "dirty".red().bold());
            if !checkout_stdout.is_empty() {
                println!("stdout: {checkout_stdout}");
            }
            if !checkout_stderr.is_empty() {
                println!("stderr: {checkout_stderr}");
            }
            continue;
        }

        let pull_output = git::run(&dir, &["pull"])?;
        let pull_stdout = git::stdout(&pull_output);
        let pull_stderr = git::stderr(&pull_output);
        print_pull_result(
            parse::classify_pull(&pull_stdout, &pull_stderr),
            &pull_stdout,
            &pull_stderr,
        );
    }

    Ok(())
}
