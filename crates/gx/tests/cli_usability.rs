use std::process::{Command, Output};

use tempfile::TempDir;

fn run(home: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gx"))
        .args(args)
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env_remove("GX_AGENT")
        .current_dir(home.path())
        .output()
        .unwrap()
}

#[test]
fn subcommand_help_is_read_only_and_successful() {
    let home = TempDir::new().unwrap();
    for command in [
        "clone",
        "open",
        "init",
        "index",
        "rebuild",
        "recent",
        "resume",
        "doctor",
        "ls",
        "config",
        "shell-init",
        "resolve",
    ] {
        for flag in ["--help", "-h"] {
            let out = run(&home, &[command, flag]);
            assert!(out.status.success(), "{command} {flag}: {out:?}");
            assert!(
                String::from_utf8_lossy(&out.stdout).contains(&format!("gx {command}")),
                "{out:?}"
            );
            assert!(out.stderr.is_empty(), "{out:?}");
        }
    }
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
}

#[test]
fn malformed_config_commands_fail_without_writing() {
    let home = TempDir::new().unwrap();
    for args in [
        vec!["config", "set"],
        vec!["config", "set", "editor"],
        vec!["config", "unknown"],
        vec!["config", "set", "editor", "code", "extra"],
    ] {
        let out = run(&home, &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: gx config"));
    }
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
    assert!(run(&home, &["config"]).status.success());
    assert!(run(&home, &["config", "set", "editor", "code"])
        .status
        .success());
    assert!(String::from_utf8_lossy(&run(&home, &["config"]).stdout).contains("code"));
}

#[cfg(windows)]
#[test]
fn powershell_wrapper_forwards_help_without_changing_directory() {
    let home = TempDir::new().unwrap();
    let generated = run(&home, &["shell-init", "powershell"]);
    assert!(generated.status.success());
    let script = home.path().join("help.ps1");
    std::fs::write(
        &script,
        format!(
            "{}\n{}",
            String::from_utf8_lossy(&generated.stdout),
            r#"
$before = (Get-Location).Path
gx clone --help
if ($LASTEXITCODE -ne 0) { exit 1 }
gx resume --help
if ($LASTEXITCODE -ne 0) { exit 1 }
if ((Get-Location).Path -ne $before) { exit 1 }
"#
        ),
    )
    .unwrap();
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(script)
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env_remove("GX_AGENT")
        .current_dir(home.path())
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Usage: gx clone"), "{out:?}");
    assert!(stdout.contains("Usage: gx resume"), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
}
