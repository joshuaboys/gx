//! `gx ls` — list indexed projects, name-padded. Ported from
//! `src/commands/ls.ts`.

use std::path::Path;

use crate::index_store::ProjectIndex;

pub fn ls(index_path: &Path) {
    let idx = ProjectIndex::load(index_path);
    let entries = idx.list();
    if entries.is_empty() {
        eprintln!("No projects indexed. Clone a repo or run 'gx rebuild'.");
        return;
    }
    let max_name = entries
        .iter()
        .map(|(name, _)| name.chars().count())
        .max()
        .unwrap_or(0);
    for (name, entry) in &entries {
        println!("{name:<max_name$}  {}", entry.path);
    }
}
