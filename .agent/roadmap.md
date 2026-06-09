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

- [x] plan-boilerplate [user-selected] 77% 155K/200K: establish project boilerplate and seed
  the forward build plan. Boilerplate — `.gitignore` (per CLAUDE.md `.serena/`;
  per SPEC §6 `corpus/raw/`, `runs/`; plus toolchain/build caches) and the
  SPEC §6 repository skeleton + tooling init (uv `pyproject.toml`/`uv.lock`;
  Cargo workspace over `crates/{ckc-core,ckc-core-cli,ckc-smt}`; `ckc/`
  adapter/runner tree; `registry/ corpus/ schemas/ examples/ tests/` dirs;
  `Makefile`). Plan — author the M1 build units into this backlog from the §19
  acceptance criteria. Reading: SPEC §5–§6, §19; CLAUDE.md. Gate: tree commits
  clean with `.gitignore` excluding `.serena/`, and forward units authored
  below this line.

Seed batch (§19 criteria 1-3: Rust typed core, schema export, registry).
Dependency-ordered; sized per the spec02 calibration in `.agent/memory.md`
Lessons (canonical JSON is a three-unit deliverable; an "implements algorithm
AND authors a table" unit splits in two). `corecli` and `core-ir` carry
JIT-split notes for their Implement sessions.

- [x] core-ids 48% 96K/200K: `ckc-core` crate skeleton + workspace wiring (edition 2024,
  resolver, root `Cargo.toml` member, `Cargo.lock`); `Id` (`[a-z][a-z0-9_.:-]*`),
  `Hash` (`sha256:`+64hex), exact-reduced `Rational` (gcd-normalized, positive
  den) value types with validation. Files: `crates/ckc-core/{Cargo.toml,
  src/lib.rs,src/id.rs}`, `Cargo.toml`. Reading: SPEC §9, §5. Gate:
  `cargo test -p ckc-core`.
- [x] core-strings 55% 109K/200K: the seven string policies (`raw_source`, `source_nfkc`,
  `semantic_ja`, `semantic_en`, `identifier_ascii`, `diagnostic_text`,
  `view_text`) as deterministic normalizers. Files:
  `crates/ckc-core/src/strings.rs`. Reading: SPEC §10. Gate:
  `cargo test -p ckc-core`.
- [ ] core-canon-writer: canonical JSON writer core — byte-sorted object fields,
  optional-omit, null/unknown rejection, integers as decimal strings, rationals
  as exact-reduced objects, strings under declared policy. Files:
  `crates/ckc-core/src/canon.rs`. Reading: SPEC §10, §9. Gate:
  `cargo test -p ckc-core`.
- [ ] core-canon-collections: canonical collections — semantic-ordered arrays,
  sets sorted by `canonical_sort_key`, maps (identifier_ascii keys -> sorted
  objects; other keys -> sorted key/value arrays). Files:
  `crates/ckc-core/src/canon.rs`. Reading: SPEC §10. Gate:
  `cargo test -p ckc-core`.
- [ ] core-canon-hash: tagged unions `{tag,value}`, strict reading, content
  hashing (`content_hash = sha256(canonical_payload_bytes)`,
  `canonicalization_policy_hash`). Files: `crates/ckc-core/src/{canon.rs,
  hash.rs}`. Reading: SPEC §10. Gate: `cargo test -p ckc-core`.
- [ ] core-enums-envelope: shared enums (`Outcome`+ordering, `Origin`,
  `Authority`, `BindingStatus`, `Direction`, `ClaimTier`, `ReviewClassification`,
  `AttemptClassification`, `PromotionDecision`, `PromotionScope`); artifact
  envelope struct + envelope invariants; `TotalOperationResult`. Files:
  `crates/ckc-core/src/{enums.rs,envelope.rs}`. Reading: SPEC §9, §10. Gate:
  `cargo test -p ckc-core`.
- [ ] core-ir: `IRBundle` with the five layers (`DocIR`/`SegmentIR`/`ClinicalIR`/
  `NormIR`/`FormalIR`), IR-invariant validation (source-region/synthetic-fixture
  grounding, stable reusable IDs), per-layer + bundle structural hashes.
  JIT-split candidate (per-layer). Files: `crates/ckc-core/src/ir.rs`. Reading:
  SPEC §13, §9. Gate: `cargo test -p ckc-core`.
- [ ] core-plans: `ExperimentPlan`, `RunManifest`, `EvaluatorLock` structs +
  run-plan canonicalization (stable canonical run-plan bytes/hash). Files:
  `crates/ckc-core/src/plans.rs`. Reading: SPEC §8, §9, §10. Gate:
  `cargo test -p ckc-core`.
- [ ] core-registry-types: registry & experiment entry Rust types (method,
  component, pipeline, corpus, experiment, evaluator, prompt, policy, index,
  schema, source_processor, gate) with adapter_status / compatibility / gate_refs
  fields. Files: `crates/ckc-core/src/registry.rs`. Reading: SPEC §7, §8. Gate:
  `cargo test -p ckc-core`.
- [ ] smt-emit: `ckc-smt` crate + `CompiledArtifact` + SMT-LIB emission (named
  assertions, narrowest-logic profile, separately named contradiction queries) +
  `assertion_map.json` (assertion_id -> IR rule IDs + source region IDs). Files:
  `crates/ckc-smt/{Cargo.toml,src/lib.rs}`, `Cargo.toml`. Reading: SPEC §14, §13,
  §9. Gate: `cargo test -p ckc-smt`.
- [ ] review rust-kernel: review the typed kernel group (core-ids .. smt-emit;
  crates `ckc-core` + `ckc-smt`) across bugs, SPEC conformance, CLAUDE.md/memory
  conformance, inconsistencies, token-inefficiency, obsolescence.
- [ ] corecli: `ckc-core-cli` crate — `schema export` (JSON Schema from Rust
  types -> committed `schemas/` + `registry/schemas.yaml`), `artifact validate`,
  `artifact canonicalize`, hash, run-plan normalize, SMT emit. JIT-split
  candidate (schema-export vs ops). Files: `crates/ckc-core-cli/{Cargo.toml,
  src/main.rs}`, `schemas/*.json`, `registry/schemas.yaml`, `Cargo.toml`.
  Reading: SPEC §6, §10, §19.2. Gate: `cargo test --workspace` + clean
  `schema export` diff.
- [ ] registry-validate: Rust registry-validation logic (entry-field
  requirements, `m1_required` runnable-adapter rule, ID grammar, gate_refs) +
  `registry check` semantics in `ckc-core-cli`, tested on inline fixtures. Files:
  `crates/ckc-core/src/registry.rs`, `crates/ckc-core-cli/src/main.rs`. Reading:
  SPEC §7, §8, §19.1. Gate: `cargo test --workspace`.
- [ ] registry-seed: author the ten registry YAMLs with SPEC §7-§8/§20 seed
  content — methods (families + adapter_status), candidates
  (`pipe.direct_rule_to_smt`, `pipe.layered_ckcir_to_smt` + required component
  IDs), corpora, experiments (`exp.m1_public_smoke`, `exp.m1_autonomous_smoke`),
  evaluators, prompts, policies, indexes, source_processors
  (`source_processor.minds_html_pdf_baseline`), gates (§20). Files:
  `registry/*.yaml`. Reading: SPEC §7, §8, §20. Gate:
  `cargo run -p ckc-core-cli -- registry check registry/`.
- [ ] ckc-cli: minimal Python `uv` project (deps in `pyproject.toml`, `uv.lock`)
  + `ckc` console entrypoint dispatching `schema export` and `registry check` to
  `ckc-core-cli`, emitting per-command JSONL events + one total outcome (§6 CLI
  invariants). Closes §19.1-2 at the `uv run ckc` level. Files: `pyproject.toml`,
  `uv.lock`, `ckc/{__init__.py,cli.py}`, `ckc/runner/`. Reading: SPEC §6, §5.
  Gate: `uv run ckc schema export --out schemas/` (no diff) + `uv run ckc
  registry check`.
- [ ] review cli-registry: review the CLI/registry/orchestration group (corecli
  .. ckc-cli; schema export, registry validation/seed, Python `ckc` bootstrap)
  across bugs, SPEC conformance, CLAUDE.md/memory conformance, inconsistencies,
  token-inefficiency, obsolescence.
