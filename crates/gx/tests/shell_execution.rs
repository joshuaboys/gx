#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn generated(shell: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_gx"))
        .args(["shell-init", shell])
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

fn available(shell: &str) -> bool {
    Command::new(shell).arg("--version").output().is_ok()
}

#[test]
fn clone_wrapper_preserves_failure() {
    for shell in ["bash", "zsh"] {
        if !available(shell) {
            eprintln!("skipping unavailable {shell}");
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let stub = dir.path().join("stub");
        script(&stub, "exit 7");
        let body = format!(
            "{}\n_GX_BIN=\"$1\"\ngx clone example/repo\nexit $?",
            generated(shell)
        );
        let out = Command::new(shell)
            .args(["-f", "-c", &body, "gx-test"])
            .arg(&stub)
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(7),
            "{shell}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn zsh_resume_changes_directory_and_preserves_failure() {
    if !available("zsh") {
        eprintln!("skipping unavailable zsh");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().join("destination");
    fs::create_dir(&destination).unwrap();
    let destination = destination.canonicalize().unwrap();
    let stub = dir.path().join("stub");
    script(&stub, "printf '%s\\n' \"$DESTINATION\"\nexit \"$RESULT\"");
    let body = format!("{}\n_GX_BIN=\"$1\"\ngx resume project\nresult=$?\n[ \"$result\" -eq 0 ] || exit \"$result\"\n[ \"$PWD\" = \"$DESTINATION\" ]", generated("zsh"));
    for code in [0, 7] {
        let out = Command::new("zsh")
            .args(["-f", "-c", &body, "gx-test"])
            .arg(&stub)
            .env("DESTINATION", &destination)
            .env("RESULT", code.to_string())
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn executable_path_is_literal_shell_data() {
    let dir = tempfile::tempdir().unwrap();
    // Substitutions are deliberately inert: no injected command writes files.
    let bin_dir = dir
        .path()
        .join("space ' quote \" $HOME $(printf injected) \u{60}printf injected\u{60} back\\slash");
    fs::create_dir(&bin_dir).unwrap();
    let binary = bin_dir.join("gx");
    fs::copy(env!("CARGO_BIN_EXE_gx"), &binary).unwrap();
    let binary = binary.canonicalize().unwrap();
    for shell in ["bash", "zsh", "fish"] {
        if !available(shell) {
            eprintln!("skipping unavailable {shell}");
            continue;
        }
        let out = Command::new(&binary)
            .args(["shell-init", shell])
            .output()
            .unwrap();
        assert!(out.status.success());
        let mut source = String::from_utf8(out.stdout).unwrap();
        source.push_str("\nprintf '%s' \"$_GX_BIN\"\n");
        let out = Command::new(shell)
            .args(["-c", &source])
            .env("HOME", dir.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{shell}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            String::from_utf8(out.stdout).unwrap(),
            binary.to_str().unwrap(),
            "{shell}"
        );
    }
}

#[test]
fn terminal_editor_exit_status_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let editor = dir.path().join("editor");
    for code in [0, 7] {
        script(&editor, &format!("exit {code}"));
        let out = Command::new(env!("CARGO_BIN_EXE_gx"))
            .args(["open", "--editor"])
            .arg(&editor)
            .env("HOME", dir.path())
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("GX_AGENT")
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn aps_relative_invocation_ignores_cdpath() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let baseline = Command::new("bash")
        .args(["bin/aps", "--help"])
        .current_dir(&root)
        .env_remove("CDPATH")
        .output()
        .unwrap();
    assert!(baseline.status.success());
    let out = Command::new("bash")
        .args(["bin/aps", "--help"])
        .current_dir(&root)
        .env("CDPATH", &root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, baseline.stdout);
    let links = tempfile::tempdir().unwrap();
    // Both the invocation and the symlink target exercise directory capture.
    std::os::unix::fs::symlink(root.join("bin/aps"), links.path().join("aps")).unwrap();
    let parent = links.path().parent().unwrap();
    let relative = links
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        + "/aps";
    let out = Command::new("bash")
        .arg(relative)
        .arg("--help")
        .current_dir(parent)
        .env("CDPATH", parent)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, baseline.stdout);
}
