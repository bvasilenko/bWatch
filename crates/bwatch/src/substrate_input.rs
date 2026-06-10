use crate::BwatchError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SubstrateInput {
    pub source: Option<String>,
    pub mission: Option<String>,
}

impl SubstrateInput {
    pub fn new(source: Option<String>, mission: Option<String>) -> Result<Self, BwatchError> {
        if matches!(source.as_deref(), Some("")) {
            return Err(BwatchError::SourceMalformed(
                "source must not be empty".to_owned(),
            ));
        }

        if matches!(mission.as_deref(), Some("")) {
            return Err(BwatchError::MissionMalformed(
                "mission must not be empty".to_owned(),
            ));
        }

        Ok(Self { source, mission })
    }
}
