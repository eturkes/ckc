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
  63% 127K/200K 86414f1
- [x] stage-normalize.2b: NormRule derivation + normalize() stage envelope.
  74% 149K/200K dcfe7e4
- [x] smt-emit.1: ckc-smt crate foundation: CompiledArtifact + VerifierResult.
  >=90% compacted/200K 511b002
- [x] smt-emit.2: plan module: eligibility scan + pair/query-id minting. 60% 120K/200K 22787f9
- [x] smt-emit.3a: §8.6 smt2 re-pin + emit-module query texts. 76% 153K/200K 2d190a6
- [x] smt-emit.3b: CompiledArtifact assembly completing compile. 75% 151K/200K b662324
- [x] smt-verify.a: Z3 adapter: live identity probe + budgeted invocation. 53% 106K/200K 92e9c4b
- [x] smt-verify.b: verdict parsing + §6 categories completing verify. 74% 147K/200K 4487787
- [x] cli-runner.2a: run module document half: resolution + per-doc stage chain, strict
  read-back landings + stage events. >=90% compacted/200K c28dab5
- [x] cli-runner.2b: group stages + total outcome completing ckc run: per-group compile/verify
  landings + byte-identical smt bodies, severity-folded total. >=90% compacted/200K 9fe4145
- [x] cli-runner.2c: workspace run oracle — exp.m1_spine sweep + gold assert. 47% 93K/200K 7cae297
- [x] cli-runner.3a.1: trace module types — DAG/claim/lineage shapes + validation.
  71% 142K/200K _
- [ ] cli-runner.3a.2a: Trace assembly in the trace module (still no run.rs contact). Reference
  implementation from the reverted first attempt: .agent/wip-3a.2a.patch — apply or transcribe,
  verify against this line, trim its test imports to the synthetic surface, delete it in the
  closing commit. Pub hand-off structs DocTrace (document_id, fixture_path: corpus-relative
  String, source_hash: raw-byte Hash, source_graph/segments/normalization: Option<(Id, Hash)>,
  bundle: Option<ArtifactEnvelope<IrBundle>> — the whole envelope; claims/lineage read payload
  layers) and GroupTrace (group_id, fixtures, compiled + verifier_results envelope Options);
  infallible assemble_trace(&[DocTrace], &[GroupTrace]) -> (TraceBundle, LineageIndex) — trace
  shapes emit canonically by construction, expect() inside. DAG: the one static report node
  always present (static id, path report.json, hash None); per doc a source node (id =
  document_id, path = fixture_path, hash = source_hash) then present landings in stage order as
  nodes (id/hash from the landing, path = artifacts/<doc-id>/<kind>.json, ir_bundle node from
  the envelope's artifact_id/content_hash), each edged from the nearest present predecessor
  with the target kind's operation(); per group compiled + verifier_results nodes (paths
  groups/<gid>/{compiled,verifier_results}.json), compile edges from each member's present
  ir_bundle node (lookup keyed by document_id), compiled →verify→ verifier_results →report→
  report. Claims iff compiled AND results present, one row per result: ordinal = index in the
  results vector; pair = the query_plan entry whose overlap or deontic slot equals the row's
  query_id (no match → skip); finding_id = finding.<group_id>.<ordinal>; evidence = recorded
  unsat core verbatim, else both pair constraint ids fc.-stripped and prefixed ctx. (overlap
  slot matched) / a. (deontic); rule/region ids = assertion_map union over the pair's two
  constraints; conflict_kind = deontic_direction_conflict iff category semantic_contradiction;
  report_ref = the report node. Lineage per claim x member fixture: member bundle required,
  doc_rules = the claim's rules prefixed <fixture>.rule. (empty → skip row); each rule's
  position k in norm.rules gives its source_region_ids and clinical.statements[k] →
  statement_id + source_segment_ids. Sort every emitted set by canonical_sort_key, then
  validate() both. Tests stay synthetic — no fixtures, no Z3, no IrBundle values: empty inputs
  (lone report node); a bundle-less full chain; a gapped doc (segments absent → one
  normalize-operation edge bridging source_graph → normalization); a bare group (no nodes); a
  results-only group (node + report edge, no verify edge, no claims); hand-built
  CompiledArtifact + VerifierResults pinning ordinals, evidence fallback naming, assertion_map
  unions, conflict_kind, and the lineage skip paths — live fixture values land in
  cli-runner.3a.2b. Reading: trace.rs, ckc-smt artifact.rs + result.rs, SPEC §7.1, §8.3.
  Consumes cli-runner.3a.1. Gate: `cargo test -p ckc-cli trace::`.
- [ ] cli-runner.3a.2b: Live fixture pins for assemble_trace (tests only; production trace.rs
  and run.rs untouched): trace.rs test helpers mirroring run.rs's two pipelines through the pub
  stage surface — fixture/lexicon/producer helpers per the normalize.rs test pattern, a generic
  envelope wrap (schema id schema.<kind>, content_hash(payload), canonicalization_policy_hash(),
  empty effects/trace_refs/diagnostics/runtime_metadata); live_doc = extract
  (synthetic_fixture_html family, Provenance::Synthetic, DataClass::None) → segment → normalize
  → DocIr::from_graph + canonical union of the segment/normalization envelope diagnostics →
  ckc_core::assemble → bundle.validate against the source graph → DocTrace with every landing
  Some, fixture_path corpus/fixtures/<file>, source_hash = hash_bytes(raw); live_group = compile
  over member (formal, norm) pairs → verify under a live Z3Adapter with a generous budget →
  GroupTrace. One test: docs m1_guideline_a/b + m1_control, group.m1_conflict = [a, b],
  group.m1_null = [a, control] → assemble_trace, both validate Ok; assert the node/edge census
  with §8.3 paths and chain/compile/verify/report edge spot checks; claims:
  finding.group.m1_conflict.0 = the overlap row (sat, no conflict_kind, evidence = both ctx.*
  assertions), finding.group.m1_conflict.1 = the deontic row (semantic_contradiction, unsat,
  deontic_direction_conflict, core verbatim `[a.fixture.m1_guideline_a.rule.0,
  a.fixture.m1_guideline_b.rule.0]`), finding.group.m1_null.0 = the documented-null overlap row
  (unsat, evidence [ctx.fixture.m1_control.rule.0, ctx.fixture.m1_guideline_a.rule.0]); lineage:
  docA rows carry the §8.6 regions [r.2, r.3] and the normalize.rs-pinned statement + segments
  [seg.2, seg.3]; docB/control row values pinned from observed gate output, never hand-computed.
  Pattern: the verdict.rs live tests. Reading: trace.rs, run.rs (mirror only), ckc-smt
  verdict.rs live tests, normalize.rs test helpers, SPEC §8.6. Consumes cli-runner.3a.2a. Gate:
  `cargo test -p ckc-cli trace::`.
- [ ] cli-runner.3a.3: Trace stage wired into ckc run: STAGE_KINDS gains trace as the seventh
  resolved component (stage.m1.trace in registry/candidates.yaml); document_pipeline returns its
  DocTrace (corpus path, hash_bytes source hash, landed ids+hashes, bundle envelope),
  group_pipeline its GroupTrace; after the group loop assemble_trace → validate both → two
  envelopes (kinds trace_bundle + lineage_index, producer = the trace component, input_hashes =
  the node content-hash set) → land trace_bundle.json + lineage_index.json at the run root (§8.3)
  → one trace stage event. Update run.rs document_stages test (18 events, command event last) and
  the run_oracle.rs §8.3 sweep (+2 files, strict-read, assert the §8.6 finding row and the
  hashless report node). Reading: run.rs, run_oracle.rs, SPEC §8.3 + §4.6 events. Consumes
  cli-runner.3a.2b. Gate: `cargo test --workspace`.
- [ ] cli-runner.3b: ckc trace command: --run + --finding resolve through LineageIndex to the full
  chain source spans → segments → statements → rules → named assertions → solver verdict →
  finding, printed in both directions (§8.5 item 7 shape). Closes §8.5 item 7. Reading: SPEC §7.1,
  §8.5 item 7. Consumes cli-runner.3a.3 artifacts, cli-runner.1.1 dispatch. Gate:
  `cargo test -p ckc-cli trace::`.
- [ ] cli-runner.4.1a: Report payload in a report module: canonical Report type + assembly from
  the run artifact set — findings keyed by trace finding ids carrying conflict kind, rule ids,
  region ids, quoted Japanese spans resolved from source graphs by region id, assertion names,
  core; documented null results; code-keyed diagnostics summary; corpus/lexicon hashes; solver
  identity; replay status slot; §0 vocabulary wording — written as report.json in the run layout.
  Reading: SPEC §7.2, §5 Report row, §8.6 finding example, §0 vocabulary. Consumes
  cli-runner.2b/.3a.3 artifacts. Gate: `cargo test -p ckc-cli report::`.
- [ ] cli-runner.4.1b: report.md + manifests completing the §8.3 artifact set: deterministic
  markdown rendering of report.json; manifest.json (RunManifest) + replay_manifest.json
  (ReplayManifest) from core-plans with real hash/identity values; report stage wired into ckc
  run. Closes §8.5 item 9 surface. Reading: SPEC §7.2, §4.6 replay manifest fields, §8.3 layout,
  §5 RunManifest row. Consumes cli-runner.4.1a, core-plans types. Gate:
  `cargo test -p ckc-cli report::`.
- [ ] cli-runner.4.2: ckc replay command: re-execute from replay_manifest.json over the same inputs
  and compare canonical content hashes for all accepted artifacts, runtime metadata excluded; emit
  symmetric-difference diagnostics on mismatch and replay_identity_unsupported for missing tools;
  matching hashes yield ok; re-run-equals-prior idempotency test over a fixture run. Closes §8.5
  item 8. Reading: SPEC §4.6 replay semantics, §8.3 layout, §7.4 replay codes. Consumes
  cli-runner.4.1b manifests and the complete run pipeline. Gate: `cargo test --workspace`.
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
