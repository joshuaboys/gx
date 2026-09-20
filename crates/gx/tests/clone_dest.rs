//! End-to-end coverage for `gx clone <repo> [dest]` (CLN-002).
//!
//! Real `git clone` runs against a local bare repo — no network. The isolated
//! `HOME` carries a `.gitconfig` that rewrites `https://github.com/` to a
//! `file://` path via `insteadOf`, so the binary exercises its normal URL
//! construction while git reads from disk.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Compare gx stdout paths with PathBufs across Unix and Windows.
/// Strips the Windows `\\?\` verbatim prefix and normalises separators so
/// JSON-stored forward-slash projectDir joins match native PathBufs.
fn path_key(p: impl AsRef<Path>) -> String {
    let raw = p.as_ref().to_string_lossy();
    let stripped = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
    if cfg!(windows) {
        stripped.replace('/', r"\")
    } else {
        stripped.replace('\\', "/")
    }
}

fn assert_path_eq(actual: &str, expected: &Path) {
    assert_eq!(
        path_key(actual),
        path_key(expected),
        "actual={actual} expected={}",
        expected.display()
    );
}

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_gx"))
}

fn git(args: &[&str], cwd: &Path) {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .expect("spawn git");
    assert!(status.success(), "git {args:?} failed in {}", cwd.display());
}

struct Env {
    home: TempDir,
    work: PathBuf,
    project_dir: PathBuf,
}

impl Env {
    /// Build an isolated HOME, a seeded bare origin for `juev/gclone`, and a
    /// working directory to run relative-path clones from.
    fn new() -> Self {
        let home = TempDir::new().expect("temp HOME");
        let root = home.path();
        fs::create_dir_all(root.join(".config/gx")).expect("mkdir .config/gx");

        let origin = root.join("origin/juev");
        fs::create_dir_all(&origin).expect("mkdir origin");
        let bare = origin.join("gclone.git");
        git(&["init", "-q", "--bare", bare.to_str().unwrap()], root);

        // Seed the bare repo with one commit so clone produces a worktree.
        let seed = root.join("seed");
        fs::create_dir_all(&seed).expect("mkdir seed");
        git(&["init", "-q", "."], &seed);
        git(
            &[
                "-c",
                "user.email=test@example.com",
                "-c",
                "user.name=test",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "init",
            ],
            &seed,
        );
        git(
            &["push", "-q", bare.to_str().unwrap(), "HEAD:refs/heads/main"],
            &seed,
        );

        // Rewrite the https remote to the local bare repo. Normalise separators
        // so Git on Windows accepts the file:// URL.
        let origin_url = root.join("origin").display().to_string().replace('\\', "/");
        fs::write(
            root.join(".gitconfig"),
            format!("[url \"file://{origin_url}/\"]\n\tinsteadOf = https://github.com/\n"),
        )
        .expect("write .gitconfig");

        let project_dir = root.join("projects");
        // Forward slashes keep this valid JSON on Windows (unescaped `\` is
        // an invalid JSON escape). PathBuf accepts both separators.
        let project_dir_json = project_dir.display().to_string().replace('\\', "/");
        fs::write(
            root.join(".config/gx/config.json"),
            format!(
                "{{\n  \"projectDir\": \"{project_dir_json}\",\n  \"structure\": \"owner\"\n}}\n"
            ),
        )
        .expect("write config.json");

        let work = root.join("work");
        fs::create_dir_all(&work).expect("mkdir work");
        // Relative destinations resolve against the cwd as the OS reports it,
        // which is fully resolved (on macOS the TempDir root `/var/...` is a
        // symlink to `/private/var/...`). Canonicalize so expectations match
        // what the binary prints. No-op on Linux.
        let work = fs::canonicalize(&work).expect("canonicalize work");
        let work = {
            let s = work.to_string_lossy();
            match s.strip_prefix(r"\\?\") {
                Some(rest) => PathBuf::from(rest),
                None => work,
            }
        };

        Env {
            home,
            work,
            project_dir,
        }
    }

    fn clone(&self, args: &[&str]) -> (bool, String, String) {
        let out = Command::new(binary_path())
            .arg("clone")
            .args(args)
            .current_dir(&self.work)
            .env("HOME", self.home.path())
            .env("USERPROFILE", self.home.path())
            .env_remove("HOMEDRIVE")
            .env_remove("HOMEPATH")
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("GX_AGENT")
            .output()
            .expect("spawn gx");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    fn index(&self) -> String {
        fs::read_to_string(self.home.path().join(".config/gx/index.json")).expect("read index.json")
    }
}

#[test]
fn no_dest_uses_configured_layout() {
    let env = Env::new();
    let (ok, stdout, stderr) = env.clone(&["juev/gclone"]);
    assert!(ok, "clone failed: {stderr}");

    let expected = env.project_dir.join("juev").join("gclone");
    assert_path_eq(&stdout, &expected);
    assert!(expected.join(".git").is_dir(), "no .git at {expected:?}");

    // Indexed under the repo name, as before.
    let index = env.index();
    assert!(index.contains("\"gclone\""), "index: {index}");
}

#[test]
fn relative_dest_lands_beside_cwd_and_indexes_by_basename() {
    let env = Env::new();
    let (ok, stdout, stderr) = env.clone(&["juev/gclone", "put-it-here"]);
    assert!(ok, "clone failed: {stderr}");

    let expected = env.work.join("put-it-here");
    assert_path_eq(&stdout, &expected);
    assert!(expected.join(".git").is_dir(), "no .git at {expected:?}");

    // The configured layout must be untouched by an override.
    assert!(
        !env.project_dir.join("juev").join("gclone").exists(),
        "override still wrote into projectDir"
    );

    // Reachable by the name the user chose.
    let index = env.index();
    assert!(index.contains("\"put-it-here\""), "index: {index}");
}

#[test]
fn absolute_dest_is_honoured() {
    let env = Env::new();
    let dest = env.home.path().join("elsewhere").join("deep").join("repo");
    let (ok, stdout, stderr) = env.clone(&["juev/gclone", dest.to_str().unwrap()]);
    assert!(ok, "clone failed: {stderr}");

    assert_path_eq(&stdout, &dest);
    // Missing parents are created, same as the default path.
    assert!(dest.join(".git").is_dir(), "no .git at {dest:?}");
    assert!(env.index().contains("\"repo\""));
}

#[test]
fn dest_does_not_write_config() {
    let env = Env::new();
    let config_path = env.home.path().join(".config/gx/config.json");
    let before = fs::read_to_string(&config_path).expect("read config");

    let (ok, _, stderr) = env.clone(&["juev/gclone", "one-off"]);
    assert!(ok, "clone failed: {stderr}");

    let after = fs::read_to_string(&config_path).expect("read config");
    assert_eq!(before, after, "clone with dest mutated config.json");
}

#[test]
fn existing_clone_at_dest_is_skipped() {
    let env = Env::new();
    let (ok, _, stderr) = env.clone(&["juev/gclone", "twice"]);
    assert!(ok, "first clone failed: {stderr}");

    let (ok, stdout, stderr) = env.clone(&["juev/gclone", "twice"]);
    assert!(ok, "second clone failed: {stderr}");
    assert_path_eq(&stdout, &env.work.join("twice"));
    assert!(
        stderr.contains("already exists"),
        "expected skip notice, got: {stderr}"
    );
}
