// The publication job runs on Ubuntu and uses GNU sha256sum.
#![cfg(target_os = "linux")]

use std::os::unix::fs::PermissionsExt;
use std::{fs, path::PathBuf, process::Command};

struct ReleaseEnv {
    dir: tempfile::TempDir,
}

impl ReleaseEnv {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("dist")).unwrap();
        fs::create_dir(dir.path().join("remote")).unwrap();
        fs::create_dir(dir.path().join("bin")).unwrap();
        for name in [
            "gx-linux-x64",
            "gx-linux-aarch64",
            "gx-darwin-x64",
            "gx-darwin-aarch64",
            "gx-windows-x64.exe",
        ] {
            fs::write(dir.path().join("dist").join(name), name).unwrap();
        }
        let stub = dir.path().join("bin/gh");
        fs::write(
            &stub,
            r#"#!/bin/bash
set -eu
echo "$*" >> "$FIXTURE/calls"
case "$1 $2" in
  "release view")
    test -f "$FIXTURE/state" || exit 1
    if [[ "$*" == *"--json assets"* ]]; then
      for file in "$FIXTURE/remote/"*; do basename "$file"; done
    else
      test "$(cat "$FIXTURE/state")" = draft && echo true || echo false
    fi ;;
  "release create")
    if [[ "$*" == *"--draft"* ]]; then echo draft; else echo published; fi > "$FIXTURE/state" ;;
  "release upload")
    test "$FAILURE" != upload || exit 7
    for arg in "$@"; do
      if [[ -f "$arg" ]]; then cp "$arg" "$FIXTURE/remote/"; fi
    done ;;
  "release download")
    while [[ "$1" != "--dir" ]]; do shift; done
    shift
    cp "$FIXTURE/remote/"* "$1/"
    if [[ "$FAILURE" == corrupt ]]; then echo corrupt > "$1/gx-linux-x64"; fi ;;
  "release edit")
    echo published > "$FIXTURE/state" ;;
  *) exit 9 ;;
esac
"#,
        )
        .unwrap();
        fs::set_permissions(stub, fs::Permissions::from_mode(0o755)).unwrap();
        Self { dir }
    }

    fn run(&self, failure: &str) -> bool {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        // Execute the actual publish job's shell steps; replace only gh and
        // the hosted attestation action, so ordering is part of the test.
        let workflow = fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();
        let job = workflow.split("  publish:\n").nth(1).unwrap();
        let mut lines = job.lines().peekable();
        let mut working_dir = self.dir.path().to_path_buf();
        while let Some(line) = lines.next() {
            if line.starts_with("      - ") {
                working_dir = self.dir.path().to_path_buf();
            }
            if line.contains("uses: actions/attest-build-provenance@") && failure == "attest" {
                return false;
            }
            if let Some(dir) = line.strip_prefix("        working-directory: ") {
                working_dir = self.dir.path().join(dir);
            }
            let Some(run) = line.strip_prefix("        run: ") else {
                continue;
            };
            let body = if run == "|" {
                let mut body = String::new();
                while let Some(next) = lines.peek() {
                    if let Some(code) = next.strip_prefix("          ") {
                        body.push_str(code);
                        body.push('\n');
                        lines.next();
                    } else {
                        break;
                    }
                }
                body
            } else {
                run.to_string()
            };
            // Scripts live in the source checkout; assets/state are isolated.
            let body = body.replace(
                "scripts/release.sh",
                &root.join("scripts/release.sh").to_string_lossy(),
            );
            let paths = std::env::join_paths(
                std::iter::once(self.dir.path().join("bin"))
                    .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
            )
            .unwrap();
            let out = Command::new("bash")
                .args(["-e", "-c", &body])
                .current_dir(&working_dir)
                .env("PATH", paths)
                .env("FIXTURE", self.dir.path())
                .env("FAILURE", failure)
                .env("GITHUB_REF_NAME", "v0.0.0-test")
                .env("GITHUB_REPOSITORY", "test/gx")
                .output()
                .unwrap();
            if !out.status.success() {
                return false;
            }
        }
        true
    }

    fn state(&self) -> String {
        fs::read_to_string(self.dir.path().join("state"))
            .unwrap()
            .trim()
            .to_string()
    }
}

#[test]
fn failed_release_stays_private_and_retry_can_publish() {
    for failure in ["upload", "attest", "corrupt"] {
        let env = ReleaseEnv::new();
        assert!(!env.run(failure), "{failure} must fail");
        assert_eq!(env.state(), "draft", "{failure} exposed a public release");
        assert!(env.run(""), "draft retry must succeed after {failure}");
        assert_eq!(env.state(), "published");
        let calls = fs::read_to_string(env.dir.path().join("calls")).unwrap();
        let before = calls
            .lines()
            .filter(|l| l.starts_with("release upload"))
            .count();
        assert!(!env.run(""), "published releases must not be replaced");
        let calls = fs::read_to_string(env.dir.path().join("calls")).unwrap();
        assert_eq!(
            calls
                .lines()
                .filter(|l| l.starts_with("release upload"))
                .count(),
            before
        );
    }
}
