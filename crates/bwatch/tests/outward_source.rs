mod common;

use bwatch::OutwardSourceSubstrate;
use common::{assert_public_name_contract, assert_rejects};
use proptest::prelude::*;
use std::str::FromStr;

#[test]
fn substrate_names_cover_exact_closed_set() {
    assert_eq!(7, OutwardSourceSubstrate::ALL.len());
    assert_public_name_contract(&OutwardSourceSubstrate::ALL);
}

#[test]
fn substrate_stable_names_are_pinned() {
    let cases = [
        (OutwardSourceSubstrate::GitlabCe, "gitlab-ce"),
        (OutwardSourceSubstrate::Github, "github"),
        (OutwardSourceSubstrate::Linear, "linear"),
        (OutwardSourceSubstrate::Jira, "jira"),
        (OutwardSourceSubstrate::Slack, "slack"),
        (OutwardSourceSubstrate::Notion, "notion"),
        (
            OutwardSourceSubstrate::PayloadMarketplace,
            "payload-marketplace",
        ),
    ];

    assert_eq!(OutwardSourceSubstrate::ALL.len(), cases.len());

    for (variant, expected_name) in cases {
        assert_eq!(expected_name, variant.to_string());
        assert_eq!(expected_name, variant.stable_name());
    }
}

#[test]
fn substrate_kebab_routes_resolve_exact_variants() {
    let cases = [
        ("gitlab-ce", OutwardSourceSubstrate::GitlabCe),
        ("github", OutwardSourceSubstrate::Github),
        ("linear", OutwardSourceSubstrate::Linear),
        ("jira", OutwardSourceSubstrate::Jira),
        ("slack", OutwardSourceSubstrate::Slack),
        ("notion", OutwardSourceSubstrate::Notion),
        (
            "payload-marketplace",
            OutwardSourceSubstrate::PayloadMarketplace,
        ),
    ];

    assert_eq!(OutwardSourceSubstrate::ALL.len(), cases.len());

    for (name, expected) in cases {
        let parsed = OutwardSourceSubstrate::from_str(name)
            .unwrap_or_else(|_| panic!("kebab name {name:?} must parse"));
        assert_eq!(expected, parsed);
    }
}

#[test]
fn substrate_url_routes_resolve_by_host_substring() {
    let cases = [
        (
            "https://github.com/org/repo",
            OutwardSourceSubstrate::Github,
        ),
        ("http://github.com/org/repo", OutwardSourceSubstrate::Github),
        (
            "https://gitlab.example.com/group/project",
            OutwardSourceSubstrate::GitlabCe,
        ),
        (
            "https://gitlab.example.com:8080/group/project",
            OutwardSourceSubstrate::GitlabCe,
        ),
        (
            "https://linear.app/team/issues/PROJ-1",
            OutwardSourceSubstrate::Linear,
        ),
        (
            "https://mycompany.atlassian.net/jira/software/projects/PROJ",
            OutwardSourceSubstrate::Jira,
        ),
        (
            "https://myworkspace.slack.com/archives/C12345",
            OutwardSourceSubstrate::Slack,
        ),
        (
            "https://www.notion.so/workspace/page-id",
            OutwardSourceSubstrate::Notion,
        ),
        (
            "https://payloadcms.com/marketplace/plugin",
            OutwardSourceSubstrate::PayloadMarketplace,
        ),
    ];

    for (url, expected) in cases {
        let parsed = OutwardSourceSubstrate::from_str(url)
            .unwrap_or_else(|_| panic!("URL {url:?} must resolve"));
        assert_eq!(expected, parsed);
    }
}

#[test]
fn substrate_plain_hostname_routes_resolve_by_host_substring() {
    let cases = [
        ("github.com", OutwardSourceSubstrate::Github),
        ("gitlab.mycompany.com", OutwardSourceSubstrate::GitlabCe),
        ("linear.app", OutwardSourceSubstrate::Linear),
        ("mycompany.atlassian.net", OutwardSourceSubstrate::Jira),
        ("myworkspace.slack.com", OutwardSourceSubstrate::Slack),
        ("www.notion.so", OutwardSourceSubstrate::Notion),
        ("payloadcms.com", OutwardSourceSubstrate::PayloadMarketplace),
    ];

    assert_eq!(OutwardSourceSubstrate::ALL.len(), cases.len());

    for (host, expected) in cases {
        let parsed = OutwardSourceSubstrate::from_str(host)
            .unwrap_or_else(|_| panic!("hostname {host:?} must resolve"));
        assert_eq!(expected, parsed);
    }
}

#[test]
fn substrate_rejects_unrecognised_names_and_urls() {
    assert_rejects::<OutwardSourceSubstrate>(&[
        "",
        "unknown-tool",
        "GitlabCe",
        "Github",
        "https://example.com/no-match",
        "https://unknown-tracker.io/project",
        "https://",
        "http://",
        "https:///path-without-host",
    ]);
}

proptest! {
    #[test]
    fn substrate_round_trip(index in 0usize..OutwardSourceSubstrate::ALL.len()) {
        let substrate = OutwardSourceSubstrate::ALL[index];
        let parsed = OutwardSourceSubstrate::from_str(&substrate.to_string())
            .expect("stable name must round-trip through FromStr");
        prop_assert_eq!(substrate, parsed);
    }
}

#[test]
fn substrate_url_routes_resolve_through_query_string_and_fragment() {
    let cases = [
        (
            "https://github.com/org/repo?tab=issues",
            OutwardSourceSubstrate::Github,
        ),
        (
            "https://github.com/org/repo#comment-42",
            OutwardSourceSubstrate::Github,
        ),
        (
            "https://gitlab.example.com/group/project?branch=main&mr=7",
            OutwardSourceSubstrate::GitlabCe,
        ),
        (
            "https://linear.app/team/issues/PROJ-1?status=done",
            OutwardSourceSubstrate::Linear,
        ),
        (
            "https://myworkspace.slack.com/archives/C12345?thread_ts=1234567890",
            OutwardSourceSubstrate::Slack,
        ),
    ];

    for (url, expected) in cases {
        let parsed = OutwardSourceSubstrate::from_str(url)
            .unwrap_or_else(|_| panic!("URL with query/fragment {url:?} must resolve by host"));
        assert_eq!(expected, parsed, "URL {url:?}");
    }
}

#[test]
fn substrate_rejects_dotless_inputs_that_are_not_kebab_names() {
    assert_rejects::<OutwardSourceSubstrate>(&[
        "github com",
        "not a url",
        "https://",
        "http://",
        " github",
        "github ",
    ]);
}

#[test]
fn substrate_plain_hostname_with_port_resolves_by_stripping_port() {
    let cases = [
        ("github.com:443", OutwardSourceSubstrate::Github),
        (
            "gitlab.mycompany.com:8080",
            OutwardSourceSubstrate::GitlabCe,
        ),
        ("linear.app:443", OutwardSourceSubstrate::Linear),
        ("mycompany.atlassian.net:8443", OutwardSourceSubstrate::Jira),
        ("myworkspace.slack.com:443", OutwardSourceSubstrate::Slack),
    ];

    for (host_with_port, expected) in cases {
        let parsed = OutwardSourceSubstrate::from_str(host_with_port)
            .unwrap_or_else(|_| panic!("plain hostname with port {host_with_port:?} must resolve"));
        assert_eq!(
            expected, parsed,
            "port must be stripped before host-substring matching for {host_with_port:?}"
        );
    }
}

#[test]
fn substrate_host_substring_priority_follows_declaration_order() {
    let cases = [
        (
            "gitlab.github.example.com",
            OutwardSourceSubstrate::GitlabCe,
        ),
        (
            "github.gitlab.example.com",
            OutwardSourceSubstrate::GitlabCe,
        ),
        ("slack.notion.example.com", OutwardSourceSubstrate::Slack),
        ("notion.slack.example.com", OutwardSourceSubstrate::Slack),
        (
            "linear.atlassian.example.com",
            OutwardSourceSubstrate::Linear,
        ),
    ];

    for (host, expected) in cases {
        let parsed = OutwardSourceSubstrate::from_str(host)
            .unwrap_or_else(|_| panic!("host {host:?} must resolve"));
        assert_eq!(
            expected, parsed,
            "host {host:?}: earlier variant in declaration order must win when multiple substrings match"
        );
    }
}
