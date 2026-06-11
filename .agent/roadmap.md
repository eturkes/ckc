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
  71% 142K/200K 01317b9
- [x] cli-runner.3a.2a: assemble_trace + DocTrace/GroupTrace hand-off, synthetic battery.
  76% 153K/200K d6fd71b
- [x] cli-runner.3a.2b: live fixture pins for assemble_trace. 71% 142K/200K f93bfe6
- [x] cli-runner.3a.3: trace stage wired into ckc run. 75% 150K/200K 49b0930
- [x] cli-runner.3b: ckc trace command, chain in both directions. 82% 164K/200K _
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
