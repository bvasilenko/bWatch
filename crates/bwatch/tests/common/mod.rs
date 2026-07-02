#![allow(dead_code)]

pub mod test_server;

use assert_cmd::Command;
use bsuite_core::ExitCode;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{collections::BTreeSet, fmt::Display, str::FromStr};
use tempfile::TempDir;

pub fn bwatch_command() -> Command {
    Command::cargo_bin("bwatch").expect("binary exists")
}

pub fn bwatch_command_in_dir(dir: &TempDir) -> Command {
    let mut cmd = bwatch_command();
    cmd.env("BSUITE_TRANSCRIPT_DIR", dir.path());
    cmd
}

pub fn command_output_with_dir(
    args: &[&str],
    exit_code: ExitCode,
    dir: &TempDir,
) -> std::process::Output {
    bwatch_command_in_dir(dir)
        .args(args)
        .assert()
        .code(exit_code.as_i32())
        .get_output()
        .clone()
}

pub fn command_stdout_with_dir(args: &[&str], exit_code: ExitCode, dir: &TempDir) -> String {
    String::from_utf8(command_output_with_dir(args, exit_code, dir).stdout)
        .expect("stdout is UTF-8")
}

pub fn poll_stdout_with_dir(category: &str, extra_args: &[&str], dir: &TempDir) -> String {
    let mut args = vec!["poll", "--category", category];
    args.extend_from_slice(extra_args);
    command_stdout_with_dir(&args, ExitCode::Finding, dir)
}

pub fn json_stdout_with_dir(args: &[&str], exit_code: ExitCode, dir: &TempDir) -> Value {
    let stdout = command_stdout_with_dir(args, exit_code, dir);
    serde_json::from_str(stdout.trim()).expect("stdout is valid JSON")
}

pub fn command_output(args: &[&str], exit_code: ExitCode) -> std::process::Output {
    bwatch_command()
        .args(args)
        .assert()
        .code(exit_code.as_i32())
        .get_output()
        .clone()
}

pub fn command_stdout(args: &[&str], exit_code: ExitCode) -> String {
    String::from_utf8(command_output(args, exit_code).stdout).expect("stdout is UTF-8")
}

pub fn poll_stdout(category: &str, extra_args: &[&str]) -> String {
    let mut args = vec!["poll", "--category", category];
    args.extend_from_slice(extra_args);
    command_stdout(&args, ExitCode::Finding)
}

pub fn json_stdout(args: &[&str], exit_code: ExitCode) -> Value {
    let stdout = command_stdout(args, exit_code);
    serde_json::from_str(stdout.trim()).expect("stdout is valid JSON")
}

pub fn assert_public_name_contract<T>(values: &[T])
where
    T: Copy + Display + FromStr + Eq + std::fmt::Debug + Serialize + DeserializeOwned,
    <T as FromStr>::Err: std::fmt::Debug,
{
    assert_display_names_are_unique(values);
    assert_round_trip(values);
    assert_json_names_match_display(values);
    assert_rejects_surrounding_whitespace(values);
}

pub fn assert_display_names_are_unique<T>(values: &[T])
where
    T: Copy + Display,
{
    let names = values
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();

    assert_eq!(values.len(), names.len());
}

pub fn assert_round_trip<T>(values: &[T])
where
    T: Copy + Display + FromStr + Eq + std::fmt::Debug,
    <T as FromStr>::Err: std::fmt::Debug,
{
    for value in values {
        let parsed = T::from_str(&value.to_string()).expect("display name must parse");
        assert_eq!(*value, parsed);
    }
}

pub fn assert_rejects<T>(invalid_names: &[&str])
where
    T: FromStr,
{
    for invalid_name in invalid_names {
        assert!(
            T::from_str(invalid_name).is_err(),
            "accepted {invalid_name:?}"
        );
    }
}

fn assert_json_names_match_display<T>(values: &[T])
where
    T: Copy + Display + Eq + std::fmt::Debug + Serialize + DeserializeOwned,
{
    for value in values {
        let encoded = serde_json::to_string(value).expect("json encode");
        let decoded: T = serde_json::from_str(&encoded).expect("json decode");

        assert_eq!(format!("\"{value}\""), encoded);
        assert_eq!(*value, decoded);
    }
}

fn assert_rejects_surrounding_whitespace<T>(values: &[T])
where
    T: Copy + Display + FromStr,
{
    for value in values {
        let display_name = value.to_string();
        for invalid_name in [
            format!(" {display_name}"),
            format!("{display_name} "),
            format!("\t{display_name}"),
            format!("{display_name}\n"),
        ] {
            assert!(
                T::from_str(&invalid_name).is_err(),
                "accepted whitespace-padded name {invalid_name:?}"
            );
        }
    }
}
