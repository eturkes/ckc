# CKC — Clinical Knowledge Compiler

Design authority for this repository. Sole implementers and readers: Claude
sessions operating under AGENTS.md/CLAUDE.md, `.claude/commands/session-prompt.md`, and `.agent/`.
The document is optimized for machine reading: stable `§` anchors, tables over prose, one fact in
one place, sections sized for selective loading.

## §0 Mission, thesis, posture

CKC is a clinical knowledge compiler: a headless research harness that translates clinical text
in any language (public Japanese guideline corpora through M7) into compact, reusable,
source-grounded IR components, surfaced as a clinician-auditable controlled natural language
(ClinicalCNL, §10 — prose a clinician reads and audits; parse and render are mutual inverses
over the CNL AST, a deterministic bridge maps it to and from the IR under §10's laws, so a
reviewed, accepted CNL document is the locked knowledge base); compiles them
deterministically to machine-evaluable targets (SMT-LIB first; a Prolog-family
execution/explanation lane joins per §11, proof assistants such as Lean 4 per §13); and surfaces
contradictions and documented no-conflict results with end-to-end machine-checkable evidence.
The probabilistic step is confined to one boundary — source text into a route's constrained
emission surface; every layer below an accepted artifact is a deterministic compiler.

Thesis under test, as three initial experiments, ordered by dependency — each consumes its
predecessors' machinery. §2 schedules them, §7.3 defines the metrics. The layered component IR
is the instrument, not a hypothesis: layering is a route variable inside experiment 1, reuse
generalization is what experiment 2's coverage measures, and the layered-versus-direct payoff
lands as M4's architecture ablation (§11):

1. Multi-hop translation (models in the loop): does translation through multiple small model
   calls between specific intermediate representations beat direct one-leap emission into a
   machine-evaluable formal target, and which route configuration is best? Reliability is
   hypothesized to depend on the IR configuration the route targets: staged,
   grammar-constrained, short-hop routes — layered and hop-chain forms included — tame model
   non-determinism on the §7.3 route-quality and conflict-quality families. Tested at M2
   (minimal pair); ClinicalCNL (§10) lands as the flagship invented form at M3; the
   configuration ranking widens over existing and invented forms at M4; experiment 2's protocol
   extends the configuration search at M5.
2. Deterministic mapping by optimization (models at development time only): an optimization
   protocol designs and maintains a deterministic mapping — a compact accepted expert system
   with minimized surface area, authored by AI agents where hand-encoding once made expert
   systems prohibitive — that covers fresh documents with zero runtime model calls. The
   objective: maximize coverage and reuse, minimize mapping-set size. Coverage tested at M4;
   the bounded autonomous protocol — declared surfaces, immutable evaluator, every attempt
   ledgered and replayable, from lexicon repair up to search over the IR-combination space —
   lands at M5.
3. Revision surfacing (the compiler applied): once a corpus is compiled into the target
   mathematical representation, does the result highlight guidelines and companion sources in
   need of revision? Seeded at M1 (one synthetic contradiction, one documented no-conflict result);
   answered on real public corpora at M6–M7, where findings become source-grounded,
   human-reviewed revision candidates.

Documented no-conflict results are first-class outcomes for all three experiments.

Stage arc: the plan realizes CKC in three stages, each gated by its predecessor. Stage I —
research instrument (M1–M5): prove experiments 1–2 on synthetic corpora; "machine-evaluable"
means compiled to solver-checked formal targets; locked Stage I measurements anchor a methods
manuscript. Stage II — guideline auditor (M6–M7): experiment 3 answered at full strength —
static analysis of real public corpora surfacing source-grounded revision candidates for
guideline authors, human-reviewed and rendered bilingual; Stage I is the auditor's validity
argument (measured translation reliability, deterministic coverage with zero runtime model
calls), and the auditor's human-reviewed findings anchor a second manuscript. Stage III — CDS
backend (§13.4, requirements-only): the compiler's runtime target — accepted knowledge evaluated
over patient contexts; machine evaluation extends to runtime evaluation; every capability sits
behind §15 gates.

ideal demonstration (M6 era, experiment 3's flagship): cross-source conflict surfacing over
real public corpora — e.g. a guideline recommendation versus a PMDA drug labeling
contraindication — traced from Japanese source text spans through IR and named SMT assertions to
solver cores, replayable from content hashes alone. Behind it sits an asymptotic ideal: ever
more minimal accepted mapping and axiom sets representing ever more clinical knowledge — a
single global axiom in the unreachable limit — which orients experiment 2's compactness
objective and optimization protocol (`G-MDL`) while staying outside every report's claims.

Research position: every output is research evidence. Accepted
semantics come from acceptance (schema validity, source linkage, canonical bytes, applicable
compiler/verifier checks, trace, replay, evidence criteria) — independent of proposer identity.
AI, retrieval, agents, and humans all propose; acceptance decides. Reports describe results with this
vocabulary: `research harness`, `candidate`, `review candidate`, `formalization-QA`,
`text-quality analysis`, `source-grounded`, `schema-valid`, `verifier-checked`, `replayable`,
`requires human review`, `locked measurement`, `synthetic test source measurement`,
`raw benchmark output`, `documented no-conflict result`. Clinical, patient-care, CDS-runtime, SaMD,
deployment, and regulatory claims sit behind the gates in §15 and enter reports only after their
gates pass.

Claim tiers:

| Tier | Meaning |
| --- | --- |
| `s0_replayable` | Byte-for-byte reproducibility; schema/trace checks pass over locked inputs. |
| `s1_accepted` | s0 plus deterministic validation/acceptance. |
| `s2_research_evidence` | s1 plus valid benchmark/gate evidence for a stated research claim. |
| `s3_clinical_regulatory` | s2 plus clinical/regulatory/deployment assurance evidence. |

## §1 Operating requirements

Intent: every session behaves the same way, learns from prior sessions, and leaves the repository
in a state the next session can trust.

Sources of truth, in order: user instructions > AGENTS.md/CLAUDE.md > this spec > `.agent/roadmap.md`
(build plan) > `.agent/memory.md` (lessons). Sessions load §1–§2 plus the reading slice their
roadmap unit names; wider loading is reserved for spec-maintenance sessions.

Unit discipline:

- One roadmap unit = one conceptual deliverable + one gate command, finishable and committable in
  a single 200K context window with margin. Calibrate from neighbouring units' `NN%` annotations
  and the sizing lessons in memory; pre-split units that stack a crate foundation, a
  writer-inverse, a recursive type family, or an algorithm plus a second authored artifact.
- Build exactly the unit's deliverable; choose the simplest implementation that passes the gate.
  Record genuine future needs as roadmap candidates for the unit that will consume them.
- Every Rust unit runs `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  its gate before staging.
- A projected unit overrun is a stop-and-report: recovery (restore to the last commit, re-scope
  the roadmap) is always user-initiated.

Working style:

- When you have enough information to act, act; bring operation-changing decisions, destructive
  actions, and genuine scope changes to the user, and proceed on everything else.
- Audit every progress claim against a tool result from the current session; report failures with
  their output, and state verified results plainly.

Session shapes — context sizing, plan-session workflows, subagent policy — live in the session
command; commit, compaction, and memory
discipline live in AGENTS.md/CLAUDE.md and `.agent/memory.md`.

Spec evolution: the spec grows in place. When a milestone closes (its closing review lands), the
plan session that opens the next milestone is an elaboration session while that milestone's
requirements section is still compact: expand it into full normative text (workflow-driven; mine
`docs/` through subagents), present the diff to the user for review, then seed
`.agent/roadmap.md` with the milestone's header and units. Elaboration sessions may also amend
earlier sections when implementation evidence justifies it; requirements-affecting amendments reach
the user before any unit consumes them. Acceptance sessions mark the milestone header in the
roadmap with the evidence run id and add the local tag `accept/m<n>`.

Normative language: declarative present tense states binding requirements. SHOULD marks a strong
default whose alternative is recorded in a registry, manifest, or gate evidence. MAY marks
options. Versions of packages, solvers, models, and tools live in lockfiles and manifests; prose
names only stable public standards.

## §2 Build plan

Intent: prove the thesis through vertical slices, each ending in a runnable artifact, instead of
assembling the full harness before the first end-to-end result.

| Stage | Deliverable | Proof |
| --- | --- | --- |
| M1 scaffold | Layered pipeline end-to-end on synthetic Japanese test sources: extract → segment → normalize → assemble → compile → verify; one deontic contradiction found, one no-conflict result documented, full trace, deterministic replay. Pure Rust. | `ckc run --experiment exp.m1_scaffold` + §8 checklist |
| M2 multi-hop PoC | Experiment 1's minimal pair: a weak local model (laptop CPU, grammar-constrained, recorded I/O) translates the M1 test sources via `route.direct_smt` versus one IR-mediated route; scored on validity/acceptance/verdict-accuracy/stability raw rows; research report in English and Japanese. | `ckc run --experiment exp.m2_multihop` + §9 |
| M3 ClinicalCNL | Experiment 1's flagship invented form and the product's knowledge surface: bilingual ClinicalCNL (JA primary, EN mirror) — deterministic parse to ClinicalIR, canonical render from guarded-route accepted IR (off-corpus M1: typed omission, §7.2), round-trip laws property-tested, per-document audit views; `route.single_cnl` scored against the §9 pair on the M2 harness over locked M1 inputs. | `ckc run --experiment exp.m3_cnl` + §10 |
| M4 route field + comparison | Route axis widened over existing IR forms (stacked, hop-chain, CKC-layered) plus invented ablations (compact record DSL, labeled-slot CNL) versus `single_cnl`; direct-formalization ablation pipeline; reuse/compactness/hash-convergence/conflict-detection metrics; metamorphic variant test sources; ranked comparison report; model-free coverage experiment; LP explanation lane (Prolog/s(CASP) fixture queries, CNL-verbalized proofs). | `ckc run --experiment exp.m4_routes` / `exp.m4_compare` / `exp.m4_coverage` + §11 |
| M5 optimization PoC | Bounded autonomous-optimization loop (§12) over declared surfaces against a fixed evaluator, optimizing translation reliability, reuse, and coverage; append-only ledger; driver-independent — local driver for acceptance, Claude-agent driver defined (experiment 2's optimization protocol). | `ckc research loop --experiment exp.m5_loop` + §12 |
| M6 sources + expansion | Public corpus ingestion (fetch/cache, permission records, real Minds/J-STAGE HTML+PDF extraction, tables and DecisionTable IR, MEDIS-anchored terminology, e-PI XML source family, drift checks), then registry-guided expansion: retrieval, richer rule semantics, additional solvers/targets, corpus scale, experiment-matrix expansion, the cross-source flagship experiment, deeper DSL capabilities. | §13.1 requirements elaborated at M5 acceptance; §13.2 per candidate |
| M7 auditor product | Reviewer-facing audit over M6 corpora (experiment 3): finding classification (severity, bilingual review questions), weighted minimal-correction-set revision targeting, cvc5 cross-check on blocking/major findings, Lean per-finding proof anchor, human review records, self-contained bilingual `report.html`, auditor manuscript bundle. | `ckc run --experiment exp.m7_audit` + §13.3 |

Scope note: M1–M5 are the current PoC horizon (Stage I); M6–M7 stay in this file as compact
forward requirements (Stage II) so PoC decisions remain production-compatible; Stage III is
requirements-only (§13.4).

Roadmap protocol: `.agent/roadmap.md`, consumed by the session command, carries one milestone at
a time: a header stamped with the commits that open (`plan`) and close (`review`) it, over an
ordered unit checklist whose completed items record context usage and commit hash; closed
milestones persist as bare headers. The plan session that opens a milestone authors its whole
checklist from the milestone's spec section; the milestone is complete when its acceptance item
passes in a dedicated acceptance session and the closing review stamps the header. Lines marked
`user-selected` get scope confirmation from the user before work begins.

Conservation rule: deferred capabilities remain represented — each appears in a
milestone requirements (§9–§13), the registry backlog (§14), or a gate (§15). Elaboration sessions
check deferred items against this rule as the sole scope source.

## §3 Architecture and repository

Intent: one toolchain until evidence demands a second; durable semantics in typed Rust; every
processing stage boundary a validated, content-addressed artifact.

Stack: a Rust workspace (edition 2024) implements everything through M5; external engines (Z3,
the M2 local-model runtime) join as recorded subprocess adapters rather than language bindings.
M6's elaboration decides per extraction/NLP adapter whether to stay Rust or admit a `uv`-managed
Python adapter layer joined only through canonical artifacts and exported JSON Schema; the
decision criteria are determinism, lockability, test source-tested quality, and maintenance cost,
recorded in the registry. Every milestone runs on a single CPU laptop: small quantized local
models, bounded test source sets, and per-query solver resource limits are the standing sizing assumptions
until the user provides larger hardware.

Crates:

| Crate | Owns |
| --- | --- |
| `ckc-core` | IDs, hashes, exact fractions, string policies, canonical bytes, artifact wrappers, enums, source linkage types, IR, plans/manifests, registry types, validation. |
| `ckc-smt` | FormalIR → SMT-LIB emission, solver-query planning, assertion-to-source maps, solver invocation, solver-result parsing. |
| `ckc-cli` | `ckc` binary: pipeline processing stages, runner, trace/report/replay, registry check. |

Pipeline shape (M1–M4; later processing stages splice in without reshaping):

```text
corpus test_source -> extract -> segment -> normalize -> assemble(IR) -> compile(SMT) -> verify -> trace -> report
```

Boundary invariants:

```text
Rust computes every accepted artifact hash.
All semantic state lives in canonical artifacts on disk; processes hold only caches of them.
Every processing stage reads declared inputs, validates them by strict canonical read, and writes only under
its run directory.
Runtime metadata stays outside content hashes.
Accepted artifacts carry wrappers (§4.4); streams (events, diagnostics) are runtime evidence.
Model calls run only inside recorded route stages; every derivation below an accepted artifact
— parse, bridge, compile, verify, render — is deterministic.
```

Repository layout (target state; built up by the M1 units):

```text
.
├── SPEC.md  AGENTS.md  CLAUDE.md  LICENSE  .gitignore
├── Cargo.toml  Cargo.lock
├── crates/{ckc-core,ckc-smt,ckc-cli}/
├── corpus/{test_sources,lexicon,reference}/        # committed, license-clean
├── registry/                              # corpora.yaml candidates.yaml experiments.yaml at M1;
│                                          # grows per milestone (§14)
├── runs/                                  # gitignored run outputs
├── .agent/{context.sh,memory.md,roadmap.md}
└── .claude/
```

CLI (M1 surface; later milestones extend):

```text
ckc registry check
ckc run --experiment <experiment-id> --out runs/<run-id>
ckc replay runs/<run-id>
ckc trace --run runs/<run-id> --finding <finding-id>
```

CLI invariants: each command validates inputs, emits JSONL events, writes only under its output
directory, and ends with exactly one total outcome (§4.4). `registry check` verifies every
registry entry referenced by an experiment resolves and is well-formed.

## §4 Kernel requirements

Intent: a small, stable core that every milestone reuses; stability here is what lets artifacts
survive spec evolution.

### §4.1 IDs, hashes, exact fractions

```text
Id       = lowercase ASCII matching [a-z][a-z0-9_.:-]*
Hash     = "sha256:" + 64 lowercase hex digits
Rational = exact reduced { "num": "<int>", "den": "<positive-int>" }
```

Semantic IDs use lowercase path-like segments; deterministic disambiguation uses source order,
then hash order.

### §4.2 String policies

| Policy | Requirements |
| --- | --- |
| `raw_source` | Preserve the extractor-emitted Unicode scalar sequence exactly. |
| `source_nfkc` | Unicode NFKC only. |
| `semantic_ja` | NFKC; fold whitespace to U+0020; collapse runs; trim; fold common Japanese/fullwidth punctuation to deterministic ASCII equivalents. |
| `semantic_en` | NFKC, whitespace/punctuation folding, lowercase ASCII; for controlled-vocabulary identifiers. |
| `identifier_ascii` | Require `[a-z0-9_:./-]+`; store bytes exactly. |
| `diagnostic_text` | NFKC plus semantic whitespace folding. |
| `rendered_text` | NFKC rendered text with renderer provenance. |

### §4.3 Canonical payload bytes

```text
Objects: UTF-8 field names sorted by byte order; the strict reader rejects unknown fields.
Optional fields: omitted when absent; the strict reader rejects JSON null.
Arrays: ordered when order is semantic.
Sets: arrays sorted by canonical_sort_key.
Maps: identifier_ascii keys as sorted objects; other keys as sorted key/value arrays.
Strings: UTF-8 under the schema-declared string policy before hashing.
Integers: decimal strings in accepted artifacts.
Rationals: exact reduced objects; the strict reader rejects bare numeric tokens.
Unions: tagged objects with exactly "tag" and "value".
Fieldless enums: identifier_ascii strings.
content_hash = sha256(canonical_payload_bytes(payload)).
```

Each payload type has exactly one byte serialization; the canonical writer and strict reader are
mutual inverses, round-trip tested. Structural hashes of IR components derive from canonical
bytes with locally indexed references, so a component's hash is stable under semantic-id renames.

### §4.4 Artifact wrapper, enums, outcomes

Every accepted artifact is one canonical JSON artifact wrapper:

| Field | Semantics |
| --- | --- |
| `schema_version` | `"ckc.1"`; bumped on breaking schema change. |
| `schema_id` | Schema identifier, e.g. `schema.ir_bundle`. |
| `artifact_id`, `artifact_kind` | Stable semantic id and kind. |
| `producer` | `{pipeline_id, pipeline_step_id, toolchain_manifest_hash}`. |
| `input_hashes` | Content hashes of consumed accepted artifacts. |
| `content_hash` | Hash of canonical payload bytes. |
| `canonicalization_policy_hash` | Hash of the policy descriptor in force. |
| `origin` | See Origin enum. |
| `evidence_status` | See Evidence status enum. |
| `external_effects` | `[]` for accepted semantic artifacts; evidence-discovery artifacts may record `network`, `clock`, `ai`, `tool`. |
| `trace_refs`, `diagnostics` | Trace links; structured diagnostics (stable codes, §7.4). |
| `runtime_metadata` | Excluded from `content_hash`. |
| `payload` | The typed content. |

Fields ending in `_hash` reference accepted-artifact content hashes unless their schema declares
raw-byte hashing. `compiler_evidence_status` is reserved for compiled artifacts and
`verifier_evidence_status` for verifier results. Accepted semantic evidence status begins after applicable
schema validation, source linkage (§4.5), canonicalization, applicable compiler/verifier checks,
trace/replay recording, and evidence criteria.

Enums (stage column = first milestone that uses the value set):

| Enum | Values | Stage |
| --- | --- | --- |
| `Outcome` | `ok residual ambiguity incoherence unsupported invalid`; severity order `invalid > incoherence > unsupported > ambiguity > residual > ok` | M1 |
| `Origin` | `human_authored ai_assisted ai_generated external_adapter_generated deterministic_compiler` | M1 |
| `EvidenceStatus` | `source_evidence_status mechanical_evidence_status evidence_discovery_only accepted_evidence_status compiler_evidence_status verifier_evidence_status view_only` | M1 |
| `BindingStatus` | `exact synonym ambiguous unmapped` | M1 |
| `Direction` | `for against contraindicate require permit avoid` | M1 |
| `ClaimTier` | `s0_replayable s1_accepted s2_research_evidence s3_clinical_regulatory` | M1 |
| `ReviewClassification` | `candidate residual ambiguity incoherence replay_failure documented_no_conflict_result` | M1 |
| `AttemptOutcome` | `improved equivalent dominated regression invalid unsupported timeout crash no_conflict_result near_miss unreproducible unauthorized gate_required` | M5 |
| `PromotionDecision` | `promote reject quarantine defer_gate request_replay` | M5 |
| `PromotionScope` | `run_local registry_status` | M5 |
| `Severity` | `blocking major moderate minor info` — finding classification labels | M7 |

Outcome meanings:

| Outcome | Meaning |
| --- | --- |
| `ok` | Output valid for the declared processing stage. |
| `residual` | Schema-valid but incomplete: permission-limited, missing evidence, missing policy, partial extraction. |
| `ambiguity` | Multiple admissible readings, bindings, spans, or normalizations remain. |
| `incoherence` | Accepted harness inputs collide (e.g. incompatible policy rows); source-level conflicts between guideline rules are findings, delivered as values under `ok`. |
| `unsupported` | Schema-valid construction outside implemented semantics. |
| `invalid` | Schema, hash, canonicalization, registry, or command validation fails. |

Every operation has one total operation result — an outcome plus the hash sets below. A
processing stage's result rides its §4.6 EventRecord (the §8.3 run layout adds no per-stage
result artifact); commands materialize the standalone record:

```json
{"operation_id":"compile","outcome":"ok","value_hashes":["sha256:..."],"diagnostic_hashes":[],
 "residual_hashes":[],"ambiguity_hashes":[],"incoherence_hashes":[]}
```

`value_hashes` hash the produced value artifacts (empty when a separate manifest already
attests them, as the run command's manifest does); `diagnostic_hashes` the §7.4 records;
`residual_hashes`/`ambiguity_hashes`/`incoherence_hashes` the typed-placeholder payloads, empty
until a milestone materializes such placeholders. Partial success is expressed through typed
residual/ambiguity payloads (typed placeholders) so downstream processing stages keep operating
on the valid remainder and traces stay complete.

### §4.5 Source linkage

| Object | Requirements |
| --- | --- |
| `SourceDocument` | Document identity: source family, provenance (`synthetic` or `public`), raw/content hashes, `data_class` (default `none`; populated under §15 gates). |
| `SourceDocumentGraph` | Finite node graph (document, section, paragraph, list, table, cell, caption, footnote, CQ, recommendation) plus the spans, anchors, and regions below; one artifact per document, emitted by extract. |
| `SourceTextSpan` | Stable text span: node, offsets, `raw_text`, `nfkc_text`, `search_text`, reading order, text hash. |
| `SourceAnchor` | Subspan anchor for mentions, quantities, modalities, negation, temporal cues, table values. |
| `EvidenceRegion` | Closed support set over nodes/spans/anchors/cells; the unit of evidence. |

Source linkage invariants:

```text
Every extracted textual unit has a SourceTextSpan or a typed extraction_uncertain residual.
Every semantic claim carries source_region_ids, or synthetic_test_source_id when injected without a
document form.
Identical source bytes and extraction config produce identical SourceDocumentGraph canonical bytes.
Every rule in every report finding exposes its source-grounded rationale (region ids resolve to
quotable spans).
```

### §4.6 Events, replay, provenance

Event fields (JSONL, `events.jsonl`):

```text
event_id run_id pipeline_id pipeline_step_id processing_stage log_level event_sequence_number started_at ended_at duration_ms
input_hashes output_hashes outcome diagnostics resource_counters
```

Logs are runtime evidence; accepted semantics live only in validated artifacts.
`diagnostics.jsonl` carries §7.4 diagnostic records as JSONL. Run ids are runtime metadata,
excluded from content hashes.

`replay_manifest.json` records command, input hashes, lexicon/corpus hashes, toolchain manifest
hash, environment profile, lockfile hashes, solver identity, and expected output hashes — a
provenance/attestation record over content hashes. `ckc replay` re-executes and compares
canonical content hashes (timestamps and other runtime metadata excluded); mismatches emit
mismatch diagnostics; missing external tools emit `replay_identity_unsupported`.
Repeated deterministic runs over the same inputs produce matching hashes — re-run-equals-prior is
the standing idempotency property check.

## §5 Domain model and IR

Intent: a typed bridge from Japanese prose to formal constraints, with reusable components as the
unit of the thesis.

| Object | Requirements (M1 unless tagged) |
| --- | --- |
| `ClinicalSegment` | CQ, recommendation, evidence, exception, definition, table-row, or metadata segment with region refs. |
| `TerminologyBinding` | Mention → concept binding: `system` (M1: `ckc.lex`), code, status (BindingStatus), alternatives, region refs — provenance at the producer's grain: M1 normalize grounds the mention, M3's CNL bridge mints citing statements' segment closures (§10). |
| `ClinicalStatement` | Normalized population, condition, action, modality, strength (`strong\|weak`), certainty (`high\|moderate\|low\|very_low`), exceptions, source refs; comparator/outcome/temporal slots optional at M1. |
| `Action` | Action kind + target concept + distinguishing fields (M4) + normalized target key. |
| `ContextExpr` | Finite DNF over atoms: concept predicate, negated concept predicate, quantity interval; M4 adds slot equality and temporal interval (difference-logic) atoms. |
| `NormativeRule` | `rule_id, context, direction, action, strength, source_region_ids` + optional at M1 `certainty, exception_refs`; exceptions compile per positive concept atom to negated context conjuncts (§10 keeps clauses single-concept; wider clause shapes sit outside the compile contract), their regions joining `source_region_ids`. |
| `FactualRule` (M4) | Context → factual consequent, strictness. |
| `DecisionTable` (M6) | Input variables, units, rows, guards, outputs, source rows; DMN-style overlap semantics. |
| `IRBundle` | The five layers below + reusable component records + assumptions + diagnostics + per-layer and whole-bundle structural hashes. |
| `CompiledArtifact` | Target id, logic, query plan, query bodies, named-assertion records (assertion id → rule ids → region ids), diagnostics. |
| `VerifierResult` | Per-query status (§6 categories), model or unsat core, solver identity, diagnostics. |
| `TraceBundle` | Derivation DAG + claim-evidence rows; M4 adds reuse/compactness graphs. |
| `LineageIndex` | Query index: artifact/finding ↔ source text spans ↔ rules ↔ assertions ↔ verdicts ↔ report. |
| `RunPlan` | Experiment id, test source groups, pipeline(s), seed, budget; canonical bytes hashed into the manifest. |
| `RunManifest` | Run plan hash, git commit, toolchain/lockfile/corpus/lexicon hashes, environment profile, solver identity, output hashes. |
| `Report` | report.json (canonical) + report_en.md (derived view): findings, no-conflict results, diagnostics, metrics (M2+), wording per §0. |
| `CnlDocument` (M3) | Canonical ClinicalCNL text (JA primary, EN mirror) + the CNL AST it parses to, over an IRBundle's ClinicalIR content: grammar id + hash, per-rule AST + canonical text, text hash; §10 text↔AST inverse laws, AST↔ClinicalIR bridge. Not a new IR layer — ClinicalIR's second concrete syntax beside canonical JSON. |

IR layers in one `IRBundle` per document:

| Layer | Content |
| --- | --- |
| `DocIR` | Layout-preserving text/table view over SourceDocumentGraph refs with extraction diagnostics. |
| `SegmentIR` | ClinicalSegments. |
| `ClinicalIR` | ClinicalStatements + TerminologyBindings (+ CQ/PICO/EtD slots, optional M1). |
| `NormIR` | NormativeRules (+ FactualRules M4, DecisionTables M6). |
| `FormalIR` | Target-independent constraints, normalized actions/contexts, contradiction-query plan. |

IR invariants:

```text
Every reusable action, condition, population, concept, rule, and constraint has a stable Id and a
normalized structural hash; component records list use sites.
IRBundle validates (source linkage, references, policy completeness) before compilation.
Assumptions and uncertainty are explicit payload fields.
Layered pipelines expose component reuse metadata; M4 metrics consume it.
```

Lexicon: `corpus/lexicon/ja_core.yaml` is the M1 terminology and modality reference file
(system `ckc.lex`): concept entries (id, surface forms, optional interval semantics such as
成人 → `age >= 18`), action verbs, modality phrases mapped to (direction, strength) — e.g.
推奨する → (`for`, strong); 提案する/考慮してもよい → (`for`/`permit`, weak); 禁忌/投与しないこと →
(`contraindicate`, strong); 推奨しない → (`against`, strong); 提案しない → (`against`, weak) —
and certainty phrases (エビデンスの確実性:中 → `moderate`) feeding `certainty` when present.
Versioned by content hash in every manifest. Binding statuses: `exact`/`synonym` satisfy concept
demands (after representative normalization); `ambiguous` emits
`Ambiguity(terminology_ambiguous)` and `unmapped` emits `Residual(terminology_unmapped)` when one
concept is required. External terminologies (MEDIS standard code tables first: license-clean, MHLW-designated)
join at M6 as additional systems behind the same TerminologyBinding requirements.

Semantic policy invariants:

```text
Action sameness = same action kind + terminology-representative target + (M4) distinguishing
fields, via normalized target keys.
Strength and certainty are proof-visible annotations; conflict logic consumes direction and
normalized action/context.
Missing required policy emits Residual(missing_policy); duplicate policy keys with different
payloads emit Incoherence(incompatible_policy_rows) quarantining only the conflicting rows.
Residual/Ambiguity/Incoherence diagnostics are §7.4 records.
```

## §6 Conflict semantics and formal profile

Intent: small, decidable, satisfying example-producing checks; every verdict names its evidence.

Direction groups:

| Group | Directions |
| --- | --- |
| positive | `for require permit` |
| against | `against avoid` |
| contraindicating | `contraindicate avoid` |

A rule pair is conflict-eligible when normalized actions are the same and one direction is in
`positive` while the other is in `against` or `contraindicating`.

Two-query check per eligible pair (the contradiction-query plan):

```text
Q1 context_overlap: assert both rules' conditioned contexts (exceptions as §5 negated conjuncts);
  sat -> overlap satisfying_example model recorded; unsat -> pair closed as documented no-conflict result
  (no shared context).
Q2 deontic_consistency: for pairs with a sat Q1, assert each rule's direction as a polarity
  literal on the shared action, each as a :named assertion; unsat -> semantic_contradiction with
  unsat core naming the contributing assertions; sat -> documented no-conflict result. M4 conflict
  kinds extend Q2 with threshold, slot, and factual constraints.
M4 adds a per-rule Q0 self-check (own conditioned context satisfiability; unsat ->
  condition_unsatisfiable) and a raw Q1 view with exception conjuncts stripped: raw sat with
  the conditioned view unsat closes the pair as documented no-conflict result carrying an
  exception_resolved_conflict finding.
```

Conflict kinds (stage = first milestone that detects them):

| Kind | Stage | Idea |
| --- | --- | --- |
| `context_compatibility` | M1 | Finite context overlap over concept and interval atoms. |
| `normalized_action_sameness` | M1 | Eligibility via normalized action keys. |
| `deontic_direction_conflict` | M1 | Opposed direction groups under satisfiable shared context. |
| `numeric_threshold_empty_intersection` | M4 | Same action+direction, disjoint quantity/temporal intervals. |
| `strict_factual_contradiction` | M4 | Strict factual consequents jointly inconsistent. |
| `terminology_incoherence` | M4 | Functional key collision or mutually exclusive mapping. |
| `condition_unsatisfiable` | M4 | A rule's own context unsatisfiable (Q0): extraction or normalization defect, or genuine source defect. |
| `exception_resolved_conflict` | M4 | Raw overlap sat, conditioned overlap unsat: the exception averts a live conflict; informational finding on the closed pair. |
| `table_value_disagreement` | M6 | Overlapping table guards, incompatible outputs. |
| `source_metadata_disagreement` | M6 | Singleton metadata values disagree after normalization. |
| `gloss_drift` | M6 | Rendered view diverges from semantic payload. |
| `replay_or_certificate_failure` | M1 | Replay mismatch or certificate check failure. |
| `package_insert_vs_guideline_conflict` | M6 | Cross-source flagship (e-PI test sources registered). |
| `priority_unresolved` | M7 | Conflicting defeasible rules with insufficient rule-priority metadata (ASP lane, behind the §13.2 richer-rule-semantics trigger). |
| `source_support_mismatch` | M7 | Accepted IR whose cited spans fail human-reviewed support review — acceptance audit over model-assisted authoring. |

SMT profile:

```text
Target: SMT-LIB 2 text artifacts, embedded in CompiledArtifact payloads and materialized
byte-identically under groups/<gid>/smt/ for solver consumption.
Logic: narrowest sufficient logic, recorded per query; M1 default QF_LRA (Bool constants +
linear-real interval atoms). M4 adds difference-logic temporal atoms; declared target profiles
gate anything richer, which otherwise returns unsupported_fragment.
Symbols: SMT symbols are |-quoted canonical Ids, so assertions remain self-identifying.
Every assertion that can influence a query is :named and mapped in the assertion map to IR rule
ids and source region ids; assertion ids form a.<rule_id> (polarity/factual) and ctx.<rule_id>
(context).
Emission is deterministic and byte-pinned (§8.6 listings are exact emitter output): one
s-expression command per line, no comments, the file ending in a newline; command order
set-logic, :print-success false, :produce-models / :produce-unsat-cores as used, declarations
sorted by symbol bytes one per line, named assertions in pair order, check-sat, then the
result command. Guard conjuncts render in stored ContextExpr order, a single-disjunct any
collapses to a bare and, and Rational numerals render as a plain integer when the denominator
is 1, else (/ n d).
Solvers: Z3 required first (binary invocation; identity+version recorded in manifests and
verifier results); cvc5 registered as the second solver (Alethe/LFSC certificate path, §13).
(get-unsat-core) on unsat; satisfying_example model on sat where relevant. Verifier adapters parse the
verdict token and result s-expressions, normalize core tokens (strip |…|) to Ids, and record
cores as canonical sets sorted by canonical_sort_key; core comparison is set-based.
```

Verifier result categories: `schema_failure compiler_failure target_syntax_failure
solver_execution_failure semantic_no_conflict semantic_contradiction unknown
unsupported_fragment` — with `sat/unsat/unknown/timeout` and solver diagnostics preserved
distinctly.

LP profile (M4; a second target lane — execution and explanation over unchanged semantics):

```text
Target: Prolog-family logic program emitted deterministically from NormIR — rules-as-data
facts (rule, population, condition, action, direction, strength, certainty, exception,
source) over a small fixed committed kernel deriving applicability and advice, with
negation-as-failure guarding labeled exception predicates (the PROLEG pattern; §5
exceptions stay separate labeled payloads: the SMT lane expands them to negated context
conjuncts, the LP lane keeps them as NAF guards).
Engines: SWI-Prolog and s(CASP) join as recorded subprocess adapters (the Z3 pattern);
s(CASP) proof trees verbalize through CNL lexicon templates so justifications read as §10
audit prose.
Lane separation: the SMT lane remains the conflict oracle — LP verdicts never replace §6
SMT verdicts absent an explicit §13.2 richer-rule-semantics adoption; differential tests
cover only shared fragments (context satisfaction over finite fixture contexts, exception
expansion equivalence). Fixture contexts are synthetic; patient-context rule-evaluation
semantics in any shipped output stays behind G-RULE-EVAL (§15).
```

## §7 Evidence: traces, reports, metrics, diagnostics

Intent: every claim is a path through artifacts; every miss is a typed data.

### §7.1 Trace

`trace_bundle.json` holds the derivation DAG (source → extraction → segment → normalization → IR
→ compile → verify → report nodes with operation-labeled edges) and claim-evidence rows (finding
→ region ids → rule ids → assertion ids → verdict → report ref). `lineage_index.json` is its
query index; `ckc trace` resolves a finding to the full chain in both directions. M4 adds
component-reuse and compactness exports plus deterministic path visualizations: `trace_graph.dot`
(sorted nodes/edges) and per-finding Mermaid blocks in `report_en.md`, rendering the chain from
Japanese source text span to solver verdict and the convergence of documents onto shared mapping
components; rendering to images is a view concern with renderer identity recorded. M5 adds
attempt-ledger rows to the trace exports (§12). The lineage index and derivation DAG subsume
dedicated mapping-hypergraph and axiom-dependency exports at test source scale; those exports
re-stage with M6 corpus scale if measurement demands them.

### §7.2 Reports

`report.json` is canonical; `report_en.md` is a deterministic rendering; from M2, `report_ja.md`
joins it as a deterministic Japanese rendering of the same canonical content. Contents: corpus and
lexicon hashes, findings (each with conflict kind, rules, regions, quoted spans under permission
rules, assertion names, core), documented no-conflict results, a diagnostics summary (code-keyed
failure-taxonomy summary), solver identity, replay status; from M2, raw metric rows before any
weighted ranking; from M3, per-document CNL audit views, per-rule CNL text keyed by
normative rule id via §10's origin map (a (pipeline, document) whose accepted IR the §10
predicate rejects at `from_ir` — a guard-less route, the M1 route off its locked corpus —
carries no CNL entries: omitted under one typed `cnl_inexpressible_ir` diagnostic (§7.4),
never an assembly failure), and a `findings_owner_pipeline_id` field naming the
findings-view owner (findings and documented no-conflict results quote their rules as CNL —
single-backtick inline code spans, the normative delimiter; code-span-inert = no literal
backtick, no line break, validated at report shape — beside the quoted source spans from the owner pipeline's entry alone, owner-labeled —
normative rule ids are route-local positional identities, never cross-route alignment keys:
a same-numbered id under another route may hold different content, so cross-route CNL
comparison requires an explicit alignment map, out of scope through M3, and non-owner
entries feed audit views only; a rule id the owner's entry does not carry — the owner's
view CNL-inexpressible — renders without CNL); from
M4, ablations; from M5,
attempt-ledger summaries; from M6, matrix coverage.
Finding ids form `finding.<group_id>.<sequence_number>` with sequence numbers in source-then-hash order (§4.1).
A multi-route run keeps exactly one findings view: the first compiled (bundle-bearing) pipeline
in experiment binding order feeds the findings body, the documented no-conflict results, and
the report's verifier-result identity — `findings_owner_pipeline_id` records that pipeline
canonically, present whenever the run lands a findings view — keeping payload query and
finding ids route-unprefixed;
every other compiled route lands route-namespaced artifacts feeding audit views, metrics, and
ledgers only.
From M7, findings carry `severity` (§4.4) and a bilingual suggested review question. From M5,
publication-designated runs export a manuscript bundle — figure-ready CSV/JSON metric tables,
corpus/permission summaries, replay instructions, limitations derived from typed
residual/ambiguity statistics — extended at M7 with finding and human review tables (§0 stage
arc: Stage I methods paper, Stage II auditor paper).
Report wording stays within the §0 vocabulary.

### §7.3 Metrics (M2 onward)

Metric values are exact fractions; unavailable values are omitted with a diagnostic; zero
denominators emit `not_applicable` per metric schema. Raw rows always accompany rankings. Core
metric families: reuse (component reuse rate, duplicate rate), compactness (component count,
mapping-set size versus coverage, reuse degree, MDL proxies), convergence (normalized hash
agreement across variants), compilation (schema/compile/parse/solver pass rates), conflict
quality (precision/recall and conflict-task accuracy over test source expectations), trace
completeness, determinism (hash stability), baseline delta (per-metric route-versus-baseline deltas over
identical test sources: model routes from M2, layered-minus-direct from M4), route quality
(schema-valid rate, acceptance rate, repair count, recorded-call counts, k-sample convergence;
from M2), surface quality (round-trip identity rate, surface tokens per accepted rule —
surface tokens = the committed ClinicalCNL JA lexer's token count over the accepted
document's stored canonical JA rule texts, deterministic and runtime-free: a model-runtime
token count would bind the metric to a versioned runtime-tokenizer replay dependency,
tokenizer identity + counts separately attested; from
M3), translation faithfulness (share of a route's accepted documents whose IR content equals
the deterministic reference derivation over identical inputs under the §10 faithfulness
projection, binding region provenance excluded — conflict-quality verdicts
saturate while faithfulness still separates routes, `docs/poc-archive.md`; from M3),
model-free coverage (share of fresh-document semantics produced deterministically from
accepted mappings, with zero application phase model calls; from M4), claim completeness (share
of normative-candidate source regions claimed by an accepted rule or covered by a typed
residual, §11; from M4), and loop outcomes (from M5).

### §7.4 Diagnostics

Every diagnostic carries a stable code, a structured payload, region/artifact refs, and maps to
exactly one Outcome. Base code set (some codes first emit with the M4 comparison):

```text
extraction_uncertain table_structure_uncertain span_source_linkage_missing segmentation_boundary_error
terminology_unmapped terminology_ambiguous terminology_incoherent semantic_slot_missing
missing_policy incompatible_policy_rows unsupported_ir_fragment schema_invalid compiler_error
target_parse_error solver_timeout solver_unknown solver_execution_failure process_crash
trace_incomplete replay_mismatch replay_identity_unsupported deferred_gate_required
false_positive_conflict false_negative_conflict metamorphic_instability
```

M2 adds model-route codes (`ai_schema_violation`, `ai_hallucinated_source`,
`repair_limit_exceeded`); M3 adds CNL codes (`cnl_parse_error`, `cnl_round_trip_mismatch`,
`cnl_unregistered_concept`, `cnl_inexpressible_ir`);
M4 adds invented-DSL route codes and the claim-completeness code
(`normative_region_unclaimed`, §11); M5 adds loop/budget/surface codes
(`unauthorized_surface_edit`, `budget_exhausted`); M6 adds source/permission/drift codes; each
is defined in its milestone section at elaboration time.

## §8 M1 — Scaffold

Intent: the smallest complete instance of the thesis machinery — one layered pipeline, real
Japanese text shapes, a real solver, full trace, deterministic replay. Everything later is
measured against this scaffold, so its requirements are exact.

### §8.1 Scope

Pipeline `pipe.layered_ckcir_to_smt` over synthetic test sources; deterministic throughout
(`runtime_ai: false` is the M1 scaffold condition; recorded model artifacts first appear under §9
requirements at M2). Experiment `exp.m1_scaffold`.

### §8.2 Test sources

Committed under `corpus/test_sources/` as minimal well-formed HTML (headings, paragraphs, one list,
one small table) with `registry/corpora.yaml` entries; origin `ai_generated`, evidence status
`source_evidence_status` on acceptance, provenance `synthetic` — a working example of
acceptance-over-proposer precedence.

| Test source | Content | Role |
| --- | --- | --- |
| `test_source.m1_guideline_a` | Minds-style synthetic guideline: CQ + recommendation 「成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する(強い推奨)」 with exception 「ただし、重度腎機能障害のある患者を除く」, plus one definitions table and an evidence list for layout coverage. | Recommendation source. |
| `test_source.m1_guideline_b` | Synthetic companion document: 「成人の敗血症患者のうち、妊娠中の患者には抗菌薬Aを投与しないこと(禁忌)」. | Overlap + contraindication → expected contradiction with A. |
| `test_source.m1_control` | Synthetic document: 「小児(18歳未満)の敗血症患者には抗菌薬Aは禁忌である」. | Age intervals disjoint with A → expected no-conflict result. |

Test source groups in `exp.m1_scaffold`: `group.m1_conflict = [a, b]` expecting one
`deontic_direction_conflict` finding; `group.m1_no_conflict = [a, control]` expecting
`semantic_no_conflict` + `documented_no_conflict_result`. Expected outcomes live in
`corpus/reference/m1_expected.yaml`, asserted by the acceptance tests, one entry per test source group:

```yaml
- group_id: group.m1_conflict
  expected_outcome: semantic_contradiction
  expected_conflict_kind: deontic_direction_conflict
  expected_unsat_core: [a.test_source.m1_guideline_a.rule.0, a.test_source.m1_guideline_b.rule.0]   # compared as a set
- group_id: group.m1_no_conflict
  expected_outcome: semantic_no_conflict
  expected_no_conflict_result: true
```

### §8.3 Processing stage requirements

| Processing stage | Requirements | Artifact (per document) |
| --- | --- | --- |
| extract | Parse test source HTML (real HTML parser) → SourceDocumentGraph with nodes, spans, anchors, regions; tables preserve row/column/cell/header relations; uncertainty emits typed residuals. | `source_document_graph.json` |
| segment | Rule-based segmentation keyed on test source structure (CQ headings, recommendation/exception sentence markers) → ClinicalSegments with region refs. | `segments.json` |
| normalize | Lexicon-driven: create terminology bindings (TerminologyBindings), normalize statements (ClinicalStatements), derive NormativeRules with conditioned contexts; interval semantics from lexicon (成人/小児 → age bounds). | `normalization.json` |
| assemble | Validate and assemble the five-layer IRBundle; per-layer and bundle structural hashes; component records. | `ir_bundle.json` |
| compile | Across each test source group: eligibility scan, contradiction-query plan, deterministic SMT emission, assertion map. | `groups/<gid>/compiled.json` + `groups/<gid>/smt/<query>.smt2` |
| verify | Invoke Z3 per query; parse verdicts, cores, models into VerifierResults. | `groups/<gid>/verifier_results.json` |
| trace | Assemble TraceBundle + LineageIndex across the run. | `trace_bundle.json`, `lineage_index.json` |
| report | Render report.json/report_en.md; write run + replay manifests. | `report.json`, `report_en.md`, `manifest.json`, `replay_manifest.json` |

Run layout:

```text
runs/<run-id>/
├── manifest.json  replay_manifest.json  report.json  report_en.md
├── trace_bundle.json  lineage_index.json
├── artifacts/<doc-id>/{source_document_graph,segments,normalization,ir_bundle}.json
├── groups/<group-id>/{compiled.json,verifier_results.json,smt/<query>.smt2}
└── logs/{events.jsonl,diagnostics.jsonl}
```

### §8.4 Registries at M1

`registry/corpora.yaml` (test sources above), `registry/candidates.yaml` (the pipeline and its processing stage
entries with ids, kinds, determinism, input/output artifact kinds), `registry/experiments.yaml`
(`exp.m1_scaffold`: test source groups, pipeline, seed, budget, expected-outcome ref). `ckc registry
check` validates all three and verifies each pipeline's processing-stage chain: every processing stage's
declared input artifact kinds are produced by its predecessors.

### §8.5 Acceptance checklist

1. `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
   `cargo test --workspace` pass.
2. `ckc registry check` passes.
3. `ckc run --experiment exp.m1_scaffold --out runs/m1` completes with outcome `ok`, emitting the
   §8.3 artifact set with every accepted artifact passing strict canonical read (enforced by a
   workspace test over the run directory).
4. Every named assertion in each `compiled.json` maps to IR rule ids and source region ids.
5. `group.m1_conflict` yields `semantic_contradiction` with an unsat core naming assertions
   derived from both documents.
6. `group.m1_no_conflict` yields `semantic_no_conflict` and a `documented_no_conflict_result` entry in the
   report, evidenced by the Q1 unsat (disjoint age intervals).
7. `ckc trace --run runs/m1 --finding <finding-id>` prints the complete chain: source text spans →
   segments → statements → rules → named assertions → solver verdict → report finding.
8. `ckc replay runs/m1` reports matching canonical content hashes for all accepted artifacts.
9. `report_en.md`/`report.json` carry findings, the no-conflict result, diagnostics, solver identity, and
   §0-vocabulary wording, with quoted Japanese spans resolving to test source bytes.

### §8.6 Worked thread (docA × docB)

Source text span (docA): 「成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する」 + exception
span 「ただし、重度腎機能障害のある患者を除く」.

Ids: `rule_id = <document_id>.rule.<k>` in derivation order (`rules[k]` derives from
`statements[k]`; document ids are the corpora test source ids), so rule ids — and the assertion
names built from them — stay unique when one SMT file cores several documents; every other id
is a document-local counter (regions `r.<k>`, exception clauses `exc.<k>`).

NormativeRule (canonical payload: fields byte-sorted, atoms as §4.3 tagged unions, conjunct sets
sorted by canonical_sort_key):

```json
{"action":{"key":"act.administer:drug.abx_a","kind":"act.administer","target":"drug.abx_a"},
 "context":{"any":[{"all":[
   {"tag":"concept","value":"cond.sepsis"},
   {"tag":"concept_negated","value":"cond.renal_severe"},
   {"tag":"interval","value":{"ge":"18","var":"q.age_years"}}]}]},
 "direction":"for","exception_refs":["exc.0"],
 "rule_id":"test_source.m1_guideline_a.rule.0",
 "source_region_ids":["r.2","r.3"],
 "strength":"strong"}
```

docB yields `test_source.m1_guideline_b.rule.0`: context `cond.sepsis ∧ age ≥ 18 ∧ cond.pregnancy`,
direction `contraindicate`, same action key → pair eligible.

Q1 `q.m1_conflict.pair1.overlap` (QF_LRA; expected sat, satisfying example model recorded):

```smt2
(set-logic QF_LRA)
(set-option :print-success false)
(set-option :produce-models true)
(declare-const |cond.pregnancy| Bool)
(declare-const |cond.renal_severe| Bool)
(declare-const |cond.sepsis| Bool)
(declare-const |q.age_years| Real)
(assert (! (and |cond.sepsis| (not |cond.renal_severe|) (>= |q.age_years| 18)) :named |ctx.test_source.m1_guideline_a.rule.0|))
(assert (! (and |cond.pregnancy| |cond.sepsis| (>= |q.age_years| 18)) :named |ctx.test_source.m1_guideline_b.rule.0|))
(check-sat)
(get-model)
```

Q2 `q.m1_conflict.pair1.deontic` — polarity literals on the shared action (overlap witnessed
by Q1); expected unsat, the core recorded as the canonical set:

```smt2
(set-logic QF_UF)
(set-option :print-success false)
(set-option :produce-unsat-cores true)
(declare-const |pos:act.administer:drug.abx_a| Bool)
(assert (! |pos:act.administer:drug.abx_a| :named |a.test_source.m1_guideline_a.rule.0|))
(assert (! (not |pos:act.administer:drug.abx_a|) :named |a.test_source.m1_guideline_b.rule.0|))
(check-sat)
(get-unsat-core)
```

VerifierResult: `semantic_contradiction`, core
`[a.test_source.m1_guideline_a.rule.0, a.test_source.m1_guideline_b.rule.0]`. Report finding
`finding.group.m1_conflict.1` cites both rules, their regions, the quoted spans, the core, and
classifies as `deontic_direction_conflict`, claim tier `s1_accepted`, wording `synthetic test source
measurement`. The control group's Q1 is unsat (`age >= 18` vs `age < 18`), closing as
`documented_no_conflict_result`. `ckc trace` walks the chain from 「妊娠中の患者には…投与しないこと」 to
the core and back.

## §9 M2 — Multi-hop translation PoC (requirements; elaborate at M1 acceptance)

Intent: experiment 1's minimal pair on this laptop. Establish as a locked measurement that a
weak local model translating clinical Japanese directly into a machine-evaluable formal target
is unreliable, and that one IR-mediated route measurably improves reliability on the same
inputs; publish the result as a bilingual research report. The M1 scaffold is the instrument:
its deterministic pipeline supplies the reference verdicts and the calibrated compile → verify back
end that scores both routes, so route failures attribute to translation, not to the instrument.

Committed direction:

- Model harness: a local-model runtime invoked as a recorded subprocess (the Z3
  pattern): greedy decoding with a fixed seed (k-sample convergence draws k recorded samples via
  per-sample seeds), grammar-constrained output via a grammar or JSON-Schema compiled from the §4/§5
  type schemas — the committed `schemas/` export and `registry/{schemas,prompts}.yaml` land here
  (§14) to feed grammars and prompt templates. Model identity, quantization, and runtime version
  live in manifests; the baseline SHOULD be a small Japanese-capable instruct model (sub-4B, CPU
  quantized) weak enough that direct-route failures are common — that headroom is the
  experiment. Model I/O records as test source artifacts (origin `ai_generated`, evidence status
  `evidence_discovery_only`, prompt-template hashes in manifests); recorded bytes replay
  deterministically; live calls run only under an explicit experiment flag with full recording.
- Exactly two routes — the minimal pair (further routes are §10/§11 scope):

| Route | Shape |
| --- | --- |
| `route.direct_smt` | Model emits SMT-LIB text directly — the weak baseline. |
| `route.single_ir` | Model fills one grammar-constrained IR schema; deterministic compile from there. Elaboration picks the layer: a CKC IR layer (the §6 compiler takes over) is the default; an existing-IR shape (e.g. DMN-style condition/action rows) is the registered alternative. |

- Inputs: the M1 test sources, lexicon, and reference, locked under a minimal measurement record —
  test source/reference/schema/prompt/model/runtime hashes in the run manifest (the evaluator identity
  that §11 formalizes and §12 locks).
- Scoring (§7.3 route-quality and baseline-delta metrics, raw rows before any ranking): target syntactic
  validity (solver parse), acceptance rate — model output passes the same §4 acceptance checks as
  any artifact — conflict-verdict accuracy against reference over the §8 conflict and no-conflict groups,
  and k-sample verdict stability; §6 categories and the §7.4 M2 model-route codes carry the
  failure taxonomy; documented no-conflict results are first-class.
- Report: `report_en.md` (English) and `report_ja.md` (Japanese) render deterministically from one
  canonical `report.json` (§7.2): per-route raw rows, the baseline-delta table, findings with quoted
  Japanese spans and named assertions, failure-taxonomy summary, model and solver identities,
  replay status; wording per §0 (locked measurement, synthetic test source measurement; no clinical
  claims).
- Deliberately out of scope, landing in §10/§11: additional routes, metamorphic test sources, the
  component store, the deterministic direct pipeline, model-free coverage, ablations.

Acceptance themes (finalized at elaboration): both routes execute over identical locked inputs
(`exp.m2_multihop`); recorded model I/O replays byte-stably; raw rows emit before the
baseline-delta table; expected conflict/no-conflict outcomes hold per reference for accepted translations;
the bilingual report renders deterministically from `report.json`; §0 vocabulary holds.

## §10 M3 — ClinicalCNL v1 (elaborated 2026-07-07; M3 planning seeds units from this section)

Intent: land ClinicalCNL — the clinician-auditable controlled natural language, the §0
knowledge surface; named to mirror ClinicalIR, the layer it serializes — as experiment 1's
flagship invented form. One content layer, two concrete syntaxes: ClinicalIR serializes as
canonical JSON (§4.3) for machines and as ClinicalCNL for clinicians. Parse and render are
mutual inverses between canonical text and the CNL AST; a deterministic bridge maps that AST
to and from a ClinicalIR projection under the round-trip laws below. A clinician audits
controlled prose beside quoted source spans — never JSON, IR slots, SMT, or Prolog — and an
accepted, reviewed CNL document is the locked knowledge base from which every target
regenerates. This section is the design
authority for M3 planning; once landed, the committed grammar files and renderer are the byte
authority (worked text here is illustrative, the §8.6 rule).

Research base (`docs/cnl-attempto.md`, `docs/cnl-multilingual-ja.md`, `docs/cnl-landscape.md`,
`docs/cnl-design-codex.md` — the second-opinion design): no surveyed system offers
deterministic CNL parse ⇄ verbalize ⇄ executable formal target, no deterministic
Japanese-parsing CNL exists, and no published system wires a CNL grammar into LLM constrained
decoding — the gaps this milestone occupies. Direct evidence for the architecture: constrained
decoding into a canonical CNL then deterministically mapping to logic beats decoding formal
syntax directly (Shin et al. 2021, in cnl-landscape.md). Measured priors from the archived
throwaway PoC (`docs/poc-archive.md` — 9-route field, weak local model, real solver): one
constrained hop with the source sentence and full vocabulary in view beat every constrained-hop
stack; a hand-written grammar mask closed the validity→acceptance gap wholesale; and invented
ASCII record DSLs under that mask stably emitted the WRONG deontic direction token while
verdicts stayed well-formed — the failure classes this CNL's design answers. Mined patterns: ACE
interpretation-rules-by-decree + verbalize-the-canonical-form; FRET/BRIDGE-Wiz slotted prose;
PENG single bidirectional grammar; 産業日本語/Miyata JA sentence patterns; PROLEG separate
labeled exceptions; AceWiki/Codeco depth-bounded enumeration testing. PENS target: P5 E3 N4 S4
D (v1 realizes the M1/M2 semantic subset; E3 completes as §11 conflict kinds land).

Committed direction:

- Surfaces: Japanese primary, English mirror — two concrete syntaxes over one AST (ClinicalIR
  statement content). Both parse and both render; the canonical renders of one AST in the two
  languages parse back to the same AST. Japanese is the corpus and clinician language; its
  generation is deterministic by construction (no agreement morphology; 以上/以下/未満/超 are
  lexically exact interval-endpoint markers), and its parse stays tractable because CKC parses
  only its own controlled surface — grammar-constrained emission and canonical rendering both
  land inside the grammar language, so open-Japanese parsing (research-grade: zero anaphora,
  attachment, scope) is designed out rather than solved.
- Sentence model: one rule = one sentence group — recommendation sentence + its basis
  bracket + zero or more exception sentences, each carrying its own basis bracket
  (per-sentence provenance: the bridge reads each exception clause's region_ids off its own
  bracket verbatim, while the rule bracket normalizes to the segment-closed remainder — a
  deterministic evidence cover, not authorial attribution, since the IR keeps no rule-level
  region set; a single rule-global
  bracket would leave multi-exception provenance unreconstructible). Fixed clause order,
  closed connective set, no
  pronouns, no anaphora, no definite references, no ellipsis (overt subject every sentence);
  bare (unescaped) out-of-lexicon or unresolvable text is a parse error, never a guess
  (fail-closed — the anti-ACE lesson; the sole registered exit is the unregistered-concept
  escape below, which parses and fails at acceptance). Multiword concepts are single
  lexicon terminals, never parsed compounds.

| Slot | JA canonical shape | EN canonical shape | AST target |
| --- | --- | --- | --- |
| context | `<dnf>患者には、` | `for patients <dnf>,` | population/condition DNF |
| action | `<target>の<action-noun>` (例 `抗菌薬Aの投与`) | `<action-noun> of <target>` | Action kind + target |
| deontic tail | `を強く推奨する` / `を提案する` / `を推奨しない` / `は禁忌である` … | `is strongly recommended` / `is suggested` / `is not recommended` / `is contraindicated` … | (direction, strength) via the §5 lexicon modality table |
| certainty | `(エビデンスの確実性:中)`, optional | `(certainty: moderate)`, optional | certainty |
| exception | `ただし、<exception-atom>患者を除く。[根拠 <id> …]` per entry | `exception: patients <exception-atom>. [basis <id> …]` per entry | one single-concept ExceptionClause per entry per split statement (disjunct splits clone entries under fresh ids, each clone keeping its sentence's basis) — separate labeled payload (PROLEG pattern); the entry's own bracket = the clause's region_ids; `<exception-atom>` = positive registered concept or escape incl. its composition glue (EN `with`, escape's patient-adjacent の) |
| basis | `[根拠 <id> …]` after each sentence, sorted per bracket, ≥1 ref | `[basis <id> …]` after each sentence, sorted per bracket, ≥1 ref | rule bracket = the rule sentence's region refs (normal form: the segment-closed remainder); exception brackets = per-clause region refs; statement source segments derive from their union |

- DNF prose: conjuncts join with `かつ`/`and`; disjunct groups join with `、または`/`; or`;
  precedence by decree (`かつ` binds tighter), no nesting beyond flat two-level DNF — each
  disjunct maps to one ClinicalStatement's flat population/condition sets (multi-disjunct
  rules split into statements, the bridge law; DNF `ContextExpr` is the §5 norm layer's;
  the bridge partitions a disjunct's atoms into those sets by the lexicon slot-role view —
  a concept atom by its row's context role, an interval atom by its quantity row's context
  role, never by id-namespace convention — the typed-slot-roles paragraph below).
  Atoms: concept (lexicon adnominal surface), negated concept
  (lexicon negated-adnominal surface), quantity interval (`<var-surface>が<n><unit><bound>`,
  `<n>` an ASCII-digit leading-zero-free canonical decimal — token-inventory bullet), and
  the unregistered-concept escape (own bullet below). Punctuation: `、` `。`
  plus ASCII brackets/parens — width-folding ambiguity is
  kept out of the surface by construction.
- Surface composition (concrete-syntax decree; the committed grammars pin the bytes, this
  bullet pins shapes and wording): composition is plain concatenation of whole-surface
  terminals — fluency is a lexicon-authoring concern, never a grammar property (the ACE
  lesson: composition rules by decree). JA: adnominal and negated-adnominal surfaces
  compose directly in every position — mid-chain (before かつ/、または) and
  patient-adjacent (the last atom of the last disjunct, before 患者には、; the exception
  concept slot, before 患者を除く。) — while interval and escape atoms take the fixed
  linking terminal の exactly patient-adjacent (年齢が18歳未満の患者には、 /
  ただし、未登録概念「…」の患者を除く。) and compose bare elsewhere
  (成人かつ年齢が18歳以上、または…): the JA grammar (cnl-grammar.1) carries mid vs
  patient-adjacent atom alternations (two nonterminals), so a stray or missing の is a
  parse error like any other byte. Interval bound markers: JA `以上`/`以下`/`未満`/`超` ↔ ge/le/lt/gt after the
  unit (`18歳以上`); EN `at least`/`at most`/`less than`/`more than` ↔ ge/le/lt/gt before
  the numeral (`at least 18 years`); units render invariant (`years`), so `at least 1
  years` is accepted mirror stiltedness — numeral agreement deliberately unhandled,
  one-form determinism over fluency. EN atoms are position-invariant under one
  prepositional frame — a positive concept atom and the exception concept slot render
  `with <gloss_en>`, a negated atom `without <gloss_en>` (the fixed negator replaces
  `with`, never stacks), an interval atom `with <quantity-surface> <bound> <n> <unit>`,
  the escape atom `with unregistered concept "<payload>"`; the action target stays the
  bare `gloss_en` (or bare escape) after `of` — one gloss serves every position because
  the frame glue lives in fixed terminals, which is what keeps EN negation field-less.
  `gloss_en` authoring contract: a lowercase-ASCII article-free noun phrase naming the
  condition/entity (`sepsis`, `severe renal impairment`, `adult status`, `antibiotic-a`)
  that reads after `with`/`without`/`of`; the reserved-token and prefix-overlap lints
  (lexicon-cnl.2) bar glosses containing connective/punctuation terminals (hyphenate:
  `head-and-neck injury`) or prefixing another lexer-visible token, and the EN-value
  shape lint (same unit) enforces the normalized form — ASCII-only lowercase
  word/digit/hyphen tokens, single-space-separated, `:` admitted for certainty labels,
  leading article (a/an/the) rejected on `gloss_en` — because SemanticEn lowercases ASCII
  yet never rejects non-ASCII, the shape needs its own lint. The adnominal contract: a
  prenominal form reading directly before 患者 (成人, 敗血症のある); mid-chain
  composition (before かつ/、または) is decree-uniform concatenation — unambiguous in
  every atom order, and clause-form adnominals read stilted there (敗血症のあるかつ…, the
  guideline_b golden shape), an accepted controlled-language cost: fluency stays
  unclaimed off the patient-adjacent slot, and cnl-laws permutes conjunct atom order.
  Interval-carrying context concepts author negated forms too — the tokens must parse for
  acceptance to reject their use repairably with the complement-interval repair (the
  negative-occurrence bar is acceptance-enforced, never a token-table gap). The
  negated-adnominal contract mirrors the adnominal one (a prenominal negation reading
  directly before 患者), under one authoring law the prefix lint makes structural: no
  lexer-visible surface may extend another as a proper prefix, so suffix negations of
  bare-noun positives (成人 → 成人でない) are excluded by construction — bare-noun
  positives negate by 非-prefix (非成人, 非小児), verb-form adnominals flip the verb
  (敗血症のある → 敗血症のない), copula-state forms flip the copula (妊娠中である →
  妊娠中でない); every authored positive/negated pair diverges before either string
  ends. Adnominals also avoid a trailing の (妊娠中である, never 妊娠中の): a
  surface-final の reads as the fixed linking terminal — visually ambiguous
  patient-adjacent and dangling mid-chain (妊娠中のかつ…); the lexicon lint enforces this
  as its own rule (surface-final の on adnominal/negated forms = finding), since neither
  the prefix rule nor the reserved-token class catches it. Certainty
  parenthetical = `(` + the certainty row's surface + `)` — the committed JA surfaces
  already carry the label (`エビデンスの確実性:中`) and `surface_en` mirrors that form
  (`certainty: moderate`) — placed between the deontic tail and the sentence terminator in
  both languages. Spacing: JA composition inserts no separator bytes outside bracket
  internals — 。 abuts its basis bracket and the next sentence (surface- and
  payload-internal spaces stay data; SemanticJa and the escape contract admit internal
  normalized spaces); EN inter-terminal separators are exactly one space, owned by the
  fixed terminals with exactly two composed lexicon-slot exceptions below — frame and
  connective terminals carry their delimiting spaces
  (`with `, `without `, ` and `, `; or `, ` of `, ` [basis `; exact inventory pinned in the
  token-inventory bullet below, committed grammar bytes the final authority), which is also
  what keeps the fixed inventory prefix-free under its own
  lint (`with` vs `without` collide only space-less; the prefix rule runs over fixed
  terminals too); two EN slots have no adjacent fixed terminal to supply their leading
  separator — the deontic tail (after the bare target gloss or the escape's closing
  quote) and the unit (after the numeral) — and carry it themselves by emitter
  composition: the emitted terminal is the space plus the surface
  (` is strongly recommended`, ` years`); the raw lexicon field carries no edge spaces
  (the EN shape lint's token form enforces the edges; internal single spaces stay data),
  and the token inventory, prefix lint, and
  parser tables all carry the composed forms — while surface- and payload-internal
  spaces stay data (`adult status`);
  bracket internals = `[根拠 `/`[basis ` + space-separated sorted ids + `]`, both
  languages.
- Token inventory (the shared terminal layer; lexicon-cnl.2's typed inventory module
  transcribes this bullet verbatim — lint, grammar emitter, and parser all consume that one
  module, and the committed grammar bytes remain the final authority). Main-mode fixed
  terminals, exact bytes including each side's owned space:

| Role | JA | EN |
| --- | --- | --- |
| context opener | — (a rule starts at its first atom) | `for patients ` |
| atom frame (positive concept, exception concept, interval, escape) | — (bare concatenation; の rows below) | `with ` |
| negated atom frame | — (lexical `negated_ja`) | `without ` (replaces `with `) |
| conjunction | `かつ` | ` and ` |
| disjunction | `、または` | `; or ` |
| context close | `患者には、` | `, ` |
| patient-adjacent link (interval/escape atoms only) | `の` | — (the frame is position-invariant) |
| action link (JA target-first, EN noun-first) | `の` (same token, second grammar position) | ` of ` |
| bound markers ge / le / lt / gt | `以上` / `以下` / `未満` / `超` (after the unit) | ` at least ` / ` at most ` / ` less than ` / ` more than ` (before the numeral; leading + trailing spaces owned) |
| certainty parens | `(` `)` (ASCII, abutting) | ` (` `)` (space-led open) |
| sentence terminator | `。` | `.` |
| basis open | `[根拠 ` | ` [basis ` |
| basis close | `]` | `]` |
| exception opener | `ただし、` | ` exception: patients ` |
| exception close | `患者を除く。` | `.` (the terminator token) |
| escape open | `未登録概念「` | `unregistered concept "` |
| escape close | `」` | `"` |
| rule terminator | LF | LF |

  Digits `0`–`9` are single-char tokens in both languages under the leading-zero-free
  numeral register: a numeral is `0` or a nonzero digit followed by any digits — canonical
  decimal, so zero-led runs (`018`) sit outside BOTH languages (grammar production and
  parser alike), the numeral's sole grammar-over-parser divergence stays the parser's
  0..=i64::MAX value bound, no third over-approximation class arises, and no undeclared
  parse-normalization variation joins the declared set (modality synonyms, basis-ref
  order); render writes the value's decimal form, trivially fixpoint. Mode-scoped content
  stays outside the main token table and the collision/prefix domain: the escape payload is
  a free scan to the closing delimiter under the payload contract, and basis-bracket
  internals are Id-grammar refs separated by single ASCII spaces — the bracket's own
  separator, so the main tables keep no bare-space token. Lexicon-token categories join the
  main mode from the typed role view — Concept (adnominal/negated/citation), ActionNoun,
  Tail, Certainty, QuantityVar, Unit — and the VIEW serves the EN tail and unit forms
  space-led-composed (` is strongly recommended`, ` years`): one composition point, with
  emitter, lint, and parser consuming view output verbatim, never re-composing. Prefix
  audit over this inventory (re-verified mechanically at impl with the lexicon tokens):
  every near-pair diverges before either string ends — `以上`/`以下` (2nd scalar),
  `未満`/`未登録概念「` (2nd), `患者には、`/`患者を除く。` (3rd), ` and `/` at least `
  (3rd), ` at least `/` at most ` (5th), `with `/`without ` (5th) — and no committed
  lexicon surface starts with a fixed terminal or a digit (the lint owns that thereafter).
  Schematic grammar skeleton — factoring and slot order normative, bytes the committed
  grammar's; `⟨…⟩` = lexicon alternations served by the role view, quoted strings = the
  inventory above:

```text
JA:
  <document>     ::= <rule> <nl> | <rule> <nl> <document>
  <rule>         ::= <disjuncts> "患者には、" <target> "の" ⟨action-noun⟩ ⟨tail⟩
                     <cert-opt> "。" <basis> <exceptions>
  <disjuncts>    ::= <conjunct-mid> "、または" <disjuncts> | <conjunct-adj>
  <conjunct-mid> ::= <atom-mid> "かつ" <conjunct-mid> | <atom-mid>
  <conjunct-adj> ::= <atom-mid> "かつ" <conjunct-adj> | <atom-adj>
  <atom-mid>     ::= ⟨adnominal⟩ | ⟨negated⟩ | <interval> | <escape>
  <atom-adj>     ::= ⟨adnominal⟩ | ⟨negated⟩ | <interval> "の" | <escape> "の"
  <interval>     ::= ⟨var-surface⟩ "が" <numeral> ⟨unit⟩ ("以上"|"以下"|"未満"|"超")
  <numeral>      ::= "0" | <nonzero> <digit-rest>
  <digit-rest>   ::= "" | <digit> <digit-rest>
  <target>       ::= ⟨citation⟩ | <escape>
  <escape>       ::= "未登録概念「" <payload> "」"        ; payload = the open production
  <cert-opt>     ::= "" | "(" ⟨certainty⟩ ")"
  <basis>        ::= "[根拠 " <id-list> "]"
  <id-list>      ::= <basis-id> | <basis-id> " " <id-list>
  <exceptions>   ::= "" | <exception> <exceptions>
  <exception>    ::= "ただし、" <exc-atom> "患者を除く。" <basis>
  <exc-atom>     ::= ⟨adnominal⟩ | <escape> "の"
EN mirrors the frame with its own terminals and one position-invariant atom nonterminal:
  <rule>         ::= "for patients " <disjuncts> ", " ⟨action-noun⟩ " of " <target> ⟨tail⟩
                     <cert-opt> "." <basis> <exceptions>
  <atom>         ::= "with " ⟨gloss⟩ | "without " ⟨gloss⟩ | "with " <interval> | "with " <escape>
  <interval>     ::= ⟨var-surface⟩ (" at least "|" at most "|" less than "|" more than ")
                     <numeral> ⟨unit⟩
  <exception>    ::= " exception: patients " <exc-atom> "." <basis>
  <exc-atom>     ::= "with " ⟨gloss⟩ | "with " <escape>
```

- Exception register (v1, deliberately narrower than the context DNF): each exception sentence
  carries exactly one concept slot — a positive registered concept (adnominal surface) or the
  escape — no connectives, no negated-concept atoms, no quantity intervals. Multiple exception
  sentences per rule remain available and read disjunctively (any listed exception exempts).
  Soundness anchor: §5 compiles exceptions to negated context conjuncts inside the rule's one
  conjunction, negating exactly the clause's positive concept atoms — sound because
  ¬(E1 ∨ … ∨ En) = ¬E1 ∧ … ∧ ¬En when every Ei is a single atom, while a conjunctive
  exception `A ∧ B` would need the De Morgan disjunction ¬A ∨ ¬B the locked tail never
  builds, and negated/interval exception atoms sit outside its negation domain entirely.
  Negative-occurrence bar (same v1 register): interval-carrying lexicon entries (成人, 小児)
  are excluded from the exception slot AND from context negated-concept atoms at acceptance
  (repairable; the repair is the complement context interval — 「ただし、成人患者を除く。」 →
  context 年齢が18歳未満): the locked tail interval-lowers positive Concept occurrences only,
  every negative occurrence staying a bare Bool literal with no axiom linking it to the Real
  interval variable, so a rule excluding 成人 would overlap-check as an unlinked Bool against
  another rule's 年齢が18歳以上 — spurious overlaps, missed disjointness. A
  conjunctive, negated, or interval exemption is authored as context refinement instead (the
  context DNF already admits negated concepts and intervals) — noting the provenance trade:
  the locked tail joins exception regions into rule source_region_ids only through
  ExceptionClause region_ids, so a context-authored exemption keeps statement-level segment
  linkage but adds no rule-level regions, and transcribing a SOURCE exception segment
  therefore uses the exception sentence; widening the exception register
  is an explicit lowering change (M4+ candidate), never a silent grammar widening over the
  unchanged compile tail.
- Ids: the parser and the model mint no ids — the bridge derives statement/exception/binding
  ids deterministically from document order as `stmt.<k>`/`exc.<k>`/`bind.<k>` document-local
  counters, mirroring the deterministic M1 derivation exactly (§8.6 reserves
  `<document_id>.rule.<k>` for norm-layer rule ids) when
  mapping AST → ClinicalIR; a multi-disjunct rule splits into one statement per disjunct,
  each cloning every exception entry under a fresh id ((D1 ∨ D2) ∧ ¬E =
  (D1 ∧ ¬E) ∨ (D2 ∧ ¬E), and exception ids are bundle-unique) — `exc.<k>` counts emitted
  clauses statement-major then sentence order (worked 2 × 2 — two context disjuncts × two
  exception sentences: stmt.0 owns exc.0, exc.1 and stmt.1 owns exc.2, exc.3, sentence
  order within each statement, clone content + provenance duplicated per statement),
  `bind.<k>` at first reference in the same
  post-split emission order (statement-major; within a statement population atoms, then
  condition atoms, then the action target, then exception clauses, each in emitted order — a
  concept exclusive to a later disjunct mints after the earlier disjunct's atoms), clause region_ids = its own sentence's basis
  refs verbatim (per-sentence brackets; clones share their sentence's basis), and statement
  source_segment_ids derive region→segment (the segments artifact) over the union of the
  rule's and its exceptions' basis refs — bridge preconditions, acceptance-enforced: every
  cited region anchored in exactly one segment, the derived segments' region sets unshared
  (closure-functional), and basis ownership KIND-aware over the derived segments, PER
  STATEMENT (each statement's own citations + clauses — coverage never pools across
  statements) — writing
  R = the cited Recommendation segments' region union, E = the cited Exception segments',
  X = the exception brackets'/clauses' union: every cited segment's kind Recommendation or
  Exception, R nonempty, and the two in-closure ownership primitives — X ∩ R == ∅ (no
  exception bracket cites a Recommendation-owned region) and E ⊆ X (no cited Exception
  segment carries a region absent from every exception bracket). With containment
  (X ⊆ closure, the pre-existing class) the primitives derive X == E, and
  closure-functionality (giving R ∩ E == ∅) then forces full closure − X == R, so the
  normalized rule bracket is exactly the Recommendation-kind closure. The ownership laws
  exist because the locked compile tail is kind-sensitive — norm-rule provenance = cited
  Recommendation segments' full region sets, then each clause's region_ids in clause
  order — so kind-blind citation would silently corrupt provenance: a non-normative cited
  segment's clause-uncited regions (Evidence/Cq/Definition/table-row/Metadata — clause
  appends are kind-blind, so clause-cited ones still land) vanish from it, a
  Recommendation-owned exception region lands at least twice (recommendation walk + each
  citing clause), and a clause-uncovered Exception-segment region is dropped from the rule
  bracket the normal form would widen it into. Exception containment holds by
  construction on this side — the closure derives from the
  bracket union — and is the predicate's closure-containment class on the IR side, where
  authored source_segment_ids can uncite an exception's segment. Binding region_ids = the
  union over the citing emitted statements of each
  statement's segment closure (its source segments' full region sets — the closure the
  statement's rendered brackets jointly cover: the rule bracket carries it minus the
  exception-owned regions — under the ownership laws exactly the Recommendation-kind
  closure, the split the locked compile tail consumes — exception sentences the rest),
  never the authored brackets —
  statement-grain provenance, coarser than M1's §5 mention grounding (the §5 field is
  producer-graded): the closure is invariant under bracket normalization, keeping
  `to_ir(from_ir(ir)) == ir` exact — a bracket-union binding breaks the law whenever the
  citing statements' brackets jointly under-cover the closure union (minimal case: an
  exception-free rule citing one region of a two-region Recommendation segment re-bridges
  wider). The bridge also derives the normative-rule origin map —
  `<document_id>.rule.<k>` → originating CNL rule index, a pure function of the document
  (rule k = the k-th post-split statement, mirroring the §8.6 derivation-order mint), so a
  multi-disjunct rule originates several rule ids that legitimately share its text —
  non-core, consumed by §7.2's per-rule CNL text. Basis refs are the only
  generated references, grounded by the §9 scaffold (`ai_hallucinated_source` on a miss). This
  removes the §9 generated-Id instability class from the emission surface.
- Grammar and lexicon: `schemas/clinical_cnl_ja.grammar` + `schemas/clinical_cnl_en.grammar`
  (id forms follow the `ClinicalIR ↔ clinical_ir` precedent; singular registry schema id
  `schema.clinical_cnl`), emitter-backed from
  the lexicon (bless + drift guard + hash pin — the M2 `schemas/` pattern); the
  `registry/schemas.yaml` entry binds the JA grammar — the route's decoding constraint,
  consumed exactly as §9 consumes the IR schema — while the EN grammar stays committed,
  drift-guarded, and hash-pinned with no route binding (its own non-route entry only if the
  coverage check demands one — the route's singular id stays JA-bound). Every linguistic
  form lives in lexicon DATA and the grammar stays purely concatenative: concept entries
  gain adnominal / negated-adnominal / EN-gloss surfaces plus a validated slot-role set
  (typed slot roles, below); action
  kinds gain JA/EN noun forms (`noun_ja`/`noun_en` — a required nonempty pair on every
  action row: integrity, never luck, keeps the `<target>の<noun_ja>` / `<noun_en> of
  <target>` render surface total); the modality table gains canonical deontic-tail fields
  (`tail_ja`/`tail_en` — CNL tails are grammatical phrases distinct from the §8 source-match
  surfaces: 「を強く推奨する」 carries particle + strength adverb; rows carrying tails parse
  as tail synonyms, the first tail-bearing row per `(direction, strength)` pair is the
  canonical render row — pinned against this section's worked renders — and tail-less rows
  stay source-match-only); certainty phrases as committed, every row gaining a required
  nonempty EN surface (`surface_en`; the first row per value stays the canonical render
  row). Concept citation forms mint no new field: an action target renders as the concept
  row's representative surface (`surfaces[0]`, JA) and its EN gloss (`gloss_en` — one EN
  form serves context and citation); synonym surfaces (`surfaces[1..]`) stay §8
  source-match vocabulary, never CNL terminals. EN negation mints no field either: the EN
  negated atom is a fixed-negator composition over the same `gloss_en` (the negator is the
  fixed inventory terminal `without`, replacing the positive frame's `with` — the surface
  composition decree above; the emitter stays the byte authority), while JA negation stays
  lexical (`negated_ja` — morphological, not composable).
  Typed slot roles (the CNL slot-legality classification — lexicon data like every other
  linguistic form): every concept row carries a nonempty validated role set over
  `population` / `condition` / `action_target` — `population` and `condition` mutually
  exclusive per row (the bridge partition stays a function of the concept; a dual-use
  classifier is a lexicon-review decision, never emission-time freedom), `action_target`
  free to combine with either (multi-role is deliberate — e.g. a drug concept as action
  target and, separately authored, as an on-drug condition atom) — and every quantity row
  carries exactly one context role (`population` | `condition`) placing its interval atoms
  under the bridge partition. The loader exposes ONE typed role view that every CNL
  consumer reads — grammar emitter, AST validation, parser slot legality, bridge partition,
  acceptance wrong-slot checks — so no CNL module tests id prefixes: the M1 id namespaces
  (`pop.`/`cond.`/`drug.`) demote to naming convention, pinned by a lexicon data test —
  `pop.*` → population, `cond.*` → condition, `drug.abx_a` → action_target, `q.age_years`
  → population — so the frozen M1 normalize partition and the role-driven bridge agree
  over the locked corpus, and a future concept namespace never silently falls through to
  `condition`. Roles live in lexicon data and the CNL layer only: the committed ClinicalIR
  schema's enums stay role-agnostic and byte-frozen — slot legality belongs to the bridge
  and the acceptance closures, and a per-slot schema re-derivation would re-bless
  committed §9-pinned schema bytes. The
  grammar emitter derives slot-specific terminal alternations from the view — context
  concept and negated-concept atoms enumerate context-role (population|condition)
  surfaces, the action-target slot enumerates `action_target`-role surfaces,
  exception-sentence concepts enumerate context-role surfaces — so wrong-slot vocabulary
  is unparseable, not merely invalid; the escape stays admitted in every concept slot (it
  names no lexicon row and carries no role).
  Modality totality: the lexicon pair set stays the corpus register — no artificial rows for
  the full §5 Direction × Strength domain; instead every IR-landing route guarantees
  lexicon-backed pairs at acceptance (M1 derives pairs from lexicon rows by construction;
  single_cnl's grammar admits only lexicon deontic tails; single_ir acceptance rejects every
  CNL-inexpressible shape as a repairable schema violation naming the offense — one per
  predicate class below — mirroring the off-lexicon id check). CNL expressibility is ONE
  executable predicate, never a hand-maintained rejection list per consumer:
  `check_cnl_expressible(clinical, lexicon (role + tail view), segments (segment_id →
  (kind, region_ids) map — id uniqueness by construction; the basis-ownership classes
  read the kind)) -> Result<(),
  CnlExpressibilityError>`, home the bridge module (it shares the segment-closure
  computation `from_ir`'s rule bracket takes its exception-owned remainder from and
  `to_ir`'s binding region_ids consume whole), defined over grounded, lexicon-valid
  ClinicalIR (vocabulary membership + grounding are the acceptance stages ahead of it);
  its error taxonomy carries one variant per CNL-inexpressible class — a
  `(direction, strength)` pair without a tail-bearing lexicon row, empty statement sets,
  statements with empty population+condition, quantity intervals without exactly one
  unsigned bound (signed / two-sided / boundless / same-side ge+gt or le+lt doubles —
  only `var` is schema-required over four independent optional bound fields; §5 bundle
  coherence catches boundless + doubled shapes only terminally at bundle time, the
  predicate rejects all four repairably at acceptance), exception clauses that are not
  exactly one positive concept atom (structural class — multi-atom, atomless,
  negated-concept, or quantity-interval shapes), negative occurrences of
  interval-carrying entries — context negated-concept atoms or the sole exception
  concept of a structurally valid clause (disjoint from the structural class),
  exception clauses with empty region_ids, statements citing no Recommendation segment
  (R empty ⇔ the wholly-exception-owned empty rule bracket under the ownership laws,
  closure − X == R; covers empty source_segment_ids), statements citing a segment of
  non-normative kind (neither Recommendation nor Exception — the compile tail's
  provenance walk would silently drop its clause-uncited regions), exception regions
  Recommendation-owned (X ∩ R ≠ ∅ — a clause citing a region a cited Recommendation segment owns —
  the tail would emit it at least twice), cited Exception segments not clause-covered (a region of
  a cited Exception segment absent from every clause — the E ⊄ X direction, a region
  the normal form would widen into a rule bracket the tail never reads Exception segments
  into; the clause set scoped per statement — statements sharing an Exception segment
  each cover it with their OWN clauses, pooled coverage passes only a document-global
  checker), statements with an exception region outside
  their segment closure (a clause citing a grounded region of an UNCITED segment —
  membership, grounding, kind, R-nonemptiness, and both ownership directions all pass
  (the region's segment is uncited, so neither sees it — containment stays non-redundant
  under the ownership laws), yet from_ir would render
  the region only on its exception sentence and the re-bridge would derive a wider
  segment set — a provenance-unfaithful render; bridge-image IR contains by construction,
  so the identity law never holds the shape and the predicate rejects it at
  acceptance/from_ir; jointly with the R-empty class this enforces exception-owned as
  a proper subset of the closure — the pair co-occurs on an exception union blanketing the
  closure with uncited excess, R-empty preceding containment as the predicate's
  single reported variant, the blanketing fixture citing Exception-kind segments only — a
  cited Recommendation segment would keep R nonempty yet still report containment while
  the uncited excess remains (containment precedes the ownership pair);
  Recommendation-owned surfaces only once the blanket is trimmed to the closure; the
  pinned first-failing-check order over the topology classes runs
  closure-functionality, non-normative kind, R-empty, containment, Recommendation-owned,
  clause-uncovered — containment ahead of the two ownership-mismatch directions so
  out-of-closure excess names containment, and the ownership pair is checked over the
  in-closure residue, disjoint from containment by construction), statements whose cited segments carry
  no region or share a region with another segment (closure-nonfunctional — breaks segment
  recovery from region-level basis; the empty-region segment is synthetic-only — the
  segmenter mints only from grounded spans and bundle validation rejects empty segment
  support — while the shared region is bundle-valid, no cross-segment disjointness check;
  the predicate owns both fail-closed over its raw view), and wrong-slot vocabulary — population atoms whose concept
  or quantity role is not `population`, condition atoms not `condition`-role, action
  targets not `action_target`-role, exception concepts outside the context roles
  (lexicon-MEMBER ids in slots no role admits pass the membership check yet sit outside
  the bridge image — accepted IR stays partition-normal), the v1 register.
  `single_ir_accept` and `from_ir` both call the one predicate — the acceptance and
  renderer domains sit one function apart — definitional drift structurally excluded,
  behavioral agreement law-tested below — so audit rendering is defined over guarded-route
  accepted IR + locked-corpus M1 (off-corpus M1: the §7.2 typed omission)
  and a missing-row render error is a fail-closed instrument path, barred from accepted
  artifacts by lexicon integrity (pair coverage, render-surface totality), the zero-finding
  view gate (the §10 typed role view refuses construction on any lint finding — every CNL
  module consumes the view, so lint-owned role-scoped surfaces gate like hard errors), and
  the predicate. Lexicon
  integrity checks: reserved-token collisions (a surface containing a connective/punctuation
  terminal or a backtick — §7.2 validation renders rule text inside Markdown code spans, so
  every surface stays code-span-inert; escape payloads never reach report surfaces — the
  escape is terminal at acceptance and accepted/report-rendered rule text is registered
  vocabulary only; pre-accept escapes render + round-trip off the report surface),
  missing surface fields (role-scoped: a context-role concept needs its
  adnominal / negated-adnominal / EN-gloss forms, an `action_target`-role concept its
  target citation forms — `surfaces[0]`, already row-required, plus `gloss_en`),
  render-surface totality (every action row carries the nonempty `noun_ja`/`noun_en` pair,
  every certainty row a nonempty `surface_en`: a membership-valid action kind or certainty
  value never reaches `from_ir` without its bilingual render surface), per-language
  duplicate-literal rejection by semantic token over the lexer-visible surfaces (concept
  adnominal/negated forms and `action_target`-role citation forms, action nouns, tails,
  certainty phrases, quantity surfaces/units): a literal is a hard error exactly when its
  occurrences denote two distinct tokens — Concept(row) (one row's citation and adnominal
  forms collapse; which slot admits which form is grammar/parser business),
  NegatedConcept(row), ActionNoun(row), Tail(direction, strength), Certainty(value),
  QuantityVar(var), Unit(literal) (deliberately var-free: the per-var interval production
  pairs each var's surface with its own row's unit terminal, so rows sharing a unit
  literal stay unambiguous), plus Fixed(terminal) and Digit(char) as their own categories
  (a lexicon literal equal to a fixed terminal or a digit token is a cross-category hard
  error — equality escapes the proper-prefix rule; escape payloads and basis-bracket id
  content stay delimiter-scoped at the lexer, outside the collision domain, though the
  grammar spells them as open/char-alternation productions) — same-token occurrences deduplicate into one token-table
  entry (a multi-role concept's surface parses in every slot its roles admit; same-pair
  tail synonyms and shared units collapse; cross-row, cross-value, and cross-category
  duplicates reject), per-language proper-prefix overlaps
  rejected across the finite longest-match token inventory — that deduplicated token table
  plus the fixed grammar terminals and digit tokens, same- and cross-category; escape payload =
  delimiter-scanned free content, outside the inventory (a prefix-free inventory keeps
  maximal-munch tokenization agreeing with the grammar's intended segmentation — the failure
  mode is a longer token stolen from another category), `implies_action`
  resolving to an action entry, `tail_ja`/`tail_en` present together or absent together (a
  row is tail-bearing iff both — per-language tail-bearing would let canonical-row selection
  diverge between languages or leave one language's pair coverage partial), every
  `(direction, strength)` pair present carrying ≥1
  tail-bearing row, certainty-table render totality — every §5 `Certainty` value carries a
  row, first row per value = canonical render row (a closed 4-value enum; a gap would leave
  `from_ir`'s certainty parenthetical surface-less on in-domain IR; the committed table is
  already total), concept intervals CNL-representable (v1: one unsigned bound), slot-role
  integrity — every concept row a nonempty deduped set of known roles with
  `population`/`condition` mutually exclusive, every quantity row exactly one context role
  agreeing with the context role of each interval-carrying concept using its var — and
  quantity-table integrity — unique `var_id`, the quantity var set EQUAL to the set of
  interval variables concepts use (exactly one row per used var, zero orphan rows: an
  orphan quantity row would emit grammar-parseable interval vocabulary outside the
  committed schema's concept-derived interval-var enum and `off_lexicon_ids`' universe —
  parseable yet unacceptable by construction), nonempty normalized surfaces and units in
  both languages.
- Unregistered-concept escape (off-lexicon posture): wherever the grammar demands a lexicon
  concept surface (context atom, exception concept, action target; action kinds stay a
  small closed class — extending them is lexicon review, not emission), one escape
  production is admitted — JA
  `未登録概念「<surface>」`, EN `unregistered concept "<surface>"`, free quoted surface — so
  constrained decoding is never forced to alias an off-lexicon source concept to the nearest
  registered terminal (silent substitution — the exact failure class the fail-closed design
  exists to prevent, and the one a fully closed grammar would otherwise manufacture).
  Quoted-surface contract (v1, one payload shared by both languages): nonempty, at most 80
  Unicode scalar values, single line — control characters and the quote delimiters `「` `」`
  `"` are excluded, and there is no escape mechanism (out-of-contract text is a plain parse
  error, never an alias) — normalized under the §4.2 JA semantic policy, so canonical fixpoint
  and cross-language agreement hold; the parser enforces the contract while the grammar keeps
  one open production per language (notation decided at the grammar emitter). The
  escape parses and round-trips like any atom and always fails accept with
  `cnl_unregistered_concept`, terminal for the run: repair prompts never mint or steer concept
  identity — resolving a gap is a lexicon-review decision, not a retry. Its payload (quoted
  surface + atom position) is exactly the lexicon-entry proposal artifact: over the locked M1
  inputs any occurrence is instrument signal (vocabulary covered by construction); from M4 the
  §11 accretion loop consumes it — propose → lint → review → accepted entries join the
  lexicon, grammar re-emitted under the drift guard — so vocabulary growth stays amortized,
  ledgered data accretion inside the single probabilistic boundary, never a precomputed
  corpus-wide lexicon.
- Canonical text: render emits exactly one text per AST per language (stored ContextExpr order
  preserved — §4.3 ordered arrays; canonicalization never reorders semantics). Document bytes
  are pinned past rule cardinality: each rule renders as one line terminated by exactly one
  LF — the uniform rule terminator, the last rule included, no other inter-rule bytes — so
  the grammars carry document = (rule <nl>)+ (the smt_query.grammar literal-LF `<nl>`
  convention; the bnf dialect lacks postfix repetition, so the committed form is the
  right-recursive lowering — smt_query.grammar's <assertions> pattern), document text = the
  stored per-rule texts each plus one LF, and the per-language text hashes and the audit
  `.txt` views cover exactly those assembled bytes — an executable invariant: validate
  recomputes the hashes from the stored texts under the frame, and the audit writer
  re-hashes its read-back against them. Line breaks (LF and CR both) cannot occur inside a
  rule: lexicon surfaces are whitespace-folded (§4.2), fixed terminals carry none, and the
  escape payload contract bars control characters. Parse accepts declared bounded variation
  inside a rule (modality synonyms, basis-ref order — whitespace variation: none; the
  parser's language equals the grammar's, keeping differential parser-vs-oracle agreement
  total, and stray whitespace is a repairable parse error); the document frame is exact — a
  missing terminal LF is a repairable parse error; accept re-renders, so every accepted
  `CnlDocument` is canonical bytes, hash-locked beside its AST content hash.
- Determinism laws (the M3 contract, property-tested):

```text
Single parse: every parser-accepted string yields exactly one AST; the runtime grammar
over-approximates the parser in exactly two classes — the open escape production (payload
contract parser-enforced) and unbounded leading-zero-free numerals (value bound 0..=i64::MAX
parser-enforced; zero-led digit runs sit outside BOTH languages — never a third class) —
grammar-emitted, parser-rejected strings are repairable parse errors, and these laws quantify
over the parser-accepted language.
Round trip: parse(render(ast)) == ast for every valid AST, both languages — equality over
the semantic AST (document-frame members — per-rule texts, text hashes — are derived
canonical bytes, recomputed identically on both sides). AST validity is
two-layered, structural first (lexicon-free, the grammar-image shape up to parse
normalization): nonempty rules and
basis brackets — every bracket sorted + deduplicated (set semantics: parse normalizes the
admitted basis-order variation, from_ir emits sorted) — nonempty context DNF — the outer
disjunction AND every conjunction (the
grammar writes neither empty) — interval atoms carrying exactly one unsigned bound
(one of ge|gt|le|lt with a nonnegative value: the v1 register, deliberately narrower than
§5 interval coherence, which admits the signed and two-sided shapes the grammar has no
surface for) — and escape payloads in contract (nonempty ≤80 scalars, single line,
control/quote-delimiter chars excluded, SemanticJa-normal); then lexicon-scoped: modality
pairs tail-backed, concept/action refs resolved, interval vars resolving to quantity rows,
slot roles admitting every atom and target position, negated/exception concept refs
interval-free — the negative-occurrence bar, the sole lexicon-scoped clause the
lexicon-projected token tables leave open; acceptance enforces it post-parse. A
structurally invalid AST
has no rendering, so render asserts the structural SHAPE sublayer fail-closed (frame
members are render's own output, validated on stored documents).
Canonical fixpoint: render(parse(t)) == t exactly when t is canonical.
Cross-language agreement: parse_en(render_en(ast)) == parse_ja(render_ja(ast)) == ast,
over the same valid-AST domain.
Bridge round trip (over ACCEPTED escape-free ASTs — single_cnl_accept's closure supplies the
bridge preconditions: cited regions anchored, closure-functional segments, normative-kind
cited segments with a nonempty Recommendation closure, exception brackets owning exactly
the Exception-kind closure — exception containment by construction, source segments derive
from the bracket union; to_ir is Err on any escape occurrence — acceptance
is already terminal there): from_ir(to_ir(ast)) == the bridge normal form of ast — disjunct
split, per-statement atom canonicalization (population before condition, §4.3 set order,
byte-identical duplicates collapsed; the partition + set emission are lossy exactly there),
basis refs segment-closed and exception-owned-split (a labeled cover, not a partition —
clauses may share a region; from_ir renders each clause's own
region_ids verbatim on its exception sentence and the segment-closed remainder — every cited
segment's full region set minus the exception-owned regions — on the rule bracket; from_ir's
sole Err source is `check_cnl_expressible` at entry — the shared taxonomy: empty clause
region sets, no cited Recommendation segment (the empty rule bracket), non-normative cited
segment kinds, exception-ownership mismatches (a Recommendation-owned clause region, a
clause-uncovered Exception segment), an exception region outside the statement closure (the
re-bridge would derive wider), atom / action-target / interval placement contradicting
the §10 role view (wrong-slot IR is
CNL-inexpressible, any rendering re-parses into a different partition), and the remaining
classes — past a passing check the projection constructs no Err, a residual failure a
fail-closed instrument bug (house panic style) — the one predicate
serves from_ir and single_ir_accept, single_cnl_accept's grounding rejecting each
CNL-reachable mirror, containment CNL-unreachable by construction) — identity
exactly on bridge-normal documents; to_ir(from_ir(ir)) == ir exactly for bridge-image IR
(the image of accepted ASTs; exact including binding region_ids — closure-derived, invariant
under bracket normalization).
Render totality: acceptance admits exactly the CNL-expressible ClinicalIR domain — the domain
`check_cnl_expressible` accepts (tail-backed
modality pairs, ≥1 statement each with a nonempty context, single-unsigned-bound quantity
intervals, single-concept interval-free exception clauses each carrying nonempty
region_ids inside the statement's segment closure, negated atoms over interval-free
entries, slot-role-conformant atom and target placement, cited segments region-bearing,
unshared, and normative-kind (Recommendation | Exception), ≥1 cited Recommendation segment
(the nonempty rule bracket under the exception-owned split), exception clauses owning
exactly the Exception-kind closure — v1) — so render is defined
for every accepted ClinicalIR on every guarded route:
single_cnl by grammar + acceptance, single_ir by the accept-total closure, M1 over its locked
corpus by derivation + lexicon integrity + the corpus render audit (derivation mints positive
concept atoms only and each locked exception segment matches exactly one concept; a document
deriving a wider clause sits outside the locked M1 contract — arbitrary M1-route inputs carry
no totality claim — and surfaces as from_ir's typed Err: the report omits that
(pipeline, document) CNL entry under cnl_inexpressible_ir (§7.4), never an assembly failure).
Expressibility agreement (render totality, executable): over canonical (ClinicalIr, lexicon,
regions, segments) tuples passing vocabulary membership and grounding — membership ONLY,
role/tail legality stays inside the predicate, keeping every variant reachable in-domain —
single_ir acceptance over the value's canonical bytes succeeds ⇔ from_ir succeeds — both
call the one predicate, from_ir Err-free past it — property-tested over a bounded IR
enumeration: bridge-image positives (to_ir over enumerated accepted ASTs) + the locked
corpus's derived IR + per-class mutations landing in every `CnlExpressibilityError` variant
while staying in-domain; the Ok side must also render in both languages.
Audit honesty: audit views render only from accepted artifacts, never from raw model output.
```

- `route.single_cnl` — the flagship route, mirroring `route.single_ir`'s stage shape: scaffold
  assemble → model fill (CNL text under the JA grammar constraint) → accept = parse + lexicon
  membership + grounding → bridge (AST → ClinicalIR + derived exact-status
  TerminologyBindings) → the unchanged M1 compile/verify tail. Repair loop per §9 mechanics:
  `cnl_parse_error` repairable under derived seeds, `ai_hallucinated_source` and
  `cnl_unregistered_concept` terminal, `repair_limit_exceeded` on exhaustion. `exp.m3_cnl` binds `[direct_smt (baseline), single_ir,
  single_cnl]` over the locked M1 inputs — the §9 measurement record extended by one route,
  scored by the same reference.
- Audit artifacts, route-independent: every CNL-expressible accepted ClinicalIR — the M1
  deterministic pipeline's included — renders to `audit/<pipeline-id>/<doc-id>.cnl.{ja,en}.txt` (keyed by pipeline AND
  document: a multi-route experiment accepts the same document several times, so views stay
  separately auditable; non-IR routes land none; an accepted IR the predicate rejects at
  `from_ir` — reachable only on a guard-less route, the M1 route off its locked corpus —
  omits exactly that (pipeline, document) entry, audit `.txt` and report CNL members alike,
  under one typed `cnl_inexpressible_ir` diagnostic (§7.4) — report assembly never fails on
  it, and a finding rule id without an owner entry already follows §7.2's omit-the-quote
  fallback); `report.json` carries the CNL text
  hashes and per-rule CNL strings under the same (pipeline, document) key; report_{en,ja}.md
  quote each finding's and documented no-conflict result's rules as CNL beside the quoted
  source spans from the findings-owner pipeline's entry alone (§7.2
  `findings_owner_pipeline_id`; positional rule ids never align routes — every route's
  views remain in the audit surfaces). The clinician-facing rule restatements are
  owner-route CNL from M3 on wherever the owner's entry carries the rule; an owner-entry
  miss omits the quote (§7.2).
- Emission-target posture (honest framing): the CNL is the committed audit surface by design;
  WHICH surface the weak model emits most reliably is the §11 measured question. Recorded
  hypothesis: a Japanese-capable weak model emits grammar-constrained Japanese CNL more
  reliably than JSON — controlled prose sits near its pretraining distribution and the grammar
  only keeps it on the rails (Shin et al. 2021; the KGQA CNL result; the cnl-landscape.md
  evidence table). The archived PoC pins the two risks this bet must clear
  (docs/poc-archive.md): grammar-masked ASCII record DSLs saturated validity yet stably
  emitted the wrong direction token — every conflict pair missed as same-direction — and
  verbose forms degenerately looped at grammar repetition points until the token budget
  truncated them. ClinicalCNL answers the first by carrying direction as the source register's
  own deontic phrases (lexicon modality surfaces, not abstract tokens) — the §11 ablation
  measures whether that works — and the second is watched directly: the record-time constraint
  audit probes DNF/exception repetition points for degeneration, and
  `surface_tokens_per_accepted_rule` plus the §9 truncation diagnostic expose it in recorded
  runs. If a §11 ablation (compact record DSL) wins emission instead, the CNL stays
  the audit surface via render-from-accepted-IR and the probabilistic step retargets — the
  architecture is invariant to that outcome.
- §7.4 M3 codes: `cnl_parse_error` (repairable; reason in payload, empty refs — mirrors
  `ai_schema_violation` conventions), `cnl_round_trip_mismatch` (fail-closed instrument code:
  an accepted AST whose canonical render fails to re-parse identically — grammar/lexicon
  drift, never a model failure), `cnl_unregistered_concept` (terminal; the escape-production
  reject — payload = the lexicon-entry proposal, quoted surface + atom position),
  `cnl_inexpressible_ir` (report-stage: an accepted ClinicalIR the expressibility predicate
  rejects at `from_ir` on a route without the acceptance guard — payload names the predicate
  class and the (pipeline, document) key, refs empty; record outcome `unsupported` — §4.4
  schema-valid construction outside implemented semantics, the stage-event outcome then
  derived per §4.4's severity order; lands in the report diagnostics summary + report-stage
  event, excluded from RouteTaxonomy (fill/accept failure classes only); the audit-view
  omission fallback above — an honest expressibility boundary, never a model failure, never
  an instrument bug).
- §7.3 additions: the surface-quality family — `round_trip_identity_rate`,
  `surface_tokens_per_accepted_rule` — beside the §9 route-quality rows; and the
  translation-faithfulness family — `ir_faithfulness_rate`: the share of a route's accepted
  documents whose ClinicalIR equals the deterministic M1 derivation recomputed over the run's
  own landed extract/segment artifacts, under the faithfulness projection — binding
  `region_ids` excluded, all else exact, ids included: CNL carries per-sentence basis refs
  (rule + exception brackets — exception provenance therefore reconstructs exactly), never
  mention-level regions — the bridge mints segment-closure binding regions, M1 mention-level
  ones — so binding region provenance is the §5 field divergent by construction, the sole
  exclusion; the remaining binding fields can also diverge off the locked corpus
  (canonical-label CNL mints one Exact binding per distinct concept, M1 one per (segment,
  candidate set) with surface-derived status) — measured misses, never
  asserted (single_ir compares its accepted fill,
  single_cnl its bridged IR; direct_smt lands no IR → not_applicable). Exact-reproduction rate,
  strict by design (§11 may grade partial faithfulness); rationale: verdict-level conflict
  metrics saturate while faithfulness still separates routes, and round-trip identity alone
  certifies the SURFACE, never the translation (docs/poc-archive.md — grammar-masked routes
  held 100% round-trip-stable emission over 50% wrong verdicts). The golden-cassette
  reproduce-M1 path pins this rate at 1.0.
- Validation program (sophistication over example count): depth-bounded AST enumeration →
  render → parse == identity with a single-parse assertion (the Codeco method);
  malformed-input battery (bare off-lexicon surface = parse error vs escaped = accept-time
  reject, wrong-slot registered surfaces, dangling refs, duplicate slots, bad bounds,
  connective misuse); an early runtime feasibility probe — the constraint mechanism compiles
  the emitted JA grammar and demonstrates one bounded constrained emission — immediately after
  the grammar emitter, before parser/bridge investment (multibyte whole-surface terminals and
  the open escape production are the risk it retires); tokenizer audit of every grammar
  terminal against the runtime constraint mechanism (§9 truncation lesson); canonical-render
  byte pins for the three M1 documents;
  reproduce-M1 gate — golden CNL cassettes for the M1 sources parse, bridge, and reproduce the
  M1 verdicts through the locked tail; lexicon lint gates.

Worked example (illustrative; the committed grammar pins the bytes).
`test_source.m1_guideline_a` accepted content renders canonically as:

```text
成人かつ敗血症のある患者には、抗菌薬Aの投与を強く推奨する。[根拠 r.2]ただし、重度腎機能障害のある患者を除く。[根拠 r.3]
```

```text
for patients with adult status and with sepsis, administration of antibiotic-a is strongly recommended. [basis r.2] exception: patients with severe renal impairment. [basis r.3]
```

Both parse to the §8.6 rule content — atoms `pop.adult` (lexicon interval semantics
`age >= 18`) and `cond.sepsis`, action `act.administer`/`drug.abx_a`, direction `for`, strength
`strong`, certainty absent (parenthetical omitted), one labeled exception
(`cond.renal_severe`), rule basis `r.2` and exception basis `r.3` (the clause's own
region_ids; statement source segments derive from their union) — and the bridge feeds the unchanged §6/§8
compile → verify chain to the same verdicts. `test_source.m1_guideline_b` renders the
contraindication tail: 「…抗菌薬Aの投与は禁忌である。」

Acceptance themes (finalized at M3 planning): `exp.m3_cnl` executes all three routes over the
locked M1 inputs with raw rows before deltas; the determinism laws hold as property tests;
every accepted document round-trips (rate 1.0 on accepted docs, emitted as a metric);
faithfulness rows emit beside the surface rows (measured, never gated — the weak baseline may
honestly read low or not_applicable; the golden path reads 1.0); audit views render
deterministically for every IR-bearing acceptance — single_ir + single_cnl in the recorded
run, M1's via its re-blessed golden run, direct_smt lands no IR hence none — and
finding/no-conflict Markdown quotes owner-route CNL only; the golden-cassette reproduce-M1 gate passes; recorded model I/O
replays byte-stably; grammar/lexicon exports carry drift guards; §0 vocabulary holds.

## §11 M4 — Route field: variation and comparison (requirements; elaborate at M3 acceptance)

Intent: widen experiment 1 across the full route axis — existing IR forms, the §10 CNL, and
further invented forms — and take experiment 2's measurements with the full evaluator; the
layered-versus-direct architecture ablation quantifies reuse and convergence on a corpus
designed to exercise them. The §10 emission-surface question settles here: prose CNL versus
compact record DSL versus JSON-IR as the weak model's target, over identical locked inputs. A
documented no-conflict result — no invented form beats the existing-IR field — stays a
first-class outcome.

Committed direction:

- Routes extending §9/§10 (concrete existing-IR schemas picked at elaboration from `docs/`,
  registered as §8.4 candidate entries):

| Route | Shape |
| --- | --- |
| `route.stacked_ir` | Model fills a stack of existing IR forms (e.g. PICO framework → rule rows); deterministic compile. |
| `route.ir_hop_chain` | Model translates across a chain of adjacent, deliberately similar IR dialects — several small constrained hops, each a minimal semantic delta — testing whether short hops tame model non-determinism better than one long jump. |
| `route.ckc_layered` | Model fills CKC layers stage by stage (segment → statement → rule), each grammar-constrained; the §6 compiler takes over. |
| `route.ckc_rec_dsl` | Model emits a compact line-oriented record DSL (id-forward, minimal terminal set; `docs/cnl-design-codex.md` carries the sketch); deterministic parse → IR. The CNL's token-compactness ablation. |
| `route.slot_cnl` | Labeled-slot CNL variant (BRIDGE-Wiz/FRET-style explicit slot lines) — the readability-versus-emission midpoint between `single_cnl` and `ckc_rec_dsl`. |
| `route.reason_ir` | Unconstrained free-text reasoning stage → constrained single-IR commit; only the commit is accepted/bridged. The constraint-placement axis: reasoning room BEFORE the grammar, versus more constrained hops. Archived-PoC prior: the sole form to beat single-IR faithfulness, on indirect surfaces (0.70→0.90 exact-IR match, verdicts saturated at 1.0 for both), at a sampling-variance cost on the free stage. |

Archived-PoC priors for this field (docs/poc-archive.md; throwaway harness, so priors, not
locked measurements): constrained-hop stacking hurt weak-model verdict accuracy (every
multi-stage failure at the final typing stage); the grammar mask closed the validity→acceptance
gap that JSON-Schema constraint cannot (var/op/value coupling); compact beat verbose under
token budgets (verbose forms loop at repetition points and truncate); a JSON-IR landing beat
the invented-DSL landing on the final typing hop; and the invented-DSL conflict miss was a
stable direction-polarity collapse — assess every candidate against BOTH §7.3 families (route
quality AND conflict quality) before any beats/does-not-beat claim, since grammar-driven
stability without faithfulness is the stability of a wrong answer.

- Every route registers its schemas/grammars and a deterministic bridge into the §6 profile,
  keeping conflict-task scoring identical across routes; all §9–§11 routes run `exp.m4_routes`
  under one locked-measurement identity, and §7.3 route-quality, baseline-delta, conflict-task
  accuracy, k-sample convergence, and §10 surface-quality metrics emit as raw rows before
  ranking. Invented candidates run singular and layered — stacked and hop-chain compositions
  over invented and existing dialects — with design dimensions recorded per candidate: token
  compactness, grammar constraint strength, semantic distance per hop, constraint placement
  (free-reasoning versus constrained stages), layer composability
  (the §12 search-space seed coordinates). Baseline deltas measure against both
  `route.direct_smt` and the best existing-IR route.
- `pipe.direct_rule_to_smt` (`exp.m4_compare`, the deterministic architecture-ablation
  baseline): extract → segment → direct phrase-normalization → FormalIR → SMT, bypassing shared
  ClinicalIR/NormIR component reuse; unused processing stages emit pass-through artifacts (outcome `ok`,
  payload marker `not_applicable`) under the same artifact wrapper rules.
- Test source growth: 4–6 additional synthetic documents sharing populations/actions/conditions
  across documents (reuse pressure), plus deterministic metamorphic variants of M1 documents
  (punctuation, kana/kanji, section order) committed as test-source variants with declared
  provenance, plus indirect-rendering variants — semantic indirection with unchanged reference
  semantics: registered-concept synonyms, oblique deontic phrasing, convention terms whose
  numeric semantics live in the lexicon, negated phrasing — the axis that dents faithfulness
  while surface-metamorphic variants leave verdicts intact (docs/poc-archive.md), plus
  threshold-conflict and factual-conflict cases for the M4 conflict kinds.
- Component store: run-scoped index of reusable components keyed by normalized structural hash;
  layered pipeline records hits/misses; `component_reuse_graph.json` and
  `compactness_front.json` join the trace exports — the front doubles as the
  mapping-minimization view (experiment 2's optimization objective, measured
  deterministically here).
- Path visualizations per §7.1 (per-finding chain; cross-document component convergence).
- Metrics per §7.3 over both pipelines and every route; the per-metric layered-minus-direct
  deltas are the staged pipeline's §7.3 baseline-delta measurement; `candidate_diff.json` compares segment,
  binding, rule, assertion, verdict, and metric levels; `ranking.csv` + `score_breakdown.json`
  with raw rows.
- Locked-measurement record: the run manifest freezes the M4 evaluator identity — test source,
  reference, lexicon, and metric-code hashes (`evaluator_lock.json` extends this identity with full
  semantics in M5).
- M4 conflict kinds (§6 table) implemented: `numeric_threshold_empty_intersection`,
  `strict_factual_contradiction`, `terminology_incoherence`, `condition_unsatisfiable`,
  `exception_resolved_conflict`; ambiguous/unmapped binding paths exercised by test sources.
- Deterministic ablations reported alongside metrics: `exceptions_off`,
  `terminology_source_linkage_off`.
- Model-free coverage experiment (`exp.m4_coverage`, experiment 2): test source set A builds mappings
  and accepted entries join the lexicon/component store (`cnl_unregistered_concept` payloads =
  the standing proposal stream, §10); test source set B (fresh documents sharing
  components) then runs `runtime_ai: false` (§8.1). Metrics: model-free coverage of B,
  accuracy versus reference, mapping-set size versus coverage on the compactness front, and
  application phase model-call count (zero) against a model-per-document baseline. Application phase path
  graphs (§7.1) contain zero model nodes — the runtime removal made visible.
- LP explanation lane per the §6 LP profile: NormIR → Prolog-family emission, SWI-Prolog and
  s(CASP) as recorded subprocess adapters, fixture-context queries, proof trees verbalized
  through the CNL lexicon so explanations speak the audit language; lane separation and
  `G-RULE-EVAL` boundary per §6.
- Informalization round-trip metamorphic, concrete via the CNL: accepted IR → canonical CNL →
  route re-parse → normalized-hash compare, reported as a route-stability metric.
- Claim-completeness instrument (route-independent; the recall converse of §9 grounding, which
  checks emitted→source only): a recorded classifier pass enumerates the SourceDocumentGraph's
  regions deterministically and labels each normative-candidate or non-normative; every
  normative-candidate region ends claimed — referenced from an accepted rule's brackets
  (rule or exception sentence) — or
  covered by a typed residual; unclaimed candidates emit `normative_region_unclaimed`
  (residual-class, §7.4). This narrows the silent-loss surface from "model skipped a
  recommendation" to "classifier mislabeled a region" — a simpler, independently k-sampled
  judgment riding the same recorded model boundary. Calibrated here against set A/B references
  (flag rate versus known reference misses — measured false-flag and miss rates); load-bearing
  from M6, where real documents carry no reference (§13.1). Metric: claim completeness (§7.3).
- `registry/methods.yaml` seeded from the `docs/` compendium (§14).
- Wording: route results are locked measurements (s0/s1 raw rows); runtime reference fidelity
  claims sit behind `G-RUNTIME-REFERENCE`.

Acceptance overview (finalized when M4's roadmap units are authored): all registered routes and
both deterministic pipelines run over all test source groups; at least two invented candidates
(§10 `single_cnl` plus one of `ckc_rec_dsl`/`slot_cnl`) rank against the existing-IR field with
raw rows visible; metrics emit exact-fraction raw rows; hash-convergence asserts identical
component hashes across metamorphic variants for the layered pipeline; recorded model I/O
replays byte-stably; path and reuse visualizations emit with deterministic bytes; expected
conflict/no-conflict outcomes hold per reference; LP-lane fixture queries replay with verdicts
lane-labeled; replay holds for both pipelines; `candidate_diff.json` is complete; the coverage
report emits with raw rows first; the claim-completeness instrument calibrates against set A/B
references (false-flag and miss rates reported).

## §12 M5 — Autonomous optimization PoC (requirements; elaborate at M4 acceptance)

Intent: experiment 2's optimization protocol — `ckc research loop --experiment exp.m5_loop`
runs a bounded propose → patch → run → score → classify → promote/reject → replay → ledger
cycle that improves experiments 1–2's objectives under an immutable evaluator. The PoC runs on
laptop budgets; the loop requirements are built to outgrow them.

Committed direction:

- `EvaluatorLock` (`evaluator_lock.json`, extending the §11 M4 identity) materialized before
  attempts: test source/reference/schema/metric/evaluator-code/toolchain/seed/budget hashes, immutable
  per experiment; per-attempt `attempt_run_lock` records evaluator-lock, candidate-graph (the
  resolved §8.4 pipeline+config identity hash), and patch/workspace hashes.
- Declared editable surfaces for the PoC: lexicon entries, prompt templates, route/DSL
  configuration (grammar included). The evaluator stays outside every candidate's editable
  surfaces; an attempt editing locked surfaces classifies as `unauthorized` (diagnostic
  `unauthorized_surface_edit`) and stays unscored.
- Objectives: §7.3 baseline delta, route quality, model-free coverage, and reuse — promotion requires
  (improvement on at least one objective, or Pareto-frontier membership), every objective within
  regression thresholds, schema validity, trace completeness, and replay success.
- Ledger: every attempt, whatever its AttemptOutcome (§4.4), lands in append-only
  `experiment_ledger.jsonl` (+ derived CSV/MD) with a run-local PromotionDecision (§4.4);
  locally promoted attempts replay deterministically.
- Budgets: max attempts/promotions/failures, wall-clock, and token counters per attempt and per
  loop; exhaustion stops the loop with `Residual(budget_exhausted)`, preserving completed and
  partial evidence.
- Evidence status: run-local promotion changes ledgers/reports only. Registry/status promotion carries
  from/to status, evidence and replay hashes, rollback, and evidence criteria (`G-AUTO-PROMOTE`);
  evaluator-identity changes (test sources, reference, schemas, metrics, evaluator code, thresholds)
  score only in a separate `G-EVALUATOR-MIGRATION` experiment.
- Mapping-gap repair (council pattern): unmapped/ambiguous residuals from new documents seed
  proposals; several independent proposer agents draft mapping deltas, and a convergence
  criterion — agreement over normalized proposal hashes — gates patch acceptance; dissenting
  proposals stay in the ledger.
- Loop drivers: the loop requirements (lock, surfaces, budgets, ledger, acceptance) are
  driver-independent, with the driver an ExperimentPlan field recorded in manifests.
  `driver.local` — this PoC's acceptance driver — runs recorded local models on the laptop.
  `driver.claude_session`, a §8.4 candidate entry, runs proposer/council/patch steps as Claude
  agent sessions (a slash command under `.claude/commands/` plus headless invocation, authored
  at this milestone's elaboration), with API cost in the budget fields; it ships authored and
  registered, exercised on user request. Long-horizon loops run on the agent driver when scale
  demands; evaluator locks, acceptance, and ledgers stay identical across drivers.
- Standing long-run objectives: route/IR-combination search over the `registry/methods.yaml`
  universe (§14) — existing formalisms and the §10/§11 invented-form program; the experiment-1
  configuration space is combinatorial — and mapping-set minimization toward the §0 asymptotic
  ideal, under `G-MDL` for any calibrated minimality claim.
- Scale-out — `ExperimentPlan` matrices with compatibility filters, pairwise/fractional designs,
  Pareto/beam narrowing, and coverage classification (untested, skipped-incompatible,
  unsupported, failed, dominated, equivalent, Pareto-front, promising) — extends these requirements
  when candidate spaces outgrow the PoC (M6).

Acceptance themes: the loop executes on `driver.local` within budgets over at least two
surfaces, with the driver named in the manifest; the ledger holds at least one valid scored
attempt and one rejected or dominated attempt; an unauthorized-surface patch is classified and
stays unscored; at least one locally promoted attempt replays; ledger summaries emit as CSV/MD.

## §13 Stage II and beyond — sources (M6), auditor product (M7), CDS-backend requirements

Intent: the scaffold, comparison, and accepted translation routes run end-to-end on real public
Japanese guideline material with permission-aware caching and richer extraction, followed by
registry-guided expansion where every candidate enters behind benchmark evidence and applicable
gates. M7 turns the audited corpus into the reviewer-facing auditor product; §13.4 keeps the
Stage III CDS-backend target visible behind gates.

### §13.1 Public sources (requirements; elaborate at M5 acceptance)

- Fetch/cache: content-addressed store under `corpus/raw/` (gitignored), resumable, with
  `PermissionRecord` per source (rights holder, access ref, license label,
  `redistribution_status ∈ redistributable|reconstructable|restricted_internal_only`, allowed
  artifact classes) and deterministic redaction policy; blocked exports emit
  `Residual(permission_limited)` and continue. New source families/export classes trigger
  `G-SOURCE-PERMISSION`.
- Source families: Minds-style guideline HTML/PDF (full text treated internal-only with
  offsets/hashes/derived labels exportable; spans quoted in reports only where permitted),
  J-STAGE/JATS XML, and PMDA e-PI XML (license-clean structured sections — 禁忌/効能/用法 — and
  the future cross-source counterpart). Licensed textbook EPUB/PDF joins as a
  `restricted_internal_only` family when rights evidence exists — corpus expansion, not M6
  acceptance; textbooks need the permission machinery, not new schema.
  `registry/source_processors.yaml` declares per-family adapters, processing stages, permission behavior,
  drift policy, diagnostics.
- Extraction: real HTML/XML parsing extended to PDF text/layout and table structure with
  uncertainty diagnostics; DecisionTable IR + `table_value_disagreement`; scanned-page OCR as a
  separate low-trust lane (engine identity and confidence recorded; OCR text feeds review
  surfaces and mapping authoring, with accepted formalization requiring validated spans);
  quantity/unit normalization (exact fractions plus canonical unit codes with committed
  UCUM-compatible conversion tables; raw Japanese unit strings preserved; threshold conflicts
  compare unit-normalized values only); reference segment/statement labels for at least one real
  test source; extractor promotion claims trigger `G-EXTRACTOR-ADAPTER`.
- Completeness: the §11 claim-completeness instrument is load-bearing here — real corpus slices
  carry reference labels for at most a few sources, so every normative-candidate region must
  end claimed or typed-residual (`normative_region_unclaimed`); a skipped recommendation is a
  visible diagnostic, never a silent loss.
- Terminology: MEDIS standard code tables (病名/HOT) as the first external systems behind the
  TerminologyBinding requirements; version-pinned snapshots; JLAC10/11 laboratory codes registered
  next; license-encumbered vocabularies (SNOMED CT, MedDRA/J, LOINC) stay registry-listed until
  licensing evidence exists.
- Drift: source hash changes emit `source_drift.json` and mark dependent scores stale.
- Boundary: the committed schemas exported since M2 govern any cross-language boundary; the
  Rust-vs-Python adapter decision per §3 is made and recorded here.

M6 acceptance themes (finalized at elaboration): the M1–M5 experiment set re-runs end-to-end
over at least one real Minds-family corpus slice plus its e-PI counterpart with live permission
records, redaction, drift checks, and claim-completeness reporting; the cross-source flagship
experiment registers.

### §13.2 Expansion principles (elaborate per candidate)

| Candidate | Adoption trigger |
| --- | --- |
| Sparse retrieval (BM25); license-clean dense/rerank models | Corpus scale demands navigation. |
| Richer rule semantics: defeasible priorities/superiority, ASP/Clingo, argumentation | Exception-as-context-conjunct measurably under-fits real guidelines. |
| Additional targets: cvc5 certificates → Lean/Isabelle replay; DMN table semantics; Alloy/TLA+ pipeline properties; e-graph canonicalization | Verifier-portfolio, table-semantics, or convergence evidence demands them. |
| Corpus-scale sweeps; experiment-matrix expansion, long-horizon agent-driver loops, IR-combination search (§12) | Candidate spaces outgrow the PoC. |
| `package_insert_vs_guideline_conflict` flagship | §13.1 e-PI test sources registered. |
| DSL/CKC-GEN beyond the M3–M4 program: typed-placeholder writing, proof export, full kernel | §10–§12 evidence favors deeper invented-IR investment. |

The §2 conservation rule keeps this table in sync with `registry/methods.yaml`.

### §13.3 M7 — Auditor product (requirements; elaborate at M6 acceptance)

Intent: Stage II's reviewer-facing deliverable — the compiler as guideline auditor, answering
experiment 3. Real-corpus findings become human-reviewable revision candidates with formal evidence,
rendered bilingual and exported as a manuscript bundle; a documented absence of defects in an
audited corpus is itself a publishable no-conflict result. Container probes for every committed
backend ran 2026-06-12 (r3 revision commit).

Committed direction:

- Revision targeting: per finding cluster, weighted minimal correction sets via solver
  soft-assertions (Z3 `assert-soft`; weights from strength, certainty, and source evidence status
  class, declared in the experiment registry); MARCO-style MUS/MCS enumeration in the Rust
  adapter when clusters outgrow single calls. Findings render "these k passages jointly imply
  an impossibility" with member spans.
- Verifier cross-check: cvc5 replays every blocking/major finding's queries; agreement,
  divergence, or unknown recorded per query; divergence emits a review item; evidence feeds
  `G-PORTFOLIO`. The cvc5 adapter parses verdict tokens and tolerates solver result command errors
  after non-matching verdicts (the Z3-adapter pattern); §6 emission stays byte-pinned.
- Mechanized anchor: a Lean 4 package defines the NormIR/FormalIR fragment, the §6 conflict
  predicates, and normalizer properties; per-run generated data files; per-instance checks by
  `decide`/`native_decide` recorded in the trace with replay commands (kernel reduction stalls
  on String-order-heavy computation — `native_decide` or Nat-keyed encodings); generic
  theorems land as explicit proof-debt records, never silent assumptions.
- Finding review classification: severity (§4.4) plus bilingual suggested review questions; wording stays
  §0-calibrated — `warrants review` by default, contradiction vocabulary only for proven
  inconsistency within supported semantics.
- `HumanReviewRecord`: append-only reviewer-role-typed records (clinician, formalist,
  terminology, final reviewer) attached to findings by hash, never mutating them; agreement
  statistics in reports; human-reviewed-corpus claims trigger `G-REFERENCE-CORPUS`.
- `report.html`: one self-contained deterministic bilingual review artifact per run — embedded
  canonical report/trace data plus committed content-hashed viewer assets; Japanese spans
  primary, English glosses linked per span; corpus overview, rule browser (rules quoted as §10
  CNL, Japanese primary), finding list with
  filters, finding detail with formal evidence and revision candidates, metrics; renderer
  identity recorded; zero servers, zero network, zero external toolchain in the build path.
- Auditor manuscript bundle: extends the §7.2 bundle with finding/human review tables and
  per-kind defect statistics for the Stage II paper.

Acceptance themes (finalized at elaboration): an auditor run over the M6 corpus emits review-classified,
localized, cross-checked, anchor-checked findings; a reviewer walks source text span → rule →
assertion → verdict → review question entirely offline in `report.html`; human review records
round-trip without mutating findings; the bundle exports; replay holds.

### §13.4 Stage III — CDS-backend requirements (gated; no build commitment)

The compiler's runtime target, kept visible so Stage I/II decisions stay compatible.
Capabilities enter only behind their gates, each as a registered candidate citing its
compendium row: a three-valued rule evaluator over typed patient contexts (`applicable |
not_applicable | unknown` plus conflict statuses; open-world missing-data semantics;
`G-RULE-EVAL` adoption defines the semantics before any build), FHIR-family interop exports
(JP Core patient substrate, Clinical Reasoning packaging, CQL/ELM where expressible, lossiness
recorded per export), EHR ingestion (SS-MIX2, CDS Hooks/SMART), live-patient data
(`G-LIVE-PATIENT`), probabilistic and world-model semantics (`G-PROB-REASONING`, `G-WORLD-MODEL`),
clinical deployment evidence status (`G-CLINICAL-REGULATORY`).

## §14 Registries and research compendium

Registry files are data, validated by `ckc registry check`, growing per milestone: M1
`corpora|candidates|experiments`; M2 adds `prompts|schemas` (the schema export feeds M2's
grammar constraints) and generalizes the `experiments` binding from a single pipeline to a
pipeline set with a `baseline_pipeline` (the §7.3 delta baseline), so one experiment binds both
route pipelines over identical locked inputs while the M1 single-pipeline form stays valid; M3
extends `schemas|prompts` with CNL grammar and prompt entries (§10); M4 adds `methods`, the
method-universe catalogue seeded from the
compendium (families, aliases, candidate roles, adapter status
`v_required|v_optional|registered_backlog|gate_only`, benchmark tags, compatibility metadata),
and extends `schemas|prompts` with invented-DSL entries; M5 adds `evaluators|gates` (gate
evidence objects); M6 adds `source_processors|policies` and `indexes` with retrieval; M7 adds
`human_review` and `views` (content-hashed `report.html` viewer assets).

`docs/` is the research compendium — method-category deep-research reports, the CNL research
base, charters, and PoC-archive evidence — resident in git history, not the working tree: a
research artifact is committed under `docs/`, then retired once distilled into this spec.
Every `docs/…` citation in this spec resolves at commit `e8b5cf6` (the M3-planning state;
census `git ls-tree e8b5cf6 docs/`, read `git show e8b5cf6:docs/<file>`, search
`git grep <pat> e8b5cf6 -- docs/`); a future retirement records its own commit here.
Registry-seeding and elaboration units mine it through read-only subagents and
cite `file §section` in registry notes; main sessions keep their own context lean.

## §15 Gates

Gates carry every stronger-than-research claim; each is a one-line trigger plus an evidence
object, defined fully when first triggered. `GateEvidenceRef` names gate, subject hash, evidence
hash, replay identity, enabled claims, limitations, rollback/sunset. A missing gate emits
`Residual(deferred_gate_required)` for the stronger claim only — locked lower-tier measurements
stand on their own.

| Gate | Trigger | Evidence object |
| --- | --- | --- |
| `G-SOURCE-PERMISSION` | New source family, redistribution mode, or export class. | `SourcePermissionProfile` |
| `G-REFERENCE-CORPUS` | Human-reviewed/released corpus-quality claims. | `ReferenceCorpusEvidence` |
| `G-EXTRACTOR-ADAPTER` | Extractor promotion or generalized extraction-quality claims. | `ExtractorAdapterRecord` |
| `G-RET-QUALITY` | Retrieval-quality claims. | `RetrievalQualityReport` |
| `G-PORTFOLIO` | Multi-verifier agreement/robustness claims. | `VerifierPortfolioReport` |
| `G-ABSTRACT-DOMAIN-LOGIC-FULL` | Richer abstract-domain logic affecting accepted outputs. | `AbstractDomainLogicRecord` |
| `G-REBIND` | Proof/trace transport across source or terminology editions. | `RebindingEvidence` |
| `G-BENCHMARK-RELEASE` | Released benchmarks, corpus-scale or calibrated performance claims. | `BenchmarkRelease`, `BenchmarkCalibrationReport` |
| `G-EVALUATOR-MIGRATION` | Changes to test sources/reference/schemas/metrics/evaluator code for future scoring. | `EvaluatorMigrationEvidence` |
| `G-MDL` | Calibrated compression/Pareto/model-selection claims. | `MDLEvidence` |
| `G-RUNTIME-REFERENCE` | Runtime-model-call or IR-processing stage reference-fidelity claims. | `RuntimeReferenceReport` |
| `G-AUTO-PROMOTE` | Automated registry/status promotion of accepted generators, prompts, policies, compilers, verifier adapters, metric/report code. | `AutoPromotionEvidence` |
| `G-PROB-REASONING` | Probability-based reasoning changes accepted outputs. | `ProbabilisticProfileRecord` |
| `G-WORLD-MODEL` | Latent-state/multimodal observations affecting outputs. | `WorldModelProfileRecord` |
| `G-RULE-EVAL` | Patient-context rule-evaluation semantics in any shipped output. | `RuleEvalProfileRecord` |
| `G-LIVE-PATIENT` | Any patient-derived data entering CKC. | `GovernedPatientDataProfile` |
| `G-CLINICAL-REGULATORY` | Clinical/regulatory/deployment evidence status claims. | `S3AssuranceEvidence` |

Gate invariants: gate evidence is replayable or explicitly marked non-authoritative; candidate
loops run inside locked experiments, with evaluator changes governed separately before any
ranking that depends on them; regulatory-framework vocabulary (assurance cases, SaMD classes,
APPI categories, SBOM fields) enters the spec only through these evidence objects when their
gates first trigger.
