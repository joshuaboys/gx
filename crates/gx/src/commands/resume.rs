//! `gx resume <name>` — resolve a project, print git context to stderr, and
//! write the path to stdout. Ported from `src/commands/resume.ts`.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::errors::{GxError, GxResult};
use crate::index_store::ProjectIndex;
use crate::resolve_name::{
    format_ambiguous, format_auto_match, resolve_project_name, ResolveResult,
};
use crate::types::Config;

pub struct ResumeContext {
    pub branch: String,
    pub dirty_count: usize,
    pub last_commit: String,
}

fn run_git(cwd: &Path, args: &[&str]) -> String {
    match Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stderr(Stdio::null())
        .output()
    {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => String::new(),
    }
}

pub fn get_resume_context(dir: &Path) -> Option<ResumeContext> {
    match std::fs::metadata(dir) {
        Ok(meta) if meta.is_dir() => {}
        _ => return None,
    }

    let branch = run_git(dir, &["branch", "--show-current"]);
    let status_output = run_git(dir, &["status", "--porcelain"]);
    let dirty_count = if status_output.is_empty() {
        0
    } else {
        status_output.lines().filter(|l| !l.is_empty()).count()
    };
    let last_commit = run_git(dir, &["log", "-1", "--format=%h %s (%cr)"]);

    Some(ResumeContext {
        branch: if branch.is_empty() {
            "HEAD (detached)".to_string()
        } else {
            branch
        },
        dirty_count,
        last_commit,
    })
}

pub fn resume(name: &str, index_path: &Path, config: &Config) -> GxResult<()> {
    let mut idx = ProjectIndex::load(index_path);

    let (resolved_name, resolved_path) = match resolve_project_name(name, &idx, config) {
        ResolveResult::Exact { name, path } => (name, path),
        ResolveResult::Auto {
            name,
            path,
            query,
            score,
        } => {
            eprintln!("{}", format_auto_match(&query, &name, score));
            (name, path)
        }
        ResolveResult::Ambiguous { query, matches } => {
            return Err(GxError::command(format_ambiguous(&query, &matches)));
        }
        ResolveResult::Missing { .. } => {
            return Err(GxError::command(format!(
                "Project '{name}' not found in index"
            )));
        }
    };

    let ctx = get_resume_context(Path::new(&resolved_path)).ok_or_else(|| {
        GxError::command(format!(
            "Project '{resolved_name}' directory not found: {resolved_path}"
        ))
    })?;

    idx.touch(&resolved_name);
    idx.save(index_path)?;

    let dirty = if ctx.dirty_count > 0 {
        let noun = if ctx.dirty_count == 1 {
            "file"
        } else {
            "files"
        };
        format!(" — {} dirty {noun}", ctx.dirty_count)
    } else {
        String::new()
    };
    eprintln!("{resolved_name} ({}){dirty}", ctx.branch);
    if !ctx.last_commit.is_empty() {
        eprintln!("  Last commit: {}", ctx.last_commit);
    }

    println!("{resolved_path}");
    Ok(())
}
