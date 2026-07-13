# CKC /loop iteration prompt

Loop sessions run /cnl-optimize rounds ONLY. Milestone units go through the normal
/session-prompt workflow in their own sessions — never through this loop. User-side loop
config (user-managed at loop time): 1M-context session, autoCompact ON.

Each iteration:

1. Run ONE /cnl-optimize round (protocol: `.claude/commands/cnl-optimize.md` — its
   preconditions gate the round: loop-framework DONE, clean tree, green baseline).
2. Round closed (landed or banked) → reschedule self-paced (1200–1800s; nothing external is
   polled, shorter is waste). All round state lives in git + `.agent/cnl-queue.md`, so
   between-round compaction is safe by design.
3. A precondition fails or the protocol's stop rule fires → report precisely why, then stop
   the loop (ScheduleWakeup stop:true).

Never push, never touch settings files, never start initiatives outside /cnl-optimize rounds.
