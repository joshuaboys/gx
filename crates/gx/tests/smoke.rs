use std::process::Command;

#[test]
fn binary_prints_help_with_no_args() {
    let output = Command::new(env!("CARGO_BIN_EXE_gx"))
        .output()
        .expect("failed to execute gx");

    assert!(
        output.status.success(),
        "gx exited with {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("git project manager") && stdout.contains("Usage:"),
        "expected help banner, got: {stdout}"
    );
}
