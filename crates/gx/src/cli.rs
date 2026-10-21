//! CLI entrypoint and argument dispatch. Hand-rolled to mirror
//! `src/index.ts` byte-for-byte: the verbatim `--help`/`--version` text, the
//! `[gx agent: …]` stderr line, the `config set` sub-dispatch, and the
//! "unknown command resolves as a project name" fallback. clap is deliberately
//! avoided here — its generated help, version, and error formatting would
//! break the parity the snapshot harness enforces.

use std::io::IsTerminal;
use std::path::PathBuf;

use crate::commands::{
    clone, config_cmd, doctor, index_repos, init, ls, open, rebuild, recent, resolve, resume,
    shell_init,
};
use crate::config::{get_agent, get_config_path, load_config};
use crate::errors::{GxError, GxResult};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn index_path() -> PathBuf {
    crate::config::home_dir().join(".config/gx/index.json")
}

fn help_text() -> String {
    format!(
        "gx v{VERSION} — git project manager

Usage:
  gx <name>                Jump to project
  gx clone <repo>          Clone and jump to repo
  gx clone <repo> <dest>   Clone to an explicit path (one-off, no config change)
  gx open [name]           Open project in editor
  gx ls                    List indexed projects
  gx index                 Index new repos (additive scan)
  gx index <path>...       Add specific repo(s) to index
  gx rebuild               Rescan and rebuild index
  gx recent                List recently visited projects
  gx recent -n <N>         Show last N projects
  gx resume <name>         Jump to project with git context
  gx doctor                Verify installation health
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
  gx init --force          Overwrite existing agent config files"
    )
}

fn command_help(command: &str) -> Option<&'static str> {
    Some(match command {
        "clone" => "Usage: gx clone <repo> [dest]\nClone and index a repository. Shell integration enters the destination.",
        "open" => "Usage: gx open [name] [--editor <editor>]\nOpen an indexed project or the current directory in an editor.",
        "init" => "Usage: gx init [--type <type>] [--force]\nWrite .claude/CLAUDE.md and plan/review commands in the current directory.\nTypes: typescript-bun, typescript-node, rust, go, python, generic\n--force overwrites existing generated files.",
        "index" => "Usage: gx index [path...]\nAdd repositories from the project directory or explicit repository roots.",
        "rebuild" => "Usage: gx rebuild\nRescan the project directory, removing stale entries in that scope.",
        "recent" => "Usage: gx recent [-n <count>]\nList projects by most recent visit, falling back to their clone/index time.",
        "resume" => "Usage: gx resume <name>\nShow Git context and print the project path. Shell integration enters it.",
        "doctor" => "Usage: gx doctor\nCheck installation, shell configuration and indexed paths without changing them.",
        "ls" => "Usage: gx ls\nList indexed project names and paths.",
        "config" => "Usage: gx config | gx config set <key> <value>\nKeys: projectDir, defaultHost, defaultOwner, structure, shallow, similarityThreshold, editor",
        "shell-init" => "Usage: gx shell-init [zsh|bash|fish|powershell]\nPrint shell integration code for your shell configuration.",
        "resolve" => "Usage: gx resolve <name> | gx resolve --list\nPrint a project path and record a visit, or list all indexed names.",
        _ => return None,
    })
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

    if args[1..].iter().any(|arg| arg == "--help" || arg == "-h") {
        if let Some(help) = command_help(command) {
            println!("{help}");
            return Ok(());
        }
    }

    let config_path = get_config_path();
    let index_path = index_path();
    let config = load_config(&config_path);
    let agent = get_agent()?;
    if let Some(a) = &agent {
        eprintln!("[gx agent: {a}]");
    }

    match command {
        "clone" => {
            const USAGE: &str = "Usage: gx clone <repo> [dest]";
            let repo = args.get(1).filter(|r| !r.is_empty());
            let Some(repo) = repo else {
                return Err(GxError::command(USAGE));
            };
            // Optional one-off destination; empty or extra positionals are a
            // usage error rather than being silently ignored.
            let dest = match args.get(2) {
                Some(d) if d.is_empty() => return Err(GxError::command(USAGE)),
                Some(d) => Some(d.as_str()),
                None => None,
            };
            if args.len() > 3 {
                return Err(GxError::command(USAGE));
            }
            let path = clone::clone_repo(repo, dest, &config, &index_path)?;
            println!("{path}");
            Ok(())
        }
        "index" => index_repos::index_repos(&args[1..], &config, &index_path),
        "rebuild" => rebuild::rebuild(&config, &index_path),
        "open" => {
            let editor = parse_flag(args, "--editor");
            if args.iter().any(|a| a == "--editor") && editor.is_none() {
                return Err(GxError::command("Usage: gx open [name] --editor <name>"));
            }
            let editor_idx = args.iter().position(|a| a == "--editor");
            let name = args.iter().enumerate().find_map(|(i, a)| {
                if i > 0 && a != "--editor" && editor_idx.map_or(true, |ei| i != ei + 1) {
                    Some(a.as_str())
                } else {
                    None
                }
            });
            open::open_project(name, &config, &index_path, editor)
        }
        "init" => {
            let type_opt = parse_flag(args, "--type");
            if args.iter().any(|a| a == "--type") && type_opt.is_none() {
                return Err(GxError::command("Usage: gx init --type <type>"));
            }
            let force = args.iter().any(|a| a == "--force");
            let cwd = std::env::current_dir().map_err(|e| GxError::Other(format!("cwd: {e}")))?;
            init::init_agent(&cwd, type_opt, force)
        }
        "resume" => {
            let name = args.get(1).filter(|n| !n.is_empty());
            let Some(name) = name else {
                return Err(GxError::command("Usage: gx resume <name>"));
            };
            resume::resume(name, &index_path, &config)
        }
        "doctor" => doctor::doctor(&config_path, &index_path, &config),
        "ls" => {
            ls::ls(&index_path);
            Ok(())
        }
        "config" => {
            if args.len() == 1 {
                config_cmd::show_config(&config_path)
            } else if args.len() == 4
                && args.get(1).map(String::as_str) == Some("set")
                && args.get(2).is_some_and(|v| !v.is_empty())
                && args.get(3).is_some_and(|v| !v.is_empty())
            {
                config_cmd::set_config(&config_path, &args[2], &args[3])
            } else {
                Err(GxError::command(
                    "Usage: gx config | gx config set <key> <value>",
                ))
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
