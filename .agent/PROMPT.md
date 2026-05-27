# CKC Session Prompt

Continue developing CKC (Clinical Knowledge Compiler) — a research-grade,
proof-carrying clinical knowledge compiler for Japanese clinical guidelines
and medical textbooks.

## Context files (read in this order)
1. `SPEC.md` — full project specification with phases, object model, and stack
2. `CLAUDE.md` — operating instructions and agent directives
3. `.agent/memory.md` — cross-session decisions, lessons, environment notes
4. `.agent/roadmap.md` — phase and task progress tracker

## Orientation
1. Run `git log --oneline -20` to see recent work.
2. Read `.agent/roadmap.md` to find the current phase and next incomplete task.
3. Read `.agent/memory.md` for decisions, lessons, and open questions.
4. Proceed with the next task. Commit when closing out a cohesive piece of work.
5. Update `.agent/memory.md` with any new decisions, lessons, or notes.
6. Update `.agent/roadmap.md` to mark completed tasks and add new ones.

## Steering
(Append task-specific instructions below this line)
