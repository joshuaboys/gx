//! `gx clone <repo> [dest]` — clone a repo into the organised project dir (or
//! an explicit one-off destination) and index it. Ported from
//! `src/commands/clone.ts`. Returns the target path (the dispatcher writes it
//! to stdout so shell wrappers can `cd` into it).

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::get_agent;
use crate::errors::{GxError, GxResult};
use crate::index_store::{iso_now, ProjectIndex};
use crate::path::{lexical_resolve, to_path_for};
use crate::types::{Config, IndexEntry, ParsedRepo};
use crate::url::{parse_url, to_clone_url};

/// Where a clone lands and what it is called in the index.
///
/// `dest`, when given, overrides the configured `projectDir`/`structure` layout
/// for this invocation only — it never writes config. Relative destinations
/// resolve against the cwd, matching `git clone`, and the index entry takes the
/// destination's basename (consistent with `gx index <path>`) so a repo cloned
/// to a custom directory is reachable by the name the user actually chose.
/// Agent routing (`GX_AGENT`) shapes the default layout only; an explicit dest
/// is taken literally.
fn resolve_target(
    parsed: &ParsedRepo,
    dest: Option<&str>,
    config: &Config,
    agent: Option<&str>,
) -> GxResult<(PathBuf, String)> {
    match dest {
        Some(d) => {
            let dir = lexical_resolve(d);
            let name = dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    GxError::Other(format!("invalid clone destination: {}", dir.display()))
                })?;
            Ok((dir, name))
        }
        None => Ok((to_path_for(parsed, config, agent), parsed.repo.clone())),
    }
}

pub fn clone_repo(
    input: &str,
    dest: Option<&str>,
    config: &Config,
    index_path: &Path,
) -> GxResult<String> {
    let parsed = parse_url(input, &config.default_host, &config.default_owner)?;
    let agent = get_agent()?;
    let (target_dir, name) = resolve_target(&parsed, dest, config, agent.as_deref())?;
    let target_str = target_dir.to_string_lossy().into_owned();

    // Check if the target already exists (handle .git as dir or file for
    // worktrees, and refuse a symlinked marker).
    match std::fs::symlink_metadata(target_dir.join(".git")) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Err(GxError::Other(format!(
                    "Refusing to clone into symlink: {target_str}"
                )));
            }
            if meta.is_dir() || meta.is_file() {
                eprintln!("already exists: {target_str}");
                return Ok(target_str);
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(GxError::Other(format!("{e}"))),
    }

    // Create the parent directory.
    if let Some(parent) = target_dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| GxError::Other(format!("create {}: {e}", parent.display())))?;
    }

    // Clone (stdout/stderr inherited so the user sees git's progress).
    let clone_url = to_clone_url(&parsed);
    let mut args: Vec<String> = vec!["clone".to_string()];
    if config.shallow {
        args.push("--depth=1".to_string());
    }
    args.push(clone_url.clone());
    args.push(target_str.clone());

    let status = Command::new("git")
        .args(&args)
        .status()
        .map_err(|e| GxError::Other(format!("spawn git: {e}")))?;
    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(GxError::Other(format!(
            "git clone failed with exit code {code}"
        )));
    }

    // Update the index.
    let mut idx = ProjectIndex::load(index_path);
    let now = iso_now();
    idx.add(
        &name,
        IndexEntry {
            path: target_str.clone(),
            url: clone_url,
            cloned_at: now.clone(),
            last_visited: Some(now),
        },
    );
    idx.save(index_path)?;

    Ok(target_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Structure;

    fn repo() -> ParsedRepo {
        ParsedRepo {
            host: "github.com".into(),
            owner: "juev".into(),
            repo: "gclone".into(),
            original_url: "https://github.com/juev/gclone".into(),
        }
    }

    fn config() -> Config {
        Config {
            project_dir: "/home/user/src".into(),
            ..Config::default()
        }
    }

    #[test]
    fn no_dest_uses_configured_layout() {
        let (dir, name) = resolve_target(&repo(), None, &config(), None).unwrap();
        assert_eq!(dir, PathBuf::from("/home/user/src/juev/gclone"));
        assert_eq!(name, "gclone");
    }

    #[test]
    fn no_dest_still_honours_structure_and_agent() {
        let cfg = Config {
            structure: Structure::Flat,
            ..config()
        };
        let (dir, name) = resolve_target(&repo(), None, &cfg, Some("morgan")).unwrap();
        assert_eq!(dir, PathBuf::from("/home/user/src/.morgan/gclone"));
        assert_eq!(name, "gclone");
    }

    #[test]
    fn absolute_dest_is_used_verbatim() {
        let (dir, name) = resolve_target(&repo(), Some("/tmp/scratch"), &config(), None).unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/scratch"));
        assert_eq!(name, "scratch");
    }

    #[test]
    fn dest_ignores_project_dir_and_agent_routing() {
        // An explicit destination must not pick up projectDir or the agent
        // dot-dir — that is the whole point of the override.
        let (dir, _) =
            resolve_target(&repo(), Some("/tmp/scratch"), &config(), Some("morgan")).unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/scratch"));
    }

    #[test]
    fn relative_dest_resolves_against_cwd_not_project_dir() {
        let cwd = std::env::current_dir().unwrap();
        let (dir, name) = resolve_target(&repo(), Some("put-it-here"), &config(), None).unwrap();
        assert_eq!(dir, cwd.join("put-it-here"));
        assert_eq!(name, "put-it-here");
    }

    #[test]
    fn dest_collapses_dot_segments() {
        let cwd = std::env::current_dir().unwrap();
        let (dir, name) = resolve_target(&repo(), Some("./a/../b"), &config(), None).unwrap();
        assert_eq!(dir, cwd.join("b"));
        assert_eq!(name, "b");
    }

    #[test]
    fn dest_trailing_slash_keeps_basename() {
        let (dir, name) = resolve_target(&repo(), Some("/tmp/scratch/"), &config(), None).unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/scratch"));
        assert_eq!(name, "scratch");
    }

    #[test]
    fn index_name_follows_dest_basename_when_it_differs_from_repo() {
        let (_, name) =
            resolve_target(&repo(), Some("/tmp/totally-different"), &config(), None).unwrap();
        assert_eq!(name, "totally-different");
    }

    #[test]
    fn root_dest_is_rejected() {
        let err = resolve_target(&repo(), Some("/"), &config(), None);
        assert!(err.is_err(), "expected root destination to be rejected");
    }
}
