use crate::BwatchError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutwardSourceState {
    Actionable,
    Informational,
    WrongTime,
    Misfire,
}

impl OutwardSourceState {
    pub const ALL: [Self; 4] = [
        Self::Actionable,
        Self::Informational,
        Self::WrongTime,
        Self::Misfire,
    ];

    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::Actionable => "actionable",
            Self::Informational => "informational",
            Self::WrongTime => "wrong-time",
            Self::Misfire => "misfire",
        }
    }
}

impl fmt::Display for OutwardSourceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.stable_name())
    }
}

impl FromStr for OutwardSourceState {
    type Err = BwatchError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "actionable" => Ok(Self::Actionable),
            "informational" => Ok(Self::Informational),
            "wrong-time" => Ok(Self::WrongTime),
            "misfire" => Ok(Self::Misfire),
            _ => Err(BwatchError::OutwardStateUnknown(value.to_owned())),
        }
    }
}

impl From<OutwardSourceState> for bsuite_core::ExitCode {
    // Both states signal a finding the calling loop must read; the directive text carries the timing distinction.
    fn from(value: OutwardSourceState) -> Self {
        match value {
            OutwardSourceState::Actionable => Self::Finding,
            OutwardSourceState::Informational => Self::Success,
            OutwardSourceState::WrongTime => Self::Finding,
            OutwardSourceState::Misfire => Self::Success,
        }
    }
}
