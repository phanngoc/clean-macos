//! Headless front-end for the cache cleaner (no webview needed).
//!
//! Run without arguments in a terminal for the interactive `suggest` flow: scan everything,
//! tick what to delete, and let the cleaner learn from the choice for next time.

mod classic;
mod memory_cmd;
mod suggest;
mod view;

use clap::{CommandFactory, Parser, Subcommand};
use std::io::IsTerminal;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "cache-cleaner-cli",
    version,
    about = "Find what eats your disk (caches, Docker, WSL, build output of idle projects), \
             pick what to delete with checkboxes, and let the cleaner learn your habits.",
    after_help = "Without a command, an interactive terminal runs `suggest`."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Scan everything, suggest what to delete (pre-ticked from what it learned), clean the picks
    Suggest(suggest::SuggestArgs),
    /// Show or edit what the cleaner learned (hot spots, habits)
    Memory(memory_cmd::MemoryArgs),
    /// List caches on this machine and their size
    Scan,
    /// List cache types supported on this platform
    Types,
    /// Empty the given caches (names from `types`, case-insensitive)
    Clean {
        /// Report what would be freed without deleting
        #[arg(long)]
        dry_run: bool,
        #[arg(required = true, value_name = "TYPE")]
        types: Vec<String>,
    },
    /// Build output (node_modules, target, .next, .venv, CACHEDIR.TAG dirs) of idle projects
    Artifacts {
        /// A project is idle when no file changed for this many days
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
        idle_days: u64,
        /// Also trim files unread for N days inside active CACHEDIR.TAG dirs (0 = off)
        #[arg(long, default_value_t = 0)]
        sweep_days: u64,
        /// Delete what is found (lists only otherwise)
        #[arg(long)]
        delete: bool,
        /// Folders to search; prefix with `wsl:` (e.g. wsl:/home/me) to search inside WSL
        #[arg(required = true, value_name = "ROOT")]
        roots: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let command = match Cli::parse().command {
        Some(command) => command,
        None if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() => {
            Cli::parse_from(["cache-cleaner-cli", "suggest"]).command.expect("suggest")
        }
        None => {
            let _ = Cli::command().print_help();
            return ExitCode::from(2);
        }
    };
    match command {
        Command::Suggest(args) => suggest::run(args).await,
        Command::Memory(args) => memory_cmd::run(args),
        Command::Scan => classic::scan().await,
        Command::Types => classic::types(),
        Command::Clean { dry_run, types } => classic::clean(&types, dry_run).await,
        Command::Artifacts { idle_days, sweep_days, delete, roots } => {
            classic::artifacts(roots, idle_days, sweep_days, delete)
        }
    }
}
