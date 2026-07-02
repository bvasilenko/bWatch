use bsuite_core::{BsuiteCoreError, ExitCode};
use bwatch::BwatchError;
use bwatch::error::process_exit_code;

fn assert_error_exit_code(error: BwatchError, expected: ExitCode) {
    assert_eq!(expected, error.exit_code());
    assert_eq!(
        std::process::ExitCode::from(expected.as_i32() as u8),
        error.process_exit_code()
    );
}

#[test]
fn usage_errors_map_to_usage_exit_code() {
    for error in [
        BwatchError::Usage("bad arguments".to_owned()),
        BwatchError::TaxonomyUnknown("bad category".to_owned()),
    ] {
        assert_error_exit_code(error, ExitCode::Usage);
    }
}

#[test]
fn domain_errors_map_to_internal_error_exit_code() {
    for error in [
        BwatchError::SourceMalformed("bad source".to_owned()),
        BwatchError::MissionMalformed("bad mission".to_owned()),
        BwatchError::OutwardStateUnknown("bad state".to_owned()),
        BwatchError::SubstrateUnknown("bad substrate".to_owned()),
        BwatchError::CorpusLoad("corpus load failure".to_owned()),
        BwatchError::Core(BsuiteCoreError::PromptResolution("bad prompt".to_owned())),
        BwatchError::Core(BsuiteCoreError::Update("update failure".to_owned())),
        BwatchError::Core(BsuiteCoreError::Transcript("transcript failure".to_owned())),
        BwatchError::Core(BsuiteCoreError::ExitCode("exit code failure".to_owned())),
        BwatchError::Core(BsuiteCoreError::VisibilityEvidence(
            "visibility failure".to_owned(),
        )),
        BwatchError::Core(BsuiteCoreError::AdapterHostBinding(
            "binding failure".to_owned(),
        )),
    ] {
        assert_error_exit_code(error, ExitCode::InternalError);
    }
}

#[test]
fn process_exit_code_maps_all_variants_to_raw_codes() {
    for exit_code in ExitCode::ALL {
        let expected_raw = exit_code.as_i32() as u8;
        assert_eq!(
            std::process::ExitCode::from(expected_raw),
            process_exit_code(exit_code),
            "{exit_code:?} must produce process exit code {expected_raw}"
        );
    }
}

#[test]
fn into_core_passes_through_bsuite_core_error_unchanged() {
    let original = BsuiteCoreError::Update("network unavailable".to_owned());
    let converted = BwatchError::Core(original.clone()).into_core();
    assert_eq!(
        format!("{original:?}"),
        format!("{converted:?}"),
        "BwatchError::Core wrapping must unwrap to the original BsuiteCoreError"
    );
}

#[test]
fn into_core_maps_corpus_load_to_deserialization_failed() {
    let msg = "bad corpus bytes".to_owned();
    let converted = BwatchError::CorpusLoad(msg.clone()).into_core();
    assert!(
        matches!(converted, BsuiteCoreError::CorpusDeserializationFailed(ref m) if *m == msg),
        "CorpusLoad must map to CorpusDeserializationFailed, got: {converted:?}"
    );
}

#[test]
fn into_core_maps_all_non_core_domain_variants_to_prompt_resolution() {
    let domain_errors = [
        BwatchError::SourceMalformed("src".to_owned()),
        BwatchError::MissionMalformed("mis".to_owned()),
        BwatchError::TaxonomyUnknown("tax".to_owned()),
        BwatchError::OutwardStateUnknown("state".to_owned()),
        BwatchError::SubstrateUnknown("sub".to_owned()),
        BwatchError::Usage("usage".to_owned()),
    ];

    for error in domain_errors {
        let display = error.to_string();
        let converted = error.into_core();
        assert!(
            matches!(converted, BsuiteCoreError::PromptResolution(ref m) if *m == display),
            "domain variant must map to PromptResolution carrying the Display string: {converted:?}"
        );
    }
}

#[test]
fn is_malformed_input_returns_true_only_for_usage_discriminating_variants() {
    let malformed = [
        BwatchError::TaxonomyUnknown("bad-category".to_owned()),
        BwatchError::Usage("bad argument shape".to_owned()),
    ];
    let not_malformed = [
        BwatchError::SourceMalformed("empty source".to_owned()),
        BwatchError::MissionMalformed("empty mission".to_owned()),
        BwatchError::OutwardStateUnknown("unknown state".to_owned()),
        BwatchError::SubstrateUnknown("unknown substrate".to_owned()),
        BwatchError::CorpusLoad("load failure".to_owned()),
        BwatchError::Core(BsuiteCoreError::PromptResolution(
            "resolution error".to_owned(),
        )),
    ];

    for error in malformed {
        assert!(
            error.is_malformed_input(),
            "{error:?} must report is_malformed_input = true"
        );
    }
    for error in not_malformed {
        assert!(
            !error.is_malformed_input(),
            "{error:?} must report is_malformed_input = false"
        );
    }
}
