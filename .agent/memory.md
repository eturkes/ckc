# Agent Memory

Reset to a clean slate on 2026-06-17 (CLAUDE.md and model-line update); prior entries live
in git history. This file is the sole memory store for CKC — Serena's memory system is
disabled. Record only what earns its place: lessons and reminders that add value beyond
SPEC.md, CLAUDE.md, the codebase, git history, and the runtime environment, especially
hard-won facts easily re-forgotten under token pressure. Consolidate aggressively; git
history retains pre-consolidation text.

## Policy
- Branch poc-m2-3-4 (the M2-M4 PoC) runs sessions at 1M context (user-launched,
  overriding the default 200K); size units for 1M headroom.

## Lessons
- Subagents inherit the launching session's context-window size (launch-set and
  process-wide); one that exhausts its window dies mid-task with no result, so
  size each subagent's reading slice with margin.
- `Explore`-type subagents are edit-restricted but still hold `Bash`, so they can
  mutate the tree; after any subagent fan-out, `git status` and reconcile stray
  paths before staging.
- Headroom round-trips unicode for `\uXXXX` ASCII-escape literals (report.py JA
  strings, ui/ i18n): author Edit old/new_string in DECODED unicode (the match
  layer accepts it) and the write path re-escapes to ASCII on disk (verified 0
  non-ASCII bytes). JA edits stay ASCII-clean without hand-escaping; still
  byte-check after (`open(...,"rb")`, count b>=0x80).
- `Read(./runs/**)`-style denies also block Bash file-readers (`grep PAT file`,
  cat/tail) on those paths, but Python `open()` bypasses -- inspect run
  records/reports with a `python3 - <<'PY'` snippet, not grep-on-file. grep from a
  pipe (stdin) is fine.
- Harness blocks `sleep N; <cmd>` chains (use the completion notification or a
  single poll command) and denies compound bash mixing `$(...)` with denied-path
  args -- keep polls to one plain command.
- Run-dir seeding (copy a prior run's records; resume skips by exact filename)
  reuses records by PATH only -- score.py trusts each record's content/keys while
  stamping the CURRENT prompt/schema/grammar shas + identities, so a stale seeded
  record (authored under different prompts) would score silently under fresh shas.
  matrix9's reuse of matrix5 was verified clean (records byte-identical, rev-2
  metrics + shas match); re-verify whenever seeding spans a prompt/schema change.
