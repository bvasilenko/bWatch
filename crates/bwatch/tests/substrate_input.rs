use bwatch::{BwatchError, substrate_input::SubstrateInput};

#[test]
fn substrate_input_accepts_every_supported_presence_combination() {
    for (source, mission) in [
        (None, None),
        (Some("github".to_owned()), None),
        (None, Some("v0.1".to_owned())),
        (
            Some("https://github.com/org/repo".to_owned()),
            Some("v0.1".to_owned()),
        ),
    ] {
        let input = SubstrateInput::new(source.clone(), mission.clone()).expect("input is valid");

        assert_eq!(source, input.source);
        assert_eq!(mission, input.mission);
    }
}

#[test]
fn substrate_input_accepts_whitespace_only_fields() {
    for (source, mission) in [
        (Some(" ".to_owned()), None),
        (None, Some("  ".to_owned())),
        (Some("\t".to_owned()), Some("\n".to_owned())),
    ] {
        let input = SubstrateInput::new(source.clone(), mission.clone())
            .expect("whitespace-only is a valid non-empty string");

        assert_eq!(source, input.source);
        assert_eq!(mission, input.mission);
    }
}

#[test]
fn substrate_input_rejects_empty_source() {
    let error = SubstrateInput::new(Some(String::new()), None).expect_err("empty source rejected");

    assert!(matches!(error, BwatchError::SourceMalformed(_)));
}

#[test]
fn substrate_input_source_error_takes_priority_when_both_fields_empty() {
    let error = SubstrateInput::new(Some(String::new()), Some(String::new()))
        .expect_err("both empty fields are rejected");

    assert!(
        matches!(error, BwatchError::SourceMalformed(_)),
        "source guard fires before mission guard: {error:?}"
    );
}

#[test]
fn substrate_input_rejects_empty_mission() {
    let error = SubstrateInput::new(None, Some(String::new())).expect_err("empty mission rejected");

    assert!(matches!(error, BwatchError::MissionMalformed(_)));
}
