//! `gx recent [-n <count>]` — list recently visited projects with relative
//! timestamps. Ported from `src/commands/recent.ts`.

use std::path::Path;

use crate::index_store::ProjectIndex;
use crate::time::relative_time;

pub fn recent(index_path: &Path, limit: Option<usize>) {
    let idx = ProjectIndex::load(index_path);
    let entries = idx.recent(limit);
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
        // TS: relativeTime(entry.lastVisited || entry.clonedAt) — an empty
        // lastVisited is falsy and falls back to clonedAt.
        let stamp = match entry.last_visited.as_deref() {
            Some(s) if !s.is_empty() => s,
            _ => entry.cloned_at.as_str(),
        };
        let time = relative_time(stamp);
        let time_str = if time.is_empty() {
            String::new()
        } else {
            format!("  {time}")
        };
        println!("{name:<max_name$}  {}{time_str}", entry.path);
    }
}
