//! `gx open [name] [--editor <name>]` — open a project (or the cwd) in an
//! editor. Ported from `src/commands/open.ts`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::errors::{GxError, GxResult};
use crate::index_store::ProjectIndex;
use crate::resolve_name::{
    format_ambiguous, format_auto_match, resolve_project_name, ResolveResult,
};
use crate::types::Config;

/// Look up `name` in the editor table. Returns `(command, is_gui)`. Unknown
/// names map to themselves as a non-GUI command, matching the TS
/// `EDITORS[name] ?? { cmd: name, gui: false }`.
fn editor_info(name: &str) -> (String, bool) {
    match name {
        "code" => ("code".to_string(), true),
        "cursor" => ("cursor".to_string(), true),
        "zed" => ("zed".to_string(), true),
        "vim" => ("vim".to_string(), false),
        "nvim" => ("nvim".to_string(), false),
        "nano" => ("nano".to_string(), false),
        "emacs" => ("emacs".to_string(), false),
        "emacs-gui" => ("emacs".to_string(), true),
        "subl" => ("subl".to_string(), true),
        other => (other.to_string(), false),
    }
}

/// `None` only on Windows with nothing configured: it has no built-in
/// terminal editor, and Notepad cannot open a project directory.
pub fn resolve_editor(config: &Config, override_: Option<&str>) -> Option<String> {
    if let Some(o) = override_ {
        if !o.is_empty() {
            return Some(o.to_string());
        }
    }
    if !config.editor.is_empty() {
        return Some(config.editor.clone());
    }
    if let Ok(v) = std::env::var("VISUAL") {
        if !v.is_empty() {
            return Some(v);
        }
    }
    if let Ok(e) = std::env::var("EDITOR") {
        if !e.is_empty() {
            return Some(e);
        }
    }
    default_editor().map(str::to_string)
}

#[cfg(windows)]
fn default_editor() -> Option<&'static str> {
    None
}

#[cfg(not(windows))]
fn default_editor() -> Option<&'static str> {
    Some("nano")
}

/// Minimal `Bun.which` equivalent: resolve `cmd` against `PATH` (or treat a
/// `cmd` with a path separator as a direct path), requiring an executable file.
fn which(cmd: &str) -> Option<PathBuf> {
    which_in(cmd, std::env::var_os("PATH").as_deref())
}

fn which_in(cmd: &str, paths: Option<&OsStr>) -> Option<PathBuf> {
    let p = Path::new(cmd);
    if p.components().count() > 1 {
        return executable_candidates(p)
            .into_iter()
            .find(|c| is_executable_file(c));
    }
    std::env::split_paths(paths?).find_map(|dir| {
        executable_candidates(&dir.join(cmd))
            .into_iter()
            .find(|c| is_executable_file(c))
    })
}

/// On Windows an extensionless name resolves through `PATHEXT` only: editor
/// bin dirs (VS Code, Cursor) ship an extensionless shell script beside the
/// `.cmd` shim, and that script cannot be spawned natively.
#[cfg(windows)]
fn executable_candidates(p: &Path) -> Vec<PathBuf> {
    if p.extension().is_some() {
        return vec![p.to_path_buf()];
    }
    let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
    pathext
        .split(';')
        .filter(|ext| !ext.is_empty())
        .map(|ext| {
            let mut s = p.as_os_str().to_owned();
            s.push(ext);
            PathBuf::from(s)
        })
        .collect()
}

#[cfg(not(windows))]
fn executable_candidates(p: &Path) -> Vec<PathBuf> {
    vec![p.to_path_buf()]
}

fn is_executable_file(p: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(p) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

pub fn open_project(
    name: Option<&str>,
    config: &Config,
    index_path: &Path,
    editor_override: Option<&str>,
) -> GxResult<()> {
    let project_path = match name {
        Some(name) => {
            let idx = ProjectIndex::load(index_path);
            match resolve_project_name(name, &idx, config) {
                ResolveResult::Exact { path, .. } => path,
                ResolveResult::Auto {
                    name,
                    path,
                    query,
                    score,
                } => {
                    eprintln!("{}", format_auto_match(&query, &name, score));
                    path
                }
                ResolveResult::Ambiguous { query, matches } => {
                    return Err(GxError::command(format_ambiguous(&query, &matches)));
                }
                ResolveResult::Missing { .. } => {
                    return Err(GxError::command(format!("Project '{name}' not found")));
                }
            }
        }
        None => std::env::current_dir()
            .map_err(|e| GxError::Other(format!("cwd: {e}")))?
            .to_string_lossy()
            .into_owned(),
    };

    let Some(editor_name) = resolve_editor(config, editor_override) else {
        return Err(GxError::command(
            "No editor configured. Pass --editor, set \"editor\" in ~/.config/gx/config.json, or set VISUAL/EDITOR.",
        ));
    };
    let (cmd, gui) = editor_info(&editor_name);

    // Spawn the resolved path: `Command::new` alone only appends `.exe` on
    // Windows, so `.cmd` shims such as `code.cmd` would not launch.
    let Some(program) = which(&cmd) else {
        return Err(GxError::command(format!(
            "Editor '{cmd}' not found on PATH"
        )));
    };

    if gui {
        Command::new(&program)
            .arg(&project_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| GxError::Other(format!("spawn {cmd}: {e}")))?;
        eprintln!("Opened {project_path} in {editor_name}");
    } else {
        let status = Command::new(&program)
            .arg(&project_path)
            .status()
            .map_err(|e| GxError::Other(format!("spawn {cmd}: {e}")))?;
        if !status.success() {
            return Err(GxError::command_with_code(
                format!("Editor '{cmd}' failed: {status}"),
                status.code().unwrap_or(1),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(editor: &str) -> Config {
        Config {
            editor: editor.to_string(),
            ..Config::default()
        }
    }

    // Override and config.editor short-circuit before any env read, so these
    // are deterministic regardless of the test process's $VISUAL/$EDITOR.
    #[test]
    fn override_beats_config_editor() {
        assert_eq!(
            resolve_editor(&cfg("vim"), Some("code")).as_deref(),
            Some("code")
        );
    }

    #[test]
    fn config_editor_used_when_no_override() {
        assert_eq!(resolve_editor(&cfg("nvim"), None).as_deref(), Some("nvim"));
    }

    #[test]
    fn empty_override_falls_through_to_config_editor() {
        assert_eq!(
            resolve_editor(&cfg("emacs"), Some("")).as_deref(),
            Some("emacs")
        );
    }

    #[test]
    fn editor_table_gui_flags() {
        assert_eq!(editor_info("code"), ("code".to_string(), true));
        assert_eq!(editor_info("cursor"), ("cursor".to_string(), true));
        assert_eq!(editor_info("zed"), ("zed".to_string(), true));
        assert_eq!(editor_info("subl"), ("subl".to_string(), true));
        assert_eq!(editor_info("vim"), ("vim".to_string(), false));
        assert_eq!(editor_info("nano"), ("nano".to_string(), false));
        assert_eq!(editor_info("emacs"), ("emacs".to_string(), false));
    }

    #[test]
    fn emacs_gui_maps_to_emacs_command() {
        assert_eq!(editor_info("emacs-gui"), ("emacs".to_string(), true));
    }

    #[test]
    fn unknown_editor_maps_to_itself_non_gui() {
        assert_eq!(editor_info("helix"), ("helix".to_string(), false));
    }

    #[test]
    fn which_in_misses_unknown_command() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(which_in("no-such-editor", Some(tmp.path().as_os_str())).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn which_in_resolves_direct_path_without_path_var() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::TempDir::new().unwrap();
        let bin = tmp.path().join("fake-editor");
        std::fs::write(&bin, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

        assert_eq!(which_in(bin.to_str().unwrap(), None), Some(bin));
    }

    #[cfg(windows)]
    #[test]
    fn which_in_resolves_direct_path_without_path_var() {
        let tmp = tempfile::TempDir::new().unwrap();
        let bin = tmp.path().join("fake-editor.exe");
        std::fs::write(&bin, "").unwrap();

        assert_eq!(which_in(bin.to_str().unwrap(), None), Some(bin));
    }

    #[cfg(unix)]
    #[test]
    fn which_in_finds_executable_on_path() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::TempDir::new().unwrap();
        let bin = tmp.path().join("fake-editor");
        std::fs::write(&bin, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

        assert_eq!(
            which_in("fake-editor", Some(tmp.path().as_os_str())),
            Some(bin)
        );
    }

    #[cfg(windows)]
    #[test]
    fn which_in_prefers_pathext_shim_over_extensionless_script() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("fake-editor"), "#!/bin/sh\n").unwrap();
        let shim = tmp.path().join("fake-editor.cmd");
        std::fs::write(&shim, "@echo off\r\n").unwrap();

        let found = which_in("fake-editor", Some(tmp.path().as_os_str())).unwrap();
        // PATHEXT entries are upper-case (`.CMD`); the filesystem is not.
        assert!(
            found
                .to_string_lossy()
                .eq_ignore_ascii_case(&shim.to_string_lossy()),
            "found {found:?}"
        );
    }
}
