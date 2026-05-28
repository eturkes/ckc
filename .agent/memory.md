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
  project-scope `.claude/settings.json` (path stored relative,
  `.claude/marketplace`). Adding/enabling plugins mid-session does NOT take
  effect — the Claude LSP host caches plugin discovery at session start, so
  any new `*-lsp@ckc-lsps` plugin requires a Claude Code restart before
  `LSP` calls find it. Use `claude plugin list` to confirm enablement state
  cheaply without restarting. `claude plugin marketplace add … --scope project`
  + `claude plugin install <name> --scope project` is the supported automation
  path (avoids hand-editing settings.json beyond fixing absolute→relative
  marketplace path).
- [2026-05-29] LSP server install gotchas hit on this project:
  * `taplo` LSP is gated behind `--features lsp`; default `cargo install
    taplo-cli` ships only the CLI. Use `cargo install --locked taplo-cli
    --features lsp`.
  * `cargo install marksman` resolves a placeholder crates.io package
    (`marksman v0.0.1`, no targets). Install via release binary from
    `github.com/artempyanykh/marksman/releases/latest` into `~/.local/bin/`
    instead.
  * `vscode-langservers-extracted` (single pnpm package) provides four
    binaries: `vscode-{html,css,json,eslint}-language-server`. Reuse across
    JSON/HTML/CSS plugins instead of installing each LSP separately.
  * Lean 4 LSP is `lake serve`; install elan + a toolchain only after a
    `lakefile.lean` exists in the repo (deferred until Phase 4).

## Mistakes

- [2026-05-27] `replace_all` is case-sensitive. A single replace_all pass can
  silently miss case variants. Always read the result or verify match count
  covers all intended occurrences.
- [2026-05-27] User gave a directive and the agent acknowledged it verbally
  but failed to persist it to memory. Any user directive meant for future
  sessions must be written to memory immediately, in the same turn.
