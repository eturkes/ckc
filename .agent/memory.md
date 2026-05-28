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
  * Lean 4 LSP is `lake serve`; elan provides `lake`/`lean`/`leanc` as
    multiplexer hardlinks. Install via the upstream `elan-init.sh` script with
    `-y --default-toolchain stable`, then symlink `~/.elan/bin/{lake,lean}`
    into `~/.local/bin/`. `lake serve` works at repo root even without a
    `lakefile.lean` — it just won't index project files until one exists.
  * Alloy 6 LSP ships inside the dist jar as the `lsp` subcommand:
    `java -jar org.alloytools.alloy.dist.jar lsp`. Get the jar from
    `github.com/AlloyTools/org.alloytools.alloy/releases/download/v6.2.0/org.alloytools.alloy.dist.jar`
    and wrap it as `~/.local/bin/alloy-lsp`. Requires `openjdk-21-jre-headless`.
  * LemMinX (generic XML LSP) is NOT on Maven Central and the GitHub releases
    page lists no jar assets. Use the Eclipse Nexus direct URL:
    `https://repo.eclipse.org/content/repositories/lemminx-releases/org/eclipse/lemminx/org.eclipse.lemminx/<ver>/org.eclipse.lemminx-<ver>-uber.jar`
    (verified for 0.31.1). Covers XML/XSD/DMN/BPMN/SHACL-XML.
  * SWI-Prolog `lsp_server` pack expects `library(json)`, but the Debian
    `swi-prolog-core-packages` splits the JSON library into
    `/usr/lib/swi-prolog/library/ext/http/http/` and registers it only as
    `library(http/json)`. Wrap `swipl` to prepend that dir to
    `user:file_search_path(library, ...)` before `use_module(library(lsp_server))`.
    Without the patch, `pack install lsp_server -y` succeeds but loading the
    pack fails with `Cannot find source for library(json)`.
- [2026-05-29] LSP coverage matrix for SPEC verification-target formats —
  wired: Lean 4 (lake serve), Alloy 6 (dist-jar `lsp` subcommand), SWI-Prolog
  (jamesnvc/lsp_server pack), LemMinX (DMN/BPMN/SHACL-XML/XSD), Dolmen
  (SMT-LIB/TPTP/DIMACS/Zipperposition), Soufflé Datalog
  (jdaridis/souffle-lsp-plugin), egglog (hatoo/egglog-language-server).
  No standalone LSP exists as of 2026-05: TLA+ (vscode-tlaplus is a TS
  extension shelling out to tla2tools.jar — no LSP server), ASP/Clingo
  (CaptainUnbrauchbar/ASP-Language-Support and ffrankreiter extension both
  just JS extensions calling clingo binary), Categorical CQL
  (categoricaldata.net repo is a Java Swing IDE only; cqframework/cql-language-server
  is HL7 Clinical Quality Language, not Categorical). Revisit per-format
  before the corresponding SPEC phase lands; do not redo the survey from
  scratch — start from these names.
- [2026-05-29] Additional LSP install gotchas (round 2):
  * Dolmen opam package is `dolmen_lsp` (underscore), not `dolmen-lsp`
    (hyphen). Binary it installs is `dolmenls`. Path:
    `~/.opam/<switch>/bin/dolmenls`. Symlink as `~/.local/bin/dolmen-lsp`.
    `dolmenls --version` exits with `error: End_of_file` because dolmenls is
    LSP-only (reads stdin); that exit is success, not failure. Total disk
    footprint: ~1.5 GB for the OCaml 5.2.0 switch + dolmen toolchain.
  * Soufflé apt package targets Ubuntu 20.04 and depends on libffi7; Debian
    13 only ships libffi8. Workaround: install libffi7 3.3-6 from Debian
    snapshot.debian.org, then `apt install souffle`. snapshot URL:
    `http://snapshot.debian.org/archive/debian/20210602T144247Z/pool/main/libf/libffi/libffi7_3.3-6_amd64.deb`.
    The Souffle LSP repo gradle-builds `Souffle_Ide_Plugin-1.0-SNAPSHOT.jar`;
    main-class is `SouffleLanguageServerLauncher`. The LSP parses with an
    in-process ANTLR grammar, so most features work without the `souffle`
    binary; only `souffle-lint`-driven diagnostics require an external
    binary (souffle-lint is a separate, optional install).
  * egglog binary: `cargo install egglog` (egglog 2.0.0, Feb 2026). The LSP
    workspace at hatoo/egglog-language-server is a tree-sitter-backed Rust
    server, built via `cargo build --release -p egglog-language-server`.
    Drop into `~/.local/bin/egglog-lsp`. File extension is `.egg` (not
    `.egglog`).

## Mistakes

- [2026-05-27] `replace_all` is case-sensitive. A single replace_all pass can
  silently miss case variants. Always read the result or verify match count
  covers all intended occurrences.
- [2026-05-27] User gave a directive and the agent acknowledged it verbally
  but failed to persist it to memory. Any user directive meant for future
  sessions must be written to memory immediately, in the same turn.
