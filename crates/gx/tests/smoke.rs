use std::process::Command;

#[test]
fn binary_prints_version_banner() {
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
        stdout.contains("gx (rust port)"),
        "expected version banner, got: {stdout}"
    );
}
