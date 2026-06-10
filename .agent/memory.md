# Agent Memory

Every entry must provide value beyond what the project specification, CLAUDE.md,
the codebase, git history, config files, tool `--version` output, and the
runtime environment already provide. Exception: high-value reminders that are
technically derivable but easily forgotten under token pressure.

## Lessons

- [2026-06-10] Work-unit sizing calibration (from roadmap `NN%`
  annotations). Target one conceptual deliverable + one gate per unit, finishable AND
  committable in one context window with margin; prefer more, smaller units.
  Anchors: canonical JSON is FIVE units (writer / collections / unions /
  strict-reader / hash, each ~62-69%); a writer-inverse (strict reader) unit
  fills a window alone — schedule it solo, never with its writer or a sibling;
  crate-foundation units run hot (~81%) — pair them only with a small type
  surface; a unit that implements a nontrivial algorithm AND authors a second
  artifact splits in two; registry entry types alone fit one unit (~69%);
  five-layer/recursive type families (IR) split into ~three units. Pre-split
  such stacks BEFORE scheduling; reserve JIT-splitting for hot-but-fitting
  single-deliverable units (~70-85%). Calibrate finer from neighbouring units'
  actual `NN%` once they exist. Archive roadmaps mix measured lines (checked,
  `NN%` annotated) with planned-only lines (unchecked, no `NN%`); cite only
  checked lines as measured anchors — planned scopes are projections.
- [2026-06-09] RTK proxy mangles `git commit` with multiple `-m` flags whose
  values carry non-ASCII (`§`, em-dash `—`): args get dropped/split, leaving a
  bare space git reads as a pathspec ("pathspec ' ' did not match any file(s)"),
  so `git add` stages but the commit never lands — and RTK still prints
  "ok N files changed" from the add step, masking the failure. Always commit
  such messages from a file: write the message, then `git commit -F <path>` (one
  flag + one path survives the rewrite), and `rm` the temp file after. Plain
  single-`-m` ASCII commits are unaffected.
- [2026-06-09] Enabling a Serena LSP language that isn't set up yet. Symptom: a
  Serena symbolic tool (`get_symbols_overview`/`find_symbol`/reference lookups)
  errors `Active languages: ['bash']`. Fix: add the language to
  `.serena/project.yml` `languages:` (list it first = default/fallback LS). The
  MCP server reads `project.yml` only at startup (`--project-from-cwd`), so the
  edit applies only after a Serena reconnect, which you cannot trigger from a
  tool — ask the user to `/mcp` reconnect serena (or restart Claude), then
  verify with a symbol/reference call (first call may lag during indexing).
  Confirm the LS binary exists first; for rust, Serena launches the rustup
  toolchain's `rust-analyzer` directly, so `rustup update` already keeps it
  current. `.serena/project.yml` is git-tracked so the set persists across fresh
  checkouts. Apply this whenever a needed Serena LSP isn't enabled.
- [2026-06-10] Markdown authoring: wrap regexes/grammars in backticks — bare
  adjacent bracket groups (`[a-z][a-z0-9_.:-]*`) parse as Marksman reference
  links and emit phantom-label warnings. Verify markdown fixes with grep for
  bare `][` outside code spans; serena `get_diagnostics_for_file` is useless
  there — it routes non-code files to the fallback LS (first entry in
  `.serena/project.yml` `languages:`, currently rust-analyzer), which emits
  pages of phantom syntax errors. Marksman diagnostics reach the session only
  via the harness new-diagnostics channel.
- [2026-06-10] Serena `replace_symbol_body` on a Rust item spans its preceding
  doc comment AND outer attributes (`#[derive(...)]`): the body you supply
  replaces all of them, so omitting them deletes them. Dropped a
  `#[derive(Debug, Clone, PartialEq, Eq)]` off an enum this way and broke the
  build (no `Debug` for `.unwrap()` in tests). Always include the leading `///`
  lines and every `#[...]` attribute in the replacement body, or edit the inner
  variant/field region with `replace_content` instead. Recurs with derive-heavy
  enums and the envelope/IR structs.
- [2026-06-10] RTK command rewriting can falsify tool output. Observed: `diff a
  b` printed "[ok] Files are identical" for files that differ (cmp/sha256sum
  disagreed; differences were single-line multibyte `§` edits), and a piped
  `diff | grep | head` chain panicked on broken pipe. Also `grep` is rewritten
  to `rg`, so BRE escapes (`\(`, `\|`) become regex parse errors — write
  rg-syntax patterns. Always prove byte-equality with `cmp` or `sha256sum`,
  never the plain `diff` wrapper; for real diffs use `git diff --no-index` or
  `rtk proxy diff`. Critical wherever byte-compatibility with the archive is
  the acceptance bar (canon reader/hash, future wire formats).
- [2026-06-10] Editing-tool string parameters decode `\uXXXX` Unicode escapes,
  and only those — `\n`, `\"`, `\xNN` pass through literally. So writing source
  that must contain a literal `\uXXXX` (e.g. a byte/raw-string test asserting
  that an uppercase-A escape is non-canonical) through
  `replace_content`/`Edit`/`Write` silently decodes it: the A-escape collapses
  to the byte `A`, and a C0-control escape collapses to a real newline,
  corrupting both code and comments. The corrupted forms often still compile, so
  only a test failure or a read-back catches code damage while comment damage
  ships silently. When edit content must carry a literal backslash-u, express
  the bytes without that substring — a byte-array literal with `0x5c` for the
  backslash, e.g. `&[b'"', 0x5c, b'u', b'0', b'0', b'4', b'1', b'"']` — and read
  the region back after writing. Recurs in any unit whose tests assert on escape
  syntax (the canonical string reader/writer, later wire parsers).

