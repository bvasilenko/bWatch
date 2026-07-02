# bwatch

Closed finding-category taxonomy plus corpus-driven directive emission wired to the bsuite-core 0.2.0 engine surface.

Prompt lookup tool. Agent names a finding category from a fixed list; bwatch returns the prompt for that finding category. The prompt tells the agent how to check the external tracker for that finding.

Built for agentic loops. The agent supplies a finding category, bwatch emits the directive for that category on stdout, and exits with a discriminating code so the calling agent can branch on the result.

```
bwatch poll --category <finding-category>   emit the directive for the named finding category; exit 0 / 1 / 2 / 64
bwatch finding-categories                   list the 9 supported finding-category identifiers
bwatch update                               self-update to the latest published version
bwatch init                                 scaffold a manifest in the current directory
bwatch tail                                 stream recent verdict transcripts
bwatch explain                              print taxonomy + exit-code reference
bwatch process                              process a queued finding batch
```

Exit code contract: `0` no action required, `1` finding requires action, `2` internal error, `64` malformed input.

## Install

```sh
cargo install --git https://github.com/bvasilenko/bWatch
```

## Use

```sh
bwatch poll --category sprint-conflict --source github --mission v0.1
# stdout: FINDING-DETECTED: sprint-conflict. Compare the work items currently open ...
# exit: 1

bwatch poll --category runbook-update --source https://gitlab.example.com/group/wiki
# stdout: FINDING-DETECTED: runbook-update. Read the updated runbook ...
# exit: 1
```

Required flag: `--category <finding-category>`. Optional flags: `--source <name-or-url>`, `--mission <path-or-name>`, `--manifest <path>`, `--json`, `--quiet`, `--reason <text>`.

## Finding taxonomy

Closed 9-variant `FindingCategory` enum. The taxonomy is fixed at this version; widening lands in a later version.

| Group | Variants |
|---|---|
| Coordination | `sprint-conflict`, `collaborator-took-issue`, `collaborator-finished-related-work`, `external-spec-change`, `runbook-update` |
| CMS context | `cms-plugin-marketplace-release`, `competitor-product-launch`, `editorial-workflow-signal`, `agent-skill-published-with-vulnerability` |

`bwatch finding-categories` prints the full list.

## Corpus

The v0 corpus is hand-authored fixture material. It covers all 9 finding categories with substantive directives and is signed with the `bwatch-fixture-v0` key. This corpus is seed material only; an evolved corpus ships at a later cycle. The fixture key and corpus are committed for reproducibility and are not used for production trust.

The outward-source-state quartet (`Actionable`, `Informational`, `WrongTime`, `Misfire`) is carried by the type surface but is not used as a secondary routing axis in the v0 corpus. The evolved corpus will introduce per-state variants once the evaluator-harness cycle produces sufficient empirical lift; using a single-axis corpus at v0 keeps the fixture burden proportional and matches the shape proven by sibling binaries at the same lifecycle stage.

## Environment variables

| Variable | Purpose |
|---|---|
| `BSUITE_UPDATE_BASE_URL` | Override the base URL used by `bwatch update` to fetch the signed release manifest. Defaults to a placeholder constant. Intended for testing and enterprise mirror deployments. |
| `BSUITE_TRANSCRIPT_DIR` | Override the directory where invocation transcript records are written. |
| `BSUITE_TRANSCRIPT_RETENTION_DAYS` | Number of days to retain transcript records (default: 90). |

## License

MIT.
