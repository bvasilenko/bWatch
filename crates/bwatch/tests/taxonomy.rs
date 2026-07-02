mod common;

use bwatch::FindingCategory;
use common::{assert_public_name_contract, assert_rejects};
use proptest::prelude::*;
use std::str::FromStr;

proptest! {
    #[test]
    fn category_round_trip(index in 0usize..FindingCategory::ALL.len()) {
        let category = FindingCategory::ALL[index];
        let parsed = FindingCategory::from_str(&category.to_string()).expect("category must parse");
        prop_assert_eq!(category, parsed);
    }
}

#[test]
fn category_names_cover_exact_closed_set() {
    assert_eq!(9, FindingCategory::ALL.len());
    assert_public_name_contract(&FindingCategory::ALL);
}

#[test]
fn category_stable_names_are_pinned() {
    let cases = [
        (FindingCategory::SprintConflict, "sprint-conflict"),
        (
            FindingCategory::CollaboratorTookIssue,
            "collaborator-took-issue",
        ),
        (
            FindingCategory::CollaboratorFinishedRelatedWork,
            "collaborator-finished-related-work",
        ),
        (FindingCategory::ExternalSpecChange, "external-spec-change"),
        (FindingCategory::RunbookUpdate, "runbook-update"),
        (
            FindingCategory::CmsPluginMarketplaceRelease,
            "cms-plugin-marketplace-release",
        ),
        (
            FindingCategory::CompetitorProductLaunch,
            "competitor-product-launch",
        ),
        (
            FindingCategory::EditorialWorkflowSignal,
            "editorial-workflow-signal",
        ),
        (
            FindingCategory::AgentSkillPublishedWithVulnerability,
            "agent-skill-published-with-vulnerability",
        ),
    ];

    assert_eq!(FindingCategory::ALL.len(), cases.len());

    for (variant, expected_name) in cases {
        assert_eq!(expected_name, variant.to_string());
        assert_eq!(expected_name, variant.stable_name());
    }
}

#[test]
fn category_rejects_names_outside_closed_set() {
    assert_rejects::<FindingCategory>(&[
        "unknown",
        "",
        "sprint_conflict",
        "SprintConflict",
        " sprint-conflict",
        "sprint-conflict ",
        "collaborator",
        "cms-plugin",
        "Sprint-Conflict",
        "SPRINT-CONFLICT",
        "sprint-CONFLICT",
        "Runbook-Update",
        "RUNBOOK-UPDATE",
    ]);
}

proptest! {
    #[test]
    fn category_from_str_rejects_every_string_not_in_the_closed_set(s in "\\PC*") {
        let is_member = FindingCategory::ALL.iter().any(|c| c.stable_name() == s);
        if !is_member {
            prop_assert!(
                FindingCategory::from_str(&s).is_err(),
                "from_str must reject any string that is not a stable name, but accepted {s:?}"
            );
        }
    }
}
