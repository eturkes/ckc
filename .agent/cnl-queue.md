# ClinicalCNL optimization queue + ledger

Consumed by `/cnl-optimize` (protocol: `.claude/commands/cnl-optimize.md`). Queue entries are
one bullet each: `- [<category a–f|major>] <what> — <where> — <why/next action>`. Ordered by
value, top = next pick. `major:` items are roadmap-session material, never round material.
Ledger lines: `- R<n> <category> <commit subject gist>`. Reject-ledger lines: surface/idea +
reason + reopen trigger.

## Queue

- [major] IR bridge (APE line → ClinicalIR): Z3 cross-check + harness metrics/provenance —
  roadmap Backlog carries the trigger; do not start in rounds.
- [major] ClinicalCNL JA surface: mine `git show ecc19d3:SPEC.md` §10 + banked JA
  lexicon/prefix pins — roadmap Backlog carries the trigger.
- [major] Clex candidate-mining seed: §11.5 evidence row (upstream commit 20960a5c grant +
  LDC-derivation authority) before any use.

## Reject ledger

(none yet)

## Ledger

(no rounds yet — loop-framework unit runs R1)
