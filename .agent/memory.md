# Agent Memory

Entries must add value beyond the spec, CLAUDE.md, codebase, git history, and runtime
environment. Exception: high-value reminders that are derivable but easily forgotten under
token pressure. Entries are consolidated aggressively; full pre-consolidation text lives in
git history.

## Policy

- [2026-06-11] Context hygiene (user directive; background: `git show 46d95e2`): keep every
  session lean and phrased in project vocabulary (stages, units, gates, artifacts) — plain
  operational words over research jargon in memory, roadmap, commits, and code alike.
  Consult `docs/` through read-only subagents so its vocabulary stays out of
  the main window. Root `.ignore` keeps ripgrep-backed sweeps (subagent Grep, `rtk proxy
  rg`) out of `docs/`; Bash `grep -r` still enters it — scope Bash greps by path; deliberate
  docs searches use `git grep <pat> -- docs/`, `rg --no-ignore`, or explicit file paths.
  Checked roadmap items collapse to one-line stubs (full unit text in git history).
  Implement sessions match patterns from the latest unit-scoped commit (`git log
  --oneline`), not bare HEAD, when HEAD is hygiene/memory work.
- [2026-06-11] Lexgate guards the durable tree: `bash .agent/lexgate.sh` with modes `hook`
  (write-time, wired in settings.json), `pre-commit` (installed in .git/hooks; `install`
  restores it), `sweep` (review sessions run it), `check` (missing parts → stop and ask the
  user), `scan <path>`. Patterns live in `.agent/lexgate.d/` — local-only, gitignored,
  Read-denied, user-maintained (recalibrated 2026-06-11 to vocabulary with no legitimate
  in-project use); a clean session treats the gate as pass/fail only and a failure as
  rewording work on the cited lines (the gate cites file:line, never echoes matches). When a
  cited line's flagged term names a legitimate project component, report a pattern bug
  instead of rewording. Sanctioned containers: docs/ and corpus/fixtures/ — consult them via
  read-only subagents instructed to answer in project vocabulary without verbatim quotes.
- [2026-06-11] LSP coverage criterion: ckc-lsps plugins and Serena languages track formats
  whose concrete syntax gets hand-authored or byte-pinned in-repo (active:
  rust, bash, json, yaml, toml, markdown, html, xml, smt2 via dolmen; §13-named targets:
  lean4, alloy, egglog) — compendium-catalogued families whose registry presence is YAML
  data carry no plugin. TLA+, ASP/Clingo, and categorical-CQL have no standalone LSP server
  (2026-05 audit); Isabelle's LSP and any Python LSP land
  only with their adoption decisions (§13 additional-targets row; §13.1 adapter boundary).
  dolmen-lsp deploys as a standalone copied binary with the opam tree removed — rebuild
  recipe in its plugin README.

## Lessons

- Unit sizing rules (consolidated 2026-06-11 from roadmap `NN%` annotations plus three
  observed 200K overruns; case studies in git history). Target: one conceptual deliverable
  plus one gate, finishable AND committable in one window with margin; prefer more, smaller
  units. Plan-time obligations (a violation is a planning bug): resolve semantic contract
  decisions INTO the roadmap line (more than ~2 left open = re-scope); research and pin any
  new external dependency (exact version + features) in the line; pre-split
  multi-deliverable stacks BEFORE scheduling — mid-session overrun recovery is
  user-initiated (stop, bring the tree clean, report). Split rules: a feature needing a
  refactor of existing code to share internals takes the refactor as its own
  behavior-frozen unit FIRST (existing tests the gate, zero test edits); a format walker
  plus committed-fixture integration = walker-core (inline-literal tests) then
  format-completion + fixture-integration; a nontrivial algorithm plus a second authored
  artifact = two units; a multi-invariant validator plus full rejection coverage = two
  units; a derivation fn with its fixture-pinned battery plus an attachment sub-feature =
  two units; a type family plus assembly plus validation = three units. Measured anchors:
  canonical JSON = five units (~62-69% each); a strict reader (writer-inverse) fills a
  window solo; crate foundations ~81% (pair only with a small type surface); registry entry
  types ~69%; a five-layer recursive type family ~3 units; a lexicon-driven derivation half
  (loader / binding / builder) = three units; statement builder over a prebuilt binding
  core = one unit; exception attachment + determinism tests = one unit. Practices: house
  new type families in fresh modules (extending a ~2K-line module costs a full-file read);
  land a compiling skeleton before the full test battery; pin expected shapes from observed
  output, never hand-computed; cite only checked roadmap lines as measured anchors.
- [2026-06-10] WebSearch 400s on this model line (the API rejects the forced tool_choice
  the search sub-request uses; the error arrives INLINE in an ok-looking result — read
  result bodies). Still broken 2026-06-11; re-test after a Claude Code update or model-line
  change, drop this clause when healed. Workflow agent() `schema` verified unaffected.
  Working channels: WebFetch on `https://lite.duckduckgo.com/lite/?q=<query>` (sandbox curl
  gets a bot wall); crates.io via curl with a `-A` user-agent header (403 without) — detail
  /api/v1/crates/NAME, search /crates?q=; GitHub /search/repositories?q=; Wikipedia
  opensearch. Meta-rule: the session that FIRST hits an environment/tool failure records it
  in that same session.
- [2026-06-10] Turn-halting `API Error: <ConnectionTerminated error_code:0 ...>` = the
  upstream HTTP/2 connection rotated mid-stream (GOAWAY surfacing through the Headroom
  proxy); mid-stream POSTs are SDK-unretryable, so the turn halts. Transient and
  content-independent. Recovery: session context survives — `git status` to confirm tree
  state, then continue the interrupted action.
- [2026-06-09] RTK mangles `git commit` carrying multiple `-m` values with non-ASCII (§,
  em-dash): args drop/split, the commit silently never lands while RTK prints ok (the add
  staged). Commit such messages from a file — `git commit -F <path>`, then rm it. Plain
  single-`-m` ASCII commits are safe.
- [2026-06-09] Serena symbolic tools erroring `Active languages: [...]`: add the language
  to `.serena/project.yml` `languages:` (first entry = fallback LS), then ask the user to
  /mcp-reconnect serena (config is read only at startup); verify with a symbol call (the
  first may lag on indexing). rustup keeps rust-analyzer current. Serena startup
  regenerates project.yml to its full annotated template whenever keys are missing — track
  the file exactly as Serena writes it; stripping it re-dirties the tree every session.
- [2026-06-10] Headroom-compressed Reads re-wrap long prose lines (roadmap.md, Rust
  doc-comment blocks), so an Edit old_string assembled from such a Read can miss the file's
  real wrap points. Print the target lines raw (`sed -n`/grep) and anchor on those bytes;
  the Edit error's `\uXXXX` hint points the wrong way. Recurs in every closing commit that
  edits roadmap.md.
- [2026-06-10] Backtick-wrap regexes/grammars in markdown — bare adjacent bracket groups
  parse as reference links (phantom Marksman warnings; verify with grep for `][` outside
  code spans). Serena get_diagnostics_for_file routes non-code files to the fallback LS and
  is useless there; Marksman diagnostics reach the session only via the harness
  new-diagnostics channel.
- [2026-06-10] Serena replace_symbol_body spans the preceding doc comment AND outer
  `#[...]` attributes — a replacement body omitting them deletes them (lost a derive this
  way). Include the leading `///` lines and every attribute, or edit inner regions with
  replace_content instead. Recurs with derive-heavy enums.
- [2026-06-10] RTK rewriting can falsify output: `diff` printed "identical" for differing
  files; standalone `grep` becomes rg (write rg-syntax patterns), while piped/compound grep
  falls through to real grep (write plain `grep -E`; rg-only flags fail). Prove
  byte-equality with `cmp`/`sha256sum` only; real diffs via `git diff --no-index` or `rtk
  proxy diff`. Critical wherever byte-compatibility is the acceptance bar.
- [2026-06-10] Editing-tool string parameters decode `\uXXXX` escapes — and only those
  (`\n`, `\xNN` pass through) — so source that must contain a literal backslash-u gets
  silently corrupted and often still compiles. Express such bytes without that substring
  (byte-array literal with `0x5c` for the backslash) and read the region back after
  writing. Recurs in escape-syntax tests (wire parsers).
- [2026-06-11] Agent-tool subagents run at the stock 200K context window even when the main
  session runs at 1M — the 1M window does not propagate. The overflow surfaces as an INLINE
  `Prompt is too long` string in the Agent result (tool_uses > 0, subagent_tokens 0), with
  no exception raised. Evidence: two whole-file rewrite agents died after last successful
  calls at 176K/145K input tokens (next request projected past 200K) while the same
  session's main loop ran at 485K; no subagent call ever exceeded 200K. Per-agent
  transcripts: `~/.claude/projects/<project>/<session-id>/subagents/agent-<id>.jsonl`
  (assistant `.message.usage` records; rejected requests log no usage). Sizing rule: budget
  every subagent at 200K — a read+rewrite agent handles ~40KB of text comfortably (~100K
  peak); chunk larger rewrites at section boundaries and stage outputs for main-session
  assembly. compaction.sh gauges the main loop only.
- [2026-06-11] Fable 5 refusal fallback silently switches a session to Opus mid-flight
  (`type=system subtype=model_refusal_fallback` transcript event, flagged "cybersecurity or
  biology topics"). One-way and in-context invisible — the post-switch assistant continues
  unaware. Trigger: raw Japanese clinical fixture text entering context via whole-fixture
  reads; probabilistic, not deterministic (three switches, then an identical-vocabulary
  session stayed on fable). Mitigation: `Read(./corpus/fixtures/**)` is settings-denied
  (2026-06-11) — work with fixtures through code, tests, and path-scoped greps. Handling:
  Opus-mode output is discarded; the user monitors for the switch — on any suspicion of
  degraded-model output, stop and report. A session's self-report is zero evidence;
  evidence channels: the Headroom proxy log (request bytes, per-response model ids),
  transcript jq over assistant records (`jq '.message.model'` in
  `~/.claude/projects/-run-host-home-eturkes-Projects-ckc/<session>.jsonl`; the fallback
  event is the switch marker), and paired A/B task batteries scored by objective gates;
  CKC's deterministic gates and replay manifests are the standing mitigation. Expect
  recurrence in fixture-reading units (stage-normalize.2, cli-runner.2, acceptance-m1).
  Full case detail (transcript ids, timestamps, trigger tokens): `git show 14e520b`.
