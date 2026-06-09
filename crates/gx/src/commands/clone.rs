//! `gx clone <repo>` — clone a repo into the organised project dir and index
//! it. Ported from `src/commands/clone.ts`. Returns the target path (the
//! dispatcher writes it to stdout so shell wrappers can `cd` into it).

use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use crate::config::get_agent;
use crate::errors::{GxError, GxResult};
use crate::index_store::{iso_now, ProjectIndex};
use crate::path::to_path_for;
use crate::types::{Config, IndexEntry};
use crate::url::{parse_url, to_clone_url};

pub fn clone_repo(input: &str, config: &Config, index_path: &Path) -> GxResult<String> {
    let parsed = parse_url(input, &config.default_host, &config.default_owner)?;
    let agent = get_agent()?;
    let target_dir = to_path_for(&parsed, config, agent.as_deref());
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
        &parsed.repo,
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
