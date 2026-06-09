//! `gx open [name] [--editor <name>]` — open a project (or the cwd) in an
//! editor. Ported from `src/commands/open.ts`.

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

pub fn resolve_editor(config: &Config, override_: Option<&str>) -> String {
    if let Some(o) = override_ {
        if !o.is_empty() {
            return o.to_string();
        }
    }
    if !config.editor.is_empty() {
        return config.editor.clone();
    }
    if let Ok(v) = std::env::var("VISUAL") {
        if !v.is_empty() {
            return v;
        }
    }
    if let Ok(e) = std::env::var("EDITOR") {
        if !e.is_empty() {
            return e;
        }
    }
    "nano".to_string()
}

/// Minimal `Bun.which` equivalent: resolve `cmd` against `PATH` (or treat a
/// slash-containing `cmd` as a direct path), requiring an executable file.
fn which(cmd: &str) -> Option<PathBuf> {
    if cmd.contains('/') {
        let p = PathBuf::from(cmd);
        return if is_executable_file(&p) {
            Some(p)
        } else {
            None
        };
    }
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let full = dir.join(cmd);
        if is_executable_file(&full) {
            return Some(full);
        }
    }
    None
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

    let editor_name = resolve_editor(config, editor_override);
    let (cmd, gui) = editor_info(&editor_name);

    if which(&cmd).is_none() {
        return Err(GxError::command(format!(
            "Editor '{cmd}' not found on PATH"
        )));
    }

    if gui {
        Command::new(&cmd)
            .arg(&project_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| GxError::Other(format!("spawn {cmd}: {e}")))?;
        eprintln!("Opened {project_path} in {editor_name}");
    } else {
        Command::new(&cmd)
            .arg(&project_path)
            .status()
            .map_err(|e| GxError::Other(format!("spawn {cmd}: {e}")))?;
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
        assert_eq!(resolve_editor(&cfg("vim"), Some("code")), "code");
    }

    #[test]
    fn config_editor_used_when_no_override() {
        assert_eq!(resolve_editor(&cfg("nvim"), None), "nvim");
    }

    #[test]
    fn empty_override_falls_through_to_config_editor() {
        assert_eq!(resolve_editor(&cfg("emacs"), Some("")), "emacs");
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
}
