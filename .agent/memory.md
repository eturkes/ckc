# Agent Memory

Every entry must provide value beyond what SPEC.md, CLAUDE.md, the codebase,
git history, config files, tool `--version` output, and the runtime
environment already provide. Exception: high-value reminders that are
technically derivable but easily forgotten under token pressure.

## Lessons

- [2026-05-27] Every piece of natural language in the project (docs, comments,
  diagnostics) must use positive constraint framing per the pink elephant
  principle in CLAUDE.md. Audit before committing.
- [2026-05-27] Memory entries, roadmap descriptions, and doc preambles are the
  highest-risk locations for redundancy with SPEC.md and config files. Default
  assumption: if a fact lives in a file, it belongs only in that file.
- [2026-05-27] Spend freely on reasoning and iteration within a session, but
  all content written to files must be token-efficient for future LLM readers.
  Self-audit this balance before committing; the user expects it done
  proactively.
- [2026-05-27] When a mistake is caught after the fact, record it here so
  future sessions avoid repeating it. Verify tool operations against their
  actual output before reporting success.
- [2026-05-28] Subtask sizing directive (user): size each roadmap subtask so a
  fresh agent completes AND commits it within one context window with margin; if
  it would need compaction, split it. One conceptual deliverable + one gate per
  subtask; prefer more, smaller subtasks. Operationalized in `.agent/prompt.md`
  → "Subtask sizing". Apply when planning (Type A) and any re-planning.
- [2026-05-28] CAS determinism pitfall: `StoreManifest` entries carry
  `stored_at_epoch` (file mtime), so raw manifest canonical bytes differ across
  runs/machines. Assert cross-run determinism on per-artifact
  `content_hash`/`envelope_hash` (or manifest entries minus timestamp), as in
  ckc-store `cas_manifest_hash_is_stable` and `all_fixtures_have_deterministic_hashes`.
  Avoid asserting raw manifest-byte equality across independent runs.
- [2026-05-28] Fresh-container toolchain drift: `rust-toolchain.toml` pins
  channel `stable` (not a specific version), so a new container's rustup may
  resolve a newer stable than committed code was formatted/linted under. Result:
  `just fmt-check` and `just clippy` may fail on otherwise-correct code while
  `just test` still passes. Treat such failures as drift, not regressions;
  remediation is `cargo fmt --all` + manual clippy fixes (then commit), or pin
  the toolchain to a specific version in `rust-toolchain.toml`. Verify tooling
  via `just test` first when bringing up a fresh environment.
- [2026-05-28] Non-interactive `Bash` tool invocations do NOT source
  `~/.bashrc`, so `PATH` additions appended there (e.g. for mise shims) are
  invisible per-command. Either prefix commands with
  `PATH="$HOME/.local/share/mise/shims:$PATH"` or rely on tools already in the
  base PATH (`~/.cargo/bin`, `~/.local/bin`, `~/.local/share/pnpm/bin`).
- [2026-05-29] The Claude `rust-analyzer-lsp` plugin requires the real
  `rust-analyzer` binary on PATH; `~/.cargo/bin/rust-analyzer` is a rustup
  multiplexer symlink that exists even when the component is not installed,
  and invoking it without the component yields `Unknown binary 'rust-analyzer'
  in official toolchain`. The Claude LSP host swallows this and surfaces only
  a generic `Tool result missing due to internal error` after a long hang.
  Fix: `rustup component add rust-analyzer`. Reproduce-detect cheaply by
  running `rust-analyzer --version` before debugging the LSP host. Note this
  supersedes the stance in commit 2fa9393, which deferred the install when
  Claude's diagnostic loop relied only on `cargo check`/`clippy`/`test`.
- [2026-05-29] Project-local LSP marketplace lives at
  `.claude/marketplace/.claude-plugin/marketplace.json` and is wired in via
  project-scope `.claude/settings.json` (relative path,
  `.claude/marketplace`). The Claude LSP host caches plugin discovery at
  session start, so any new `*-lsp@ckc-lsps` plugin requires a Claude Code
  restart before `LSP` calls find it. Use `claude plugin list` to confirm
  enablement state cheaply without restarting. `claude plugin marketplace
  add … --scope project` + `claude plugin install <name> --scope project`
  is the supported automation path. Per-LSP install steps and gotchas live
  in `.claude/marketplace/plugins/<name>-lsp/README.md`.
- [2026-05-29] `bgcmd` (single-file bash script from
  github.com/izabera/bgcmd) is installed at `~/.local/bin/bgcmd` for driving
  interactive REPLs across separate `Bash` tool calls (solvers like
  z3/cvc5/clingo in interactive mode, lean REPL, duckdb, psql, gdb/rr, python).
  Required env: `BGCMDPROMPT` must equal the REPL's exact prompt string;
  `BGCMDDIR` (defaults `$HOME/.bgcmd`) holds the in/out fifos + pid file.
  Pattern: `BGCMDDIR=path BGCMDPROMPT='>>> ' bgcmd START python3 -i -q` then
  `BGCMDDIR=path BGCMDPROMPT='>>> ' bgcmd '<line>'` per turn; the prompt env
  vars must be re-exported on each call because non-interactive `Bash` does not
  preserve shell state. Per-REPL wrapper scripts (README pattern) eliminate
  this repetition when used heavily. To run multiple concurrent REPLs, give
  each a distinct `BGCMDDIR`. REPLs that suppress the prompt under non-TTY
  stdout require a force-interactive flag (e.g. python `-i`, bash
  `--noediting -i`). No pty is spawned, so ANSI escapes do not contaminate
  output. Cleanup: `kill "$(cat $BGCMDDIR/pid)"` then `rm -rf $BGCMDDIR`.
- [2026-05-29] No standalone LSP exists as of 2026-05 for these
  SPEC verification-target formats (audited at source):
  * TLA+ — `tlaplus/vscode-tlaplus` is a TS extension that shells out to
    `tla2tools.jar`; no LSP server component, no separate language-server
    repo under the `tlaplus` or `tlaplus-community` orgs.
  * ASP/Clingo — `CaptainUnbrauchbar/ASP-Language-Support` and
    `ffrankreiter/answer-set-programming-language-support` are JS extensions
    that call the `clingo` binary directly; no LSP.
  * Categorical CQL — `CategoricalData/CQL` is a Java Swing IDE only;
    `cqframework/cql-language-server` targets HL7 Clinical Quality Language
    (FHIR), unrelated to Categorical CQL.
  Revisit per-format before the corresponding SPEC phase lands.

- [2026-05-29] Test-suite KISS gatekeeper directive (user, review pass):
  golden bytes + proptests are the durable assets; per-type unit tests should
  be one roundtrip + one optional-field-omission max. Skip these categories
  unless a specific behavior actually motivates them: (1) canonical-stability
  self-equality `assert_eq!(to_canonical_bytes(&x), to_canonical_bytes(&x))`
  — trivially true once serializer is deterministic; (2)
  distinct-types-distinct-hashes loops — assumes SHA-256 non-collision;
  (3) cross-type referential-consistency tests over hardcoded fixtures
  (`claim.rule_ids.contains(&rule.rule_id)`) — fixtures are self-consistent
  with themselves by construction; (4) serde-behavior tests
  (`invalid_variant_rejects_deserialization`) — tests the library, not CKC;
  (5) "empty arrays are valid" smokes — covered by serde derive. Cross-bundle
  reference invariants (catching fixture programmer error before regen) are
  legitimate and should be kept. Roadmap-gate phrasing like "round-trip serde
  tests for every type" means "for each type that has roundtrip", not
  "produce N×3 mechanical assertions per type". Apply this when planning
  and when implementing.
- [2026-05-29] Repo-layout scaffolding policy (this session): empty crates
  and `.gitkeep`-only directories in the SPEC §19 target layout are added
  by the roadmap phase that needs them, not pre-scaffolded. Phase 0 keeps
  only `crates/{ckc-cli,ckc-core,ckc-store,ckc-retrieve,ckc-term}/`,
  `examples/research_kernel/`, `schemas/`, `runs/`. SPEC.md §19 remains the
  target-layout source; current state lags intentionally.
- [2026-05-29] When sed-deleting a multi-line comment block, also delete the
  continuation indent line that follows it (`        //         …`) — it
  becomes an orphaned half-sentence otherwise. Verify with grep before
  committing.

## Mistakes

- [2026-05-27] `replace_all` is case-sensitive. A single replace_all pass can
  silently miss case variants. Always read the result or verify match count
  covers all intended occurrences.
- [2026-05-27] User gave a directive and the agent acknowledged it verbally
  but failed to persist it to memory. Any user directive meant for future
  sessions must be written to memory immediately, in the same turn.
