# Plan session protocol (routed from /session-prompt)

A plan session opens the next milestone. When that milestone's contract
section is still compact, elaborate it first per SPEC §1 — the spec diff
reaches the user before any unit consumes it. Then author the milestone header
and its full unit checklist.

Planning is the one task that runs as a **dynamic workflow**: unit authoring
is a discrete task that divides cleanly among subagents without loading the
main context. Author a script inline and pass it to the `Workflow` tool; the
session command routing here is your standing opt-in to multi-agent
orchestration — call `Workflow` directly, no confirmation needed. There are no
saved workflows; every script is written fresh for the task at hand. Subagent
model and effort (fable/max) come from the user's global Claude settings, so
omit per-call `model`. A judge-panel shape fits: fan out agents that each
propose a full decomposition from a different boundary (specification-section
family, artifact layer, deliverable type), score the candidates with parallel
judges, and return the winner as structured unit specs (use the `schema`
option). Keep finders read-only (`agentType: 'Explore'`); when the workflow
returns, `git status` and reconcile every stray path before staging — Explore
agents are edit-restricted but still hold `Bash`, so a finder can mutate the
tree. The main session owns every mutation — spec edits, roadmap edits — and
the closing commits.

Calibrate unit sizing against the previous milestone's checklist, still
present while you plan — its per-item usage annotations are ground truth for
what fits a window — plus the memory.md sizing anchors. Each unit line is one
conceptual deliverable with explicit file paths, real identifiers, its reading
slice, and exactly one gate command.

Closing: one commit, scoped `plan-m<n>:` — fill the previous milestone's
pending `review _`, delete its items keeping the bare header (a closed
milestone collapses to that one line; git history retains the rest), and
append the new milestone header `## <milestone> — plan _` + checklist. End the
session.
