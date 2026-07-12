# CKC /loop iteration prompt

Each iteration, exactly ONE of, checked in order:

1. `.agent/roadmap.md` active milestone has an OPEN unit → invoke the session-prompt skill
   with empty task (WORK-UNIT mode, lowest OPEN unit) and drive it to its scoped commit.
   Then STOP this loop (ScheduleWakeup stop:true): milestone units are session-sized, the
   next unit wants a fresh session — tell the user to relaunch /loop in a new session.
2. Active milestone units all DONE (or only BLOCKED remain) and M3 `loop-framework` is
   DONE → run ONE /cnl-optimize round (protocol: `.claude/commands/cnl-optimize.md`). Then
   if total context usage < 80%: reschedule self-paced (1200–1800s; nothing external is
   being polled, so shorter is waste). Else: stop per the protocol's ceiling rule.
3. Neither applies (blocked milestone, dirty tree, red baseline) → report precisely what
   blocks, then stop the loop.

Never push, never touch settings files, never start initiatives outside the two flows above.
