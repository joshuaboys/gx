//! Behavior-parity snapshot harness.
//!
//! Each test runs the binary under test in an isolated `HOME` populated from
//! `tests/fixtures/`, captures stdout/stderr/exit, and asserts byte-for-byte
//! against a golden snapshot under `tests/snapshots/`.
//!
//! By default the binary under test is the Rust crate's `gx`
//! (`CARGO_BIN_EXE_gx`). Set `GX_SNAPSHOT_BIN` to point at any other build —
//! typically the TypeScript binary produced by `bun run build` — to lock in
//! goldens against it:
//!
//! ```sh
//! bun run build
//! GX_SNAPSHOT_BIN="$PWD/gx" cargo test -p gx --test snapshots -- --ignored
//! ```
//!
//! Tests are `#[ignore]` by default so `cargo test` stays green while the
//! Rust port is incomplete. Once a command's Rust implementation lands, the
//! corresponding `#[ignore]` is removed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

fn binary_path() -> PathBuf {
    if let Ok(p) = std::env::var("GX_SNAPSHOT_BIN") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_BIN_EXE_gx"))
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
}

struct Harness {
    home: TempDir,
}

impl Harness {
    fn new() -> Self {
        let home = TempDir::new().expect("create temp HOME");
        fs::create_dir_all(home.path().join(".config/gx")).expect("mkdir .config/gx");
        Self { home }
    }

    fn with_fixture_index(self) -> Self {
        self.copy_fixture("index.json")
    }

    fn with_fixture_config(self) -> Self {
        self.copy_fixture("config.json")
    }

    fn copy_fixture(self, name: &str) -> Self {
        let src = fixtures_dir().join(name);
        let dst = self.home.path().join(".config/gx").join(name);
        fs::copy(&src, &dst).unwrap_or_else(|e| panic!("copy fixture {name}: {e}"));
        self
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new(binary_path());
        // Inherit PATH (so spawned `git` etc resolve) and TMPDIR, but pin every
        // gx-relevant env to a known value.
        cmd.env("HOME", self.home.path())
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("GX_AGENT")
            .env_remove("GX_SHELL_OVERRIDE")
            .env("SHELL", "/bin/zsh");
        cmd
    }
}

struct Captured {
    code: i32,
    stdout: String,
    stderr: String,
}

impl From<Output> for Captured {
    fn from(out: Output) -> Self {
        Captured {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }
}

fn run(mut cmd: Command) -> Captured {
    cmd.output().expect("spawn gx").into()
}

fn format_capture(cap: &Captured) -> String {
    format!(
        "exit: {code}\n---- stdout ----\n{stdout}\n---- stderr ----\n{stderr}",
        code = cap.code,
        stdout = cap.stdout,
        stderr = cap.stderr
    )
}

/// Wrap a snapshot assertion with shared insta settings: snapshots live next
/// to the test fixtures, no inline expressions, and `_GX_BIN`/relative-time
/// noise is scrubbed before comparison.
fn assert_snapshot(name: &str, value: &str) {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(snapshot_dir());
    settings.set_prepend_module_to_snapshot(false);
    settings.set_omit_expression(true);
    // Resolved binary path leaks into `gx shell-init` as `_GX_BIN="..."`.
    settings.add_filter(r#"_GX_BIN="[^"]*""#, r#"_GX_BIN="<BIN>""#);
    settings.add_filter(r#"set -g _GX_BIN "[^"]*""#, r#"set -g _GX_BIN "<BIN>""#);
    // `gx recent` prints relative times against Date.now(); normalize.
    settings.add_filter(r"\d+ (minute|hour|day|week|month)s? ago", "<TIME_AGO>");
    settings.add_filter(r"just now", "<TIME_AGO>");
    settings.bind(|| insta::assert_snapshot!(name, value));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_help() {
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.arg("--help");
        c
    });
    assert_snapshot("help", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_version() {
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.arg("--version");
        c
    });
    assert_snapshot("version", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_shell_init_zsh() {
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.args(["shell-init", "zsh"]);
        c
    });
    assert_snapshot("shell_init_zsh", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_shell_init_bash() {
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.args(["shell-init", "bash"]);
        c
    });
    assert_snapshot("shell_init_bash", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_shell_init_fish() {
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.args(["shell-init", "fish"]);
        c
    });
    assert_snapshot("shell_init_fish", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_shell_init_unsupported() {
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.args(["shell-init", "powershell"]);
        c
    });
    assert_snapshot("shell_init_unsupported", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_ls() {
    let h = Harness::new().with_fixture_index();
    let cap = run({
        let mut c = h.command();
        c.arg("ls");
        c
    });
    assert_snapshot("ls", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_ls_empty() {
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.arg("ls");
        c
    });
    assert_snapshot("ls_empty", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_resolve_list() {
    let h = Harness::new().with_fixture_index();
    let cap = run({
        let mut c = h.command();
        c.args(["resolve", "--list"]);
        c
    });
    assert_snapshot("resolve_list", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_resolve_missing() {
    let h = Harness::new().with_fixture_index();
    let cap = run({
        let mut c = h.command();
        c.args(["resolve", "nonexistent-xyzzy"]);
        c
    });
    assert_snapshot("resolve_missing", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_recent() {
    let h = Harness::new().with_fixture_index();
    let cap = run({
        let mut c = h.command();
        c.arg("recent");
        c
    });
    assert_snapshot("recent", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_recent_limit() {
    let h = Harness::new().with_fixture_index();
    let cap = run({
        let mut c = h.command();
        c.args(["recent", "-n", "2"]);
        c
    });
    assert_snapshot("recent_limit", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_config_show() {
    let h = Harness::new().with_fixture_config();
    let cap = run({
        let mut c = h.command();
        c.arg("config");
        c
    });
    assert_snapshot("config_show", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_config_show_defaults() {
    // No fixture config — should fall through to DEFAULT_CONFIG.
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.arg("config");
        c
    });
    assert_snapshot("config_show_defaults", &format_capture(&cap));
}

#[test]
#[ignore = "RST-2 snapshot harness — run with --ignored against a built binary"]
fn snapshot_unknown_arg_resolves_as_name() {
    // `gx <unknown>` falls through to resolve; with empty index, that's an error.
    let h = Harness::new();
    let cap = run({
        let mut c = h.command();
        c.arg("zzz-not-a-project");
        c
    });
    assert_snapshot("unknown_arg_resolves_as_name", &format_capture(&cap));
}
