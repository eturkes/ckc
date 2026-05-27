# Agent Memory

Every entry must provide value beyond what SPEC.md, CLAUDE.md, the codebase,
git history, config files, tool `--version` output, and the runtime
environment already provide. Exception: high-value reminders that are
technically derivable but easily forgotten under token pressure.

## Decisions

## Lessons

- [2026-05-27] Every piece of natural language in the project (docs, comments,
  diagnostics) must use positive constraint framing per the pink elephant
  principle in CLAUDE.md. Audit before committing.
- [2026-05-27] LLMs habitually duplicate information across files and add
  redundant recap/summary sections that restate what was just said. Audit every
  draft for cross-file duplication and filler steps before committing. If a
  fact lives in one file, other files must reference it, not restate it.

## Open Questions
