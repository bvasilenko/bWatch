# bwatch

For agents that act blind to outward state. bwatch consults an outward tracker (GitHub issues, Linear, Jira, Slack, Notion) for related work and emits a finding directive: ACTIONABLE (a related thread is open and the agent should reference it), INFORMATIONAL, WRONG-TIME, or MISFIRE. The finding taxonomy is closed (9 variants at v0.1); the prompt library evolves continuously via empirical-lift evaluation, so the same `bwatch verify` invocation gets stricter at surfacing relevant outward state as the corpus matures.


Prompt lookup tool. Agent names a finding category from a fixed list; bwatch returns the prompt for that finding category. The prompt tells the agent how to check the external tracker for that finding.

Built for agentic loops. Polls an external tracker source, matches the observation against a closed 9-category finding taxonomy, writes a directive on stdout, exits with a discriminating code so the calling agent can branch on the triage result.

```
bwatch poll                       poll an outward tracker source against the finding taxonomy; exit 0 / 1 / 2 / 64
bwatch finding-categories         list the 9 supported finding-category identifiers
bwatch init                       scaffold a manifest in the current directory
bwatch update                     self-update to the latest published version
bwatch tail                       stream recent verdict transcripts
bwatch explain                    print taxonomy + exit-code reference
bwatch process                    process a queued finding batch
```

Exit code contract: `0` no action required, `1` finding requires action, `2` internal error, `64` malformed input.

## Install

```sh
cargo install --git https://github.com/bvasilenko/bWatch
```

## Use

```sh
bwatch poll --source github --mission v0.1
# stdout: [bwatch placeholder directive - pre-corpus output] ...
# exit: 1

bwatch poll --source https://gitlab.example.com/group/project --mission v0.1
# stdout: [bwatch placeholder directive - pre-corpus output] ...
# exit: 1
```

Optional flags: `--source <name-or-url>`, `--mission <path-or-name>`, `--manifest <path>`, `--json`, `--quiet`, `--reason <text>`. Subcommands consume the same flag set; defaults are sane.

## Finding taxonomy

Closed 9-variant `FindingCategory` enum. The taxonomy is fixed at this version; widening lands in a later version.

| Category | Variants |
|---|---|
| Coordination | `sprint-conflict`, `collaborator-took-issue`, `collaborator-finished-related-work`, `external-spec-change`, `runbook-update` |
| CMS context | `cms-plugin-marketplace-release`, `competitor-product-launch`, `editorial-workflow-signal`, `agent-skill-published-with-vulnerability` |

`bwatch finding-categories` prints the full list.

## License

MIT.
