//! Execute generated shell integration; snapshots alone cannot verify shell semantics.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn isolated<'a>(command: &'a mut Command, home: &Path) -> &'a mut Command {
    command
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("GX_AGENT")
        .env_remove("GX_SHELL_OVERRIDE")
        .env_remove("BASH_ENV")
        .env_remove("ENV")
        .current_dir(home)
}

fn run_shell(shell: &str, home: &Path, body: &str) -> Output {
    let integration = isolated(
        Command::new(env!("CARGO_BIN_EXE_gx")).args(["shell-init", shell]),
        home,
    )
    .output()
    .expect("generate shell integration");
    assert!(integration.status.success());
    let script = home.join("test-shell");
    fs::write(
        &script,
        format!("{}\n{body}\n", String::from_utf8_lossy(&integration.stdout)),
    )
    .unwrap();
    isolated(Command::new(shell).arg(&script), home)
        .output()
        .expect("execute shell")
}

// Shells may preserve a logical path (macOS /var -> /private/var). Compare
// directory identity while still rejecting extra stdout or a missing directory.
fn assert_directory(actual: &str, expected: &Path) {
    assert_eq!(
        Path::new(actual.trim()).canonicalize().unwrap(),
        expected.canonicalize().unwrap()
    );
}

fn shells() -> Vec<&'static str> {
    ["bash", "zsh", "fish"]
        .into_iter()
        .filter(|shell| {
            let available = Command::new(shell).arg("--version").output().is_ok();
            if !available {
                assert!(std::env::var_os("CI").is_none(), "CI must install {shell}");
                eprintln!("Skipping unavailable shell: {shell}");
            }
            available
        })
        .collect()
}

#[test]
fn failed_clone_stops_chained_scaffolding() {
    for shell in shells() {
        let home = TempDir::new().unwrap();
        let out = run_shell(shell, home.path(), "gx clone && gx init");
        assert_eq!(out.status.code(), Some(1), "{shell}: {out:?}");
        assert!(
            !home.path().join(".claude").exists(),
            "{shell} scaffolded after failure"
        );
    }
}

#[test]
fn shell_help_prints_usage_without_navigation() {
    for shell in shells() {
        let home = TempDir::new().unwrap();
        for command in ["clone", "resume"] {
            let out = run_shell(shell, home.path(), &format!("gx {command} --help"));
            assert!(out.status.success(), "{shell} {command}: {out:?}");
            assert!(String::from_utf8_lossy(&out.stdout).contains(&format!("Usage: gx {command}")));
        }
    }
}

#[test]
fn existing_clone_enters_destination_with_spaces() {
    for shell in shells() {
        let home = TempDir::new().unwrap();
        let project = home.path().join("existing repo");
        fs::create_dir_all(project.join(".git")).unwrap();
        let out = run_shell(
            shell,
            home.path(),
            "gx clone user/repo 'existing repo' && pwd",
        );
        assert!(out.status.success(), "{shell}: {out:?}");
        assert_directory(&String::from_utf8_lossy(&out.stdout), &project);
    }
}

#[test]
fn failed_resume_stops_followup_commands() {
    for shell in shells() {
        let home = TempDir::new().unwrap();
        let out = run_shell(shell, home.path(), "gx resume missing && gx init");
        assert_eq!(out.status.code(), Some(1), "{shell}: {out:?}");
        assert!(!home.path().join(".claude").exists());
    }
}

#[test]
fn worktrunk_help_is_forwarded_after_navigation() {
    for shell in shells() {
        let home = TempDir::new().unwrap();
        let project = home.path().join("project");
        fs::create_dir(&project).unwrap();
        let config = home.path().join(".config/gx");
        fs::create_dir_all(&config).unwrap();
        fs::write(
            config.join("index.json"),
            serde_json::json!({
                "version": 1,
                "projects": { "project": {
                    "path": project, "url": "", "clonedAt": "2026-01-01T00:00:00Z"
                }}
            })
            .to_string(),
        )
        .unwrap();
        let wt = home.path().join("wt");
        fs::write(&wt, "#!/bin/sh\nprintf 'wt:%s\\n' \"$1\"\npwd\n").unwrap();
        fs::set_permissions(&wt, fs::Permissions::from_mode(0o755)).unwrap();
        let prefix = if shell == "fish" {
            "set -gx PATH \"$HOME\" $PATH"
        } else {
            "export PATH=\"$HOME:$PATH\""
        };
        let out = run_shell(
            shell,
            home.path(),
            &format!("{prefix}\ngx project wt --help"),
        );
        assert!(out.status.success(), "{shell}: {out:?}");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_directory(
            stdout
                .strip_prefix("wt:--help\n")
                .expect("Worktrunk help output"),
            &project,
        );
    }
}

#[test]
fn resume_enters_project_and_prints_context() {
    for shell in shells() {
        let home = TempDir::new().unwrap();
        let project = home.path().join("project with spaces");
        fs::create_dir(&project).unwrap();
        let git = isolated(
            Command::new("git").args(["init", "-q"]).arg(&project),
            home.path(),
        )
        .output()
        .unwrap();
        assert!(git.status.success(), "{git:?}");
        let indexed = isolated(
            Command::new(env!("CARGO_BIN_EXE_gx"))
                .arg("index")
                .arg(&project),
            home.path(),
        )
        .output()
        .unwrap();
        assert!(indexed.status.success(), "{indexed:?}");
        let out = run_shell(shell, home.path(), "gx resume 'project with spaces' && pwd");
        assert!(out.status.success(), "{shell}: {out:?}");
        assert_directory(&String::from_utf8_lossy(&out.stdout), &project);
        assert!(String::from_utf8_lossy(&out.stderr).contains("project with spaces ("));
    }
}

#[test]
fn installer_enables_navigation_on_first_shell_start() {
    let home = TempDir::new().unwrap();
    let mocks = home.path().join("download-tools");
    fs::create_dir(&mocks).unwrap();
    // Exercise the actual installer, replacing only network downloads.
    let downloader = mocks.join("curl");
    fs::write(
        &downloader,
        r#"#!/bin/sh
case "$2" in
    */SHA256SUMS)
        if command -v sha256sum >/dev/null 2>&1; then
            hash=$(sha256sum "$GX_TEST_BINARY" | awk '{print $1}')
        else
            hash=$(shasum -a 256 "$GX_TEST_BINARY" | awk '{print $1}')
        fi
        for asset in gx-linux-x64 gx-linux-aarch64 gx-darwin-x64 gx-darwin-aarch64; do
            printf '%s  %s\n' "$hash" "$asset"
        done > "$4"
        ;;
    *) cp "$GX_TEST_BINARY" "$4" ;;
esac
"#,
    )
    .unwrap();
    fs::set_permissions(&downloader, fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!("{}:/usr/bin:/bin", mocks.display());
    let installer = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../install.sh");
    let out = isolated(Command::new("sh").arg(installer), home.path())
        .env("PATH", &path)
        .env("SHELL", "/bin/bash")
        .env("GX_TEST_BINARY", env!("CARGO_BIN_EXE_gx"))
        .output()
        .unwrap();
    assert!(out.status.success(), "installer: {out:?}");
    let project = home.path().join("existing repo");
    fs::create_dir_all(project.join(".git")).unwrap();
    let out = isolated(
        Command::new("bash").args([
            "--noprofile",
            "--norc",
            "-c",
            "source \"$HOME/.bashrc\"; gx clone user/repo \"$HOME/existing repo\" && pwd",
        ]),
        home.path(),
    )
    .env("PATH", &path)
    .env("SHELL", "/bin/bash")
    .output()
    .unwrap();
    assert!(out.status.success(), "first shell: {out:?}");
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("command not found"),
        "{out:?}"
    );
    assert_directory(&String::from_utf8_lossy(&out.stdout), &project);
}
