use bsuite_core::BsuiteCoreError;
use bwatch::{FindingCategory, corpus_index::FindingCategoryCorpusIndex};
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::collections::BTreeSet;

const CORPUS_TOML: &str = include_str!("../corpus/bwatch-v0.toml");
const PUBKEY_BYTES: &[u8] = include_bytes!("../corpus/bwatch-v0-pubkey.bin");

fn load_verifying_key() -> VerifyingKey {
    let bytes: [u8; 32] = PUBKEY_BYTES.try_into().expect("pubkey is 32 bytes");
    VerifyingKey::from_bytes(&bytes).expect("pubkey is valid")
}

fn load_corpus() -> FindingCategoryCorpusIndex {
    FindingCategoryCorpusIndex::from_toml_signed(CORPUS_TOML, &load_verifying_key())
        .expect("fixture corpus loads cleanly")
}

fn with_signature(corpus: &str, new_value: &str) -> String {
    let sig_line = corpus
        .lines()
        .find(|l| l.starts_with("signature = "))
        .expect("corpus has signature line");
    corpus.replacen(sig_line, &format!("signature = \"{new_value}\""), 1)
}

fn remove_entry(corpus: &str, category: &str) -> String {
    let mut sections = corpus.split("\n[[entries]]");
    let preamble = sections.next().expect("corpus has preamble");
    let mut output = preamble.to_owned();
    for section in sections {
        if !section.contains(&format!("finding_category = \"{category}\"")) {
            output.push_str("\n[[entries]]");
            output.push_str(section);
        }
    }
    output
}

#[test]
fn all_finding_categories_are_indexed_with_distinct_non_empty_directives() {
    let corpus = load_corpus();
    let mut seen = BTreeSet::new();
    for category in FindingCategory::ALL {
        let directive = corpus.resolve(category);
        let content = directive.as_str();
        assert!(!content.is_empty(), "empty directive for {category}");
        let expected_prefix = format!("FINDING-DETECTED: {}.", category.stable_name());
        assert!(
            content.starts_with(&expected_prefix),
            "directive for {category} must start with canonical prefix {expected_prefix:?}"
        );
        assert!(
            seen.insert(content.to_owned()),
            "duplicate directive for {category}"
        );
    }
    assert_eq!(FindingCategory::ALL.len(), seen.len());
}

#[test]
fn corpus_rejects_wrong_pubkey() {
    let wrong_seed = [0x00u8; 32];
    let wrong_pubkey = SigningKey::from_bytes(&wrong_seed).verifying_key();
    let result = FindingCategoryCorpusIndex::from_toml_signed(CORPUS_TOML, &wrong_pubkey);
    assert!(result.is_err(), "wrong-pubkey corpus must be rejected");
    assert!(
        matches!(
            result.unwrap_err(),
            bwatch::BwatchError::Core(BsuiteCoreError::CorpusSignatureInvalid)
        ),
        "error kind must be CorpusSignatureInvalid"
    );
}

#[test]
fn corpus_rejects_tampered_directive_content() {
    let corpus = load_corpus();
    let first_directive = corpus.resolve(FindingCategory::ALL[0]).as_str().to_owned();
    let first_word = first_directive
        .split_whitespace()
        .next()
        .expect("directive has at least one word");

    let tampered = CORPUS_TOML.replacen(first_word, "TAMPERED", 1);
    let result = FindingCategoryCorpusIndex::from_toml_signed(&tampered, &load_verifying_key());
    assert!(result.is_err(), "tampered directive must be rejected");
    assert!(
        matches!(
            result.unwrap_err(),
            bwatch::BwatchError::Core(BsuiteCoreError::CorpusSignatureInvalid)
        ),
        "tamper detection must be CorpusSignatureInvalid"
    );
}

#[test]
fn corpus_index_rejects_duplicate_finding_category() {
    let first = FindingCategory::ALL[0].stable_name();
    let second = FindingCategory::ALL[1].stable_name();
    let with_duplicate = CORPUS_TOML.replacen(
        &format!("finding_category = \"{second}\""),
        &format!("finding_category = \"{first}\""),
        1,
    );
    let result =
        FindingCategoryCorpusIndex::from_toml_signed(&with_duplicate, &load_verifying_key());
    assert!(result.is_err(), "duplicate entry must be rejected");
    assert!(matches!(
        result.unwrap_err(),
        bwatch::BwatchError::CorpusLoad(_)
    ));
}

#[test]
fn corpus_index_rejects_unknown_finding_category_string() {
    let invalid_names = [
        "not-a-valid-type",
        "",
        "SprintConflict",
        " sprint-conflict",
        "sprint-conflict ",
        "sprint_conflict",
        "42",
        "sprint",
    ];
    for name in invalid_names {
        let mutated = CORPUS_TOML.replacen(
            &format!(
                "finding_category = \"{}\"",
                FindingCategory::ALL[0].stable_name()
            ),
            &format!("finding_category = \"{name}\""),
            1,
        );
        let result = FindingCategoryCorpusIndex::from_toml_signed(&mutated, &load_verifying_key());
        assert!(result.is_err(), "invalid name {name:?} must be rejected");
    }
}

#[test]
fn corpus_index_rejects_every_individually_missing_category() {
    for category in FindingCategory::ALL {
        let without = remove_entry(CORPUS_TOML, category.stable_name());
        let result = FindingCategoryCorpusIndex::from_toml_signed(&without, &load_verifying_key());
        assert!(
            result.is_err(),
            "corpus missing {category} must be rejected"
        );
        assert!(
            matches!(
                result.unwrap_err(),
                bwatch::BwatchError::Core(BsuiteCoreError::CorpusSignatureInvalid)
            ),
            "removing a signed entry must fail at signature verification"
        );
    }
}

#[test]
fn corpus_rejects_unsigned_appended_entry() {
    let extra = "\n[[entries]]\nrouting_key = \"bwatch\"\nfinding_category = \"sprint-conflict\"\ndirective = \"Extra.\"\n[entries.provenance]\nrun_id = \"x\"\niteration = 0\nobservation_source = \"x\"\npre_compliance = 0.0\npost_compliance = 0.0\n";
    let with_extra = format!("{CORPUS_TOML}{extra}");
    let result = FindingCategoryCorpusIndex::from_toml_signed(&with_extra, &load_verifying_key());
    assert!(result.is_err(), "unsigned appended entry must be rejected");
    assert!(matches!(
        result.unwrap_err(),
        bwatch::BwatchError::Core(BsuiteCoreError::CorpusSignatureInvalid)
    ));
}

#[test]
fn corpus_rejects_signature_with_wrong_algorithm_prefix() {
    let mutated = with_signature(
        CORPUS_TOML,
        "wrong-algo:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
    );
    let result = FindingCategoryCorpusIndex::from_toml_signed(&mutated, &load_verifying_key());
    assert!(
        result.is_err(),
        "signature with an unknown algorithm prefix must be rejected"
    );
}

#[test]
fn corpus_rejects_signature_with_non_base64_body() {
    let mutated = with_signature(CORPUS_TOML, "ed25519:NOT!VALID!BASE64!CHARACTERS!@#$%^");
    let result = FindingCategoryCorpusIndex::from_toml_signed(&mutated, &load_verifying_key());
    assert!(
        result.is_err(),
        "signature whose body contains non-base64 characters must be rejected"
    );
}

#[test]
fn corpus_rejects_signature_with_wrong_decoded_byte_length() {
    // "AAAAAAAA" is valid base64 (8 chars -> 6 bytes); Ed25519 requires exactly 64 bytes.
    let mutated = with_signature(CORPUS_TOML, "ed25519:AAAAAAAA");
    let result = FindingCategoryCorpusIndex::from_toml_signed(&mutated, &load_verifying_key());
    assert!(
        result.is_err(),
        "signature with the wrong decoded byte length must be rejected"
    );
}

#[test]
fn all_directives_contain_substantive_multi_sentence_guidance() {
    let corpus = load_corpus();
    for category in FindingCategory::ALL {
        let directive = corpus.resolve(category);
        let text = directive.as_str();

        let sentence_count = text
            .split(". ")
            .count()
            .saturating_sub(1)
            .max(text.ends_with('.') as usize);

        assert!(
            sentence_count >= 2,
            "directive for {category} must contain at least 2 sentences (found ~{sentence_count}): {text:?}"
        );

        assert!(
            text.len() >= 80,
            "directive for {category} must be at least 80 characters of substantive guidance, got {} chars",
            text.len()
        );
    }
}
