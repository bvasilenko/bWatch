use base64::Engine;
use bsuite_core::{CorpusEntry, CorpusFile, ProvenanceRecord, RoutingKey};
use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;
use std::{fs, path::Path};

const FIXTURE_SEED: [u8; 32] = *b"bwatch-fixture-v0-seed-000000000";
const CANONICAL_KEY_ID: &str = "bwatch-fixture-v0";
const OBSERVATION_SOURCE: &str = "hand-authored-fixture-v0";
const RUN_ID: &str = "hand-authored-fixture-v0";

struct EntrySpec {
    finding_category: &'static str,
    directive: &'static str,
}

const ENTRIES: &[EntrySpec] = &[
    EntrySpec {
        finding_category: "sprint-conflict",
        directive: "FINDING-DETECTED: sprint-conflict. Compare the work items currently open in your sprint with the external tracker to identify any scope overlap with parallel tasks assigned to other team members. Determine whether the conflicting item shares a code path, database schema, or API contract with your current work by tracing the dependency graph through both items. Coordinate with the owner of the conflicting sprint item before proceeding: agree on integration ordering, resolve the overlap explicitly, or escalate to the sprint facilitator if the conflict blocks both workstreams. Do not merge or deploy either item until the dependency ordering is confirmed and recorded.",
    },
    EntrySpec {
        finding_category: "collaborator-took-issue",
        directive: "FINDING-DETECTED: collaborator-took-issue. Verify the current assignee of the issue or work item you were tracking and confirm whether the new owner has already begun implementing changes that affect your current branch. Review any commits, branches, or in-progress diffs the collaborator has opened and identify any overlapping file changes or interface mutations. Coordinate directly with the new owner before continuing: determine whether your work should pause, merge into their branch, or proceed independently with an explicit integration plan. Document the handoff boundary in the issue or pull request thread before marking your own work as ready.",
    },
    EntrySpec {
        finding_category: "collaborator-finished-related-work",
        directive: "FINDING-DETECTED: collaborator-finished-related-work. Pull the recently merged branch or tagged release from the collaborator and inspect its diff against your current working tree for any overlapping changes to shared files, types, or contracts. Confirm whether the completed work introduces interface changes, renamed symbols, migrated schemas, or updated dependencies that your current branch must absorb before continuing. Rebase or merge the upstream change into your branch, resolve any conflicts conservatively by preserving both parties' intended contracts, and run the test suite to confirm no regressions were introduced. Record any design decisions that changed as a result of the absorption in your commit message or pull request description.",
    },
    EntrySpec {
        finding_category: "external-spec-change",
        directive: "FINDING-DETECTED: external-spec-change. Read the updated specification document and identify every section that has changed relative to the version your current implementation was authored against. Map each changed section to the code, tests, or documentation it governs and determine whether your implementation still satisfies the updated requirements. For any section whose requirements no longer match your implementation, create a gap list with one item per divergence and address each gap before treating your work as spec-complete. Do not ship work that satisfies an outdated spec version without explicit acknowledgment from the spec owner that the old version governs.",
    },
    EntrySpec {
        finding_category: "runbook-update",
        directive: "FINDING-DETECTED: runbook-update. Read the updated runbook and identify the specific steps or decision rules that changed relative to the version used in previous operations. Determine whether any in-progress operational task, deployment, or incident-response procedure you are executing is governed by the changed steps. If you are mid-procedure and the changed steps affect the current phase, pause and re-read the updated guidance before continuing; do not apply the old runbook steps to a situation the new runbook covers differently. Update any working notes, scripts, or automation that reference the old runbook content before re-invoking.",
    },
    EntrySpec {
        finding_category: "cms-plugin-marketplace-release",
        directive: "FINDING-DETECTED: cms-plugin-marketplace-release. Inspect the release notes and changelog for the newly published marketplace plugin and identify any changes to its API surface, permissions model, or schema it writes to shared collections. Determine whether your current CMS configuration, workflow hooks, or content types depend on the plugin and would be affected by the version change. If the plugin is a dependency of your current implementation cycle, assess whether the new release introduces breaking changes, deprecations, or security fixes that require immediate action. Do not install or upgrade the plugin in a production environment without confirming compatibility with your current content schema and running the change against a staging environment first.",
    },
    EntrySpec {
        finding_category: "competitor-product-launch",
        directive: "FINDING-DETECTED: competitor-product-launch. Read the available public information about the competitor launch and identify any product capabilities, pricing changes, or positioning claims that overlap with the current feature or content you are building. Assess whether the launch affects the relevance, urgency, or framing of your current work item by checking it against your mission rail and current sprint goals. Surface the finding to the product owner or mission anchor before changing scope: competitor launches are signals to absorb, not authorization to redirect implementation. Record the observation in the relevant tracking issue or decision log so the product owner can act on it during the next planning cycle.",
    },
    EntrySpec {
        finding_category: "editorial-workflow-signal",
        directive: "FINDING-DETECTED: editorial-workflow-signal. Read the full signal including any attached comments, reviewer notes, or approval state changes and determine which content item, workflow stage, or editorial gate it is associated with. Match the signal to the content item currently open in your working session and confirm whether the signal blocks, unblocks, or redirects the authoring task. If the signal introduces a new requirement such as a mandatory review, a brand-voice exception, or a regulatory disclosure, incorporate it before advancing the content item to the next workflow stage. Record the signal handling decision in the content item's audit trail or history field so downstream reviewers can trace the editorial chain.",
    },
    EntrySpec {
        finding_category: "agent-skill-published-with-vulnerability",
        directive: "FINDING-DETECTED: agent-skill-published-with-vulnerability. Identify the published skill, plugin, or extension and determine whether it is currently installed, referenced, or scheduled for installation in any agent substrate under your mission rail. Immediately suspend any pending installation of the affected skill and revoke access if it is already installed and active. Obtain the vulnerability details from the source that flagged the publish event and determine whether a patched version is already available; if not, treat the skill as unsafe until a clean version is published and verified. Do not reinstall or re-authorize the skill without first running bspector scan against the patched version and confirming a safe exit code.",
    },
];

#[derive(Serialize)]
struct FixtureFile {
    schema_version: u32,
    signature: String,
    canonical_key_id: &'static str,
    entries: Vec<FixtureEntry>,
}

#[derive(Serialize)]
struct FixtureEntry {
    routing_key: RoutingKey,
    finding_category: &'static str,
    directive: &'static str,
    provenance: FixtureProvenance,
}

#[derive(Serialize)]
struct FixtureProvenance {
    run_id: &'static str,
    iteration: u32,
    observation_source: &'static str,
    pre_compliance: f64,
    post_compliance: f64,
}

fn main() {
    let signing_key = SigningKey::from_bytes(&FIXTURE_SEED);
    let verifying_key = signing_key.verifying_key();

    let mut corpus = CorpusFile {
        schema_version: 1,
        signature: String::new(),
        canonical_key_id: CANONICAL_KEY_ID.to_owned(),
        entries: ENTRIES
            .iter()
            .map(|spec| CorpusEntry {
                routing_key: RoutingKey::bwatch(),
                directive: spec.directive.to_owned(),
                provenance: ProvenanceRecord {
                    run_id: RUN_ID.to_owned(),
                    iteration: 0,
                    observation_source: OBSERVATION_SOURCE.to_owned(),
                    pre_compliance: 0.0,
                    post_compliance: 0.0,
                },
            })
            .collect(),
    };

    let payload_bytes = bsuite_core::corpus::canonical_payload_bytes(&corpus)
        .expect("fixture corpus canonicalization succeeds");
    let signature = signing_key.sign(&payload_bytes);
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
    corpus.signature = format!("ed25519:{sig_b64}");

    let fixture_file = FixtureFile {
        schema_version: 1,
        signature: corpus.signature,
        canonical_key_id: CANONICAL_KEY_ID,
        entries: ENTRIES
            .iter()
            .map(|spec| FixtureEntry {
                routing_key: RoutingKey::bwatch(),
                finding_category: spec.finding_category,
                directive: spec.directive,
                provenance: FixtureProvenance {
                    run_id: RUN_ID,
                    iteration: 0,
                    observation_source: OBSERVATION_SOURCE,
                    pre_compliance: 0.0,
                    post_compliance: 0.0,
                },
            })
            .collect(),
    };

    let header = "# Fixture corpus. Hand-authored seed material until an evolved corpus ships at a later cycle. Not for production trust.\n\n";
    let body = toml::to_string_pretty(&fixture_file).expect("fixture TOML serialization succeeds");
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus");
    fs::create_dir_all(&out_dir).expect("create corpus directory");
    fs::write(out_dir.join("bwatch-v0.toml"), format!("{header}{body}"))
        .expect("write corpus TOML");
    fs::write(
        out_dir.join("bwatch-v0-pubkey.bin"),
        verifying_key.to_bytes(),
    )
    .expect("write verifying key");
    fs::write(
        out_dir.join("bwatch-v0-signkey.bin"),
        signing_key.to_bytes(),
    )
    .expect("write signing key");

    println!("corpus files written to {}", out_dir.display());
}
