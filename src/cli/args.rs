use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "gitnest",
    author,
    version,
    about = "One Workspace. Multiple Git Identities.",
    long_about = "GitNest is a cross-platform CLI tool for managing directory-scoped GitHub identities without touching global git config."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize GitNest directory structure (~/.gitnest)
    #[command(alias = "i")]
    Init,

    /// Login to a GitHub account via OAuth Device Flow
    #[command(alias = "l")]
    Login,

    /// List all registered GitHub accounts
    #[command(alias = "a")]
    Accounts,

    /// Account management subcommands
    Account {
        #[command(subcommand)]
        command: AccountCommands,
    },

    /// Register or manage project mappings
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// List all project identity mappings
    #[command(alias = "ls")]
    Projects,

    /// Display GitHub identity assigned to current project directory
    #[command(alias = "c")]
    Current,

    /// Scan directory recursively for Git repositories
    #[command(alias = "sc")]
    Scan {
        /// Starting directory path to scan (defaults to current directory)
        path: Option<PathBuf>,
    },

    /// Run health diagnostics
    #[command(alias = "dr")]
    Doctor,

    /// Print version details (GitNest, Rust, Git, OS)
    #[command(alias = "v")]
    Version,

    /// Export GitNest configuration and keys backup ZIP
    #[command(alias = "ex")]
    Export {
        /// Target ZIP file output path (defaults to gitnest-backup.zip)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import GitNest backup ZIP archive
    #[command(alias = "im")]
    Import {
        /// Source ZIP file input path
        input: PathBuf,
    },

    /// Clone a repository using a selected GitNest identity
    #[command(alias = "cl")]
    Clone {
        /// Repository URL to clone
        url: String,

        /// Target directory path
        target_dir: Option<PathBuf>,

        /// Account ID or GitHub username to use
        #[arg(short, long)]
        account: Option<String>,
    },

    /// Run git push using the identity mapped to this project
    #[command(alias = "p")]
    Push {
        /// Git push extra arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Run git pull using the identity mapped to this project
    #[command(alias = "pl")]
    Pull {
        /// Git pull extra arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Show rich GitNest status, paths, active account, and repo state
    #[command(alias = "s")]
    Status,
}

#[derive(Subcommand, Debug)]
pub enum AccountCommands {
    /// Remove a registered account by ID or GitHub username
    #[command(alias = "ar")]
    Remove {
        /// Account ID or GitHub username to remove
        id_or_username: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// Register a project path and map it to a GitHub account
    #[command(alias = "pa")]
    Add {
        /// Project directory path (defaults to current directory)
        path: Option<PathBuf>,

        /// Account ID or GitHub username to associate
        #[arg(short, long)]
        account: Option<String>,
    },

    /// Remove project mapping for a path (defaults to current directory)
    #[command(alias = "pr")]
    Remove {
        /// Project directory path (defaults to current directory)
        path: Option<PathBuf>,
    },
}
