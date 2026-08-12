//! `gx index [path...]` — additive scan of the project dir, or add specific
//! repo paths. Ported from `src/commands/index-repos.ts`.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::effective_project_dir;
use crate::errors::{GxError, GxResult};
use crate::index_store::{iso_now, ProjectIndex};
use crate::path::lexical_resolve;
use crate::types::{Config, IndexEntry};

pub fn index_repos(paths: &[String], config: &Config, index_path: &Path) -> GxResult<()> {
    if paths.is_empty() {
        index_scan(config, index_path)
    } else {
        index_paths(paths, index_path)
    }
}

fn index_paths(paths: &[String], index_path: &Path) -> GxResult<()> {
    let mut idx = ProjectIndex::load(index_path);

    for raw_path in paths {
        let abs = lexical_resolve(raw_path);
        let abs_str = abs.to_string_lossy().into_owned();

        // Verify this is a real git worktree.
        let inside = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(&abs)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let ok = matches!(inside, Ok(s) if s.success());
        if !ok {
            return Err(GxError::Other(format!("{abs_str} is not a git repository")));
        }

        // Reject a symlinked .git marker for safety/consistency.
        match std::fs::symlink_metadata(abs.join(".git")) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(GxError::Other(format!(
                    "{abs_str} has a symlinked .git directory, which is not supported for indexing"
                )));
            }
            Ok(_) => {}
            Err(_) => {
                return Err(GxError::Other(format!("{abs_str} is not a git repository")));
            }
        }

        let name = abs
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let url = ProjectIndex::get_remote_url(&abs);
        let is_new = idx.merge(
            &name,
            IndexEntry {
                path: abs_str.clone(),
                url,
                cloned_at: iso_now(),
                last_visited: None,
            },
        );

        if is_new {
            eprintln!("Indexed {name} ({abs_str})");
        } else {
            eprintln!("Already indexed: {name} ({abs_str})");
        }
    }

    idx.save(index_path)
}

fn index_scan(config: &Config, index_path: &Path) -> GxResult<()> {
    let project_dir = effective_project_dir(config)?;
    let mut idx = ProjectIndex::load(index_path);
    let before = idx.list().len();
    idx.additive_scan(&project_dir)?;
    idx.save(index_path)?;
    let after = idx.list().len();
    let added = after - before;
    let plural = if added != 1 { "s" } else { "" };
    eprintln!("Found {added} new project{plural} ({after} total)");
    Ok(())
}
