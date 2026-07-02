use bwatch::{
    BwatchCli, BwatchError, Cmd, FindingCategory, OutwardSourceState, OutwardSourceSubstrate,
    PollArgs, routing_key,
};

#[test]
fn library_reexports_public_contract_types() {
    assert_eq!(9, FindingCategory::ALL.len());
    assert_eq!(4, OutwardSourceState::ALL.len());
    assert_eq!(7, OutwardSourceSubstrate::ALL.len());
}

#[test]
fn library_reexports_cli_surface_types() {
    let _error: BwatchError = BwatchError::Usage("test".to_owned());
    let _poll_args = PollArgs {
        category: "sprint-conflict".to_owned(),
        source: None,
        mission: None,
        manifest: None,
        json: false,
        quiet: false,
        reason: None,
    };
    let _cmd: fn(Cmd) = |_| {};
    let _cli: fn(BwatchCli) = |_| {};
}

#[test]
fn routing_key_uses_bwatch_core_entry_point() {
    assert_eq!(bsuite_core::RoutingKey::bwatch(), routing_key());
}

#[test]
fn routing_key_stable_name_is_bwatch() {
    assert_eq!("bwatch", routing_key().stable_name());
}

#[test]
fn substrate_input_module_is_accessible_through_public_path() {
    let input = bwatch::substrate_input::SubstrateInput::new(
        Some("github".to_owned()),
        Some("v0.1".to_owned()),
    )
    .expect("valid inputs construct successfully");

    assert_eq!(Some("github".to_owned()), input.source);
    assert_eq!(Some("v0.1".to_owned()), input.mission);
}
