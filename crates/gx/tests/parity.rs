//! Parity harness: spawns whichever `gx` binary `GX_BIN` (or the default
//! workspace-root `./gx`) points at, against a hermetic fixture HOME, and
//! snapshots stdout / stderr / exit-code / mutated index state.
//!
//! Today the only binary the harness runs against is the TS build of `gx`
//! (`bun run build`). As Rust ports of commands land in RST-5/6, CI re-runs
//! the same suite against `target/release/gx` and must reproduce every
//! snapshot.

use std::path::{Path, PathBuf};
use std::process::Command;

use insta::{assert_snapshot, with_settings};
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at crates/gx; go up two levels.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn gx_bin() -> PathBuf {
    if let Ok(p) = std::env::var("GX_BIN") {
        return PathBuf::from(p);
    }
    let candidate = workspace_root().join("gx");
    assert!(
        candidate.exists(),
        "TS gx binary not found at {}. Run `bun run build` at the workspace root, or set GX_BIN.",
        candidate.display()
    );
    candidate
}

fn fixture_home() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/home")
}

struct Env {
    home: PathBuf,
    // Held to keep the tempdir alive for the duration of the test.
    _tmp: TempDir,
}

impl Env {
    fn new() -> Self {
        let tmp = TempDir::new().expect("tempdir");
        let home = tmp.path().to_path_buf();
        // Copy the fixture's ~/.config/gx/ into the temp HOME.
        let src = fixture_home().join(".config/gx");
        let dst = home.join(".config/gx");
        std::fs::create_dir_all(&dst).expect("mkdir config");
        for entry in std::fs::read_dir(&src).expect("read fixture") {
            let entry = entry.expect("dirent");
            std::fs::copy(entry.path(), dst.join(entry.file_name())).expect("copy fixture");
        }
        Self { home, _tmp: tmp }
    }

    fn index_json(&self) -> String {
        std::fs::read_to_string(self.home.join(".config/gx/index.json")).unwrap_or_default()
    }

    fn config_json(&self) -> String {
        std::fs::read_to_string(self.home.join(".config/gx/config.json")).unwrap_or_default()
    }
}

struct Run {
    stdout: String,
    stderr: String,
    code: i32,
}

fn run(env: &Env, args: &[&str]) -> Run {
    // PATH excludes anything containing `gx`, so `shell-init`'s `which gx`
    // lookup falls back deterministically to the bare string "gx".
    let path = "/usr/bin:/bin";

    let out = Command::new(gx_bin())
        .args(args)
        .env_remove("GX_AGENT")
        .env_remove("GX_SHELL_OVERRIDE")
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .env("HOME", &env.home)
        .env("PATH", path)
        // SHELL is read by `shell-init` when no arg is given; fix it so a
        // bare `shell-init` invocation is deterministic.
        .env("SHELL", "/bin/zsh")
        .output()
        .expect("spawn gx");

    Run {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

fn timestamp_filters() -> Vec<(&'static str, &'static str)> {
    vec![
        // ISO 8601 timestamps inside JSON values.
        (
            r#""(lastVisited|clonedAt)": "[^"]+""#,
            r#""$1": "<TIMESTAMP>""#,
        ),
        // Relative-time strings emitted by `recent`.
        (
            r"\d+ (minute|hour|day|week|month)s? ago|just now",
            "<RELATIVE>",
        ),
    ]
}

fn assert_run(name: &str, run: &Run) {
    with_settings!({
        filters => timestamp_filters(),
    }, {
        assert_snapshot!(format!("{name}.stdout"), run.stdout);
        assert_snapshot!(format!("{name}.stderr"), run.stderr);
        assert_snapshot!(format!("{name}.exit"), run.code.to_string());
    });
}

fn assert_index(name: &str, env: &Env) {
    with_settings!({
        filters => timestamp_filters(),
    }, {
        assert_snapshot!(format!("{name}.index.json"), env.index_json());
    });
}

// --- read-only / informational ---

#[test]
fn help_flag() {
    let env = Env::new();
    let r = run(&env, &["--help"]);
    assert_run("help_flag", &r);
}

#[test]
fn version_flag() {
    let env = Env::new();
    let r = run(&env, &["--version"]);
    assert_run("version_flag", &r);
}

#[test]
fn config_show() {
    let env = Env::new();
    let r = run(&env, &["config"]);
    assert_run("config_show", &r);
}

#[test]
fn ls() {
    let env = Env::new();
    let r = run(&env, &["ls"]);
    assert_run("ls", &r);
}

#[test]
fn recent_unlimited() {
    let env = Env::new();
    let r = run(&env, &["recent"]);
    assert_run("recent_unlimited", &r);
}

#[test]
fn recent_limit_2() {
    let env = Env::new();
    let r = run(&env, &["recent", "-n", "2"]);
    assert_run("recent_limit_2", &r);
}

// --- resolve (mutates lastVisited as a side effect) ---

#[test]
fn resolve_exact() {
    let env = Env::new();
    let r = run(&env, &["resolve", "alpha"]);
    assert_run("resolve_exact", &r);
    assert_index("resolve_exact", &env);
}

#[test]
fn resolve_fuzzy_auto() {
    // "alphaa" is a near-prefix match for "alpha" (Jaro-Winkler ≥ 0.85) so
    // resolve should auto-jump and write `Fuzzy match: …` to stderr.
    let env = Env::new();
    let r = run(&env, &["resolve", "alphaa"]);
    assert_run("resolve_fuzzy_auto", &r);
}

#[test]
fn resolve_missing() {
    let env = Env::new();
    let r = run(&env, &["resolve", "zzz-no-match"]);
    assert_run("resolve_missing", &r);
}

#[test]
fn resolve_list() {
    let env = Env::new();
    let r = run(&env, &["resolve", "--list"]);
    assert_run("resolve_list", &r);
}

// --- shell-init ---

#[test]
fn shell_init_zsh() {
    let env = Env::new();
    let r = run(&env, &["shell-init", "zsh"]);
    assert_run("shell_init_zsh", &r);
}

#[test]
fn shell_init_bash() {
    let env = Env::new();
    let r = run(&env, &["shell-init", "bash"]);
    assert_run("shell_init_bash", &r);
}

#[test]
fn shell_init_fish() {
    let env = Env::new();
    let r = run(&env, &["shell-init", "fish"]);
    assert_run("shell_init_fish", &r);
}

// --- config set (mutates config.json) ---

#[test]
fn config_set_editor() {
    let env = Env::new();
    let r = run(&env, &["config", "set", "editor", "code"]);
    assert_run("config_set_editor", &r);
    with_settings!({
        filters => timestamp_filters(),
    }, {
        assert_snapshot!("config_set_editor.config.json", env.config_json());
    });
}
