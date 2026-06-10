use assert_cmd::Command;
use bwatch::FindingCategory;
use predicates::prelude::*;

const PLACEHOLDER_DIRECTIVE_HEADER: &str = "[bwatch placeholder directive - pre-corpus output]";
const ACTION_PREFIX: &str = "ACTION: This invocation reached bwatch";
const EXIT_CODE_FOOTER: &str = "Exit code carries the verdict-class signal.";
const PUBLIC_INVOCATION_SURFACE_LINE: &str = "Invocation surface: cli.";
const POLL_FINDING_EXIT_CODE: i32 = 1;
const INTERNAL_ERROR_EXIT_CODE: i32 = 2;
const INTERNAL_SURFACE_TOKENS: [&str; 6] = ["L2a", "L2b", "L2c", "l2a", "l2b", "l2c"];

#[derive(Debug, Clone, Copy)]
struct PollCase<'a> {
    args: &'a [&'a str],
    expected_source: &'a str,
    expected_mission: &'a str,
}

impl PollCase<'_> {
    fn assert(self) {
        let stdout = poll_stdout(self.args);

        assert_poll_directive(&stdout);
        assert!(
            stdout.contains(&format!("source={}", self.expected_source)),
            "stdout missing source={}: {stdout}",
            self.expected_source
        );
        assert!(
            stdout.contains(&format!("mission={}", self.expected_mission)),
            "stdout missing mission={}: {stdout}",
            self.expected_mission
        );
    }
}

fn bwatch_command() -> Command {
    Command::cargo_bin("bwatch").expect("binary exists")
}

fn command_stdout_with_code(args: &[&str], code: i32) -> String {
    let output = bwatch_command()
        .args(args)
        .assert()
        .code(code)
        .get_output()
        .clone();

    String::from_utf8(output.stdout).expect("stdout is utf8")
}

fn poll_stdout(args: &[&str]) -> String {
    command_stdout_with_code(args, POLL_FINDING_EXIT_CODE)
}

fn successful_stdout(args: &[&str]) -> String {
    command_stdout_with_code(args, 0)
}

fn assert_usage_failure(args: &[&str], stderr_fragment: &str) {
    bwatch_command()
        .args(args)
        .assert()
        .code(64)
        .stderr(predicate::str::contains(stderr_fragment));
}

fn assert_no_internal_surface_tokens(stdout: &str) {
    for token in INTERNAL_SURFACE_TOKENS {
        assert!(
            !stdout.contains(token),
            "stdout leaked internal surface token {token:?}: {stdout}"
        );
    }
}

fn assert_poll_directive(stdout: &str) {
    assert!(
        stdout.contains(PLACEHOLDER_DIRECTIVE_HEADER),
        "missing header: {stdout}"
    );
    assert!(
        stdout.contains("Parsed input: source="),
        "missing parsed input: {stdout}"
    );
    assert!(
        stdout.contains("mission="),
        "missing mission in parsed input: {stdout}"
    );
    assert!(
        stdout.contains("Routing key: FindingCategory::"),
        "missing routing key: {stdout}"
    );
    assert!(
        stdout.contains(" placeholder route."),
        "missing placeholder route: {stdout}"
    );
    assert!(
        stdout.contains(PUBLIC_INVOCATION_SURFACE_LINE),
        "missing invocation surface: {stdout}"
    );
    assert!(
        stdout.contains("Outward-source-state: Actionable."),
        "missing outward state: {stdout}"
    );
    assert!(
        stdout.contains(ACTION_PREFIX),
        "missing ACTION prefix: {stdout}"
    );
    assert!(
        stdout.contains("finding category"),
        "missing finding category in ACTION: {stdout}"
    );
    assert!(
        stdout.contains("external tracker source"),
        "missing tracker reference: {stdout}"
    );
    assert!(
        stdout.contains(EXIT_CODE_FOOTER),
        "missing exit code footer: {stdout}"
    );
    assert_no_internal_surface_tokens(stdout);
}

fn deferred_command_output(command_name: &str) -> String {
    format!("bwatch {command_name} placeholder: behavior is deferred.\n")
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
    let stdout = successful_stdout(&["finding-categories"]);
    let actual = stdout.lines().collect::<Vec<_>>();
    let expected = FindingCategory::ALL
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    assert_eq!(expected, actual);
}

#[test]
fn placeholder_commands_exit_successfully_with_stable_output() {
    for command_name in ["update", "init", "tail", "explain", "process"] {
        let stdout = successful_stdout(&[command_name]);

        assert_eq!(deferred_command_output(command_name), stdout);
    }
}

#[test]
fn poll_emits_placeholder_directive_and_finding_exit_code() {
    PollCase {
        args: &["poll"],
        expected_source: "<none>",
        expected_mission: "<none>",
    }
    .assert();
}

#[test]
fn poll_directive_reports_every_supported_input_combination() {
    for poll_case in [
        PollCase {
            args: &["poll", "--source", "github", "--mission", "v0.1"],
            expected_source: "github",
            expected_mission: "v0.1",
        },
        PollCase {
            args: &["poll", "--source", "github"],
            expected_source: "github",
            expected_mission: "<none>",
        },
        PollCase {
            args: &["poll", "--mission", "v0.1"],
            expected_source: "<none>",
            expected_mission: "v0.1",
        },
        PollCase {
            args: &[
                "poll",
                "--source",
                "https://github.com/org/repo",
                "--mission",
                "sprint-3",
            ],
            expected_source: "https://github.com/org/repo",
            expected_mission: "sprint-3",
        },
        PollCase {
            args: &["poll", "--manifest", "manifest.json"],
            expected_source: "<none>",
            expected_mission: "<none>",
        },
    ] {
        poll_case.assert();
    }
}

#[test]
fn poll_quiet_and_json_flags_keep_directive_stdout() {
    for poll_case in [
        PollCase {
            args: &["poll", "--quiet", "--reason", "review requested"],
            expected_source: "<none>",
            expected_mission: "<none>",
        },
        PollCase {
            args: &["poll", "--json", "--reason", "review requested"],
            expected_source: "<none>",
            expected_mission: "<none>",
        },
    ] {
        poll_case.assert();
    }
}

#[test]
fn poll_rejects_blank_reason() {
    for blank_reason in ["", " ", "   ", "\t", "\n", "\r", "\r\n"] {
        bwatch_command()
            .args(["poll", "--reason", blank_reason])
            .assert()
            .code(64)
            .stderr(predicate::str::contains("reason must not be empty"));
    }
}

#[test]
fn poll_accepts_every_non_blank_reason_shape() {
    for reason in [
        "review requested",
        " review requested ",
        "review\trequested",
    ] {
        PollCase {
            args: &["poll", "--reason", reason],
            expected_source: "<none>",
            expected_mission: "<none>",
        }
        .assert();
    }
}

#[test]
fn poll_rejects_empty_source_with_internal_error() {
    bwatch_command()
        .args(["poll", "--source", ""])
        .assert()
        .code(INTERNAL_ERROR_EXIT_CODE)
        .stderr(predicate::str::contains("source input is malformed"));
}

#[test]
fn poll_rejects_empty_mission_with_internal_error() {
    bwatch_command()
        .args(["poll", "--mission", ""])
        .assert()
        .code(INTERNAL_ERROR_EXIT_CODE)
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
