# CKC Roadmap

Flat ordered checklist consumed by the `/session-prompt` command
(`.claude/commands/session-prompt.md`): each line is a build unit or a
`review …` line. On completing a line the session protocol marks it `[x]` and
appends a trailing `NN% NNNK/200K` context-usage annotation (from
`.agent/compaction.sh`); splitting a unit replaces its line with `<unit-id>.z`
sub-lines per the command's Splitting rule. Build units are authored here once
the specification defines a build plan.

## Backlog

(empty — author the first build units once the specification exists.)
