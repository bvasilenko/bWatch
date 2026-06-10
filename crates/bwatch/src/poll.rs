use crate::{
    BwatchError, FindingCategory, OutwardSourceState, PollArgs, error::process_exit_code,
    substrate_input::SubstrateInput,
};
use std::fmt;

const BINARY_NAME: &str = "bwatch";
const PLACEHOLDER_CATEGORY: FindingCategory = FindingCategory::SprintConflict;
const PLACEHOLDER_STATE: OutwardSourceState = OutwardSourceState::Actionable;

pub fn run(args: PollArgs) -> Result<std::process::ExitCode, BwatchError> {
    let input = SubstrateInput::new(args.source, args.mission)?;
    validate_reason(args.reason.as_deref())?;

    println!("{}", PlaceholderDirective::new(input));

    Ok(process_exit_code(PLACEHOLDER_STATE.into()))
}

fn validate_reason(reason: Option<&str>) -> Result<(), BwatchError> {
    if matches!(reason, Some(r) if r.trim().is_empty()) {
        return Err(BwatchError::Usage("reason must not be empty".to_owned()));
    }
    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PlaceholderDirective {
    input: SubstrateInput,
    category: FindingCategory,
    state: OutwardSourceState,
}

impl PlaceholderDirective {
    fn new(input: SubstrateInput) -> Self {
        Self {
            input,
            category: PLACEHOLDER_CATEGORY,
            state: PLACEHOLDER_STATE,
        }
    }
}

impl fmt::Display for PlaceholderDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "[{BINARY_NAME} placeholder directive - pre-corpus output]"
        )?;
        writeln!(
            f,
            "Parsed input: source={}, mission={}. Routing key: FindingCategory::{:?} placeholder route. Invocation surface: cli. Outward-source-state: {:?}.",
            format_optional(self.input.source.as_deref()),
            format_optional(self.input.mission.as_deref()),
            self.category,
            self.state,
        )?;
        writeln!(
            f,
            "ACTION: This invocation reached {BINARY_NAME} at the pre-corpus phase. A real evolved directive would name the specific finding category {:?} the external tracker source surfaced and steer the calling LLM toward either coordinating with the upstream collaborator, capturing the runbook update, or deferring the response to a better moment.",
            self.category,
        )?;
        write!(
            f,
            "Re-invoke after the corpus-backed release lands. Do not treat this placeholder as ground truth. Exit code carries the verdict-class signal."
        )
    }
}

fn format_optional(value: Option<&str>) -> &str {
    value.unwrap_or("<none>")
}
