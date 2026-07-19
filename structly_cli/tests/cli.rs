//! End-to-end tests for the `structly` binary against a source fixture.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/config.rs")
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_structly"))
        .args(args)
        .output()
        .expect("binary should run")
}

#[test]
fn writes_markdown_to_stdout() {
    let output = run(&["docs", "--path", fixture(), "--struct", "AppConfig"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // First line is a markdown comment recording the generating command.
    let first_line = stdout.lines().next().unwrap();
    assert_eq!(
        first_line,
        format!("[//]: # (structly docs --struct AppConfig --path {})", fixture())
    );
    assert!(stdout.contains("# AppConfig"));
    assert!(stdout.contains("## database (section: Database)"));
    assert!(stdout.contains("Database connection settings."));
    assert!(stdout.contains("### database.cert_path (Certificate)"));
    assert!(stdout.contains("PEM certificate used for SSL."));
    assert!(stdout.contains("- Required when `self.ssl`: cert required when ssl is on"));
    assert!(stdout.contains("## server (section: Server)"));
    assert!(stdout.contains("### server.domain"));
    assert!(stdout.contains("- Required when `self.public`: domain required for public servers"));
    // List fields resolve through the container and use bracketed paths.
    assert!(stdout.contains("## categories (list of CommitCategory)"));
    assert!(stdout.contains("### categories[].name"));
    // Structs without the derive are ignored entirely.
    assert!(!stdout.contains("NotStructly"));
}

#[test]
fn writes_markdown_to_a_file() {
    let out_path: PathBuf = std::env::temp_dir().join(format!(
        "structly_cli_test_{}.md",
        std::process::id()
    ));

    let output = run(&[
        "docs",
        "--path",
        fixture(),
        "--struct",
        "AppConfig",
        "--out",
        out_path.to_str().unwrap(),
    ]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let written = std::fs::read_to_string(&out_path).expect("output file written");
    // The regeneration comment includes the --out argument when one was given.
    let first_line = written.lines().next().unwrap();
    assert_eq!(
        first_line,
        format!(
            "[//]: # (structly docs --struct AppConfig --path {} --out {})",
            fixture(),
            out_path.display()
        )
    );
    assert!(written.contains("# AppConfig"));
    assert!(written.contains("### database.cert_path (Certificate)"));

    let _ = std::fs::remove_file(&out_path);
}

#[test]
fn can_target_a_nested_struct_directly() {
    let output = run(&["docs", "--path", fixture(), "--struct", "Database"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Database"));
    assert!(stdout.contains("## cert_path (Certificate)"));
}

#[test]
fn unknown_struct_fails_and_lists_available() {
    let output = run(&["docs", "--path", fixture(), "--struct", "Nope"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no #[derive(Structly)] struct named `Nope`"));
    assert!(stderr.contains("AppConfig"));
    assert!(stderr.contains("Database"));
    assert!(stderr.contains("Server"));
}

#[test]
fn missing_struct_argument_is_a_usage_error() {
    let output = run(&["docs", "--path", fixture()]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("`--struct` is required"));
}
