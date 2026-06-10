use crate::BwatchError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutwardSourceSubstrate {
    GitlabCe,
    Github,
    Linear,
    Jira,
    Slack,
    Notion,
    PayloadMarketplace,
}

impl OutwardSourceSubstrate {
    pub const ALL: [Self; 7] = [
        Self::GitlabCe,
        Self::Github,
        Self::Linear,
        Self::Jira,
        Self::Slack,
        Self::Notion,
        Self::PayloadMarketplace,
    ];

    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::GitlabCe => "gitlab-ce",
            Self::Github => "github",
            Self::Linear => "linear",
            Self::Jira => "jira",
            Self::Slack => "slack",
            Self::Notion => "notion",
            Self::PayloadMarketplace => "payload-marketplace",
        }
    }

    fn from_host_substring(host: &str) -> Option<Self> {
        let host = host.to_ascii_lowercase();
        if host.contains("gitlab") {
            Some(Self::GitlabCe)
        } else if host.contains("github") {
            Some(Self::Github)
        } else if host.contains("linear") {
            Some(Self::Linear)
        } else if host.contains("atlassian") || host.contains("jira") {
            Some(Self::Jira)
        } else if host.contains("slack") {
            Some(Self::Slack)
        } else if host.contains("notion") {
            Some(Self::Notion)
        } else if host.contains("payload") {
            Some(Self::PayloadMarketplace)
        } else {
            None
        }
    }
}

impl fmt::Display for OutwardSourceSubstrate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.stable_name())
    }
}

impl FromStr for OutwardSourceSubstrate {
    type Err = BwatchError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "gitlab-ce" => return Ok(Self::GitlabCe),
            "github" => return Ok(Self::Github),
            "linear" => return Ok(Self::Linear),
            "jira" => return Ok(Self::Jira),
            "slack" => return Ok(Self::Slack),
            "notion" => return Ok(Self::Notion),
            "payload-marketplace" => return Ok(Self::PayloadMarketplace),
            _ => {}
        }

        if let Some(substrate) = extract_host(value).and_then(|h| Self::from_host_substring(&h)) {
            return Ok(substrate);
        }

        Err(BwatchError::SubstrateUnknown(value.to_owned()))
    }
}

fn extract_host(value: &str) -> Option<String> {
    if let Some(after_scheme) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    {
        let host = after_scheme.split('/').next()?.split(':').next()?;
        return if host.is_empty() {
            None
        } else {
            Some(host.to_owned())
        };
    }

    // PascalCase variant names and whitespace-padded inputs contain no dot, preventing false host-substring matches.
    if value.contains('.') && !value.contains(char::is_whitespace) {
        let host = value.split('/').next()?.split(':').next()?;
        if !host.is_empty() {
            return Some(host.to_owned());
        }
    }

    None
}
