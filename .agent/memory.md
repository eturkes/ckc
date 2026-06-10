# Agent Memory

Entries must add value beyond the spec, CLAUDE.md, codebase, git history, and runtime
environment. Exception: high-value reminders that are derivable but easily forgotten under
token pressure. Entries are consolidated aggressively; full pre-consolidation text lives in
git history.

## Policy

- [2026-06-11] Context hygiene (user directive; rationale and full background: `git show
  46d95e2`): keep every session's context lean and phrased in project vocabulary (stages,
  units, gates, artifacts) — plain operational words over research jargon, in memory,
  roadmap, commits, and code alike. Consult `docs/` surveys through read-only subagents
  so their vocabulary stays out of the main window; a root `.ignore`
  keeps ripgrep-backed sweeps (subagent Grep tool, `rtk proxy rg`) out of `docs/`, while
  Bash `grep -r` still enters it — scope Bash greps by path; deliberate docs searches use
  `git grep <pat> -- docs/`, `rg --no-ignore`, or explicit file paths. Checked roadmap
  items collapse to one-line stubs; full unit text lives in git history. Implement sessions
  match patterns from the latest unit-scoped commit (`git log --oneline`), not bare HEAD,
  when HEAD is hygiene/memory work. If degraded-model output is ever suspected: a session's
  self-report is zero evidence; evidence channels are the Headroom proxy log (request bytes,
  per-response model ids), transcript jq for fallback events, and paired A/B task batteries
  scored by objective gates; CKC's deterministic gates and replay manifests are the standing
  mitigation. The clinical-text fallback entry below stays authoritative for its visible
  case.
- [2026-06-11] Lexgate guards the durable tree: `bash .agent/lexgate.sh` with modes `hook`
  (write-time, wired in settings.json), `pre-commit` (installed in .git/hooks; `install`
  restores it), `sweep` (review sessions run it), `check` (when it reports missing parts,
  stop and ask the user), `scan <path>`. Patterns live in `.agent/lexgate.d/` — local-only,
  gitignored, Read-denied, user-maintained; a clean session treats the gate as pass/fail
  only and a failure as rewording work on the cited lines (the gate cites file:line and
  never echoes matches). Patterns are scoped to vocabulary with no legitimate in-project
  use (user-recalibrated 2026-06-11); when a cited line's flagged term names a legitimate
  project component, report the hit to the user as a pattern bug instead of rewording. docs/ and corpus/fixtures/ are the sanctioned containers; consult
  them via read-only subagents instructed to answer in project vocabulary without verbatim
  quotes.

## Lessons

- Unit sizing rules (consolidated 2026-06-11 from roadmap `NN%` annotations plus three
  observed 200K overruns; case studies in git history). Target: one conceptual deliverable
  plus one gate, finishable AND committable in one window with margin; prefer more, smaller
  units. Plan-time obligations (a violation is a planning bug): resolve semantic contract
  decisions INTO the roadmap line (more than ~2 left open = re-scope); research and pin any
  new external dependency (exact version + features) in the line; pre-split
  multi-deliverable stacks BEFORE scheduling — mid-session overrun recovery is
  user-initiated (stop, bring the tree clean, report; the user restores and re-scopes).
  Split rules: a feature needing a refactor of existing code to share internals takes the
  refactor as its own behavior-frozen unit FIRST (existing tests are the gate, zero test
  edits); a format walker plus committed-fixture integration splits into walker-core
  (inline-literal tests) then format-completion + fixture-integration; a nontrivial
  algorithm plus a second authored artifact is two units; a multi-invariant validator plus
  full rejection coverage is two units; a derivation fn with its fixture-pinned battery plus
  an attachment sub-feature is two units; a type family plus assembly plus validation is
  three units. Measured anchors: canonical JSON = five units (~62-69% each); a strict
  reader (writer-inverse) fills a window solo; crate foundations run ~81% (pair only with a
  small type surface); registry entry types ~69%; a five-layer recursive type family ~3
  units; a lexicon-driven derivation half (loader / binding / builder) = three units;
  statement builder over a prebuilt binding core = one unit; exception attachment +
  determinism tests = one unit. Practices: house new type families in fresh modules
  (extending a ~2K-line module costs a full-file read); land a compiling skeleton before
  the full test battery; pin expected shapes from observed output, never hand-computed;
  cite only checked roadmap lines as measured anchors.
- [2026-06-10] WebSearch 400s on this model line (the API rejects the forced tool_choice
  the search sub-request uses; the error arrives INLINE in an ok-looking result — read
  result bodies). Re-test after a Claude Code update or model-line change; drop this clause
  when healed. Workflow agent() `schema` is verified unaffected (live canary). Working
  channels: WebFetch on `https://lite.duckduckgo.com/lite/?q=<query>` (sandbox curl gets a
  bot wall); crates.io via curl with a `-A` user-agent header (403 without) — detail
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
- [2026-06-11] Fable 5 refusal fallback silently switches a session to Opus mid-flight
  (`type=system subtype=model_refusal_fallback` transcript event, flagged "cybersecurity or
  biology topics"). One-way and in-context invisible — the post-switch assistant continues
  the unit unaware. Trigger: raw Japanese clinical fixture text entering context via
  whole-fixture reads; probabilistic, not deterministic (three switches, then an
  identical-vocabulary session stayed on fable). Handling: Opus-mode output is discarded;
  the user monitors for the switch — on any suspicion of degraded-model output, stop and
  report. Expect recurrence in fixture-reading units (stage-normalize.2, cli-runner.2,
  acceptance-v1). Forensics: per-record models via `jq '.message.model'` over assistant
  records in `~/.claude/projects/-run-host-home-eturkes-Projects-ckc/<session>.jsonl`; the
  fallback event is the switch marker. Full case detail (transcript ids, timestamps,
  trigger tokens): `git show 14e520b`.
