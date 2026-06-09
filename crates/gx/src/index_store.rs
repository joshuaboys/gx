use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::atomic_write;
use crate::errors::{GxError, GxResult};
use crate::types::{Index, IndexEntry};

const MAX_SCAN_DEPTH: usize = 10;

fn skip_dirs() -> &'static [&'static str] {
    &[
        "node_modules",
        "vendor",
        "target",
        ".build",
        "dist",
        "build",
    ]
}

/// In-memory view of `~/.config/gx/index.json` with the same surface as the
/// TS `ProjectIndex` class. Insertion order is preserved on the underlying
/// map so a load → save round-trip is byte-equal.
#[derive(Debug, Clone, Default)]
pub struct ProjectIndex {
    data: Index,
}

impl ProjectIndex {
    /// Load the index from `path`. Missing file, unreadable file, or invalid
    /// JSON all yield an empty index (matching the TS fall-back).
    pub fn load(path: &Path) -> Self {
        let Ok(raw) = fs::read_to_string(path) else {
            return Self::default();
        };
        match serde_json::from_str::<Index>(&raw) {
            Ok(data) => ProjectIndex { data },
            Err(_) => Self::default(),
        }
    }

    /// Insert `entry` under `name`, overwriting any existing entry (with a
    /// stderr warning on path collisions, matching TS).
    pub fn add(&mut self, name: &str, entry: IndexEntry) {
        if let Some(existing) = self.data.projects.get(name) {
            if existing.path != entry.path {
                eprintln!(
                    "Warning: project name '{name}' collision — overwriting {old} with {new}",
                    old = existing.path,
                    new = entry.path,
                );
            }
        }
        self.data.projects.insert(name.to_string(), entry);
    }

    /// Merge `entry`. Returns `true` if the index changed (new name or
    /// different path), `false` if the entry was already tracked at the
    /// same path.
    pub fn merge(&mut self, name: &str, entry: IndexEntry) -> bool {
        if let Some(existing) = self.data.projects.get(name) {
            if existing.path == entry.path {
                return false;
            }
            eprintln!(
                "Warning: project name '{name}' collision — overwriting {old} with {new}",
                old = existing.path,
                new = entry.path,
            );
        }
        self.data.projects.insert(name.to_string(), entry);
        true
    }

    pub fn resolve(&self, name: &str) -> Option<&str> {
        self.data.projects.get(name).map(|e| e.path.as_str())
    }

    /// Stamp `lastVisited` to the current UTC time. Returns `true` when the
    /// project exists.
    pub fn touch(&mut self, name: &str) -> bool {
        match self.data.projects.get_mut(name) {
            Some(entry) => {
                entry.last_visited = Some(iso_now());
                true
            }
            None => false,
        }
    }

    /// Entries sorted by `lastVisited || clonedAt` descending. Equivalent
    /// to the TS `recent(limit?)` method.
    pub fn recent(&self, limit: Option<usize>) -> Vec<(String, IndexEntry)> {
        let mut entries: Vec<(String, IndexEntry)> = self
            .data
            .projects
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort_by(|a, b| {
            let ta =
                a.1.last_visited
                    .as_deref()
                    .unwrap_or(a.1.cloned_at.as_str());
            let tb =
                b.1.last_visited
                    .as_deref()
                    .unwrap_or(b.1.cloned_at.as_str());
            tb.cmp(ta)
        });
        if let Some(n) = limit {
            entries.truncate(n);
        }
        entries
    }

    /// Entries sorted by name ascending.
    pub fn list(&self) -> Vec<(String, IndexEntry)> {
        let mut entries: Vec<(String, IndexEntry)> = self
            .data
            .projects
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.data.projects.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn is_empty(&self) -> bool {
        self.data.projects.is_empty()
    }

    /// Drop every project and scan `project_dir` for git repos.
    pub fn rebuild(&mut self, project_dir: &Path) -> GxResult<()> {
        self.data.projects.clear();
        let mut visited = HashSet::new();
        scan_for_repos(self, project_dir, 0, &mut visited);
        Ok(())
    }

    /// Drop entries whose path lives inside `scope_dir` (skipping the
    /// dotdir agent-scope first segment) and rescan that scope.
    pub fn scoped_rebuild(&mut self, scope_dir: &Path) -> GxResult<()> {
        let prefix = scope_dir.to_string_lossy().into_owned();
        let prefix = if prefix.ends_with('/') {
            prefix
        } else {
            format!("{prefix}/")
        };
        let to_delete: Vec<String> = self
            .data
            .projects
            .iter()
            .filter_map(|(name, entry)| {
                if !entry.path.starts_with(&prefix) {
                    return None;
                }
                let rel = &entry.path[prefix.len()..];
                let first = rel.split('/').next().unwrap_or("");
                if !first.is_empty() && first.starts_with('.') {
                    return None;
                }
                Some(name.clone())
            })
            .collect();
        for name in to_delete {
            self.data.projects.shift_remove(&name);
        }
        let mut visited = HashSet::new();
        scan_for_repos(self, scope_dir, 0, &mut visited);
        Ok(())
    }

    /// Scan `project_dir` and add any newly discovered repos without
    /// removing existing entries.
    pub fn additive_scan(&mut self, project_dir: &Path) -> GxResult<()> {
        let mut visited = HashSet::new();
        scan_for_repos(self, project_dir, 0, &mut visited);
        Ok(())
    }

    /// Read `remote.origin.url` from a local repository. Returns an empty
    /// string if the directory is not a git repo or has no origin remote.
    pub fn get_remote_url(repo_path: &Path) -> String {
        let out = Command::new("git")
            .args(["config", "--get", "remote.origin.url"])
            .current_dir(repo_path)
            .output();
        match out {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            _ => String::new(),
        }
    }

    /// Atomic write: serialize to a temp file in the same directory and
    /// rename into place. JSON uses 2-space indent + trailing newline,
    /// preserving the TS `JSON.stringify(.., null, 2) + "\n"` output.
    pub fn save(&self, path: &Path) -> GxResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| GxError::Other(format!("create {}: {e}", parent.display())))?;
        }
        let mut text = serde_json::to_string_pretty(&self.data)
            .map_err(|e| GxError::Other(format!("serialise index: {e}")))?;
        text.push('\n');
        atomic_write(path, text.as_bytes())
    }
}

fn scan_for_repos(
    idx: &mut ProjectIndex,
    dir: &Path,
    depth: usize,
    visited: &mut HashSet<PathBuf>,
) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let Ok(real) = fs::canonicalize(dir) else {
        return;
    };
    if !visited.insert(real) {
        return;
    }

    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    let entries: Vec<_> = read.flatten().collect();

    // First pass: detect a .git marker (directory OR file). If present,
    // capture this repo and stop — do not descend.
    for entry in &entries {
        let Ok(ft) = entry.file_type() else { continue };
        if entry.file_name() != ".git" {
            continue;
        }
        if !(ft.is_dir() || ft.is_file()) {
            continue;
        }
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let path_s = dir.to_string_lossy().into_owned();
        let url = match idx.data.projects.get(&name) {
            Some(existing) if existing.path == path_s && !existing.url.is_empty() => {
                existing.url.clone()
            }
            _ => ProjectIndex::get_remote_url(dir),
        };
        idx.merge(
            &name,
            IndexEntry {
                path: path_s,
                url,
                cloned_at: String::new(),
                last_visited: None,
            },
        );
        return;
    }

    // Second pass: recurse into subdirectories that are visible and not in
    // the skip list.
    for entry in &entries {
        let Ok(ft) = entry.file_type() else { continue };
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if name_s.starts_with('.') {
            continue;
        }
        if skip_dirs().contains(&name_s.as_ref()) {
            continue;
        }
        scan_for_repos(idx, &dir.join(&name), depth + 1, visited);
    }
}

/// `new Date().toISOString()` parity: `YYYY-MM-DDTHH:MM:SS.sssZ`. Public so
/// commands that stamp `clonedAt`/`lastVisited` (clone, index) share it.
pub fn iso_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let millis = now.subsec_millis();
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (y, mo, d) = civil_from_days(days);
    let h = sod / 3600;
    let mi = (sod % 3600) / 60;
    let s = sod % 60;
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{millis:03}Z")
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn entry(path: &str) -> IndexEntry {
        IndexEntry {
            path: path.into(),
            url: String::new(),
            cloned_at: String::new(),
            last_visited: None,
        }
    }

    #[test]
    fn load_missing_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let idx = ProjectIndex::load(&tmp.path().join("index.json"));
        assert!(idx.is_empty());
        assert!(idx.list().is_empty());
    }

    #[test]
    fn add_and_resolve() {
        let mut idx = ProjectIndex::default();
        idx.add(
            "gx",
            IndexEntry {
                path: "/home/user/src/joshuaboys/gx".into(),
                url: "https://github.com/joshuaboys/gx".into(),
                cloned_at: "2026-02-23T00:00:00Z".into(),
                last_visited: None,
            },
        );
        assert_eq!(idx.resolve("gx"), Some("/home/user/src/joshuaboys/gx"));
    }

    #[test]
    fn resolve_returns_none_for_unknown() {
        let idx = ProjectIndex::default();
        assert!(idx.resolve("nope").is_none());
    }

    #[test]
    fn save_then_load() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("index.json");
        let mut idx = ProjectIndex::default();
        idx.add(
            "gx",
            IndexEntry {
                path: "/tmp/gx".into(),
                url: "https://github.com/joshuaboys/gx".into(),
                cloned_at: "2026-02-23T00:00:00Z".into(),
                last_visited: None,
            },
        );
        idx.save(&path).unwrap();
        let idx2 = ProjectIndex::load(&path);
        assert_eq!(idx2.resolve("gx"), Some("/tmp/gx"));
    }

    #[test]
    fn list_returns_entries_sorted_by_name() {
        let mut idx = ProjectIndex::default();
        idx.add("bravo", entry("/tmp/bravo"));
        idx.add("alpha", entry("/tmp/alpha"));
        let names: Vec<String> = idx.list().into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["alpha".to_string(), "bravo".into()]);
    }

    #[test]
    fn names_contains_added() {
        let mut idx = ProjectIndex::default();
        idx.add("gx", entry("/tmp/gx"));
        idx.add("gclone", entry("/tmp/gclone"));
        let names = idx.names();
        assert!(names.contains(&"gx".to_string()));
        assert!(names.contains(&"gclone".to_string()));
    }

    #[test]
    fn merge_new_returns_true() {
        let mut idx = ProjectIndex::default();
        let is_new = idx.merge(
            "newrepo",
            IndexEntry {
                path: "/tmp/newrepo".into(),
                url: "https://github.com/user/newrepo".into(),
                cloned_at: "2026-03-02T00:00:00Z".into(),
                last_visited: None,
            },
        );
        assert!(is_new);
        assert_eq!(idx.resolve("newrepo"), Some("/tmp/newrepo"));
    }

    #[test]
    fn merge_same_name_same_path_returns_false() {
        let mut idx = ProjectIndex::default();
        idx.add("myrepo", entry("/tmp/myrepo"));
        let is_new = idx.merge(
            "myrepo",
            IndexEntry {
                path: "/tmp/myrepo".into(),
                url: "https://github.com/user/myrepo".into(),
                cloned_at: "2026-03-02T00:00:00Z".into(),
                last_visited: None,
            },
        );
        assert!(!is_new);
    }

    #[test]
    fn merge_same_name_diff_path_overwrites() {
        let mut idx = ProjectIndex::default();
        idx.add("myrepo", entry("/old/myrepo"));
        let is_new = idx.merge("myrepo", entry("/new/myrepo"));
        assert!(is_new);
        assert_eq!(idx.resolve("myrepo"), Some("/new/myrepo"));
    }

    #[test]
    fn touch_updates_last_visited() {
        let mut idx = ProjectIndex::default();
        idx.add("gx", entry("/tmp/gx"));
        assert!(idx.touch("gx"));
        let entries = idx.list();
        let (_, e) = entries.iter().find(|(n, _)| n == "gx").unwrap();
        let lv = e.last_visited.as_deref().expect("last_visited set");
        // YYYY-MM-DDTHH:MM:SS.sssZ
        assert_eq!(lv.len(), 24, "iso length {lv}");
        assert!(lv.ends_with('Z'));
    }

    #[test]
    fn touch_unknown_returns_false() {
        let mut idx = ProjectIndex::default();
        assert!(!idx.touch("nope"));
    }

    #[test]
    fn recent_sorted_by_last_visited_desc() {
        let mut idx = ProjectIndex::default();
        idx.add(
            "old",
            IndexEntry {
                path: "/tmp/old".into(),
                url: String::new(),
                cloned_at: "2026-01-01T00:00:00Z".into(),
                last_visited: Some("2026-01-01T00:00:00Z".into()),
            },
        );
        idx.add(
            "new",
            IndexEntry {
                path: "/tmp/new".into(),
                url: String::new(),
                cloned_at: "2026-01-02T00:00:00Z".into(),
                last_visited: Some("2026-03-01T00:00:00Z".into()),
            },
        );
        idx.add(
            "mid",
            IndexEntry {
                path: "/tmp/mid".into(),
                url: String::new(),
                cloned_at: "2026-01-03T00:00:00Z".into(),
                last_visited: Some("2026-02-01T00:00:00Z".into()),
            },
        );
        let order: Vec<String> = idx.recent(None).into_iter().map(|(n, _)| n).collect();
        assert_eq!(order, vec!["new", "mid", "old"]);
    }

    #[test]
    fn recent_falls_back_to_cloned_at() {
        let mut idx = ProjectIndex::default();
        idx.add(
            "visited",
            IndexEntry {
                path: "/tmp/visited".into(),
                url: String::new(),
                cloned_at: "2026-01-01T00:00:00Z".into(),
                last_visited: Some("2026-02-01T00:00:00Z".into()),
            },
        );
        idx.add(
            "unvisited",
            IndexEntry {
                path: "/tmp/unvisited".into(),
                url: String::new(),
                cloned_at: "2026-03-01T00:00:00Z".into(),
                last_visited: None,
            },
        );
        let order: Vec<String> = idx.recent(None).into_iter().map(|(n, _)| n).collect();
        assert_eq!(order, vec!["unvisited", "visited"]);
    }

    #[test]
    fn recent_respects_limit() {
        let mut idx = ProjectIndex::default();
        idx.add(
            "a",
            IndexEntry {
                path: "/tmp/a".into(),
                url: String::new(),
                cloned_at: String::new(),
                last_visited: Some("2026-03-03T00:00:00Z".into()),
            },
        );
        idx.add(
            "b",
            IndexEntry {
                path: "/tmp/b".into(),
                url: String::new(),
                cloned_at: String::new(),
                last_visited: Some("2026-03-02T00:00:00Z".into()),
            },
        );
        idx.add(
            "c",
            IndexEntry {
                path: "/tmp/c".into(),
                url: String::new(),
                cloned_at: String::new(),
                last_visited: Some("2026-03-01T00:00:00Z".into()),
            },
        );
        let order: Vec<String> = idx.recent(Some(2)).into_iter().map(|(n, _)| n).collect();
        assert_eq!(order, vec!["a", "b"]);
    }

    // ---- scan tests -------------------------------------------------------

    fn make_repo(root: &Path, rel: &str) -> PathBuf {
        let path = root.join(rel);
        fs::create_dir_all(path.join(".git")).unwrap();
        path
    }

    #[test]
    fn rebuild_discovers_repos() {
        let tmp = TempDir::new().unwrap();
        let a = make_repo(tmp.path(), "user/repoA");
        let b = make_repo(tmp.path(), "user/repoB");

        let mut idx = ProjectIndex::default();
        idx.rebuild(tmp.path()).unwrap();
        assert_eq!(idx.resolve("repoA").map(PathBuf::from), Some(a));
        assert_eq!(idx.resolve("repoB").map(PathBuf::from), Some(b));
    }

    #[test]
    fn additive_scan_preserves_existing() {
        let tmp = TempDir::new().unwrap();
        let mut idx = ProjectIndex::default();
        idx.add("external", entry("/external/repo"));

        let a = make_repo(tmp.path(), "org/repoA");
        let b = make_repo(tmp.path(), "org/repoB");

        idx.additive_scan(tmp.path()).unwrap();
        assert_eq!(idx.resolve("external"), Some("/external/repo"));
        assert_eq!(idx.resolve("repoA").map(PathBuf::from), Some(a));
        assert_eq!(idx.resolve("repoB").map(PathBuf::from), Some(b));
    }

    #[test]
    fn scoped_rebuild_replaces_in_scope() {
        let tmp = TempDir::new().unwrap();
        let a = make_repo(tmp.path(), "org/repoA");
        let b = make_repo(tmp.path(), "org/repoB");

        let mut idx = ProjectIndex::default();
        idx.add("external", entry("/other/dir/external"));
        idx.add("repoA", entry(a.to_string_lossy().as_ref()));

        idx.scoped_rebuild(tmp.path()).unwrap();

        assert_eq!(idx.resolve("external"), Some("/other/dir/external"));
        assert_eq!(idx.resolve("repoA").map(PathBuf::from), Some(a));
        assert_eq!(idx.resolve("repoB").map(PathBuf::from), Some(b));
    }

    #[test]
    fn scoped_rebuild_skips_dotdir_entries() {
        let tmp = TempDir::new().unwrap();
        // Agent-scope repo lives under a dotdir from tmpDir's perspective.
        let agent_repo = make_repo(tmp.path(), ".morgan/org/agentrepo");
        let user_repo = make_repo(tmp.path(), "org/userrepo");

        let mut idx = ProjectIndex::default();
        idx.add("agentrepo", entry(agent_repo.to_string_lossy().as_ref()));
        idx.add("userrepo", entry(user_repo.to_string_lossy().as_ref()));

        idx.scoped_rebuild(tmp.path()).unwrap();

        // Agent-scope entry preserved (dotdir guard); user-scope entry
        // rediscovered.
        assert_eq!(
            idx.resolve("agentrepo").map(PathBuf::from),
            Some(agent_repo)
        );
        assert_eq!(idx.resolve("userrepo").map(PathBuf::from), Some(user_repo));
    }

    #[test]
    fn scoped_rebuild_from_agent_scope() {
        let tmp = TempDir::new().unwrap();
        let agent_dir = tmp.path().join(".morgan");
        let agent_repo = make_repo(&agent_dir, "org/agentrepo");
        let user_repo = make_repo(tmp.path(), "org/userrepo");

        let mut idx = ProjectIndex::default();
        idx.add("agentrepo", entry(agent_repo.to_string_lossy().as_ref()));
        idx.add("userrepo", entry(user_repo.to_string_lossy().as_ref()));

        idx.scoped_rebuild(&agent_dir).unwrap();

        assert_eq!(idx.resolve("userrepo").map(PathBuf::from), Some(user_repo));
        assert_eq!(
            idx.resolve("agentrepo").map(PathBuf::from),
            Some(agent_repo)
        );
    }

    #[test]
    fn scan_skips_node_modules_and_friends() {
        let tmp = TempDir::new().unwrap();
        // A repo nested under a skip-listed directory should NOT be picked up.
        fs::create_dir_all(tmp.path().join("node_modules/pkg/.git")).unwrap();
        // ...but a sibling repo outside the skip should be discovered.
        let good = make_repo(tmp.path(), "user/good");

        let mut idx = ProjectIndex::default();
        idx.rebuild(tmp.path()).unwrap();
        assert!(idx.resolve("pkg").is_none(), "node_modules wasn't skipped");
        assert_eq!(idx.resolve("good").map(PathBuf::from), Some(good));
    }

    #[test]
    fn scan_handles_git_marker_as_file() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("user/worktree");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join(".git"), "gitdir: /elsewhere/.git\n").unwrap();
        let mut idx = ProjectIndex::default();
        idx.rebuild(tmp.path()).unwrap();
        assert_eq!(idx.resolve("worktree").map(PathBuf::from), Some(repo));
    }

    #[cfg(unix)]
    #[test]
    fn scan_breaks_symlink_cycles() {
        let tmp = TempDir::new().unwrap();
        // a/.git is a real repo
        let a = make_repo(tmp.path(), "user/a");
        // b is a symlink back into the scan root
        std::os::unix::fs::symlink(tmp.path(), tmp.path().join("loop")).unwrap();

        let mut idx = ProjectIndex::default();
        idx.rebuild(tmp.path()).unwrap();
        // The repo is still discovered exactly once, and the scan terminates.
        assert_eq!(idx.resolve("a").map(PathBuf::from), Some(a));
    }

    #[test]
    fn get_remote_url_non_repo_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(ProjectIndex::get_remote_url(tmp.path()), "");
    }

    #[test]
    fn get_remote_url_reads_origin() {
        // Requires `git` on PATH; CI provides it.
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("fixture");
        fs::create_dir_all(&repo).unwrap();
        let init = Command::new("git")
            .args(["init"])
            .current_dir(&repo)
            .output();
        if init.is_err() || !init.as_ref().unwrap().status.success() {
            eprintln!("skipping: git unavailable");
            return;
        }
        Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/joshuaboys/gx.git",
            ])
            .current_dir(&repo)
            .output()
            .unwrap();
        assert_eq!(
            ProjectIndex::get_remote_url(&repo),
            "https://github.com/joshuaboys/gx.git"
        );
    }

    // ---- byte-equal round-trip --------------------------------------------

    /// A canonical index.json shaped exactly the way the TS code emits it:
    /// 2-space indent, `lastVisited` omitted when missing, trailing newline,
    /// insertion-order keys.
    const CANONICAL_INDEX_JSON: &str = "{\n  \"projects\": {\n    \"alpha\": {\n      \"path\": \"/snapshot/projects/github.com/acme/alpha\",\n      \"url\": \"git@github.com:acme/alpha.git\",\n      \"clonedAt\": \"2025-01-01T00:00:00.000Z\",\n      \"lastVisited\": \"2025-04-15T12:00:00.000Z\"\n    },\n    \"beta\": {\n      \"path\": \"/snapshot/projects/github.com/acme/beta\",\n      \"url\": \"https://github.com/acme/beta.git\",\n      \"clonedAt\": \"2025-02-10T00:00:00.000Z\"\n    }\n  }\n}\n";

    #[test]
    fn round_trip_is_byte_equal() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("index.json");
        fs::write(&path, CANONICAL_INDEX_JSON).unwrap();
        let idx = ProjectIndex::load(&path);
        let out_path = tmp.path().join("out.json");
        idx.save(&out_path).unwrap();
        let written = fs::read_to_string(&out_path).unwrap();
        assert_eq!(written, CANONICAL_INDEX_JSON);
    }
}
