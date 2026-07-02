mod common;

use bsuite_core::ExitCode;
use bwatch::FindingCategory;
use common::{bwatch_command_in_dir, command_stdout_with_dir, json_stdout_with_dir};
use tempfile::TempDir;

#[test]
fn plain_output_without_json_flag_is_not_json() {
    let dir = TempDir::new().unwrap();
    let output = command_stdout_with_dir(
        &["poll", "--category", "sprint-conflict"],
        ExitCode::Finding,
        &dir,
    );
    assert!(serde_json::from_str::<serde_json::Value>(output.trim()).is_err());
}

#[test]
fn json_flag_produces_valid_envelope_for_all_categories() {
    let dir = TempDir::new().unwrap();
    for category in FindingCategory::ALL {
        let envelope = json_stdout_with_dir(
            &["poll", "--category", category.stable_name(), "--json"],
            ExitCode::Finding,
            &dir,
        );
        assert_eq!(envelope["schema_version"].as_u64(), Some(1));
        assert_eq!(envelope["outcome"].as_str(), Some("finding"));
        assert!(
            envelope.get("error").is_none_or(|v| v.is_null()),
            "{category}: error field must be absent or null in a finding success envelope"
        );
        let directive = envelope["directive"].as_str().unwrap_or_default();
        let expected_prefix = format!("FINDING-DETECTED: {}.", category.stable_name());
        assert!(
            directive.starts_with(&expected_prefix),
            "directive for {category} must start with {expected_prefix:?}: got {directive:?}"
        );
    }
}

#[test]
fn json_flag_with_malformed_category_routes_error_to_stderr_not_stdout() {
    // --json only affects the success output format, not the error path.
    let dir = TempDir::new().unwrap();
    let output = bwatch_command_in_dir(&dir)
        .args(["poll", "--json", "--category", "unknown-category"])
        .assert()
        .code(ExitCode::Usage.as_i32())
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "malformed category with --json must not produce stdout: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !output.stderr.is_empty(),
        "error message must appear on stderr"
    );
}

#[test]
fn json_flag_and_quiet_flag_are_orthogonal() {
    let dir = TempDir::new().unwrap();
    for category in FindingCategory::ALL {
        let envelope = json_stdout_with_dir(
            &[
                "poll",
                "--category",
                category.stable_name(),
                "--json",
                "--quiet",
            ],
            ExitCode::Finding,
            &dir,
        );
        assert_eq!(
            envelope["schema_version"].as_u64(),
            Some(1),
            "{category}: --quiet must not suppress the JSON envelope"
        );
        assert_eq!(
            envelope["outcome"].as_str(),
            Some("finding"),
            "{category}: outcome must still be 'finding' with --quiet"
        );
        let directive = envelope["directive"].as_str().unwrap_or_default();
        assert!(
            !directive.is_empty(),
            "{category}: directive must not be empty when --quiet and --json are combined"
        );
    }
}
