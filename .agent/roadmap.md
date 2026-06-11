# CKC roadmap

Build plan for /session-prompt — the session protocol, bookkeeping format, and stamp
semantics live in that command; SPEC.md is the design authority, its §2 the milestone
sequence. One milestone at a time: header `## <name> — plan <hash> — review <hash>` over an
ordered unit checklist; unchecked lines carry the full unit spec; checked items collapse to
one-line stubs `- [x] <id>: <gist>. NN% NNNK/200K <hash>`; closed milestones persist as
bare headers; git history retains all removed text.

## M1 spine — plan 40de97c

- [x] core-ids: ckc-core value types Id/Hash/Rational + serde. 34% 67K/200K 62ee8d3
- [x] core-strings: seven StringPolicy normalizers. 30% 60K/200K 1110fc9
- [x] core-canon-writer: canonical JSON writer core + ObjectEmitter. 43% 86K/200K e6f0b47
- [x] core-canon-collections: arrays/sets/maps over the writer core. 41% 82K/200K 0620275
- [x] core-canon-unions: tagged-union + fieldless-enum emission. 30% 61K/200K 173540d
- [x] core-canon-reader: strict reader, the writer's inverse (solo). 28% 55K/200K df25224
- [x] core-canon-hash: content_hash/hash_bytes + policy descriptor. 36% 72K/200K c680b28
- [x] core-enums-envelope.1: ten §4.4 enums + DiagnosticRecord + TotalOperationResult.
  59% 118K/200K aafcfbc
- [x] core-enums-envelope.2: ArtifactEnvelope + EventRecord + JSONL. 64% 128K/200K 695ae62
- [x] core-grounding: SourceDocument/Graph/Span/Anchor/Region + invariants.
  68% 135K/200K e42da41
- [x] core-ir.1: DocIR + SegmentIR layers, structural-hash pattern. 72% 144K/200K a6842d9
- [x] core-ir.2: ClinicalIR + NormIR layers, §8.6 byte pin. 72% 145K/200K b70ae15
- [x] core-ir.3: FormalIR layer + directions_opposed + pair slots. 61% 122K/200K 40b6476
- [x] core-ir.4: IrBundle assembly + components + hashes (bundle.rs). 85% 170K/200K d32331d
- [x] core-ir.5: ten-invariant bundle validation + rejection suite.
  >=90% compacted/200K 8d0ba4e
- [x] core-plans: RunPlan/RunManifest/ReplayManifest. 53% 106K/200K bfebd7d
- [x] core-registry.1: registry entry types + YAML loading. 56% 113K/200K 49bf921
- [x] core-registry.2: registry validation + stage-chain rule. 48% 97K/200K a51cffb
- [x] fixtures-m1: three fixture HTMLs, ja_core lexicon, gold, registry seeds.
  65% 129K/200K e3f0faa
- [x] cli-runner.1.1: ckc-cli crate, four-command dispatch + CLI invariants.
  68% 137K/200K efef72f
- [x] cli-runner.1.2: ckc registry check end-to-end. 60% 121K/200K 1451a35
- [x] stage-extract.1: extract stage core, DOM walk + residuals. 70% 140K/200K 0c97ee0
- [x] stage-extract.2: table extraction + fixture pins. 62% 124K/200K 14134bc
- [x] stage-segment: rule-based segmentation stage. 69% 138K/200K a65be60
- [x] stage-normalize.1a: lexicon loader. 60% 119K/200K 690347c
- [x] stage-normalize.1b: mention binding. 69% 138K/200K 01312c0
- [x] stage-normalize.1c: behavior-frozen binding-core refactor. 53% 105K/200K e7b7acd
- [x] stage-normalize.1d: recommendation statement builder. 68% 136K/200K eca4462
- [x] stage-normalize.1e: exception clause attachment completing clinical_ir.
  51% 102K/200K 68b71e2
- [x] stage-normalize.2a: rule-id scheme + §8.6/§8.2 re-pin + Normalization payload.
  63% 127K/200K _
- [ ] stage-normalize.2b: NormRule derivation + stage envelope (cli half; consumes
  stage-normalize.2a). New ckc-cli module rules.rs: derive_norm_ir(document_id, &ClinicalIr,
  &SegmentIr, &Lexicon) -> NormIr — per statement one DNF conjunct: population+condition atoms
  interval-lowered (a positive Concept whose lexicon entry carries an interval becomes that
  Interval atom — adult ge "18" / child lt "18" on q.age_years; exception negation never
  lowers), plus one ConceptNegated per exception-clause Concept atom (non-Concept clause atoms
  contribute nothing, unreachable at M1); source_region_ids = regions of recommendation-kind
  source segments in reading order, then clause regions in clause order; exception_refs = clause
  ids; direction/strength/certainty/action flow from the statement; rule_id
  <document_id>.rule.<k>. normalize.rs gains the stage entry normalize(source, segments,
  lexicon, producer) -> ArtifactEnvelope<Normalization> mirroring segment(): schema.normalization,
  artifact id <document_id>.normalization, input_hashes [source, segments], NormalizeError::Canon
  shaped like SegmentError; lib.rs: pub mod rules + doc bullet. Tests in rules.rs: pipeline byte
  pin for guideline_a equal to the amended §8.6 listing + strict-read round trip (THE oracle);
  full value pins for guideline_b + control (predicted: source_region_ids [r.2] both, b conjunct
  {age ge "18", cond.pregnancy, cond.sepsis} contraindicate/strong, control {age lt "18",
  cond.sepsis}; pin from observed output if off); hand-built derivation-semantics case (clause
  refs, region order, certainty flow, Interval clause atom skipped); envelope contract +
  double-run determinism. Reading: SPEC §8.6 (amended), §5 NormRule/ContextExpr, §8.3 normalize
  row, §4.4; patterns: clinical_ir, segment(). Gate: `cargo test -p ckc-cli`.
- [ ] smt-emit.1: ckc-smt crate foundation: workspace member depending on ckc-core; CompiledArtifact
  (target id, logic, query plan, query bodies, named-assertion records mapping assertion ids to IR
  rule ids and source region ids, target metadata, diagnostics) and VerifierResult (per-query §6
  category, sat/unsat/unknown/timeout detail, model or unsat core, solver identity, diagnostics)
  with canonical impls, validation, round-trip tests. Reading: SPEC §3 crate table, §5
  CompiledArtifact/VerifierResult rows, §6 categories and profile fields, §8.6 payload instances.
  Consumes the canon stack. Gate: `cargo test -p ckc-smt`.
- [ ] smt-emit.2: Eligibility scan and contradiction-query planning over FormalIR per fixture group,
  in a plan module: normalized-action sameness keys, §6 direction-group opposition (positive vs
  against or contraindicating), two-query plan per eligible pair (Q1 context_overlap asserting both
  guarded contexts with exceptions as negated conjuncts, Q2 deontic_consistency on polarity literals
  over the shared action) with stable deterministic query ids; tested on the §8.6 pair and the
  disjoint-interval control pair. Reading: SPEC §6 direction groups, eligibility, two-query plan, §5
  action-sameness invariants, §8.6, §8.3 compile row. Consumes core-ir.3 FormalIR, smt-emit.1 types.
  Gate: `cargo test -p ckc-smt plan::`.
- [ ] smt-emit.3: Deterministic SMT-LIB emission in an emit module: per-query files with narrowest
  sufficient logic (QF_LRA contexts, QF_UF deontic), required set-option lines, pipe-quoted
  canonical Id symbols, sorted declarations, named assertions in the forms ctx.<rule_id> and
  a.<rule_id>; assertion map binding every named assertion to IR rule ids and source region ids
  (§8.5 item 4); unsupported_fragment path for out-of-profile constructs; emitted bytes reproduce
  the §8.6 Q1/Q2 files. Reading: SPEC §6 SMT profile, §8.6 smt2 listings, §8.3 compile row, §4.4
  envelope. Consumes smt-emit.2 query plans. Gate: `cargo test -p ckc-smt emit::`.
- [ ] smt-verify: Z3 verifier adapter in a verify module: install and pin the z3 binary, record
  solver identity and version for manifests and results; invoke per query file under per-query
  budget; parse the verdict token and result s-expressions from get-model and get-unsat-core; strip
  pipe quotes and normalize cores to canonical Id sets sorted by canonical_sort_key; map outcomes to
  the §6 categories semantic_no_conflict, semantic_contradiction, unknown, unsupported_fragment,
  target_syntax_failure, solver_execution_failure, keeping raw sat/unsat/unknown/timeout tokens and
  the §7.4 solver_timeout and solver_unknown diagnostics distinct; produce VerifierResults over the
  emitted §8.6 queries (Q1 sat with model, Q2 unsat with the expected core). Reading: SPEC §6
  solvers and verdict parsing, §8.3 verify row, §4.4 verifier_authority. Consumes smt-emit.3 smt2
  files, smt-emit.1 VerifierResult. Gate: `cargo test -p ckc-smt`.
- [ ] cli-runner.2: ckc run orchestration: resolve exp.m1_spine through the registries; execute
  extract, segment, normalize, assemble (thin wrapper emitting envelope-wrapped ir_bundle.json via
  core-ir.4 assembly and core-ir.5 validation), compile, verify per document and group; write the §8.3 run
  layout; strict-canonical-read every consumed artifact at each boundary; aggregate stage outcomes
  by severity into exactly one total operation result; stream events.jsonl and diagnostics.jsonl;
  add the workspace test that runs the experiment into a temp dir, strict-reads every accepted
  artifact by walking the run directory (later-stage artifacts join the sweep as they wire in), and
  asserts the corpus/gold/m1_expected.yaml expected outcomes for both fixture groups through the
  typed gold entries. Closes §8.5 item 3 and stands as the code oracle behind items 5 and 6.
  Reading: SPEC §8.1, §8.3 stage contracts and run layout, §8.4, §8.2 gold shape,
  §4.4, §4.6, §8.5 item 3. Consumes stage-extract/segment/normalize.1-.2, core-ir.4/.5,
  smt-emit.2/.3, smt-verify, core-registry.1 gold entries, cli-runner.1.1 shell. Gate: `cargo test
  --workspace`.
- [ ] cli-runner.3: Trace stage and command in a trace module: derive finding ids
  finding.<group_id>.<ordinal> with ordinals in source-then-hash order (§7.2) as claim-evidence rows
  are born; assemble TraceBundle (derivation DAG with operation-labeled edges from source through
  report, claim-evidence rows) and LineageIndex into trace_bundle.json and lineage_index.json in the
  run layout; ckc trace --run --finding resolves a finding to the full chain source spans, segments,
  statements, rules, named assertions, verdict, finding, in both directions. Closes §8.5 item 7.
  Reading: SPEC §7.1, §7.2 finding-id rule, §5 TraceBundle/LineageIndex rows, §8.3 layout, §4.4.
  Consumes the cli-runner.2 run artifact set. Gate: `cargo test -p ckc-cli trace::`.
- [ ] cli-runner.4.1: Report stage and manifests in a report module: render canonical report.json
  and derived report.md (findings keyed by the trace-derived finding ids with conflict kind, rules,
  regions, quoted Japanese spans, assertion names, core; documented null results; code-keyed
  diagnostics summary; corpus/lexicon hashes; solver identity; replay status; §0 vocabulary
  wording); write manifest.json and replay_manifest.json from core-plans types; wire report into ckc
  run, completing the §8.3 artifact set. Closes §8.5 item 9. Reading: SPEC §7.2, §5 Report and
  RunManifest rows, §4.6 replay manifest fields, §8.3 layout, §4.4; §0 vocabulary. Consumes
  core-plans types, cli-runner.2/.3 outputs. Gate: `cargo test -p ckc-cli report::`.
- [ ] cli-runner.4.2: ckc replay command: re-execute from replay_manifest.json over the same inputs
  and compare canonical content hashes for all accepted artifacts, runtime metadata excluded; emit
  symmetric-difference diagnostics on mismatch and replay_identity_unsupported for missing tools;
  matching hashes yield ok; re-run-equals-prior idempotency test over a fixture run. Closes §8.5
  item 8. Reading: SPEC §4.6 replay semantics, §8.3 layout, §7.4 replay codes. Consumes
  cli-runner.4.1 manifests and the complete run pipeline. Gate: `cargo test --workspace`.
- [ ] acceptance-m1: Dedicated acceptance session for the M1 milestone: execute §8.5 items 1-9 in
  order (fmt/clippy/workspace tests; ckc registry check; ckc run --experiment exp.m1_spine --out
  runs/m1 with outcome ok and strict-read artifact set; assertion-map audit; group.m1_conflict
  semantic_contradiction with cross-document unsat core matching corpus/gold/m1_expected.yaml;
  group.m1_null semantic_no_conflict with documented_null_result from the disjoint-interval Q1
  unsat; ckc trace full chain; ckc replay hash match; report content with quoted spans resolving to
  fixture bytes); mark the milestone header with the evidence run id and create the local tag
  accept/m1. Reading: SPEC §8.5, §8.1-§8.4, §8.6; §1 acceptance and tagging protocol. Consumes the
  complete built pipeline. Gate: All nine §8.5 items pass against one recorded run; roadmap
  milestone header carries the evidence run id; local tag accept/m1 exists.
