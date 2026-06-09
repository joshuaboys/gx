//! `gx resolve <name>` / `gx resolve --list` — resolve a project name to its
//! path (writing the path to stdout, hints to stderr) or list all names.
//! Ported from `src/commands/resolve.ts`.

use std::path::Path;

use crate::errors::{GxError, GxResult};
use crate::index_store::ProjectIndex;
use crate::resolve_name::{
    format_ambiguous, format_auto_match, resolve_project_name, ResolveResult,
};
use crate::types::Config;

pub fn resolve(name: &str, index_path: &Path, config: &Config, list_all: bool) -> GxResult<()> {
    let mut idx = ProjectIndex::load(index_path);

    if list_all {
        println!("{}", idx.names().join("\n"));
        return Ok(());
    }

    match resolve_project_name(name, &idx, config) {
        ResolveResult::Exact { name, path } => {
            idx.touch(&name);
            idx.save(index_path)?;
            println!("{path}");
            Ok(())
        }
        ResolveResult::Auto {
            name,
            path,
            query,
            score,
        } => {
            eprintln!("{}", format_auto_match(&query, &name, score));
            idx.touch(&name);
            idx.save(index_path)?;
            println!("{path}");
            Ok(())
        }
        ResolveResult::Ambiguous { query, matches } => {
            Err(GxError::command(format_ambiguous(&query, &matches)))
        }
        ResolveResult::Missing { .. } => {
            Err(GxError::command(format!("Project '{name}' not found")))
        }
    }
}
