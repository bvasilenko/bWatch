#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

mapfile -t files < <(
  git ls-files --cached --others --exclude-standard |
    grep -E '^(Cargo\.toml|LICENSE|README\.md|\.github/workflows/|crates/|scripts/)' |
    grep -v '^scripts/public_voice_guard\.sh$'
)

if [ "${#files[@]}" -eq 0 ]; then
  exit 0
fi

patterns=(
  'p[i]lls?'
  'Q5L R-[0-9]+'
  'projects/'"b-suite/"
  'hold'"ing/"
  'frame'"works/"
  'B:[0-9]+'
  'I[0-9]+-I[0-9]+'
  'GOV-'
  'DECISION [0-9]+'
  'implementation-'"open gate"
  '0\.1\.0-'"skeleton"
  'M1-Mz'
  'PENDING-'"OPENEVOLVE-RUN"
  'PENDING-'"FIRST-CONTRIBOT-CYCLE"
  'SCOPE-BNPM-'
  'DOMAIN-'
  'IP-TRANS-'
  'Co-Authored-By:'
  $'—'
)

failed=0
for pattern in "${patterns[@]}"; do
  if grep -InE -- "$pattern" "${files[@]}"; then
    failed=1
  fi
done

if grep -RInE -- 'OffBrandVoiceCreep' crates/bwatch/src crates/bwatch/tests; then
  failed=1
fi

if ! grep -Fq 'Prompt lookup tool. Agent names a finding category from a fixed list; bwatch returns the prompt for that finding category. The prompt tells the agent how to check the external tracker for that finding.' crates/bwatch/Cargo.toml README.md; then
  echo "public description missing" >&2
  failed=1
fi

if [ "$failed" -ne 0 ]; then
  echo "public voice guard failed" >&2
  exit 1
fi
