//! CLI entrypoint and argument dispatch. Hand-rolled to mirror
//! `src/index.ts` byte-for-byte: the verbatim `--help`/`--version` text, the
//! `[gx agent: …]` stderr line, the `config set` sub-dispatch, and the
//! "unknown command resolves as a project name" fallback. clap is deliberately
//! avoided here — its generated help, version, and error formatting would
//! break the parity the snapshot harness enforces.

use std::io::IsTerminal;
use std::path::PathBuf;

use crate::commands::{config_cmd, ls, recent, resolve, shell_init};
use crate::config::{get_agent, get_config_path, load_config};
use crate::errors::{GxError, GxResult};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

fn index_path() -> PathBuf {
    home_dir().join(".config/gx/index.json")
}

fn help_text() -> String {
    format!(
        "gx v{VERSION} — git project manager

Usage:
  gx <name>                Jump to project
  gx clone <repo>          Clone and jump to repo
  gx open [name]           Open project in editor
  gx ls                    List indexed projects
  gx index                 Index new repos (additive scan)
  gx index <path>...       Add specific repo(s) to index
  gx rebuild               Rescan and rebuild index
  gx recent                List recently visited projects
  gx recent -n <N>         Show last N projects
  gx resume <name>         Jump to project with git context
  gx config                Show config
  gx config set <key> <v>  Set config value
  gx init                  Scaffold .claude/ agent config
  gx shell-init [shell]    Print shell integration code
  gx resolve <name>        Resolve project name to path
  gx resolve --list        List all project names

Worktrees (requires shell integration):
  gx <name> wt [args...]   Jump to project and run wt (worktrunk)

Planned (not yet implemented):
  gx fork <repo>           Fork repo on GitHub, clone, and jump
  gx fork <repo> --clone-only  Clone existing fork, set upstream
  gx sync                  Fetch upstream and integrate changes
  gx sync --rebase|--merge Override sync strategy
  gx sync --push           Push to origin after syncing

Options:
  gx open --editor <name>  Override editor for this invocation
  gx init --type <type>    Override project type detection
  gx init --force          Overwrite existing CLAUDE.md"
    )
}

/// `parseFlag(args, flag)`: value following `flag`, or `None` if absent,
/// missing, or itself a flag (`-`-prefixed).
fn parse_flag<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let i = args.iter().position(|a| a == flag)?;
    let value = args.get(i + 1)?;
    if value.starts_with('-') {
        return None;
    }
    Some(value)
}

/// Parse a base-10 integer prefix the way JS `parseInt(value, 10)` does:
/// leading digits only, ignoring trailing non-digits. Returns `None` when no
/// digits are present (JS `NaN`).
fn parse_int_prefix(value: &str) -> Option<i64> {
    let digits: String = value.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<i64>().ok()
}

/// Run the CLI against `std::env::args`. Returns the process exit code.
pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

fn dispatch(args: &[String]) -> GxResult<()> {
    let command = args.first().map(String::as_str);

    match command {
        None | Some("--help") | Some("-h") => {
            println!("{}", help_text());
            return Ok(());
        }
        Some("--version") | Some("-v") => {
            println!("gx v{VERSION}");
            return Ok(());
        }
        _ => {}
    }
    let command = command.expect("non-help command present");

    let config_path = get_config_path();
    let index_path = index_path();
    let config = load_config(&config_path);
    let agent = get_agent()?;
    if let Some(a) = &agent {
        eprintln!("[gx agent: {a}]");
    }

    match command {
        "clone" | "index" | "rebuild" | "open" | "init" | "resume" => {
            // Ported in RST-6; the shipped binary remains the TS build until
            // the RST-7 cutover.
            Err(GxError::command(format!(
                "{command}: not yet ported (RST-6)"
            )))
        }
        "ls" => {
            ls::ls(&index_path);
            Ok(())
        }
        "config" => {
            if args.get(1).map(String::as_str) == Some("set")
                && args.get(2).is_some_and(|v| !v.is_empty())
                && args.get(3).is_some_and(|v| !v.is_empty())
            {
                config_cmd::set_config(&config_path, &args[2], &args[3])
            } else {
                config_cmd::show_config(&config_path)
            }
        }
        "shell-init" => shell_init::shell_init(args.get(1).map(String::as_str)),
        "resolve" => match args.get(1).map(String::as_str) {
            Some("--list") => resolve::resolve("", &index_path, &config, true),
            Some(name) => resolve::resolve(name, &index_path, &config, false),
            None => Err(GxError::command(
                "Usage: gx resolve <name> | gx resolve --list",
            )),
        },
        "recent" => {
            let n_value = parse_flag(args, "-n");
            if args.iter().any(|a| a == "-n") && n_value.is_none() {
                return Err(GxError::command("Usage: gx recent [-n <count>]"));
            }
            let limit = match n_value {
                Some(v) => {
                    let parsed = parse_int_prefix(v);
                    match parsed {
                        Some(n) if n >= 1 => Some(n as usize),
                        _ => return Err(GxError::command("Usage: gx recent [-n <count>]")),
                    }
                }
                None => None,
            };
            recent::recent(&index_path, limit);
            Ok(())
        }
        // Default: treat as a project name to resolve.
        _ => {
            resolve::resolve(command, &index_path, &config, false)?;
            if std::io::stdout().is_terminal() {
                eprintln!(
                    "Hint: add shell integration for cd support: run 'gx shell-init' and follow the printed instructions for your shell."
                );
            }
            Ok(())
        }
    }
}
