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
    binary_on_path_in(env::var_os("PATH")?)
}

fn binary_on_path_in(path_value: impl AsRef<std::ffi::OsStr>) -> Option<PathBuf> {
    for dir in env::split_paths(path_value.as_ref()) {
        for name in ["gx.exe", "gx"] {
            let candidate = dir.join(name);
            if is_executable(&candidate) {
                return Some(candidate);
            }
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

fn powershell_profiles(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1"),
        home.join("Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1"),
        home.join(".config/powershell/Microsoft.PowerShell_profile.ps1"),
    ]
}

fn shell_rc_files(shell_path: &str, home: &Path) -> Vec<PathBuf> {
    let normalized = shell_path.replace('\\', "/");
    let shell = Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(shell_path);
    let shell = shell
        .strip_suffix(".exe")
        .or_else(|| shell.strip_suffix(".EXE"))
        .unwrap_or(shell);

    match shell.to_ascii_lowercase().as_str() {
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
        "pwsh" | "powershell" => powershell_profiles(home),
        _ => Vec::new(),
    }
}

fn detect_shell_for_doctor() -> String {
    if let Ok(v) = env::var("GX_SHELL_OVERRIDE") {
        if !v.is_empty() {
            return v;
        }
    }
    if let Ok(v) = env::var("SHELL") {
        if !v.is_empty() {
            return v;
        }
    }
    if env::var_os("PSModulePath").is_some() {
        return "powershell".to_string();
    }
    String::new()
}

fn shell_check() -> Check {
    let home = crate::config::home_dir();
    let shell = detect_shell_for_doctor();
    let rc_files = shell_rc_files(&shell, &home);

    if rc_files.is_empty() {
        return Check {
            name: "shell",
            status: "warn",
            message: "unsupported shell; run gx shell-init <zsh|bash|fish|powershell>".to_string(),
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
    let index_check = if !index_path.exists() {
        Check {
            name: "index",
            status: "warn",
            message: format!("missing; run gx rebuild ({})", index_path.display()),
        }
    } else {
        match ProjectIndex::try_load(index_path) {
            Ok(idx) => {
                let entries = idx.list();
                let stale = entries
                    .iter()
                    .filter(|(_, entry)| !Path::new(&entry.path).exists())
                    .count();
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
            }
            Err(err) => Check {
                name: "index",
                status: "warn",
                message: format!("{err}; run gx rebuild ({})", index_path.display()),
            },
        }
    };

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
        index_check,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use tempfile::TempDir;

    fn make_executable(path: &Path) {
        fs::write(path, "fake").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
    }

    #[test]
    fn finds_gx_exe_on_path() {
        let tmp = TempDir::new().unwrap();
        let exe = tmp.path().join("gx.exe");
        make_executable(&exe);
        let found = binary_on_path_in(tmp.path().as_os_str());
        assert_eq!(found.as_deref(), Some(exe.as_path()));
    }

    #[test]
    fn finds_gx_when_exe_absent() {
        let tmp = TempDir::new().unwrap();
        let bin = tmp.path().join("gx");
        make_executable(&bin);
        let found = binary_on_path_in(tmp.path().as_os_str());
        assert_eq!(found.as_deref(), Some(bin.as_path()));
    }

    #[test]
    fn prefers_gx_exe_when_both_exist() {
        let tmp = TempDir::new().unwrap();
        make_executable(&tmp.path().join("gx"));
        let exe = tmp.path().join("gx.exe");
        make_executable(&exe);
        let found = binary_on_path_in(tmp.path().as_os_str());
        assert_eq!(found.as_deref(), Some(exe.as_path()));
    }

    #[test]
    fn powershell_rc_files_cover_windows_and_pwsh_paths() {
        let home = Path::new("/tmp/gx-home");
        let files = shell_rc_files("powershell", home);
        assert!(files
            .iter()
            .any(|p| p.ends_with("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")));
        assert!(files
            .iter()
            .any(|p| p.ends_with("Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1")));
        assert!(files
            .iter()
            .any(|p| p.ends_with(".config/powershell/Microsoft.PowerShell_profile.ps1")));
        assert_eq!(shell_rc_files("pwsh.exe", home), files);
    }

    #[test]
    fn unknown_shell_has_no_rc_files() {
        assert!(shell_rc_files("cmd", Path::new("/tmp")).is_empty());
    }

    #[test]
    fn empty_path_finds_nothing() {
        assert_eq!(binary_on_path_in(OsString::new()), None);
    }
}
