# Agent Memory

Entries must add value beyond the spec, AGENTS.md/CLAUDE.md, codebase, git history, and runtime
environment — project-independent tooling pitfalls (RTK, Headroom, Serena, Claude Code, web
access) live in the global `~/.claude/CLAUDE.md`, not here. Exception: high-value reminders that
are derivable but easily forgotten under token pressure. Entries are consolidated aggressively;
full pre-consolidation text lives in git history.

## Policy

- Context hygiene (user directive; background: `git show 531f586`): keep every
  session lean and phrased in project vocabulary (processing stages, units, gates, artifacts) — plain
  operational words over research jargon in memory, roadmap, commits, and code alike.
  Consult `docs/` through read-only subagents so its vocabulary stays out of
  the main window. Root `.rgignore` keeps ripgrep-backed sweeps (subagent Grep, `rtk proxy
  rg`) out of `docs/`; Bash `grep -r` still enters it — scope Bash greps by path; deliberate
  docs searches use `git grep <pat> -- docs/`, `rg --no-ignore`, or explicit file paths.
  Implement sessions match patterns from the latest unit-scoped commit (`git log
  --oneline`), not bare HEAD, when HEAD is hygiene/memory work.
- LSP coverage map (wiring is global per ~/.claude: Serena primary via per-project
  `.serena/project.yml` `languages:`, the global `global` marketplace for solidlsp gaps —
  no project marketplace). ckc's hand-authored/byte-pinned formats and their provider:
  rust, bash, json, yaml, toml, markdown (Marksman, Serena-bundled), html, lean4 sit in
  Serena `languages:`; xml, smt2 (dolmen), alloy, egglog are global plugins. Delivery
  differs: Serena formats surface only via an explicit get_diagnostics_for_file call;
  global-plugin formats also push passively through the harness new-diagnostics channel.
  Add a format = list it in `languages:` if solidlsp does it (restart Claude Code to load),
  else lean on / add a global gap plugin. §13 formal targets: alloy, egglog covered (global
  plugins); lean4 sits in `languages:` but the Lean server starts only once .lean files
  exist. No standalone LSP (audited): TLA+, ASP/Clingo, categorical-CQL; Isabelle lacks
  solidlsp (global gap plugin at adoption), Python is solidlsp-covered (add to `languages:`
  at adoption) — §13 additional-targets, §13.1 adapter boundary. Compendium families present
  only as registry YAML data carry no LSP.

## Lessons

- Unit sizing rules (consolidated from roadmap `NN%` annotations and observed 200K overruns;
  per-incident case studies live in git history — `git show 6e413f0^:.agent/memory.md`). Target: one
  conceptual deliverable plus one gate, finishable AND committable in one window with margin;
  prefer more, smaller units. Plan-time obligations (a violation is a planning bug): resolve
  semantic requirements decisions INTO the roadmap line (more than ~2 left open = re-scope);
  research and pin any new external dependency (exact version + features) in the line;
  pre-split multi-deliverable stacks BEFORE scheduling — mid-session overrun recovery is
  user-initiated (stop, bring the tree clean, report); minting a split rule re-audits every
  remaining unchecked line against it in the same recovery commit; a recovery split is itself
  plan work — audit its replacement lines against every standing rule and the open-decision
  ceiling within the recovery commit. Split rules: a feature needing a refactor of existing
  code to share internals takes the refactor as its own behavior-locked unit FIRST (existing
  tests the gate, zero test edits); a format walker plus committed test-source integration =
  walker-core (inline-literal tests) then format-completion + test-source integration; a
  nontrivial algorithm plus a second authored artifact = two units; a multi-invariant
  validator plus full rejection coverage = two units; a derivation fn with its
  test-source-pinned battery plus an attachment sub-feature = two units; a type family plus
  assembly plus validation = three units; an assembly fn plus its live-pipeline pin battery =
  two units; a live-pin battery over the run binary is a unit on its own (never paired with
  assembly or processing-stage wiring); a spec-byte amendment (re-pin + reference/test mirror
  sweep) bundled with new feature code = two units — an open decision whose resolution amends
  pinned bytes is a deliverable, not a session preamble; crate foundations pair only with a
  small type surface (one payload module per foundation unit). Measured anchors (checked
  roadmap stubs carry the `NN%` figures): canonical JSON = five units; a five-layer recursive
  type family = three units; a lexicon-driven derivation half (loader / binding / builder) =
  three units; statement builder over a prebuilt binding core = one unit; exception
  attachment + determinism tests = one unit. Practices: house new type families in fresh
  modules (extending a ~2K-line module costs a full-file read); scope each split unit's
  Reading slice to exclude files its half leaves untouched; land a compiling skeleton before
  the full test battery; salvage a reverted session's compiling half as a committed
  `.agent/wip-<unit>.patch` the redo line points at (apply, verify against the line, delete in
  the closing commit) — an uncompiled draft salvages the same way flagged UNCOMPILED,
  transcription-with-verification still beating re-derivation; pin expected shapes from
  observed output, never hand-computed; cite only checked roadmap lines as measured anchors.
  At plan/re-scope time, audit any spec listing a unit must byte-reproduce: listings written
  for readability (alignment padding, inline result comments, illustrative declaration or
  conjunct order) contradict deterministic-emission rules and need a scheduled re-pin
  deliverable (caught pre-session for smt-emit.3a: §8.6 smt2 vs §6 sorted-declaration rule).
- WebSearch is denied here by a PreToolUse hook (`.claude/hooks/deny-websearch.sh`, wired in
  `settings.json`) because the tool 400s on this model line — the global `~/.claude/CLAUDE.md`
  carries the why and the web-access channels to use instead. Re-test/heal: drop the
  `settings.json` hooks entry, ToolSearch-load WebSearch, run one query; healed → delete the
  hook script + entry and this note.
- Backtick-wrap regexes/grammars in markdown — bare adjacent bracket groups
  parse as reference links (phantom Marksman warnings; verify with grep for `][` outside
  code spans). Marksman is Serena's markdown server (solidlsp bundles it), active because
  `markdown` is in `.serena/project.yml` `languages:`; its diagnostics surface only via an
  explicit get_diagnostics_for_file call (verified: source "Marksman", code 2/Warning), not
  the harness new-diagnostics channel — that passive push was the removed standalone
  markdown-lsp plugin's path (Serena is MCP, not a Claude Code LSP plugin), so query markdown
  diagnostics; they no longer auto-appear on edit. Off-switch: drop `markdown` from
  `languages:` (restart Claude Code to apply). Marksman's index honors
  `.ignore`/`.gitignore`/`.hgignore` (Folder.fs ignoreFiles); an ignored markdown target
  turns valid links into "non-existent document" warnings — hence docs/ sweep-exclusion
  lives in `.rgignore` (rg-only, Marksman-invisible) and link diagnostics are trustworthy.
  Ignore files are read at folder scan, not watched: such fixes clear at the next LSP start.
  Marksman is settled (kept deliberately 2026-06): all three warning shapes are real quick
  fixes — phantom reflink: backtick the notation; non-existent document: repair the link;
  "Ambiguous link": target doc has >1 H1 (title_from_heading registers every H1 as a title;
  keep one, demote the rest) — apply and move on, reporting only the fix. Diagnostics are
  unconfigurable (none in .marksman.toml; Diag.fs gives phantom reflinks and real broken md
  links the same code 2/Warning, so any filter kills the signal too).
- Renaming canonical (§4.3) JSON member keys is a silent test-breaker. The object emitter buffers members then sorts them by key bytes on `finish`; the reader (`canon.rs` `member`/`optional`) is positional — it peeks the next key and demands the caller request keys in ascending byte order. So a key rename moves its sort slot: the code still compiles, but round-trip reads fail `MissingField` at runtime and pinned canonical byte-string literals mismatch. Fix = re-sort each Canonical read+emit member sequence AND every pinned byte-string to the new key order (`printf '%s\n' k1 k2 … | LC_ALL=C sort`). Related: a `#[serde(rename_all="snake_case")]` enum serializes by variant name, so a snake wire-key rename must also rename the CamelCase variant (e.g. ViewText→RenderedText) — caught by name-pin asserts, never the compiler. And hyphenated scope-IDs (`stage-extract.1`, `core-grounding`, `fixtures-m1`) in roadmap+comments are git-commit-traceability keys: keep them historical on a terminology rename (rename only dotted runtime IDs `processing_stage.m1.*` and living prose).
- Test/example producer IDs: `pipe.<qual>` (`pipeline_id`) + `processing_stage.<qual>.<step>` (`pipeline_step_id`); shared `<qual>` links a pipeline to its steps. Generic unit fixtures use `qual=test`; scenario fixtures keep their own (`m1`/`t`/`base`). Never `cand.*`/`comp.*` — those echo the pre-rename `candidate`/`component` field names the terminology cleanup removed.
- "Oracle" has two senses; the `terms:`/`codex:` cleanup (`b0e51b2`/`caefcbb`/`e4f983a`) renamed only the epistemic-overclaim one — `runtime-oracle` → `runtime reference` across IDs/types/prose (results are locked measurements, not an authority on real-world truth). Scope: SPEC.md + Rust + registry/corpus/reference data + IDs + config; `docs/` excluded. The commits cite a replacement map whose contents aren't recoverable from git — so "the map omitted generic `oracle`" is inference; what's verifiable is that only runtime-oracle terms were swapped. The TEST-ORACLE sense (authority deciding a test's pass/fail vs. the reference) persists in `run_oracle.rs` (file + `run_oracle_*` fn) and `rules.rs` (`// THE oracle`); those files passed through the sweep commits unrenamed — survival, not a documented approval — and the phrasing recurs in out-of-scope `docs/` ("test oracle"/"SAT oracle"/"perfect oracle") as ordinary technical usage (corroboration, not proof). SPEC.md has zero "oracle"; no instruction mandates global removal (nearest pull: the general "plain operational words over research jargon" line above). Decision: NARROW — leave the test-sense as-is. A global test-sense retirement (`run_oracle.rs`→`run_reference_check.rs`) stays an OPEN user/style call.
- ckc-smt's `serde` dep reads as unused (no `serde::`/`Serialize`/`Deserialize` in ckc-smt/src
  beyond the `fieldless_enum!` invocations) but is REQUIRED: that ckc-core macro expands to
  `::serde::Serialize`/`Deserialize` impls *in the caller's crate*, so every fieldless_enum! user
  must depend on serde — dropping it breaks the build (`E0433` unresolved `::serde`). Holds for any
  crate adopting the macro. Those serde impls go unused there (the canonical path is
  Canonical/CanonRead), an accepted KISS cost of one shared macro over per-call serde gating; don't
  "tidy" the dep away.
- M1 review verdict (REVIEWED, zero code fixes): gates green, all nine §8.5 mechanisms + every §8.6
  byte-pin verified live through the real pipeline, determinism/canonical guarantees hold,
  data files conform to §8.2/§8.4/§8.6. Two items left OPEN for the user (not code defects).
  (1) §4.4 TotalOperationResult is sparse (one example, no field prose): M1 maps per-processing-stage
  results to §4.6 EventRecords (outcome + in/out hashes), not per-stage TotalOperationResults, and
  the single command-total leaves `value_hashes` + the typed residual/ambiguity/incoherence buckets
  empty (all diagnostics → `diagnostic_hashes`). Defensible (passes §8.5; M1 has no typed-placeholder
  artifacts; outputs attested via the run manifest) — clarify the intended per-stage result type and
  bucket partition before M2 populates them. (2) Tests are example/byte-pin only; property-based /
  fuzzing for the canon layer (round-trip identity, reject-any-mutation) and StringPolicy
  (idempotence) is the AGENTS.md-preferred strengthening, currently unscheduled.
