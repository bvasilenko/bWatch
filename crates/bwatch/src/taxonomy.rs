use crate::BwatchError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingCategory {
    SprintConflict,
    CollaboratorTookIssue,
    CollaboratorFinishedRelatedWork,
    ExternalSpecChange,
    RunbookUpdate,
    CmsPluginMarketplaceRelease,
    CompetitorProductLaunch,
    EditorialWorkflowSignal,
    AgentSkillPublishedWithVulnerability,
}

impl FindingCategory {
    pub const ALL: [Self; 9] = [
        Self::SprintConflict,
        Self::CollaboratorTookIssue,
        Self::CollaboratorFinishedRelatedWork,
        Self::ExternalSpecChange,
        Self::RunbookUpdate,
        Self::CmsPluginMarketplaceRelease,
        Self::CompetitorProductLaunch,
        Self::EditorialWorkflowSignal,
        Self::AgentSkillPublishedWithVulnerability,
    ];

    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::SprintConflict => "sprint-conflict",
            Self::CollaboratorTookIssue => "collaborator-took-issue",
            Self::CollaboratorFinishedRelatedWork => "collaborator-finished-related-work",
            Self::ExternalSpecChange => "external-spec-change",
            Self::RunbookUpdate => "runbook-update",
            Self::CmsPluginMarketplaceRelease => "cms-plugin-marketplace-release",
            Self::CompetitorProductLaunch => "competitor-product-launch",
            Self::EditorialWorkflowSignal => "editorial-workflow-signal",
            Self::AgentSkillPublishedWithVulnerability => {
                "agent-skill-published-with-vulnerability"
            }
        }
    }
}

impl fmt::Display for FindingCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.stable_name())
    }
}

impl FromStr for FindingCategory {
    type Err = BwatchError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "sprint-conflict" => Ok(Self::SprintConflict),
            "collaborator-took-issue" => Ok(Self::CollaboratorTookIssue),
            "collaborator-finished-related-work" => Ok(Self::CollaboratorFinishedRelatedWork),
            "external-spec-change" => Ok(Self::ExternalSpecChange),
            "runbook-update" => Ok(Self::RunbookUpdate),
            "cms-plugin-marketplace-release" => Ok(Self::CmsPluginMarketplaceRelease),
            "competitor-product-launch" => Ok(Self::CompetitorProductLaunch),
            "editorial-workflow-signal" => Ok(Self::EditorialWorkflowSignal),
            "agent-skill-published-with-vulnerability" => {
                Ok(Self::AgentSkillPublishedWithVulnerability)
            }
            _ => Err(BwatchError::TaxonomyUnknown(value.to_owned())),
        }
    }
}
