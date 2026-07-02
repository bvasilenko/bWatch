mod common;

use bsuite_core::ExitCode;
use bwatch::FindingCategory;
use common::bwatch_command;
use tempfile::TempDir;

#[test]
fn poll_routes_directive_to_stdout_and_nothing_to_stderr_for_all_categories() {
    let transcript_dir = TempDir::new().unwrap();
    for category in FindingCategory::ALL {
        let output = bwatch_command()
            .env("BSUITE_TRANSCRIPT_DIR", transcript_dir.path())
            .args(["poll", "--category", category.stable_name()])
            .assert()
            .code(ExitCode::Finding.as_i32())
            .get_output()
            .clone();

        assert!(!output.stdout.is_empty(), "{category}: empty stdout");
        assert!(
            output.stderr.is_empty(),
            "{category}: stderr was {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn malformed_category_routes_error_to_stderr_and_nothing_to_stdout() {
    let output = bwatch_command()
        .args(["poll", "--category", "unknown-category"])
        .assert()
        .code(ExitCode::Usage.as_i32())
        .get_output()
        .clone();

    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn deferred_verbs_produce_no_output_on_either_stream_and_exit_successfully() {
    let transcript_dir = TempDir::new().unwrap();
    for subcmd in ["init", "tail", "explain", "process"] {
        let output = bwatch_command()
            .env("BSUITE_TRANSCRIPT_DIR", transcript_dir.path())
            .arg(subcmd)
            .assert()
            .code(ExitCode::Success.as_i32())
            .get_output()
            .clone();

        assert!(output.stdout.is_empty(), "{subcmd}: stdout not empty");
        assert!(output.stderr.is_empty(), "{subcmd}: stderr not empty");
    }
}

#[test]
fn finding_categories_listing_goes_to_stdout_not_stderr_and_is_not_json() {
    let transcript_dir = TempDir::new().unwrap();
    let output = bwatch_command()
        .env("BSUITE_TRANSCRIPT_DIR", transcript_dir.path())
        .arg("finding-categories")
        .assert()
        .code(ExitCode::Success.as_i32())
        .get_output()
        .clone();

    let stdout = String::from_utf8(output.stdout).expect("finding-categories stdout is UTF-8");
    assert!(!stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert!(serde_json::from_str::<serde_json::Value>(stdout.trim()).is_err());
}

#[test]
fn update_subcommand_writes_only_to_stderr_never_stdout() {
    let transcript_dir = TempDir::new().unwrap();
    let output = bwatch_command()
        .env("BSUITE_TRANSCRIPT_DIR", transcript_dir.path())
        .env_remove("BSUITE_UPDATE_BASE_URL")
        .arg("update")
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "update failure must not write to stdout: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !output.stderr.is_empty(),
        "update failure must surface a message on stderr"
    );
}
