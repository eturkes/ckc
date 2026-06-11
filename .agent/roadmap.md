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
- [x] smt-emit.3b: CompiledArtifact assembly completing compile. 75% 151K/200K _
- [ ] smt-verify.a: Z3 adapter foundation in a verify module: install z3 via apt; SolverIdentity
  parsed live from `z3 --version` at adapter construction — code carries no version literal,
  manifests and results carry the truth; per-query subprocess invocation under a per-query
  wall-clock budget with kill-on-expiry; capture raw stdout/stderr and the leading verdict token;
  timeout maps to solver_timeout and spawn failure/nonzero exit to solver_execution_failure (§7.4
  codes, kept distinct from §6 categories). Tests: live z3 on tiny inline smt2 for sat and unsat;
  verdict-token parse incl. unknown on canned text; budget kill via a stub sleeper executable.
  Reading: SPEC §6 solvers, §7.4 solver codes, §8.3 verify row, §5 RunManifest row
  (solver identity). Consumes core-plans SolverIdentity. Gate: `cargo test -p ckc-smt verify::`.
- [ ] smt-verify.b: Verdict parsing and §6 categories completing verify: s-expression reader over
  get-model and get-unsat-core output; pipe-quote stripping; cores normalized to canonical Id sets
  sorted by canonical_sort_key and compared set-based; witness model recorded on sat; category
  mapping — Q2 unsat semantic_contradiction, Q2 sat semantic_no_conflict, Q1 unsat closes the pair
  as the documented-null path of semantic_no_conflict, unknown stays unknown — with raw
  sat/unsat/unknown/timeout tokens preserved distinctly; VerifierResult assembly + validate.
  Integration runs plan + emit + live z3 over the §8.6 pair and control pair: Q1 sat with model,
  Q2 unsat with the expected core, control Q1 unsat. Reading: SPEC §6 verdict parsing +
  categories, §8.6 expectations, §4.4 verifier_authority, §8.3 verify row. Consumes smt-verify.a,
  smt-emit.2/.3a/.3b. Gate: `cargo test -p ckc-smt`.
- [ ] cli-runner.2a: Run scaffolding + document pipeline in a run module: resolve exp.m1_spine
  through the registries (cli-runner.1.2 loaders); create the §8.3 run layout under --out; per
  document run extract → segment → normalize → assemble (thin wrapper: core-ir.4 assemble +
  core-ir.5 validate) emitting envelope-wrapped
  artifacts/<doc-id>/{source_graph,segments,normalization,ir_bundle}.json, each written canonical
  and strict-read back at the boundary; stream events.jsonl + diagnostics.jsonl
  (core-enums-envelope.2 records). Gate test runs the document stages over the three fixtures into
  a temp dir with every artifact strict-read clean. Reading: SPEC §8.1, §8.3 stage rows + layout,
  §4.4, §4.6. Consumes stage fns, core-ir.4/.5, cli-runner.1.1 shell. Gate:
  `cargo test -p ckc-cli run::`.
- [ ] cli-runner.2b: Group stages + total outcome completing ckc run: per fixture group load the
  member ir_bundles, compile (plan + emit + assertion map → groups/<gid>/compiled.json with each
  QueryBody body materialized byte-identical at groups/<gid>/smt/<query_id>.smt2) and verify
  (adapter per query under budget → groups/<gid>/verifier_results.json); aggregate every stage
  outcome by severity into exactly one TotalOperationResult; `ckc run --experiment exp.m1_spine
  --out <dir>` completes ok (trace/report artifacts join in their units). Reading: SPEC §8.3
  compile/verify rows + layout, §4.4 outcome aggregation. Consumes cli-runner.2a,
  smt-emit.2/.3a/.3b, smt-verify.a/.b. Gate: `cargo test -p ckc-cli run::`.
- [ ] cli-runner.2c: Workspace run oracle: workspace test executing exp.m1_spine into a temp dir,
  walking the run directory, strict-reading every accepted artifact (later-stage artifacts join
  the sweep as they wire in), asserting corpus/gold/m1_expected.yaml through typed GoldEntry rows
  for both groups — conflict group semantic_contradiction + deontic_direction_conflict + expected
  core compared as a set; null group semantic_no_conflict + null result. Closes §8.5 item 3 and
  stands as the code oracle behind items 5 and 6. First whole-pipeline execution — reserve the
  window's margin for cross-stage debugging. Reading: SPEC §8.5 item 3, §8.2 gold shape, §8.3
  layout. Consumes cli-runner.2a/.2b, core-registry.1 GoldEntry. Gate: `cargo test --workspace`.
- [ ] cli-runner.3a: Trace types + assembly in a trace module: finding ids
  finding.<group_id>.<ordinal> with ordinals in source-then-hash order (§7.2) as claim-evidence
  rows are born from verifier results; TraceBundle (derivation DAG source → extraction → segment →
  normalization → IR → compile → verify → report with operation-labeled edges; claim-evidence rows
  finding → region ids → rule ids → assertion ids → verdict → report ref) and LineageIndex types
  with canonical bytes — the report node and report refs are static id/path references, keeping
  trace before report in stage order; assemble both from the run artifact set, wire into ckc run,
  write trace_bundle.json + lineage_index.json. Reading: SPEC §7.1, §7.2 finding-id rule, §5
  TraceBundle/LineageIndex rows, §8.3 layout. Consumes cli-runner.2b run artifacts. Gate:
  `cargo test -p ckc-cli trace::`.
- [ ] cli-runner.3b: ckc trace command: --run + --finding resolve through LineageIndex to the full
  chain source spans → segments → statements → rules → named assertions → solver verdict →
  finding, printed in both directions (§8.5 item 7 shape). Closes §8.5 item 7. Reading: SPEC §7.1,
  §8.5 item 7. Consumes cli-runner.3a artifacts, cli-runner.1.1 dispatch. Gate:
  `cargo test -p ckc-cli trace::`.
- [ ] cli-runner.4.1a: Report payload in a report module: canonical Report type + assembly from
  the run artifact set — findings keyed by trace finding ids carrying conflict kind, rule ids,
  region ids, quoted Japanese spans resolved from source graphs by region id, assertion names,
  core; documented null results; code-keyed diagnostics summary; corpus/lexicon hashes; solver
  identity; replay status slot; §0 vocabulary wording — written as report.json in the run layout.
  Reading: SPEC §7.2, §5 Report row, §8.6 finding example, §0 vocabulary. Consumes
  cli-runner.2b/.3a artifacts. Gate: `cargo test -p ckc-cli report::`.
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
