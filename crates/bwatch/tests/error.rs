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
    assert_error_exit_code(
        BwatchError::Usage("bad arguments".to_owned()),
        ExitCode::Usage,
    );
}

#[test]
fn domain_errors_map_to_internal_error_exit_code() {
    for error in [
        BwatchError::SourceMalformed("bad source".to_owned()),
        BwatchError::MissionMalformed("bad mission".to_owned()),
        BwatchError::TaxonomyUnknown("bad category".to_owned()),
        BwatchError::OutwardStateUnknown("bad state".to_owned()),
        BwatchError::SubstrateUnknown("bad substrate".to_owned()),
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
