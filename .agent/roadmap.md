# CKC roadmap

Build plan consumed by the /session-prompt command; SPEC.md is the design
authority, its §2 the milestone sequence. One milestone at a time: a header
`## <milestone> — plan <hash> — review <hash>` (plan opens, review closes, the
pair bounds the milestone's commits; acceptance adds the evidence run id per
SPEC §1) over an ordered unit checklist. Completed items gain `[x]` plus
trailing `NN% NNNK/200K <hash>` (context usage, then the completing commit);
plan/review stamps carry no usage. A commit cannot contain its own hash, so a
session writes `_` in its own hash slot and the next unit of roadmap work
fills it (at most one `_` pending; resolve via commit scopes `<unit-id>:`,
`plan-v<n>:`, `review-v<n>:`). Closed milestones persist as bare headers; the
next plan session removes their checklists (git history retains them).

## V1 spine — plan e6523e9

- [x] core-ids: Fill the existing ckc-core stub crate: workspace dependency pins (serde, num-bigint,
  num-rational), lib.rs wiring with unsafe forbidden, and value types Id (`[a-z][a-z0-9_.:-]*`), Hash
  (sha256: plus 64 lowercase hex), exact-reduced Rational (positive denominator, decimal-string
  num/den repr) with ValidationError, serde via try_from/into String, Display/FromStr;
  acceptance/rejection tables and serde round-trip tests. Reading: SPEC §4.1, §3 crate table. Gate:
  `cargo test -p ckc-core`. 34% 67K/200K 10bf054
- [x] core-strings: StringPolicy enum (Copy, snake_case serde) with the seven policies raw_source,
  source_nfkc, semantic_ja, semantic_en, identifier_ascii, diagnostic_text, view_text as
  deterministic normalizers: pipeline NFKC, whitespace fold to single U+0020 trimmed, CJK
  punctuation fold (U+3001 to comma, U+3002 to period), ASCII lowercase for semantic_en; only
  identifier_ascii ([a-z0-9_:./-]+) fallible; per-policy exact-output and idempotence tests. Reading: SPEC §4.2. Consumes core-ids ValidationError and crate skeleton. Gate: `cargo
  test -p ckc-core`. 30% 60K/200K 3813a4d
- [x] core-canon-writer: Canonical JSON writer core in a canon module: Canonical trait and
  canonical_payload_bytes as the single byte authority; emit_string with the fixed minimal escape
  set, emit_int as quoted decimal (never bare number tokens), emit_string_policy; ObjectEmitter
  sorting members by UTF-8 name bytes, omitting absent optionals, rejecting duplicates; Rational as
  reduced {den,num} object; CanonError; exact-byte assertion tests. Reading: SPEC §4.3.
  Consumes core-ids value types, core-strings policies. Gate: `cargo test -p ckc-core canon::`.
  43% 86K/200K 1adc76b
- [x] core-canon-collections: Canonical collections over the writer core: emit_array in semantic
  order; canonical_sort_key from element canonical bytes; emit_set sorted byte-lexicographically
  with byte-identical duplicates collapsed; MapKey trait whose const IDENTIFIER_ASCII picks the map
  form for the whole map; emit_map as sorted object for identifier_ascii keys and as key/value pair
  array sorted by key bytes otherwise, duplicates rejected in both forms. Reading: SPEC §4.3. Consumes core-canon-writer core. Gate: `cargo test -p ckc-core canon::`.
  41% 82K/200K 0e13c14
- [x] core-canon-unions: Tagged-union and fieldless-enum emission: emit_union producing exactly
  {tag,value} with identifier_ascii tags via ObjectEmitter, tag byte-sorted before value; fieldless
  enums emitted as identifier_ascii strings. Reading:
  SPEC §4.3. Consumes
  core-canon-writer core. Gate: `cargo test -p ckc-core canon::`. 30% 61K/200K 97a3a46
- [x] core-canon-reader: Strict canonical reader as the writer's inverse, scheduled solo: CanonRead
  trait, Reader cursor, read_canonical requiring full consumption;
  read_string/read_int/read_string_policy admitting only writer output (minimal escapes, canonical
  decimals, already-normalized policy strings); ObjectReader demanding fields in ascending byte
  order; read_array/read_set/read_map/read_union enforcing strictly ascending sort keys and exact
  {tag,value} shape; CanonReadError taxonomy rejecting unknown/duplicate/misordered fields, null,
  bare numerics, non-reduced rationals; write-read round-trip tests. Reading: SPEC §4.3. Consumes core-canon-writer, core-canon-collections, core-canon-unions. Gate: `cargo test -p
  ckc-core canon::`. 28% 55K/200K 0353438
- [x] core-canon-hash: Hashing in a hash module: sha2 dep; content_hash as sha256 over
  canonical_payload_bytes wrapped as Hash; hash_bytes raw-byte primitive for _hash fields declared
  over raw bytes; CanonicalizationPolicy descriptor type with pinned canonical bytes and
  canonicalization_policy_hash derived through content_hash; NIST vector, determinism,
  value-sensitivity, and pinned-descriptor tests.
  Reading: SPEC §4.3, §4.4 canonicalization_policy_hash row. Consumes core-ids Hash and the canon stack. Gate: `cargo test -p
  ckc-core`. 36% 72K/200K c568017
- [x] core-enums-envelope.1: V1 enums plus result and diagnostic records in an enums module: all ten
  §4.4 enums — Outcome with the §4.4 severity order, Origin, Authority, BindingStatus, Direction,
  ClaimTier, ReviewClassification, AttemptClassification, PromotionDecision, PromotionScope — as
  fieldless identifier_ascii canonical strings; DiagnosticRecord per §7.4 (stable code from the
  V1-V2 set, structured payload, region/artifact refs, exactly one Outcome); TotalOperationResult
  with operation_id, outcome, and the five hash-list buckets; canonical round-trips and
  severity-ordering tests. Reading: SPEC §4.4 enum and outcome tables, §7.4. Consumes the canon
  stack and core-canon-hash. Gate: `cargo test -p ckc-core enums::`. 59% 118K/200K a27a723
- [x] core-enums-envelope.2: Artifact envelope and event records in an envelope module:
  ArtifactEnvelope with the §4.4 field table (schema_version ckc.1, schema_id, artifact_id/kind,
  producer triple, input_hashes, content_hash, canonicalization_policy_hash, origin, authority,
  accepted_effects, trace_refs, diagnostics, runtime_metadata excluded from content hash, payload)
  with authority and accepted_effects invariants; EventRecord with the §4.6 events.jsonl fields;
  JSONL serialization for both streams; envelope-invariant and round-trip tests. Reading: SPEC §4.4
  envelope table, §4.6. Consumes core-enums-envelope.1 enums and DiagnosticRecord, canon stack,
  core-canon-hash. Gate: `cargo test -p ckc-core`. 64% 128K/200K 0876269
- [x] core-grounding: Source grounding types in a grounding module: SourceDocument (source family,
  provenance synthetic/public, raw/content hashes, data_class default none), SourceGraph with the
  §4.5 node kinds (document through CQ and recommendation), SourceSpan (node, offsets,
  raw/nfkc/search text, reading order, text hash), SourceAnchor, SourceRegion as closed support set;
  validate grounding invariants (every textual unit spanned or typed residual, region refs resolve,
  identical bytes plus config give identical graph bytes); canonical round-trips. Reading: SPEC
  §4.5. Consumes core-strings policies, canon stack, core-canon-hash. Gate: `cargo test -p ckc-core
  grounding::`. 68% 135K/200K _
- [ ] core-ir.1: DocIR and SegmentIR layers in an ir module: layout-preserving text/table view over
  SourceGraph refs with extraction diagnostics; ClinicalSegment with the seven §5 kinds (CQ,
  recommendation, evidence, exception, definition, table-row, metadata) and region refs; canonical
  impls plus the per-layer structural hash over locally indexed references (rename-stable),
  establishing the pattern core-ir.2/.3 reuse. Reading: SPEC §5 layer table and ClinicalSegment row,
  §4.3 structural-hash tail. Consumes core-grounding refs, core-canon-hash. Gate: `cargo test -p
  ckc-core ir::`.
- [ ] core-ir.2: ClinicalIR and NormIR layers: TerminologyBinding (system ckc.lex, code,
  BindingStatus, alternatives, region refs), ClinicalStatement (population, condition, action,
  modality, strength strong/weak, certainty, exceptions, source refs), Action with normalized target
  key, ContextExpr as finite DNF over concept, negated-concept, and quantity-interval atoms,
  NormRule (rule_id, context, direction, action, strength, source_region_ids, optional certainty and
  exception_refs); per-layer hashes; canonical bytes pinned against the §8.6 NormRule JSON. Reading:
  SPEC §5, §8.6 NormRule example. Consumes core-ir.1 layer/hash pattern. Gate: `cargo test -p
  ckc-core ir::`.
- [ ] core-ir.3: FormalIR layer and IRBundle: target-independent constraints, normalized
  actions/contexts, contradiction-query plan slots shaped per §6; IRBundle assembling the five
  layers with reusable component records (use sites), assumptions, diagnostics, per-layer plus
  whole-bundle structural hashes; §5 IR-invariant validation (grounding refs, references, policy
  completeness) before compilation. Reading: SPEC §5, §6 two-query plan shape. Consumes core-ir.1/.2
  layers, core-grounding. Gate: `cargo test -p ckc-core`.
- [ ] core-plans: Plan and manifest types: RunPlan (experiment id, fixture groups, pipeline, seed,
  budget) with canonical bytes and plan hash; RunManifest (plan hash, git commit,
  toolchain/lockfile/corpus/lexicon hashes, environment profile, solver identity, output hashes);
  ReplayManifest with the §4.6 field list; canonical round-trips. Reading: SPEC §5 RunPlan and
  RunManifest rows, §4.6 replay manifest fields. Consumes canon stack, core-canon-hash. Gate: `cargo
  test -p ckc-core`.
- [ ] core-registry.1: Registry entry types in a registry module with a pinned serde-compatible YAML
  dependency: corpora entries (origin, authority, provenance per §8.2), candidates entries (pipeline
  and stage components: ids, kinds, determinism, input/output artifact kinds), experiments entries
  (fixture groups, pipeline, seed, budget, expected-outcome ref), and gold expected-outcome entries
  (group_id, expected_outcome, optional expected_conflict_kind, expected_core compared as a set,
  expected_null_result); inline-YAML loading and round-trip tests. Reading: SPEC §8.4, §8.2 corpora
  fields and gold shape. Consumes core-enums-envelope.1 enums, canon stack. Gate: `cargo test -p
  ckc-core registry::`.
- [ ] core-registry.2: Registry validation in the registry module: per-entry required fields, Id
  grammar, cross-file resolution (experiment to pipeline, corpora, and gold expected-outcome ref;
  every referenced entry resolves and is well-formed), and the §8.4 stage-chain rule that every
  stage's declared input artifact kinds are produced by predecessors; typed validation errors;
  inline-fixture tests including chain violations and dangling refs. Reading: SPEC §8.4; §3
  registry-check invariant. Consumes core-registry.1 types. Gate: `cargo test -p ckc-core`.
- [ ] fixtures-v1: Author the V1 data layer: three §8.2 fixture HTML documents under
  corpus/fixtures/ (guideline_a with CQ heading, recommendation and exception sentences, definitions
  table, evidence list; guideline_b contraindication; control with disjoint child age interval),
  corpus/lexicon/ja_core.yaml (concept entries with adult/child age-interval semantics, action
  verbs, modality phrases to direction/strength, certainty phrases, yielding the §8.6 concept and
  rule ids), corpus/gold/v1_expected.yaml in the typed gold shape, and
  registry/{corpora,candidates,experiments}.yaml seeds (pipe.layered_ckcir_to_smt stage chain;
  exp.v1_spine with group.v1_conflict and group.v1_null, seed, budget, expected-outcome ref); add
  the fixtures_v1 integration test loading every authored file through core-registry types and
  validation and resolving fixture and gold paths. Reading: SPEC §8.2, §5 lexicon paragraph, §8.6
  ids, §8.4. Consumes core-registry.1 types and core-registry.2 validation. Gate: `cargo test -p
  ckc-core --test fixtures_v1`.
- [ ] cli-runner.1.1: ckc-cli crate foundation: workspace member building the ckc binary; dispatch
  for the §3 four-command surface (registry check; run --experiment --out; replay; trace --run
  --finding argument shapes; commands pending implementation return typed unsupported total
  results); CLI invariants wired once: validate inputs, emit §4.6 JSONL events, write only under the
  output directory, end with exactly one §4.4 total operation result; dispatch and shell tests.
  Reading: SPEC §3 crate table, CLI surface and invariants, §4.4 total result, §4.6 events. Consumes
  core-enums-envelope.1 TotalOperationResult and core-enums-envelope.2 EventRecord. Gate: `cargo
  test -p ckc-cli`.
- [ ] cli-runner.1.2: Implement ckc registry check end-to-end: load the three seeded registry files
  through core-registry.1, run core-registry.2 validation including experiment resolution and the
  stage chain, map findings to the severity-aggregated §4.4 outcome inside the cli-runner.1.1 shell.
  Closes §8.5 item 2. Reading: SPEC §3 CLI surface, §8.4, §4.4 outcome severity. Consumes
  cli-runner.1.1 dispatch, core-registry.2, fixtures-v1 registry files. Gate: `cargo run -p ckc-cli
  -- registry check`.
- [ ] stage-extract: Extract stage in an extract module of ckc-cli: choose and pin a real HTML
  parser dependency; parse fixture HTML into SourceGraph (nodes, spans, anchors, regions; tables
  preserving row/column/cell/header relations); emit typed extraction_uncertain and
  table_structure_uncertain residuals on uncertainty; envelope-wrapped source_graph.json payload;
  identical input bytes and config yield identical canonical bytes. Reading: SPEC §8.3 extract row,
  §8.2, §4.5, §4.4, §7.4. Consumes core-grounding types, core-enums-envelope.2 envelope, fixtures-v1
  HTML, cli-runner.1.1 crate. Gate: `cargo test -p ckc-cli extract::`.
- [ ] stage-segment: Segment stage in a segment module: rule-based segmentation keyed on fixture
  structure (CQ headings, recommendation and exception sentence markers, definition table rows,
  evidence lists) producing ClinicalSegments with region refs; envelope-wrapped segments.json
  payload; segmentation_boundary_error diagnostics on misses. Reading: SPEC §8.3 segment row, §8.2
  markers, §5 ClinicalSegment row, §4.4. Consumes stage-extract SourceGraph, core-ir.1 types. Gate:
  `cargo test -p ckc-cli segment::`.
- [ ] stage-normalize.1: Normalize stage first half, in a normalize module: load
  corpus/lexicon/ja_core.yaml content-hash versioned; bind mentions to TerminologyBindings with
  BindingStatus mapping (ambiguous emits terminology_ambiguous ambiguity, unmapped emits
  terminology_unmapped residual when one concept is required); normalize ClinicalStatements with
  direction/strength from modality phrases and certainty phrases when present. Reading: SPEC §8.3
  normalize row, §5 lexicon and binding contract, §8.2 mention text, §4.4. Consumes fixtures-v1
  lexicon, stage-segment output, core-ir.2 types. Gate: `cargo test -p ckc-cli normalize::`.
- [ ] stage-normalize.2: Normalize stage second half: derive NormRules with guarded DNF contexts;
  exceptions compile to negated conjuncts whose regions join source_region_ids; lexicon interval
  semantics yield quantity-interval atoms (adult/child age bounds); canonical bytes reproduce §8.6
  rule.a.cq1.r1; envelope-wrapped normalization.json payload completing the stage contract. Reading:
  SPEC §8.3 normalize row, §5 NormRule/ContextExpr, §8.6 worked rule, §4.4. Consumes
  stage-normalize.1 bindings and statements. Gate: `cargo test -p ckc-cli`.
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
- [ ] cli-runner.2: ckc run orchestration: resolve exp.v1_spine through the registries; execute
  extract, segment, normalize, assemble (thin wrapper emitting envelope-wrapped ir_bundle.json via
  core-ir.3 assembly and validation), compile, verify per document and group; write the §8.3 run
  layout; strict-canonical-read every consumed artifact at each boundary; aggregate stage outcomes
  by severity into exactly one total operation result; stream events.jsonl and diagnostics.jsonl;
  add the workspace test that runs the experiment into a temp dir, strict-reads every accepted
  artifact by walking the run directory (later-stage artifacts join the sweep as they wire in), and
  asserts the corpus/gold/v1_expected.yaml expected outcomes for both fixture groups through the
  typed gold entries. Closes §8.5 item 3 and stands as the code oracle behind items 5 and 6. JIT
  seam if hot: document phase (extract..assemble) before group phase (compile/verify, aggregation,
  workspace test). Reading: SPEC §8.1, §8.3 stage contracts and run layout, §8.4, §8.2 gold shape,
  §4.4, §4.6, §8.5 item 3. Consumes stage-extract/segment/normalize.1-.2, core-ir.3, smt-emit.2/.3,
  smt-verify, core-registry.1 gold entries, cli-runner.1.1 shell. Gate: `cargo test --workspace`.
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
- [ ] acceptance-v1: Dedicated acceptance session for the V1 milestone: execute §8.5 items 1-9 in
  order (fmt/clippy/workspace tests; ckc registry check; ckc run --experiment exp.v1_spine --out
  runs/v1 with outcome ok and strict-read artifact set; assertion-map audit; group.v1_conflict
  semantic_contradiction with cross-document unsat core matching corpus/gold/v1_expected.yaml;
  group.v1_null semantic_no_conflict with documented_null_result from the disjoint-interval Q1
  unsat; ckc trace full chain; ckc replay hash match; report content with quoted spans resolving to
  fixture bytes); mark the milestone header with the evidence run id and create the local tag
  accept/v1. Reading: SPEC §8.5, §8.1-§8.4, §8.6; §1 acceptance and tagging protocol. Consumes the
  complete built pipeline. Gate: All nine §8.5 items pass against one recorded run; roadmap
  milestone header carries the evidence run id; local tag accept/v1 exists.
