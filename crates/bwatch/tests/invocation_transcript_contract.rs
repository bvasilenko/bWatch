mod common;

use bsuite_core::ExitCode;
use common::bwatch_command;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct TranscriptCase {
    name: &'static str,
    args: &'static [&'static str],
    expected_exit_code: ExitCode,
    directive_emitted: bool,
}

const TRANSCRIPT_CASES: &[TranscriptCase] = &[
    TranscriptCase {
        name: "poll-finding",
        args: &["poll", "--category", "sprint-conflict"],
        expected_exit_code: ExitCode::Finding,
        directive_emitted: true,
    },
    TranscriptCase {
        name: "poll-malformed-category",
        args: &["poll", "--category", "unknown-category"],
        expected_exit_code: ExitCode::Usage,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "finding-categories",
        args: &["finding-categories"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "init",
        args: &["init"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "tail",
        args: &["tail"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "explain",
        args: &["explain"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "process",
        args: &["process"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "update",
        args: &["update"],
        expected_exit_code: ExitCode::InternalError,
        directive_emitted: false,
    },
];

fn run_in_transcript_dir(args: &[&str], dir: &TempDir) -> assert_cmd::assert::Assert {
    let mut cmd = bwatch_command();
    cmd.env("BSUITE_TRANSCRIPT_DIR", dir.path());
    for arg in args {
        cmd.arg(arg);
    }
    cmd.assert()
}

fn collect_transcript_files(dir: &TempDir) -> Vec<PathBuf> {
    let bwatch_dir = dir.path().join("bwatch");
    if !bwatch_dir.exists() {
        return vec![];
    }
    std::fs::read_dir(&bwatch_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jsonl"))
        .collect()
}

fn read_transcript_record(path: &Path) -> Value {
    let content = std::fs::read_to_string(path).expect("transcript file is readable");
    serde_json::from_str(content.trim()).expect("transcript file is valid JSON")
}

#[test]
fn every_subcommand_appends_exactly_one_transcript_record() {
    for case in TRANSCRIPT_CASES {
        let dir = TempDir::new().unwrap();
        run_in_transcript_dir(case.args, &dir).code(case.expected_exit_code.as_i32());

        let files = collect_transcript_files(&dir);
        assert_eq!(
            1,
            files.len(),
            "{}: expected 1 transcript file, got {}",
            case.name,
            files.len()
        );

        let record = read_transcript_record(&files[0]);
        assert_eq!(
            record["binary_name"].as_str(),
            Some("bwatch"),
            "{}: binary_name",
            case.name
        );
        assert_eq!(
            record["schema_version"].as_u64(),
            Some(1),
            "{}: schema_version",
            case.name
        );
        assert_eq!(
            record["directive_emitted"].as_bool(),
            Some(case.directive_emitted),
            "{}: directive_emitted",
            case.name
        );
        assert_eq!(
            record["exit_code"].as_u64(),
            Some(case.expected_exit_code.as_i32() as u64),
            "{}: exit_code",
            case.name
        );
        assert!(
            record["invocation_id"].as_str().is_some(),
            "{}: invocation_id must be set",
            case.name
        );
        assert!(
            record["timestamp"].as_str().is_some(),
            "{}: timestamp must be set",
            case.name
        );
        assert_eq!(
            record["routing_key"].as_str(),
            Some("bwatch"),
            "{}: routing_key must be bwatch",
            case.name
        );
        let binary_version = record["binary_version"]
            .as_str()
            .expect("binary_version must be set");
        assert!(
            semver::Version::parse(binary_version).is_ok(),
            "{}: binary_version must be valid semver, got {binary_version:?}",
            case.name
        );
        assert_eq!(
            record["corpus_version"].as_u64(),
            Some(1),
            "{}: corpus_version must be 1",
            case.name
        );
        assert!(
            record["elapsed_ms"].is_number(),
            "{}: elapsed_ms must be a number",
            case.name
        );
        assert!(
            record["additional_fields"].is_object(),
            "{}: additional_fields must be an object",
            case.name
        );
    }
}

#[test]
fn sequential_invocations_each_produce_their_own_transcript_file() {
    let dir = TempDir::new().unwrap();

    run_in_transcript_dir(&["poll", "--category", "sprint-conflict"], &dir)
        .code(ExitCode::Finding.as_i32());
    run_in_transcript_dir(&["poll", "--category", "runbook-update"], &dir)
        .code(ExitCode::Finding.as_i32());

    let files = collect_transcript_files(&dir);
    assert_eq!(
        2,
        files.len(),
        "each sequential invocation must produce its own file"
    );

    let invocation_ids: std::collections::BTreeSet<String> = files
        .iter()
        .map(|f| {
            let record = read_transcript_record(f);
            record["invocation_id"]
                .as_str()
                .expect("invocation_id present")
                .to_owned()
        })
        .collect();
    assert_eq!(
        2,
        invocation_ids.len(),
        "sequential invocation_ids must be unique"
    );
}

#[test]
fn concurrent_invocations_produce_separate_non_overlapping_records() {
    let dir = TempDir::new().unwrap();

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let path = dir.path().to_path_buf();
            std::thread::spawn(move || {
                bwatch_command()
                    .env("BSUITE_TRANSCRIPT_DIR", &path)
                    .args(["poll", "--category", "sprint-conflict"])
                    .assert()
                    .code(ExitCode::Finding.as_i32());
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("thread completed");
    }

    let files = collect_transcript_files(&dir);
    assert_eq!(
        4,
        files.len(),
        "each concurrent invocation needs its own file"
    );

    let invocation_ids: std::collections::BTreeSet<String> = files
        .iter()
        .map(|f| {
            let record = read_transcript_record(f);
            record["invocation_id"]
                .as_str()
                .expect("invocation_id present")
                .to_owned()
        })
        .collect();

    assert_eq!(4, invocation_ids.len(), "invocation_ids must be unique");
}

#[test]
fn transcript_record_host_context_defaults_to_l2a_when_env_var_absent() {
    let dir = TempDir::new().unwrap();
    bwatch_command()
        .env("BSUITE_TRANSCRIPT_DIR", dir.path())
        .env_remove("BSUITE_HOST_CONTEXT")
        .args(["poll", "--category", "sprint-conflict"])
        .assert()
        .code(ExitCode::Finding.as_i32());

    let files = collect_transcript_files(&dir);
    assert_eq!(1, files.len(), "expected exactly one transcript file");
    let record = read_transcript_record(&files[0]);
    assert_eq!(
        record["host_context"].as_str(),
        Some("l2a"),
        "host_context must default to l2a when BSUITE_HOST_CONTEXT is not set"
    );
}

#[test]
fn transcript_record_host_context_reflects_bsuite_host_context_env_var() {
    let non_l2a_cases = [
        ("payload-v3", "payload-v3"),
        ("strapi-v5", "strapi-v5"),
        ("sanity-v3", "sanity-v3"),
        ("directus-v10", "directus-v10"),
    ];

    for (host_id, expected_context) in non_l2a_cases {
        let dir = TempDir::new().unwrap();
        let ctx = serde_json::json!({
            "host_id": host_id,
            "host_version": "1.0.0",
            "cycle_id": "01JXXXXXXXXXXXXXXXXXXXXXXXXX",
            "directive_field": "_bsuiteDirective",
            "document_id": "doc-1",
            "collection": "posts"
        })
        .to_string();

        bwatch_command()
            .env("BSUITE_TRANSCRIPT_DIR", dir.path())
            .env("BSUITE_HOST_CONTEXT", &ctx)
            .args(["poll", "--category", "sprint-conflict"])
            .assert()
            .code(ExitCode::Finding.as_i32());

        let files = collect_transcript_files(&dir);
        assert_eq!(1, files.len(), "{host_id}: expected one transcript file");
        let record = read_transcript_record(&files[0]);

        assert_eq!(
            record["host_context"].as_str(),
            Some(expected_context),
            "{host_id}: host_context in transcript must match the host_id from BSUITE_HOST_CONTEXT"
        );

        let context_tag = record["additional_fields"]["context_tag"].as_str();
        assert!(
            context_tag.is_some(),
            "{host_id}: context_tag must be present in additional_fields when BSUITE_HOST_CONTEXT is set"
        );
        let tag = context_tag.unwrap();
        assert!(
            tag.contains(&format!("host:{expected_context}")),
            "{host_id}: context_tag must embed the host id, got {tag:?}"
        );
        assert!(
            tag.contains("version:1.0.0"),
            "{host_id}: context_tag must embed the host version, got {tag:?}"
        );
    }
}

#[test]
fn transcript_record_additional_fields_is_empty_object_when_no_invocation_context() {
    let dir = TempDir::new().unwrap();
    bwatch_command()
        .env("BSUITE_TRANSCRIPT_DIR", dir.path())
        .env_remove("BSUITE_HOST_CONTEXT")
        .args(["poll", "--category", "runbook-update"])
        .assert()
        .code(ExitCode::Finding.as_i32());

    let files = collect_transcript_files(&dir);
    assert_eq!(1, files.len());
    let record = read_transcript_record(&files[0]);

    assert_eq!(
        record["additional_fields"],
        Value::Object(serde_json::Map::new()),
        "additional_fields must be an empty object when BSUITE_HOST_CONTEXT is absent"
    );
}
