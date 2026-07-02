mod common;

use assert_cmd::cargo::cargo_bin;
use bsuite_core::ExitCode;
use common::bwatch_command;
use common::test_server::TestServer;
use predicates::prelude::*;
use std::collections::BTreeSet;

fn install_dir_entries() -> BTreeSet<(std::ffi::OsString, u64)> {
    let dir = cargo_bin("bwatch")
        .parent()
        .expect("binary must have a parent directory")
        .to_path_buf();
    std::fs::read_dir(&dir)
        .expect("install dir must be readable")
        .filter_map(|e| e.ok())
        .map(|e| {
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            (e.file_name(), size)
        })
        .collect()
}

#[test]
fn update_subcommand_exists_as_a_clap_subcommand() {
    let output = bwatch_command()
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("update"));
}

#[test]
fn update_subcommand_uses_env_var_for_base_url() {
    let server = TestServer::start(404, "not found");

    bwatch_command()
        .env("BSUITE_UPDATE_BASE_URL", server.base_url())
        .arg("update")
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .stderr(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("update check failed"));

    assert_eq!(server.hit_count("/manifest.json"), 1);
}

#[test]
fn update_subcommand_uses_placeholder_url_when_env_var_absent() {
    bwatch_command()
        .env_remove("BSUITE_UPDATE_BASE_URL")
        .arg("update")
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .stderr(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("update check failed"));
}

#[test]
fn update_subcommand_fails_with_internal_error_on_server_error_response() {
    let server = TestServer::start(500, "internal server error");

    bwatch_command()
        .env("BSUITE_UPDATE_BASE_URL", server.base_url())
        .arg("update")
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .stderr(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("update check failed"));

    assert!(
        server.hit_count("/manifest.json") >= 1,
        "manifest endpoint must have been contacted at least once"
    );
}

#[test]
fn update_subcommand_fails_when_manifest_signature_verification_fails() {
    let manifest_json = serde_json::json!({
        "schema_version": 1,
        "binary_name": "bwatch",
        "version": "0.1.0",
        "release_at": "2025-01-01T00:00:00Z",
        "platforms": {
            "linux-x86_64": { "archive_url": "https://example.com/bwatch.tar", "sha256": "abc" }
        },
        "corpus_version": 1,
        "obfuscation_tier": "none",
        "signing_key_id": "fixture-key"
    });
    let manifest_body = serde_json::to_string(&manifest_json).unwrap();
    let server = TestServer::start(200, Box::leak(manifest_body.into_boxed_str()));
    let before = install_dir_entries();

    bwatch_command()
        .env("BSUITE_UPDATE_BASE_URL", server.base_url())
        .arg("update")
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .stderr(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("update check failed"));

    let after = install_dir_entries();
    assert_eq!(
        before, after,
        "install directory must be byte-identical before and after a manifest signature verification failure"
    );
}

#[test]
fn update_subcommand_leaves_no_partial_install_on_any_http_failure() {
    for &status in &[404u16, 500u16] {
        let server = TestServer::start(status, "failure");
        let before = install_dir_entries();

        bwatch_command()
            .env("BSUITE_UPDATE_BASE_URL", server.base_url())
            .arg("update")
            .assert()
            .code(ExitCode::InternalError.as_i32())
            .stderr(predicate::str::is_empty().not())
            .stderr(predicate::str::contains("update check failed"));

        let after = install_dir_entries();
        assert_eq!(
            before, after,
            "install directory must be byte-identical before and after a failed HTTP {status} response"
        );
    }
}

#[test]
fn update_subcommand_leaves_no_partial_install_on_unreachable_host() {
    let before = install_dir_entries();

    bwatch_command()
        .env_remove("BSUITE_UPDATE_BASE_URL")
        .arg("update")
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .stderr(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("update check failed"));

    let after = install_dir_entries();
    assert_eq!(
        before, after,
        "install directory must be byte-identical before and after an unreachable-host failure"
    );
}

#[test]
fn update_subcommand_leaves_no_partial_install_on_malformed_manifest_body() {
    let server = TestServer::start(200, "this is not json {{{");
    let before = install_dir_entries();

    bwatch_command()
        .env("BSUITE_UPDATE_BASE_URL", server.base_url())
        .arg("update")
        .assert()
        .code(ExitCode::InternalError.as_i32())
        .stderr(predicate::str::is_empty().not());

    let after = install_dir_entries();
    assert_eq!(
        before, after,
        "install directory must be byte-identical before and after a malformed-body failure"
    );
}
