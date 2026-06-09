//! `gx doctor` — read-only installation health checks.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::effective_project_dir;
use crate::errors::GxResult;
use crate::index_store::ProjectIndex;
use crate::types::Config;

struct Check {
    name: &'static str,
    status: &'static str,
    message: String,
}

fn binary_on_path() -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join("gx");
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => meta.permissions().mode() & 0o111 != 0,
        _ => false,
    }
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn shell_rc_files(shell_path: &str) -> Vec<PathBuf> {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    let shell = Path::new(shell_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(shell_path);
    match shell {
        "zsh" => vec![home.join(".zshrc")],
        "bash" => {
            if cfg!(target_os = "macos") {
                vec![
                    home.join(".bash_profile"),
                    home.join(".profile"),
                    home.join(".bashrc"),
                ]
            } else {
                vec![home.join(".bashrc")]
            }
        }
        "fish" => vec![home.join(".config/fish/conf.d/gx.fish")],
        _ => Vec::new(),
    }
}

fn shell_check() -> Check {
    let shell = env::var("GX_SHELL_OVERRIDE")
        .or_else(|_| env::var("SHELL"))
        .unwrap_or_default();
    let rc_files = shell_rc_files(&shell);
    if rc_files.is_empty() {
        return Check {
            name: "shell",
            status: "warn",
            message: "unsupported shell; run gx shell-init <zsh|bash|fish>".to_string(),
        };
    }
    for rc in &rc_files {
        if let Ok(text) = fs::read_to_string(rc) {
            if text.contains("gx shell-init") || text.contains("gx.plugin.zsh") {
                return Check {
                    name: "shell",
                    status: "ok",
                    message: format!("integration found in {}", rc.display()),
                };
            }
        }
    }
    let candidates = rc_files
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Check {
        name: "shell",
        status: "warn",
        message: format!("integration not found in {candidates}; run gx shell-init"),
    }
}

pub fn doctor(config_path: &Path, index_path: &Path, config: &Config) -> GxResult<()> {
    let project_dir = effective_project_dir(config)?;
    let idx = ProjectIndex::load(index_path);
    let entries = idx.list();
    let stale = entries
        .iter()
        .filter(|(_, entry)| !Path::new(&entry.path).exists())
        .count();

    let mut checks = vec![
        Check {
            name: "runtime",
            status: "ok",
            message: "native".to_string(),
        },
        match binary_on_path() {
            Some(path) => Check {
                name: "binary",
                status: "ok",
                message: format!("gx found at {}", path.display()),
            },
            None => Check {
                name: "binary",
                status: "warn",
                message: "gx is not on PATH".to_string(),
            },
        },
        if config_path.exists() {
            Check {
                name: "config",
                status: "ok",
                message: config_path.display().to_string(),
            }
        } else {
            Check {
                name: "config",
                status: "warn",
                message: format!("missing; defaults are in use ({})", config_path.display()),
            }
        },
        if project_dir.exists() {
            Check {
                name: "projectDir",
                status: "ok",
                message: project_dir.display().to_string(),
            }
        } else {
            Check {
                name: "projectDir",
                status: "warn",
                message: format!("missing: {}", project_dir.display()),
            }
        },
        if index_path.exists() {
            Check {
                name: "index",
                status: if stale == 0 { "ok" } else { "warn" },
                message: if stale == 0 {
                    format!("{} project(s), {}", entries.len(), index_path.display())
                } else {
                    format!(
                        "{} project(s), {} stale path(s), {}",
                        entries.len(),
                        stale,
                        index_path.display()
                    )
                },
            }
        } else {
            Check {
                name: "index",
                status: "warn",
                message: format!("missing; run gx rebuild ({})", index_path.display()),
            }
        },
        shell_check(),
    ];

    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            checks.push(Check {
                name: "configDir",
                status: "warn",
                message: format!("missing: {}", parent.display()),
            });
        }
    }

    let width = checks
        .iter()
        .map(|check| check.name.len())
        .max()
        .unwrap_or(0);
    for check in &checks {
        println!(
            "{:<width$}  {:<4}  {}",
            check.name, check.status, check.message
        );
    }

    let warnings = checks.iter().filter(|check| check.status == "warn").count();
    if warnings == 0 {
        println!("doctor: ok");
    } else {
        println!("doctor: {warnings} warning(s); gx can still run");
    }
    Ok(())
}
