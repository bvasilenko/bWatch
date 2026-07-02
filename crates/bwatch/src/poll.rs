use crate::{BwatchError, FindingCategory, PollArgs, corpus_index::FindingCategoryCorpusIndex};
use bsuite_core::{ExitCode, prompt_resolver::DirectiveString};

pub fn run(
    args: &PollArgs,
    corpus: &FindingCategoryCorpusIndex,
) -> Result<(DirectiveString, ExitCode), BwatchError> {
    crate::substrate_input::SubstrateInput::new(args.source.clone(), args.mission.clone())?;
    validate_reason(args.reason.as_deref())?;
    let category = args.category.parse::<FindingCategory>()?;
    let directive = corpus.resolve(category).clone();
    Ok((directive, ExitCode::Finding))
}

fn validate_reason(reason: Option<&str>) -> Result<(), BwatchError> {
    if matches!(reason, Some(r) if r.trim().is_empty()) {
        return Err(BwatchError::Usage("reason must not be empty".to_owned()));
    }
    Ok(())
}
