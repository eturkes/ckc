# Agent Memory

Entries must add value beyond the spec, CLAUDE.md, codebase, git history, and runtime
environment. Exception: high-value reminders that are derivable but easily forgotten under
token pressure. Entries are consolidated aggressively; full pre-consolidation text lives in
git history.

## Policy

- Context hygiene (user directive; background: `git show 531f586`): keep every
  session lean and phrased in project vocabulary (processing stages, units, gates, artifacts) — plain
  operational words over research jargon in memory, roadmap, commits, and code alike.
  Consult `docs/` through read-only subagents so its vocabulary stays out of
  the main window. Root `.rgignore` keeps ripgrep-backed sweeps (subagent Grep, `rtk proxy
  rg`) out of `docs/`; Bash `grep -r` still enters it — scope Bash greps by path; deliberate
  docs searches use `git grep <pat> -- docs/`, `rg --no-ignore`, or explicit file paths.
  Checked roadmap items collapse to one-line stubs (full unit text in git history).
  Implement sessions match patterns from the latest unit-scoped commit (`git log
  --oneline`), not bare HEAD, when HEAD is hygiene/memory work.
- LSP coverage criterion: ckc-lsps plugins and Serena languages track formats
  whose concrete syntax gets hand-authored or byte-pinned in-repo (active:
  rust, bash, json, yaml, toml, markdown, html, xml, smt2 via dolmen; §13-named targets:
  lean4, alloy, egglog) — compendium-catalogued families whose registry presence is YAML
  data carry no plugin. TLA+, ASP/Clingo, and categorical-CQL have no standalone LSP server
  (audited); Isabelle's LSP and any Python LSP land
  only with their adoption decisions (§13 additional-targets row; §13.1 adapter boundary).
  dolmen-lsp deploys as a standalone copied binary with the opam tree removed — rebuild
  recipe in its plugin README.

## Lessons

- Unit sizing rules (consolidated from roadmap `NN%` annotations and
  observed 200K overruns; case studies in git history). Target: one conceptual deliverable
  plus one gate, finishable AND committable in one window with margin; prefer more, smaller
  units. Plan-time obligations (a violation is a planning bug): resolve semantic requirements
  decisions INTO the roadmap line (more than ~2 left open = re-scope); research and pin any
  new external dependency (exact version + features) in the line; pre-split
  multi-deliverable stacks BEFORE scheduling — mid-session overrun recovery is
  user-initiated (stop, bring the tree clean, report). Split rules: a feature needing a
  refactor of existing code to share internals takes the refactor as its own
  behavior-locked unit FIRST (existing tests the gate, zero test edits); a format walker
  plus committed test-source integration = walker-core (inline-literal tests) then
  format-completion + test-source integration; a nontrivial algorithm plus a second authored
  artifact = two units; a multi-invariant validator plus full rejection coverage = two
  units; a derivation fn with its test-source-pinned battery plus an attachment sub-feature =
  two units; a type family plus assembly plus validation = three units (6th overrun,
  cli-runner.3a: trace types + assembly + ckc-run wiring scheduled as one unit
  against this anchor — 975 uncompiled lines plus full run.rs/shell.rs reads by compaction;
  reverted, split .3a.1/.2/.3 with the decisions pinned in the lines and each Reading slice
  excluding files its half leaves untouched); an assembly fn plus its live-pipeline pin
  battery = two units (7th overrun, cli-runner.3a.2: assemble_trace + hand-off
  structs compiled clean, but the live battery — reading slice alone spanning run.rs, the
  verify-stage tests, the normalize.rs helpers — was unwritten at compaction; reverted,
  split .3a.2a synthetic-tests / .3a.2b live-pins); minting a split rule re-audits every
  remaining unchecked line against it in the same recovery commit (8th overrun,
  cli-runner.4.1a: types + validation + assembly + synthetic battery as one unit —
  the exact shape the 6th-overrun rule bans, left unaudited by both later recovery sessions;
  the full report.rs draft was written but never built at compaction; reverted, split
  .4.1a.1/.4.1a.2 and the sweep pre-split .4.1b and .4.2 into core/wiring pairs); a live-pin
  battery over the run binary is a unit on its own — pairing it with assembly (7th overrun)
  or with processing stage wiring overruns (9th overrun, cli-runner.4.1b.1: the 8th-overrun
  core/wiring pre-split still left run.rs threading + the exp.m1_scaffold report pins on one
  line; compacted at the workspace-suite rerun with work otherwise done, landed
  user-accepted; anchors: trace wiring solo 75%, trace live-pins solo 71%); a recovery split
  is itself plan work — audit its replacement lines against every standing rule and the
  open-decision ceiling within the recovery commit (10th overrun, cli-runner.4.1b.2b: the 9th-overrun recovery named that rule yet left wiring + live pins paired
  and ~6 provenance decisions open on the .2b line — toolchain-manifest identity, git-commit
  source, environment profile, replay argv, hash sources; the window went to decision
  derivation + run.rs threading, compacted with the pin battery unwritten; reverted,
  decisions + salvage patch pinned into .2b.1, pins split to .2b.2); a spec-byte
  amendment (re-pin + reference/test mirror sweep) bundled with new feature code = two units —
  an open decision whose resolution amends pinned bytes is a deliverable, not a session
  preamble (4th overrun, stage-normalize.2: decision + §8.6 amendment + mirrors
  consumed ~half the window before the derivation module; compacted at the test gate,
  reverted, split into .2a/.2b with the decisions written into the roadmap lines). Measured
  anchors (checked roadmap stubs carry the `NN%` figures): canonical JSON = five units;
  crate foundations pair only with a small type surface (5th overrun, smt-emit.1:
  foundation + two durable payload modules, each canonical impls + multi-rule validator
  + pin/rejection battery, compacted at the lint step with work otherwise done; schedule
  one payload module per foundation unit); a five-layer recursive type family = three
  units; a lexicon-driven derivation half (loader / binding / builder) = three units;
  statement builder over a prebuilt binding core = one unit; exception attachment +
  determinism tests = one unit. Practices: house
  new type families in fresh modules (extending a ~2K-line module costs a full-file read);
  land a compiling skeleton before the full test battery; salvage a reverted session's
  compiling half as a committed `.agent/wip-<unit>.patch` the redo line points at (apply,
  verify against the line, delete in the closing commit) — an uncompiled draft salvages the
  same way flagged UNCOMPILED, transcription-with-verification still beating re-derivation
  (8th overrun); pin expected shapes from observed
  output, never hand-computed; cite only checked roadmap lines as measured anchors.
  At plan/re-scope time, audit any spec listing a unit must byte-reproduce: listings written
  for readability (alignment padding, inline result comments, illustrative declaration or
  conjunct order) contradict deterministic-emission rules and need a scheduled re-pin
  deliverable (caught pre-session for smt-emit.3a: §8.6 smt2 vs §6 sorted-declaration rule).
- Web search default: WebFetch on `https://lite.duckduckgo.com/lite/?q=<query>` (sandbox
  curl gets a bot wall). Targeted channels: crates.io via curl with a `-A` user-agent
  header (403 without) — detail /api/v1/crates/NAME, search /crates?q=; GitHub
  /search/repositories?q=; Wikipedia opensearch. The WebSearch tool 400s on this model
  line (the API rejects its forced tool_choice; the error arrives INLINE in an ok-looking
  result) — a PreToolUse hook (`.claude/hooks/deny-websearch.sh`, wired in settings.json)
  denies it with this redirect; Workflow agent() `schema` verified unaffected. Re-test on
  Claude Code update or model-line change (last 2026-06-12: still broken): drop the
  settings.json hooks entry, ToolSearch-load WebSearch, one query, read the body; healed →
  delete hook script + entry and collapse this entry to its working-channels half.
  Meta-rule: the session that FIRST hits an environment/tool failure records it in that
  same session.
- Turn-halting `API Error: <ConnectionTerminated error_code:0 ...>` = the
  upstream HTTP/2 connection rotated mid-stream (GOAWAY surfacing through the Headroom
  proxy); mid-stream POSTs are SDK-unretryable, so the turn halts. Transient and
  content-independent. Recovery: session context survives — `git status` to confirm tree
  state, then continue the interrupted action.
- RTK mangles `git commit` carrying multiple `-m` values with non-ASCII (§,
  em-dash): args drop/split, the commit silently never lands while RTK prints ok (the add
  staged). Commit such messages from a file — `git commit -F <path>`, then rm it. Plain
  single-`-m` ASCII commits are safe.
- Serena symbolic tools erroring `Active languages: [...]`: add the language
  to `.serena/project.yml` `languages:` (first entry = fallback LS), then ask the user to
  /mcp-reconnect serena (config is read only at startup); verify with a symbol call (the
  first may lag on indexing). rustup keeps rust-analyzer current. Serena startup
  regenerates project.yml to its full annotated template whenever keys are missing — track
  the file exactly as Serena writes it; stripping it re-dirties the tree every session.
- Headroom-compressed Reads re-wrap long prose lines (roadmap.md, Rust
  doc-comment blocks), so an Edit old_string assembled from such a Read can miss the file's
  real wrap points. Print the target lines raw (`sed -n`/grep) and anchor on those bytes;
  the Edit error's `\uXXXX` hint points the wrong way. Recurs in every closing commit that
  edits roadmap.md.
- Backtick-wrap regexes/grammars in markdown — bare adjacent bracket groups
  parse as reference links (phantom Marksman warnings; verify with grep for `][` outside
  code spans). Serena get_diagnostics_for_file routes non-code files to the fallback LS and
  is useless there; Marksman diagnostics reach the session only via the harness
  new-diagnostics channel. Marksman's index honors `.ignore`/`.gitignore`/`.hgignore`
  (Folder.fs ignoreFiles); an ignored markdown target turns valid links into "non-existent
  document" warnings — hence docs/ sweep-exclusion lives in `.rgignore` (rg-only,
  Marksman-invisible) and link diagnostics are trustworthy. Ignore files are read at folder
  scan, not watched: such fixes clear at the next LSP start. Marksman is settled (kept
  deliberately 2026-06): all three warning shapes are real quick fixes — phantom reflink:
  backtick the notation; non-existent document: repair the link; "Ambiguous link": target
  doc has >1 H1 (title_from_heading registers every H1 as a title; keep one, demote the
  rest) — apply and move on, reporting only the fix. Diagnostics are unconfigurable (none in .marksman.toml; Diag.fs gives phantom reflinks and real broken md
  links the same code 2/Warning, so any filter kills the signal too); the off-switch is
  markdown-lsp@ckc-lsps in settings.json enabledPlugins.
- Serena replace_symbol_body spans the preceding doc comment AND outer
  `#[...]` attributes — a replacement body omitting them deletes them (lost a derive this
  way). Include the leading `///` lines and every attribute, or edit inner regions with
  replace_content instead. Recurs with derive-heavy enums.
- RTK rewriting can falsify output, both directions: `diff`
  printed "identical" for differing files; standalone `grep` becomes rg (write rg-syntax
  patterns) while piped/compound grep falls through to real grep (plain `grep -E`; rg-only
  flags fail); a bare `rg` call can become real grep; combined grep short flags reparse as
  rg flags (`grep -rln` ran as rg `-r ln` = --replace ln: zero output, no error); `git log`
  piped output silently truncates at 50 entries with no marker — full listings and
  counts via `rtk proxy git log` or `git rev-list --count`. Treat
  unexpected empty/odd search output as rewrite-suspect and re-run under `rtk proxy rg`;
  prove byte-equality with `cmp`/`sha256sum` only; real diffs via `git diff --no-index` or
  `rtk proxy diff`. Critical wherever byte-compatibility is the acceptance bar.
- Editing-tool string parameters decode `\uXXXX` escapes — and only those
  (`\n`, `\xNN` pass through) — so source that must contain a literal backslash-u gets
  silently corrupted and often still compiles. Express such bytes without that substring
  (byte-array literal with `0x5c` for the backslash) and read the region back after
  writing. Recurs in escape-syntax tests (wire parsers).
- Subagent window = session window; the launch flag decides both. The user
  toggles 200K/1M solely by prefixing `claude` with `CLAUDE_CODE_DISABLE_1M_CONTEXT=1`
  (terminal-only; settings carry only model slugs); the flag gates the 1M beta header
  process-wide (v2.1.170 short-circuits both 1M paths: `[1m]` suffix parse and the
  always-1M model allowlist). Flag on — every non-review session — leaves the global
  `CLAUDE_CODE_SUBAGENT_MODEL=<model>[1m]` slug silently inert: every subagent caps
  at 200K. Flag off (review sessions): subagents get 1M via both paths. The slug keeps
  `[1m]` by user choice; a subagent env block echoing it verifies model selection only,
  never the effective window. Overflow = hard mid-task death (subagents never compact):
  the API rejects the next request, the Agent result carries an INLINE `Prompt is too
  long` (tool_uses > 0, subagent_tokens 0, no exception, no result); pinned by a probe of
  ~19K-token reads dying at read 10, the 200K boundary. Sizing: in 200K sessions budget
  every subagent at 200K with margin — a read+rewrite agent handles ~40KB text (~100K
  peak); chunk larger rewrites at section boundaries and processing stage outputs for main-session
  assembly. Per-agent transcripts:
  `~/.claude/projects/<project>/<session-id>/subagents/agent-<id>.jsonl` (assistant
  `.message.usage`; rejected requests log no usage). compaction.sh reads the same flag and
  gauges the main loop only.

- Renaming canonical (§4.3) JSON member keys is a silent test-breaker. The object emitter buffers members then sorts them by key bytes on `finish`; the reader (`canon.rs` `member`/`optional`) is positional — it peeks the next key and demands the caller request keys in ascending byte order. So a key rename moves its sort slot: the code still compiles, but round-trip reads fail `MissingField` at runtime and pinned canonical byte-string literals mismatch. Fix = re-sort each Canonical read+emit member sequence AND every pinned byte-string to the new key order (`printf '%s\n' k1 k2 … | LC_ALL=C sort`). Related: a `#[serde(rename_all="snake_case")]` enum serializes by variant name, so a snake wire-key rename must also rename the CamelCase variant (e.g. ViewText→RenderedText) — caught by name-pin asserts, never the compiler. And hyphenated scope-IDs (`stage-extract.1`, `core-grounding`, `fixtures-m1`) in roadmap+comments are git-commit-traceability keys: keep them historical on a terminology rename (rename only dotted runtime IDs `processing_stage.m1.*` and living prose).
