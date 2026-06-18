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
