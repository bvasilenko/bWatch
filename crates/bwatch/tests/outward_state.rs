mod common;

use bsuite_core::ExitCode;
use bwatch::OutwardSourceState;
use common::{assert_public_name_contract, assert_rejects};
use proptest::prelude::*;
use std::str::FromStr;

proptest! {
    #[test]
    fn outward_state_round_trip(index in 0usize..OutwardSourceState::ALL.len()) {
        let state = OutwardSourceState::ALL[index];
        let parsed = OutwardSourceState::from_str(&state.to_string()).expect("state must parse");
        prop_assert_eq!(state, parsed);
    }
}

#[test]
fn outward_state_names_cover_exact_closed_set() {
    assert_eq!(4, OutwardSourceState::ALL.len());
    assert_public_name_contract(&OutwardSourceState::ALL);
}

#[test]
fn outward_state_stable_names_are_pinned() {
    let cases = [
        (OutwardSourceState::Actionable, "actionable"),
        (OutwardSourceState::Informational, "informational"),
        (OutwardSourceState::WrongTime, "wrong-time"),
        (OutwardSourceState::Misfire, "misfire"),
    ];

    assert_eq!(OutwardSourceState::ALL.len(), cases.len());

    for (variant, expected_name) in cases {
        assert_eq!(expected_name, variant.to_string());
        assert_eq!(expected_name, variant.stable_name());
    }
}

#[test]
fn outward_state_exit_codes_match_contract() {
    let cases = [
        (OutwardSourceState::Actionable, ExitCode::Finding),
        (OutwardSourceState::Informational, ExitCode::Success),
        (OutwardSourceState::WrongTime, ExitCode::Finding),
        (OutwardSourceState::Misfire, ExitCode::Success),
    ];

    assert_eq!(OutwardSourceState::ALL.len(), cases.len());

    for (state, expected_exit_code) in cases {
        let exit_code: ExitCode = state.into();

        assert_eq!(
            expected_exit_code, exit_code,
            "{state:?} maps to wrong exit code"
        );
        assert_eq!(expected_exit_code.as_i32(), exit_code.as_i32());
    }
}

#[test]
fn outward_state_rejects_names_outside_closed_set() {
    assert_rejects::<OutwardSourceState>(&[
        "",
        "Actionable",
        "actionable ",
        " actionable",
        "wrong_time",
        "WrongTime",
        "finding",
    ]);
}
