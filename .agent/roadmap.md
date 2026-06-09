# CKC Roadmap

Flat ordered checklist consumed by the `/session-prompt` command
(`.claude/commands/session-prompt.md`): each line is a build unit or a
`review …` line. On completing a line the session protocol marks it `[x]` and
appends a trailing `NN% NNNK/200K` context-usage annotation (from
`.agent/compaction.sh`); splitting a unit replaces its line with `<unit-id>.z`
sub-lines per the command's Splitting rule.

`SPEC.md` is the design authority; M1 acceptance is its §19. Per the bootstrap
decision, units are authored incrementally rather than all upfront: the planning
unit below seeds the forward plan, and each session extends the tail before it
empties. An empty tail means *author the next units from `SPEC.md` §19*, not *M1
complete* — M1 is complete only when every §19 criterion maps to a checked unit.

## Backlog

- [ ] plan-boilerplate [user-selected]: establish project boilerplate and seed
  the forward build plan. Boilerplate — `.gitignore` (per CLAUDE.md `.serena/`;
  per SPEC §6 `corpus/raw/`, `runs/`; plus toolchain/build caches) and the
  SPEC §6 repository skeleton + tooling init (uv `pyproject.toml`/`uv.lock`;
  Cargo workspace over `crates/{ckc-core,ckc-core-cli,ckc-smt}`; `ckc/`
  adapter/runner tree; `registry/ corpus/ schemas/ examples/ tests/` dirs;
  `Makefile`). Plan — author the M1 build units into this backlog from the §19
  acceptance criteria. Reading: SPEC §5–§6, §19; CLAUDE.md. Gate: tree commits
  clean with `.gitignore` excluding `.serena/`, and forward units authored
  below this line.
