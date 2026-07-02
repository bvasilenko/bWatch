mod common;

use bsuite_core::ExitCode;
use bwatch::FindingCategory;
use common::{bwatch_command, command_stdout_with_dir, poll_stdout_with_dir};
use predicates::prelude::*;
use tempfile::TempDir;

const INTERNAL_SURFACE_TOKENS: [&str; 6] = ["L2a", "L2b", "L2c", "l2a", "l2b", "l2c"];

#[derive(Debug, Clone, Copy)]
struct PollCase<'a> {
    args: &'a [&'a str],
    category: FindingCategory,
}

impl PollCase<'_> {
    fn assert(self, dir: &TempDir) {
        let stdout = command_stdout_with_dir(self.args, ExitCode::Finding, dir);
        assert_poll_directive(&stdout, self.category);
    }
}

fn assert_no_internal_surface_tokens(stdout: &str) {
    for token in INTERNAL_SURFACE_TOKENS {
        assert!(
            !stdout.contains(token),
            "stdout leaked internal surface token {token:?}: {stdout}"
        );
    }
}

fn assert_poll_directive(stdout: &str, category: FindingCategory) {
    let expected_prefix = format!("FINDING-DETECTED: {}.", category.stable_name());
    assert!(
        stdout.contains(&expected_prefix),
        "missing finding header for {category}: {stdout}"
    );
    assert_no_internal_surface_tokens(stdout);
}

fn assert_usage_failure(args: &[&str], stderr_fragment: &str) {
    bwatch_command()
        .args(args)
        .assert()
        .code(ExitCode::Usage.as_i32())
        .stderr(predicate::str::contains(stderr_fragment));
}

#[test]
fn help_exits_successfully() {
    bwatch_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Outward tracker observation tool"));
}

#[test]
fn finding_categories_exits_successfully_and_prints_exact_closed_set() {
    let dir = TempDir::new().unwrap();
    let stdout = command_stdout_with_dir(&["finding-categories"], ExitCode::Success, &dir);
    let actual = stdout.lines().collect::<Vec<_>>();
    let expected = FindingCategory::ALL
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    assert_eq!(expected, actual);
}

#[test]
fn poll_emits_corpus_directive_and_finding_exit_code() {
    let dir = TempDir::new().unwrap();
    PollCase {
        args: &["poll", "--category", "sprint-conflict"],
        category: FindingCategory::SprintConflict,
    }
    .assert(&dir);
}

#[test]
fn poll_directive_covers_every_finding_category() {
    let dir = TempDir::new().unwrap();
    for category in FindingCategory::ALL {
        PollCase {
            args: &["poll", "--category", category.stable_name()],
            category,
        }
        .assert(&dir);
    }
}

#[test]
fn optional_poll_flags_are_orthogonal_to_directive_category() {
    let dir = TempDir::new().unwrap();
    let cases: &[(&str, &[&str])] = &[
        ("sprint-conflict", &["--source", "github"]),
        ("sprint-conflict", &["--mission", "v0.1"]),
        ("sprint-conflict", &["--manifest", "manifest.json"]),
        ("sprint-conflict", &["--quiet"]),
        ("sprint-conflict", &["--reason", "review requested"]),
        (
            "runbook-update",
            &["--source", "github", "--mission", "sprint-3"],
        ),
        (
            "external-spec-change",
            &[
                "--source",
                "https://github.com/org/repo",
                "--mission",
                "v0.1",
                "--reason",
                "API changed",
            ],
        ),
    ];
    for (category_name, extra_args) in cases {
        let category = category_name
            .parse::<FindingCategory>()
            .expect("valid category name");
        let stdout = poll_stdout_with_dir(category_name, extra_args, &dir);
        assert_poll_directive(&stdout, category);
    }
}

#[test]
fn poll_quiet_flag_preserves_directive_on_stdout() {
    let dir = TempDir::new().unwrap();
    PollCase {
        args: &[
            "poll",
            "--category",
            "sprint-conflict",
            "--quiet",
            "--reason",
            "review requested",
        ],
        category: FindingCategory::SprintConflict,
    }
    .assert(&dir);
}

#[test]
fn poll_rejects_blank_reason_regardless_of_category() {
    // Blank reason rejection is enforced before category resolution.
    for blank_reason in ["", " ", "   ", "\t", "\n", "\r", "\r\n"] {
        for category in ["sprint-conflict", "runbook-update"] {
            bwatch_command()
                .args(["poll", "--category", category, "--reason", blank_reason])
                .assert()
                .code(ExitCode::Usage.as_i32())
                .stderr(predicate::str::contains("reason must not be empty"));
        }
    }
}

#[test]
fn poll_accepts_every_non_blank_reason_shape() {
    let dir = TempDir::new().unwrap();
    for reason in [
        "review requested",
        " review requested ",
        "review\trequested",
    ] {
        PollCase {
            args: &["poll", "--category", "sprint-conflict", "--reason", reason],
            category: FindingCategory::SprintConflict,
        }
        .assert(&dir);
    }
}

#[test]
fn poll_rejects_unknown_category_with_usage_exit_code() {
    bwatch_command()
        .args(["poll", "--category", "not-a-valid-category"])
        .assert()
        .code(ExitCode::Usage.as_i32())
        .stderr(predicate::str::contains("unknown finding category"));
}

#[test]
fn poll_missing_required_category_flag_exits_with_usage() {
    assert_usage_failure(&["poll"], "--category");
}

#[test]
fn poll_rejects_empty_source_with_internal_error() {
    bwatch_command()
        .args(["poll", "--category", "sprint-conflict", "--source", ""])
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .stderr(predicate::str::contains("source input is malformed"));
}

#[test]
fn poll_rejects_empty_mission_with_internal_error() {
    bwatch_command()
        .args(["poll", "--category", "sprint-conflict", "--mission", ""])
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .stderr(predicate::str::contains("mission reference is invalid"));
}

#[test]
fn no_subcommand_uses_cli_usage_failure() {
    assert_usage_failure(&[], "Usage:");
}

#[test]
fn unknown_command_uses_cli_usage_failure() {
    assert_usage_failure(&["unknown"], "unrecognized subcommand");
}

#[test]
fn malformed_flag_shape_uses_cli_usage_failure() {
    for (args, stderr_fragment) in [
        (&["poll", "--reason"][..], "a value is required"),
        (&["poll", "--unknown"][..], "unexpected argument"),
        (&["poll", "--json=false"][..], "unexpected value"),
    ] {
        assert_usage_failure(args, stderr_fragment);
    }
}
