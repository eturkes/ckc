# CKC — Clinical Knowledge Compiler

Design authority for this repository. Sole implementers and readers: Claude
sessions operating under AGENTS.md/CLAUDE.md, `.claude/commands/session-prompt.md`, and `.agent/`.
The document is optimized for machine reading: stable `§` anchors, tables over prose, one fact in
one place, sections sized for selective loading.

## §0 Mission, thesis, posture

CKC = Clinical Knowledge Compiler, built evidence-first (architecture reset 2026-07-12): this
repository is FIRST a rigorous, representation-neutral research harness that fairly compares
candidate approaches to formalizing clinical knowledge — structured IRs, controlled natural
languages, direct formal targets, layered transformations, alternative reasoning engines — and
SECOND, once harness evidence selects an approach or composition (§11), the source of the
clinical knowledge compiler that evidence defines: a headless compiler translating clinical
text in any language (public Japanese guideline corpora through M7) into compact, reusable,
source-grounded components, compiled deterministically to machine-evaluable targets (SMT-LIB
first), surfacing contradictions and documented no-conflict results with end-to-end
machine-checkable evidence.

Standing invariant, every candidate: the probabilistic step is confined to one boundary —
source text into a route's constrained emission surface; every layer below an accepted
artifact is a deterministic compiler.

Harness/route split (§3's role table assigns every element exactly one role): the shared
harness owns only the concerns common to every candidate — source corpus, benchmark cases and
their intended semantics; source grounding and provenance; route invocation and resource
accounting; standardized outcomes, evidence, and diagnostics; evaluation, metrics, replay, and
the experiment ledger. Each candidate route owns its representations, transformations,
prompts, schemas, grammars, compiler passes, and reasoning engines. Representation choice, hop
count, constraint mechanism, normalization strategy, and backend stay independently measurable
(§10.3); representation-neutral task outcomes are the primary comparison basis and
route-specific structural checks are diagnostic evidence (§10.2). Landed M1/M2 behavior is the
reproducible baseline and control. ClinicalCNL (§10.4) holds high-priority candidate status on
the standing priors — under the same §11 promotion bar as every other approach.

Thesis under test, as three experiments ordered by dependency:

1. Translation reliability (models in the loop): which constrained emission surface and route
   composition most reliably translates clinical text into accepted, machine-evaluable
   semantics? M2 landed the minimal pair (direct SMT versus single-hop JSON-IR); M3 adds the
   ClinicalCNL slice plus the faithfulness instrument; M4 selects or widens on the evidence.
2. Deterministic mapping by optimization (models at development time only): an optimization
   protocol designs and maintains a deterministic mapping — a compact accepted expert system
   authored by AI agents — that covers fresh documents with zero runtime model calls;
   maximize coverage and reuse, minimize mapping-set size. Coverage instrument from M4; the
   bounded autonomous protocol at M5. Behind it sits an asymptotic ideal — ever more minimal
   accepted mapping and axiom sets representing ever more clinical knowledge — orienting the
   compactness objective (`G-MDL`) while staying outside every report's claims.
3. Revision surfacing (the compiler applied): once a corpus is compiled, does the result
   highlight guidelines and companion sources in need of revision? Seeded at M1 (one
   synthetic contradiction, one documented no-conflict result); answered on real public
   corpora at M6–M7. Flagship (M6 era): cross-source conflict surfacing — a guideline
   recommendation versus a PMDA drug-labeling contraindication — traced from Japanese source
   text spans through accepted semantics and named assertions to solver cores, replayable
   from content hashes alone.

Documented no-conflict results are first-class outcomes for all three experiments — including
"no invented representation beats the existing field" (§11).

Stage arc: Stage I — research harness (M1–M5): fair comparison on synthetic corpora; locked
Stage I measurements anchor a methods manuscript; the §11 selection decision closes the stage.
Stage II — guideline auditor (M6–M7): the selected architecture runs real public corpora,
surfacing source-grounded revision candidates, human-reviewed and rendered bilingual; Stage I
is the auditor's validity argument, and the auditor's findings anchor a second manuscript.
Stage III — CDS backend (§13.4, requirements-only): every capability sits behind §15 gates.

Research position: every output is research evidence. Accepted semantics come from acceptance
(schema validity, source linkage, canonical bytes, applicable compiler/verifier checks, trace,
replay, evidence criteria) — independent of proposer identity. AI, retrieval, agents, and
humans all propose; acceptance decides. Reports describe results with this vocabulary:
`research harness`, `candidate`, `review candidate`, `formalization-QA`,
`text-quality analysis`, `source-grounded`, `schema-valid`, `verifier-checked`, `replayable`,
`requires human review`, `locked measurement`, `synthetic test source measurement`,
`raw benchmark output`, `documented no-conflict result`. Clinical, patient-care, CDS-runtime,
SaMD, deployment, and regulatory claims sit behind the gates in §15 and enter reports only
after their gates pass.

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

Intent: prove the thesis through vertical slices, each ending in a runnable artifact; from M3,
evidence-first — the smallest experiment able to change the architectural decision precedes
any wider build, and each milestone's evidence is the next milestone's input.

| Stage | Deliverable | Proof |
| --- | --- | --- |
| M1 scaffold (landed) | Deterministic layered pipeline end-to-end on synthetic Japanese test sources: extract → segment → normalize → assemble → compile → verify → trace → report; one deontic contradiction found, one no-conflict result documented, full trace, deterministic replay. Pure Rust. Doubles as baseline control and reference-derivation instrument. | `ckc run --experiment exp.m1_scaffold` + §8 checklist |
| M2 multi-hop PoC (landed) | Experiment 1's minimal pair: a weak local model (laptop CPU, constrained decoding, recorded I/O) translates the M1 test sources via `route.direct_smt` versus `route.single_ir`; scored on validity/acceptance/verdict-accuracy/stability raw rows; bilingual research report. Landed the multi-route harness: recorded model adapter, cassettes, repair loop, metrics, baseline-delta table. | `ckc run --experiment exp.m2_multihop` + §9 |
| M3 route comparison | The smallest decisive experiment on the strongest open question: ClinicalCNL v1 slice (§10.4 — JA-only, closed lexicon) as `route.single_cnl`, compared against the landed §9 pair on identical locked inputs under the neutral contract (§10.1–§10.2), with the faithfulness, round-trip, and resource instruments and the run explorer (§10.5). | `ckc run --experiment exp.m3_cnl` + §10.5 |
| M4 selection + widening | §11 promotion review over M1–M3 evidence: either promote a composition toward product architecture (canonicalize, migrate, set retirement gates) or run the single next-most-decisive widening experiment from the §11.3 route field; repeat until selection. Deferred instruments land on evidence demand: M4 conflict kinds, component store, model-free coverage, claim completeness, corpus growth, LP lane (§11.4). | `ckc run --experiment exp.m4_*` + §11 |
| M5 optimization PoC | Bounded autonomous-optimization loop (§12) over declared surfaces against a fixed evaluator, optimizing translation reliability, reuse, and coverage; append-only ledger; driver-independent — local driver for acceptance, Claude-agent driver defined (experiment 2's optimization protocol). | `ckc research loop --experiment exp.m5_loop` + §12 |
| M6 sources + expansion | Public corpus ingestion (fetch/cache, permission records, real Minds/J-STAGE HTML+PDF extraction, tables and DecisionTable IR, MEDIS-anchored terminology, e-PI XML source family, drift checks), then registry-guided expansion: retrieval, richer rule semantics, additional solvers/targets, corpus scale, experiment-matrix expansion, the cross-source flagship experiment. | §13.1 requirements elaborated at M5 acceptance; §13.2 per candidate |
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
| `ckc-cli` | `ckc` binary: pipeline processing stages, runner, trace/report/replay, registry check, CNL modules (M3). |

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

Role assignments (the 2026-07-12 reset's classification; every element carries exactly one
role, and a role change is a §11 promotion/retirement event recorded here):

| Element | Role |
| --- | --- |
| Test sources, lexicon, `corpus/reference` expected outcomes | Harness: benchmark inputs + oracle annotations — intended semantics fixed at corpus authoring (acceptance-reviewed per §0, origin-independent), never derived from any route's output. |
| extract + segment stages; SourceDocumentGraph, spans, anchors, regions; ClinicalSegments | Harness: shared source grounding — every route consumes the graph + segments and cites region ids. |
| Model adapter, cassettes, `model_fill` stage core, `prompts`/`schemas` registries | Harness: route invocation, recording, resource accounting. |
| Trace, report, metrics, replay, registry check, experiment binding | Harness: evaluation contract, provenance, experiment ledger. |
| §6 conflict semantics + verdict categories | Harness: the shared task-outcome semantics every route is scored on. |
| Z3 | Harness verdict executor (§6) and candidate product backend. |
| ClinicalIR | Interchange representation: the candidate routes' common landing zone and the faithfulness-diagnostic substrate (§10.2); itself a candidate representation (`route.single_ir`). |
| NormIR, FormalIR | Route-internal deterministic compiler passes (the shared compile-tail library); no route is obliged to produce them. |
| SMT-LIB artifacts | Generated target. |
| `pipe.layered_ckcir_to_smt` (M1) | Baseline/legacy control + reference-derivation instrument. |
| `route.direct_smt` | Weak baseline control (landed). |
| `route.single_ir` | Candidate route: structured-IR emission (landed). |
| ClinicalCNL grammar/AST/bridge | Candidate representation, high priority (§10.4 slice at M3). |
| Prolog-family LP lane, cvc5, Lean | Candidate backends: registered, evidence-triggered (§6 LP profile, §13.2). |
| Attempto/ACE/APE, GF, PENG, FRET et al. | External prior art + dev-time design oracles; no build dependency (license posture §11.5). |
| `docs/` compendium | Research archive, git-resident (§14). |
| Dual-surface emission split; Ulex precedence shadowing | Retired designs (git history). |
| EN mirror surface; unregistered-concept escape; from-IR audit rendering; CNL findings quoting; lexicon accretion pipeline | Deferred CNL capabilities: promotion-phase scope (§11.3), deliberately outside the M3 slice. |

License: GPL-3.0-or-later, Copyright (C) 2026 Emir Turkes — the whole tree (code, spec,
registries, project-authored corpus) conveys under LICENSE's terms; §11.5 carries the
copyleft-source rationale and the per-resource evidence-row discipline.

Repository layout (target state; built up by the M1 units):

```text
.
├── SPEC.md  AGENTS.md  CLAUDE.md  LICENSE  .gitignore   # LICENSE = GPL-3.0-or-later (§11.5)
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
| `TerminologyBinding` | Mention → concept binding: `system` (M1: `ckc.lex`), code, status (BindingStatus), alternatives, region refs — provenance at the producer's grain: M1 normalize grounds the mention, M3's CNL bridge mints the sorted union of the citing statements' cited regions (§10.4). |
| `ClinicalStatement` | Normalized population, condition, action, modality, strength (`strong\|weak`), certainty (`high\|moderate\|low\|very_low`), exceptions, source refs; comparator/outcome/temporal slots optional at M1. |
| `Action` | Action kind + target concept + distinguishing fields (M4) + normalized target key. |
| `ContextExpr` | Finite DNF over atoms: concept predicate, negated concept predicate, quantity interval; M4 adds slot equality and temporal interval (difference-logic) atoms. |
| `NormativeRule` | `rule_id, context, direction, action, strength, source_region_ids` + optional at M1 `certainty, exception_refs`; exceptions compile per positive concept atom to negated context conjuncts (§10.4 keeps clauses single-concept; wider clause shapes sit outside the compile contract), their regions joining `source_region_ids`. |
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
| `CnlDocument` (M3) | Canonical ClinicalCNL text (v1 slice: JA; EN mirror = §11.3 promotion scope) + the CNL AST it parses to, over an IRBundle's ClinicalIR content: grammar id + hash, per-rule AST + canonical text, text hash; §10.4 text↔AST inverse laws, AST→ClinicalIR bridge. Not a new IR layer — ClinicalIR's second concrete syntax beside canonical JSON. |

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
concept is required. External terminologies (MEDIS standard code tables first: MHLW-designated; permissions
evidence-gated per §11.5) join at M6 as additional systems behind the same TerminologyBinding
requirements.

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
s(CASP) proof trees verbalize through CNL lexicon templates so justifications read as §10.4
CNL prose (a from-IR rendering consumer — §11.3 promotion scope precedes this lane).
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
weighted ranking; from M3, per-document CNL audit artifacts for CNL-route accepted documents —
the stored canonical text (§10.4); each route's own accepted surface is its audit artifact, and
cross-route CNL rendering of accepted IR is promotion-phase scope (§11.3); from M4, ablations;
from M5, attempt-ledger summaries; from M6, matrix coverage.
Finding ids form `finding.<group_id>.<sequence_number>` with sequence numbers in source-then-hash order (§4.1).
A multi-route run keeps exactly one findings view: the first compiled (bundle-bearing) pipeline
in experiment binding order feeds the findings body, the documented no-conflict results, and
the report's verifier-result identity — `findings_owner_pipeline_id` records that pipeline
canonically, present whenever the run lands a findings view — keeping payload query and
finding ids route-unprefixed;
every other compiled route lands route-namespaced artifacts feeding audit views, metrics, and
ledgers only. Normative rule ids are route-local positional identities, never cross-route
alignment keys — a same-numbered id under another route may hold different content, so
cross-route rule comparison requires an explicit alignment map (out of scope through M3).
From M7, findings carry `severity` (§4.4) and a bilingual suggested review question. From M5,
publication-designated runs export a manuscript bundle — figure-ready CSV/JSON metric tables,
corpus/permission summaries, replay instructions, limitations derived from typed
residual/ambiguity statistics — extended at M7 with finding and human review tables (§0 stage
arc: Stage I methods paper, Stage II auditor paper).
Report wording stays within the §0 vocabulary.

### §7.3 Metrics (M2 onward)

Metric values are exact fractions; unavailable values are omitted with a diagnostic; zero
denominators emit `not_applicable` per metric schema. Raw rows always accompany rankings.
Two classes (§10.2): PRIMARY metrics are representation-neutral — defined over task outcomes
and resource use, computable for every contract-conforming route — and rank routes; DIAGNOSTIC
metrics presuppose a route shape (IR-landing, CNL-landing, layered) and never rank across
shapes: `not_applicable` where the shape is absent, and a shape-applicable-but-missing value
is a fail-closed instrument error, never a silent omission.

Primary families: compilation (schema/compile/parse/solver pass rates), conflict quality
(precision/recall and conflict-task accuracy over test source expectations), route quality
(schema-valid rate, acceptance rate, repair count, recorded-call counts, k-sample convergence;
from M2), resource (recorded calls, repairs, model wall-clock, accepted-emission byte size —
byte size over the accepted artifact's stored surface, deterministic and runtime-free; from
M3), determinism (hash stability), trace completeness, and baseline delta (per-metric
route-versus-baseline deltas over identical test sources: model routes from M2,
layered-minus-direct from M4).

Diagnostic families: translation faithfulness (share of a route's accepted IR-landing
documents whose ClinicalIR equals the deterministic reference derivation over identical inputs
under the declared faithfulness projection — provenance-grain fields excluded, §10.4; verdicts
saturate on small corpora while faithfulness still separates routes, `docs/poc-archive.md`;
from M3), surface quality (round-trip identity rate over accepted CNL documents; from M3),
reuse (component reuse rate, duplicate rate), compactness (component count, mapping-set size
versus coverage, reuse degree, MDL proxies), convergence (normalized hash agreement across
variants), model-free coverage (share of fresh-document semantics produced deterministically
from accepted mappings, with zero application-phase model calls; from M4), claim completeness
(share of normative-candidate source regions claimed by an accepted rule or covered by a typed
residual, §11.4; from M4), and loop outcomes (from M5).

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
`repair_limit_exceeded`); CNL parse and validation rejects reuse `ai_schema_violation` (the §9
repairable convention, §10.4). M3 adds `cnl_round_trip_mismatch` (accept-time re-render
disagreement: fail-closed terminal instrument error, outcome `invalid`); the deferred escape
design carries `cnl_unregistered_concept` (§11.3). M4 adds invented-DSL route codes and the
claim-completeness code (`normative_region_unclaimed`, §11.4); M5 adds loop/budget/surface
codes (`unauthorized_surface_edit`, `budget_exhausted`); M6 adds source/permission/drift
codes; each is defined in its milestone section at elaboration time.

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

## §9 M2 — Multi-hop translation PoC (landed)

Status: landed — locked measurement `exp.m2_multihop`, tag `accept/m2`; the requirements
below are the faithful M2 record, and its mechanics (recorded adapter, cassettes, repair
loop, acceptance closures, metrics) are the harness machinery §10 routes reuse.

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

## §10 M3 — Candidate routes, neutral contract, route comparison (reset 2026-07-12)

Intent: the design authority for fair comparison — the route contract every candidate
satisfies, the evaluation contract that scores them, the candidate matrix, and the M3
experiment: the ClinicalCNL v1 slice against the landed §9 pair. The pre-reset ClinicalCNL
full design (bilingual mirror, unregistered-concept escape, from-IR audit rendering, findings
CNL quoting, lexicon accretion) is superseded as active authority and git-resident (§14
retrieval note); §10.4 carries its load-bearing decisions forward at slice scope, and §11.3
holds the deferred remainder behind promotion evidence.

### §10.1 Neutral route contract

A candidate route is a registered pipeline (§8.4 entries) satisfying:

- Inputs: identical benchmark inputs per experiment binding — locked test sources through the
  shared extract + segment stages (`source_document_graph`, `segments`), the committed
  lexicon, and the route's own registered constraint surfaces (schema/grammar + prompt
  template, §14), every hash in the manifest. No route receives an input another compared
  route is denied; prompt content is route-owned (part of the candidate), its hash recorded.
- Model discipline: model calls only inside recorded `model_fill` stages (§9 mechanics —
  cassettes, seeds, repair loop, §7.4 reject taxonomy); every stage below an accepted
  artifact is deterministic.
- Outputs, mandatory: per-group VerifierResults in §6 categories (or a typed §7.4 failure);
  per-document acceptance status with diagnostics; §4.6 events with resource counters; §9
  manifest identity fields (model, runtime, constraint, prompt hashes). Outputs, optional —
  diagnostic evidence: an IRBundle (enables faithfulness, audit views, findings ownership),
  route-native accepted artifacts (CNL document, SMT text).
- Scoring interface: conflict-task outcomes land through the shared §6 verdict semantics —
  IR-landing routes through the shared compile → verify tail, formal-target routes through
  direct verification of their emitted queries. A non-SMT reasoning backend joins by mapping
  its verdicts into §6 categories behind §11's backend evidence bar.
- Findings ownership (multi-route runs, §7.2): exactly one findings view — the first
  bundle-bearing pipeline in experiment binding order; every other compiled route lands
  route-namespaced artifacts feeding audit views, metrics, and ledgers.
- Fairness controls: one experiment binds all compared routes over identical locked inputs,
  seeds, budgets, model identity, and reference; §7.3 raw rows emit before any delta or
  ranking; repair budgets identical across routes.

### §10.2 Evaluation contract

- Oracle provenance (anti-circularity): expected outcomes (`corpus/reference/*.yaml`) are
  intended semantics fixed at corpus authoring (acceptance-reviewed per §0's
  proposer-independence) — never derived from any route's output. The M1 pipeline is a
  control that reproduces them deterministically, not their source. Expected unsat cores compare as sets of rule-derived assertion ids under the
  shared id convention; a route whose accepted semantics split rules differently scores on
  verdict + conflict kind, its core comparison downgraded to diagnostic evidence.
- Primary basis (ranks routes): representation-neutral task outcomes and costs — conflict-task
  accuracy, acceptance/validity rates, k-sample stability, failure-taxonomy shares, resource
  rows (§7.3 primary families).
- Diagnostic basis (explains; never ranks across shapes): translation faithfulness versus the
  reference derivation — the reference derivation is M1 normalize's deterministic output, so
  faithfulness measures agreement-with-the-instrument and inherits M1's normalization
  conventions by construction, which is exactly why it stays diagnostic — plus round-trip
  identity, structural checks, and audit-surface inspection.
- Known confounds, declared: (1) representation and constraint mechanism are partially
  conflated in the landed pair (`direct_smt` grammar-constrained, `single_ir`
  JSON-Schema-constrained); `single_cnl` (grammar-constrained prose) adds the point
  separating surface class from constraint mechanism, and a grammar-constrained JSON-IR
  control is the registered deconfounder if M3 evidence stays ambiguous (§11.3).
  (2) Prompt content necessarily differs per route — prompts are candidate-owned; hashes
  recorded; prompt-tuning effort SHOULD stay comparable and is reported. (3) The M3 corpus
  is three documents with saturated verdicts — faithfulness and failure taxonomy carry the
  discrimination (archived-PoC prior), any beats/does-not-beat claim cites BOTH §7.3
  families, and corpus growth (metamorphic + indirect variants) is §11.4 scope.
- Inspectability evidence: each route's own accepted surface is its audit artifact (CNL text,
  IR JSON, SMT text), inspected side-by-side in the explorer (§10.5) beside quoted source
  spans; human judgment on this axis is recorded in the §11 promotion review, never scored
  as a metric.

### §10.3 Candidate matrix

Experimental axes, independently measurable; a new route declares its coordinates:

| Axis | Points (landed / M3 / registered §11.3) |
| --- | --- |
| Emission surface | SMT text (landed) / JSON-IR (landed) / CNL prose (M3) / record DSL, slotted CNL (registered) |
| Constraint mechanism | BNF grammar mask (landed) / JSON-Schema (landed) / crossed surface×mechanism controls (registered) |
| Hop count + composition | single constrained hop (landed, M3) / stacked, hop-chain, free-reason + constrained-commit (registered) |
| Normalization strategy | model-normalized emission (all model routes) / deterministic lexicon normalization (M1 control) |
| Reasoning backend | Z3 SMT (landed) / Prolog-family LP lane, cvc5, Lean (registered; §6, §13.2) |

Standing evidence: M2 locked measurement — the §9 minimal pair over locked M1 inputs
(`accept/m2`). Archived-PoC priors (`docs/poc-archive.md`; throwaway harness, so priors, not
locked measurements): one constrained hop with source + full vocabulary in view beat every
constrained-hop stack; a grammar mask closed the validity→acceptance gap JSON-Schema cannot
(cross-field coupling); compact surfaces beat verbose under token budgets (verbose forms loop
at repetition points and truncate); invented ASCII record DSLs stably emitted the WRONG deontic
polarity while staying well-formed; free-reason + constrained-commit was the sole form to beat
single-IR faithfulness (0.70→0.90, verdicts saturated at 1.0 for both) at a sampling-variance
cost. CNL research base (git-resident `docs/cnl-*.md`): no surveyed system offers
deterministic CNL parse ⇄ verbalize ⇄ executable formal target; no deterministic
Japanese-parsing CNL exists; no published system wires a CNL grammar into LLM constrained
decoding; constrained decoding into canonical CNL then deterministic mapping to logic beats
decoding formal syntax directly (Shin et al. 2021, in `docs/cnl-landscape.md`).

Unresolved — what M3 can change: whether grammar-constrained CNL prose matches or beats
JSON-IR on acceptance and faithfulness while adding an audit surface a clinician can read.
That evidence justifies, redirects, or retires the CNL product bet (§11).

### §10.4 ClinicalCNL v1 slice (the M3 candidate)

One content layer, two concrete syntaxes is the design idea: ClinicalIR serializes as
canonical JSON (§4.3) for machines and as ClinicalCNL for clinicians; parse and render are
mutual inverses over a CNL AST; a deterministic bridge maps the AST into ClinicalIR. The v1
slice is the smallest version able to earn comparison evidence: Japanese only, closed lexicon
(no escape production — bare off-lexicon text is a repairable parse error), parse → bridge →
ClinicalIR only (no from-IR rendering). EN mirror, escape, and cross-route audit rendering are
§11.3 promotion scope. Grammar-constrained emission and canonical rendering both land inside
the grammar language, so open-Japanese parsing (zero anaphora, attachment, scope) is designed
out rather than solved. Named to mirror ClinicalIR; ids: grammar file
`schemas/clinical_cnl_ja.grammar`, registry schema id `schema.clinical_cnl`,
`route.single_cnl`.

Sentence model: one rule = one sentence group — a recommendation sentence with its basis
bracket, plus zero or more exception sentences, each with its own basis bracket (per-sentence
provenance: the bridge reads each exception clause's region_ids off its own bracket; a single
rule-global bracket would leave multi-exception provenance unreconstructible). Fixed clause
order, closed connective set, no pronouns, no anaphora, no ellipsis; multiword concepts are
single lexicon terminals, never parsed compounds; fail-closed throughout (the anti-ACE
lesson). Slots, JA canonical shapes:

| Slot | JA canonical shape | AST target |
| --- | --- | --- |
| context | `<dnf>患者には、` | population/condition DNF |
| action | `<target>の<action-noun>` (e.g. `抗菌薬Aの投与`) | Action kind + target |
| deontic tail | `を強く推奨する` / `は禁忌である` … (lexicon `tail_ja`) | (direction, strength) |
| certainty | `(<certainty-surface>)`, optional | certainty |
| exception | `ただし、<concept>患者を除く。[根拠 <id> …]` per entry | one single-concept ExceptionClause per entry per split statement |
| basis | `[根拠 <id> …]` after each sentence, ids sorted per bracket, ≥1 ref | rule/exception source regions |

- DNF prose: conjuncts join with `かつ`; disjunct groups join with `、または`; `かつ` binds
  tighter by decree; flat two-level DNF only — each disjunct maps to one ClinicalStatement.
  Atoms: concept (lexicon adnominal surface), negated concept (lexicon negated-adnominal
  surface), quantity interval (`<var-surface>が<n><unit><bound>`). Exception register stays
  single positive interval-free concept per sentence (§5 NormativeRule compiles exceptions
  per positive concept atom; wider clause shapes sit outside the compile contract).
  Negative-occurrence bar: interval-carrying concepts are barred from negated positions and
  exception slots (their complement is not one interval; the repair names the
  complement-interval rewrite).
- Composition decree (concrete syntax; committed grammar pins the bytes): plain concatenation
  of whole-surface terminals — fluency is a lexicon-authoring concern, never a grammar
  property. Adnominal/negated surfaces compose bare in every position; interval atoms take
  the fixed linking terminal `の` exactly patient-adjacent (`年齢が18歳未満の患者には、`)
  and compose bare mid-chain — the grammar carries mid versus patient-adjacent atom
  alternations, so a stray or missing `の` is a parse error. Interval bound markers after
  the unit: `以上`/`以下`/`未満`/`超` ↔ ge/le/lt/gt. Numerals: ASCII digits,
  leading-zero-free canonical decimals (`0` alone admits a leading zero), parser-bounded
  0..=i64::MAX — the grammar's SOLE declared over-approximation class. Punctuation `、` `。`
  plus ASCII brackets/parens; JA text is separator-free outside bracket internals (bracket
  ids space-separated, sorted); whitespace variation NONE — parser language == grammar
  language; stray whitespace is a repairable parse error. Document frame: canonical document
  bytes = one LF-terminated line per rule, LF the uniform terminator, last rule included, no
  other inter-rule bytes; stored per-rule texts are line-break-free.
- Lexicon (JA slice; `corpus/lexicon/ja_core.yaml` extends, all new fields optional at load
  so committed bytes stay green): concepts gain `adnominal_ja`, `negated_ja` (prenominal
  negation, prefix-clean against its positive: 非-prefix nouns / verb flip / copula flip;
  trailing `の` barred), and typed slot `roles` — nonempty set over
  `population|condition|action_target`, population/condition mutually exclusive per row,
  action_target free to combine; actions gain `noun_ja`; modality rows gain optional
  `tail_ja` with ≥1 tail-bearing row per present (direction,strength) pair, first
  tail-bearing row canonical; certainty renders its committed `surfaces[0]`; a NEW quantity
  table maps interval vars — `{var_id, role (population|condition), surface_ja, unit_ja}`,
  var set == the interval vars concepts use, exactly one row per var. ONE typed role view is
  the single source every CNL module reads (grammar slot alternations, AST validation,
  parser slot legality, bridge partition); integrity + lint gate view construction:
  hard-errors (refs resolve, role sets legal, quantity/var agreement, render totality,
  per-language duplicate-literal rejection by semantic token) plus lint (reserved-terminal
  collisions, trailing-`の`, pairwise proper-prefix-freedom across ALL lexer-visible tokens
  — fixed terminals and digits included — the maximal-munch determinism guard). Synonym
  `surfaces[1..]` stay source-match-only, never CNL terminals. `normalize.rs`'s frozen M1
  prefix partition stays untouched; a test pins committed-corpus role agreement.
- Grammar: emitted from the validated lexicon view by `clinical_cnl_grammar(lexicon)` —
  whole-surface string literals in slot-specific alternations (wrong-slot vocabulary is
  unparseable), per-quantity-row interval productions pairing each var's surface with its
  own unit terminal, basis-id production == Id's exact `[a-z][a-z0-9_.:-]*` grammar (the
  `smt_query.grammar` identifier shape), document production = the right-recursive
  `(rule <nl>)+` lowering. Committed + hash-pinned + drift-guarded (`schema.rs` pattern);
  registry entry `schema.clinical_cnl` is the route's decoding constraint. No open lexical
  productions.
- CNL AST (own type family, not ClinicalIr): CnlAtom `Concept(Id) | ConceptNegated(Id) |
  Interval{var, exactly one unsigned bound among ge|gt|le|lt, value ≥ 0}`; CnlContext
  `{any: Vec<Vec<CnlAtom>>}`; CnlException `{concept: Id, basis: nonempty sorted region
  refs}`; CnlRule `{context, action target + kind, direction+strength, certainty?, basis,
  exceptions}`; CnlDocument per the §5 row (document_id, grammar id+hash, per-rule AST +
  canonical JA text, text hash — acceptance re-renders and hash-locks canonical bytes).
  Validity two-layered, STRUCTURAL first (lexicon-free: nonempty rules/DNF/brackets,
  brackets sorted+deduplicated — set semantics, parse normalizes surface order — Id grammar,
  the interval register; §5 IrBundle coherence admits signed/two-sided interval shapes the
  grammar cannot write, so coherence-mirrored validity would bless unrenderable ASTs),
  then lexicon-scoped against the role view (refs resolve, slot roles admit positions,
  modality pair tail-backed, interval vars resolve, negative-occurrence bar). The parser
  and the model mint no ids.
- Bridge `to_ir(&CnlDocument, &segments) -> ClinicalIR`, deterministic: one ClinicalStatement
  per context-disjunct in document order; atoms partition into population/condition by the
  role view (never by id-namespace convention); ids `stmt.<k>`/`exc.<k>`/`bind.<k>` —
  document-order counters in `normalize.rs`'s id forms, document-local scope; each exception
  sentence yields one single-atom ExceptionClause PER split statement — a multi-disjunct
  rule clones its exception list into every emitted statement ((D1∨D2)∧¬E = (D1∧¬E)∨(D2∧¬E);
  bundle validation demands globally unique exception ids), `exc.<k>` counting emitted
  clauses statement-major then sentence order, each clause's region_ids = its own sentence's
  basis bracket verbatim; one Exact-status TerminologyBinding per distinct referenced
  concept, minted at first reference in post-split emission order (statement-major; per
  statement population, condition, action target, then exception clauses), system =
  lexicon.system, code = concept id, region_ids = the sorted union of the citing emitted
  statements' cited regions (provenance at the producer's grain, §5); statement
  source_region_ids = its rule's brackets' union (rule bracket + its clauses');
  source_segment_ids derive region→segment via the segments artifact. Preconditions,
  reject-checked at acceptance: every cited region resolves in the document graph
  (grounding: absent → `ai_hallucinated_source`, terminal) and anchors in exactly one
  segment (else a repairable schema reject naming the region).
- Acceptance (`single_cnl_accept`, the §9 accept-closure pattern): UTF-8 → parse (grammar
  language, parse errors repairable `ai_schema_violation`) → structural + lexicon-scoped
  validation (same code) → instrument self-check: re-render the parsed AST and re-parse;
  disagreement = `cnl_round_trip_mismatch`, terminal (§7.4) — the stored document carries
  the CANONICAL re-rendered text + hash (the raw emission stays in the cassette) → `to_ir`
  + grounding → accepted CnlDocument + ClinicalIR.
- Route `pipe.m3_single_cnl`, mirroring `pipe.m2_single_ir`'s shape: `m1.extract` →
  `m1.segment` → `m3.model_fill_cnl` (nondeterministic, output `cnl_document`) →
  `m3.bridge` (deterministic, `[cnl_document, segments]` → `clinical_ir`; recomputes `to_ir`
  from the accepted document — determinism makes it the accept-time result, asserted by
  hash) → `m2.assemble` → `m1.compile` → `m1.verify`. Provenance chain: `cnl_document`
  {origin `ai_generated`, `accepted_evidence_status`, inputs [graph, segments, cassette]} →
  `clinical_ir` {`deterministic_compiler`, `mechanical_evidence_status`, inputs
  [cnl_document, segments]} → `ir_bundle`; trace reuses the landed assemble/compile/verify
  node kinds — fill and bridge artifacts stay off-DAG per the landed §9 precedent, visible
  through §4.6 events, wrapper producers, and the input-hash chain.
- Determinism laws (property-tested; bounded deterministic enumeration, no new deps):
  parse∘render == id over valid ASTs; render∘parse == id over canonical documents; text-hash
  members recompute from stored texts under the frame assembly; `to_ir` is a pure function
  of (accepted document, segments); lexer segmentation is deterministic and complete over
  in-language input (pairwise prefix-freedom ⇒ maximal munch unique) — guarded by the
  lexicon lint, since the bnf Earley oracle proves membership only.
- Faithfulness projection (the §7.3 definition): compare accepted ClinicalIR against the
  reference derivation with provenance-grain fields excluded — TerminologyBinding
  `region_ids` and statement source region/segment fields — every remaining field, ids
  included, compared exactly on canonical bytes; the projection is pinned in metric code and
  named in the report.

### §10.5 M3 experiment: `exp.m3_cnl`

- Binding: `[pipe.m2_direct_smt (baseline), pipe.m2_single_ir, pipe.m3_single_cnl]` over the
  locked M1 test source groups, seed, budgets, and reference (§9 values); one run, one
  manifest, raw rows before deltas.
- New instruments: faithfulness + round-trip + emission-size rows (§7.3); the explorer.
- Explorer (`ckc explore runs/<run-id> --out <path>.html`): one deterministic self-contained
  HTML view over a completed run — per document × route the chain source spans → route-native
  accepted artifacts (CNL text / IR JSON / SMT) → verdicts, plus findings, metrics raw rows,
  and the delta table; embedded canonical data, committed hash-pinned viewer assets, zero
  network, zero servers; a view-only derived artifact (§4.4 `view_only`, renderer identity
  embedded) computed FROM attested run bytes, outside the attested output set — replay
  untouched. Exists because side-by-side inspection of unlike representations is promotion
  evidence (§10.2) the file tree serves poorly.
- Acceptance themes (roadmap unit `acceptance-m3` finalizes): all three routes execute over
  identical locked inputs; recorded model I/O replays byte-stably; round-trip identity 1.0
  over accepted CNL documents; faithfulness rows emit for both IR-landing routes (measured,
  never gated); expected conflict/no-conflict outcomes hold per reference for accepted
  translations; M1/M2 pins stay byte-identical except the scheduled lexicon-hash and
  report/metrics re-blesses; the explorer renders every route's chain; grammar/lexicon drift
  guards green; §0 vocabulary throughout; tag `accept/m3`.
- The decision this evidence feeds (§11): promote CNL investment (EN mirror, escape, from-IR
  rendering, findings quoting), hold candidate status pending the registered deconfounder,
  or retire it to the catalogue with its evidence.

## §11 M4 — Promotion, selection, and the route field

Intent: turn harness evidence into the architecture decision. M4 opens with a promotion
review over M1–M3 evidence and either selects the product composition or runs the single
next-most-decisive widening experiment; this section also carries the registered route field,
the deferred instruments, and the external-resource licensing discipline.

### §11.1 Promotion criteria

A candidate (route, representation, backend, or composition) is promoted toward product
architecture only through a review scoring EVERY criterion, evidence cited by run id or
commit; no criterion auto-scores into a single ranking — raw rows plus adversarial review
judgment (AGENTS.md criteria), landing as a SPEC amendment + registry status change through
the user:

| Criterion | Evidence source |
| --- | --- |
| Semantic fidelity | Conflict-task accuracy (primary) + faithfulness diagnostics over locked runs. |
| Clinically relevant coverage | Corpus breadth handled: growth-set variants, claim completeness at scale (§11.4). |
| Reliability + failure transparency | Acceptance rate, k-sample stability, typed failure-taxonomy shares (§7.4); silent failure classes disqualify. |
| Source-grounded inspectability | Grounding checks + recorded human inspection of the candidate's audit surface (explorer, §10.5). |
| Provenance + replay quality | Replay match rate, cassette completeness, manifest attestation coverage. |
| Implementation + conceptual complexity | Module/LOC/spec-surface counts, declared over-approximation classes, re-bless surface. |
| Model + runtime cost | §7.3 resource rows: recorded calls, repairs, wall-clock, emission bytes. |
| Performance + scalability | Runtime per document; constraint-compile and lexicon-growth cost curves. |
| Maintainability + extensibility | Measured cost of adding a concept, rule, language, or backend on a real extension. |

### §11.2 Selection, migration, retirement

- Selection trigger: a candidate dominates or acceptably trades off §11.1 across ≥2 corpus
  generations (the locked M1 set plus one growth set), with no unresolved §10.2 confound
  that a bounded experiment could remove.
- Canonicalization: the selected composition's modules become the compiler core; the harness
  (corpus, acceptance, evaluation, replay, ledger) persists as its standing benchmark and
  regression instrument; §15 gates keep governing claims. This repository evolves in place
  into the compiler — no rewrite-from-scratch.
- Baseline reproducibility: losing and retired routes stay replayable from committed
  cassettes + locked manifests at their acceptance tags (`accept/m<n>`); registry entries
  persist with status, never deleted.
- Retirement gate: a candidate layer or backend retires when dominated on every §11.1
  criterion it uniquely served across two corpus generations, or when its unique evidence is
  absorbed by a successor; retirement = registry status + §3 role-table update, design text
  git-resident.
- Migration gate: moving a capability between harness and route (either direction) requires
  demonstrating the invariant it protects still holds — in particular, anything entering the
  harness must remain representation-neutral (§10.1).

### §11.3 Route field (registered candidates; each builds only on evidence demand)

| Route | Shape / hypothesis |
| --- | --- |
| `route.grammar_ir` | Grammar-constrained JSON-IR emission — the §10.2 deconfounder isolating constraint mechanism from surface class. |
| `route.reason_ir` | Unconstrained free-text reasoning stage → constrained single-IR commit; only the commit is accepted. Archived-PoC prior: the sole form to beat single-IR faithfulness (0.70→0.90) at a sampling-variance cost. |
| `route.stacked_ir` | Model fills a stack of existing IR forms (e.g. PICO → rule rows); deterministic compile. |
| `route.ir_hop_chain` | Chain of adjacent, deliberately similar IR dialects — do short hops tame non-determinism better than one leap? |
| `route.ckc_layered` | Model fills CKC layers stage by stage (segment → statement → rule), each constrained. |
| `route.ckc_rec_dsl` | Compact line-oriented record DSL; the CNL's token-compactness ablation (PoC prior: stable wrong-polarity failure — assess both §7.3 families). |
| `route.slot_cnl` | Labeled-slot CNL (FRET/BRIDGE-Wiz style) — the readability-versus-emission midpoint. |

Design dimensions recorded per candidate: token compactness, constraint strength, semantic
distance per hop, constraint placement, layer composability (the §12 search-space
coordinates). Baseline deltas measure against both `route.direct_smt` and the best standing
route.

Deferred ClinicalCNL capabilities (promotion-phase; the pre-reset design at the §14 retrieval
commit is the seed — mine it, never re-derive): EN mirror surface (two concrete syntaxes, one
AST, cross-language parse agreement); unregistered-concept escape (sole open lexical
production, payload contract, `cnl_unregistered_concept` proposal stream feeding lexicon
growth); from-IR audit rendering (render accepted IR from ANY route as CNL: `from_ir` with
round-trip laws, segment-closure region provenance, expressibility predicate); findings CNL
quoting via the rule-origin map; lexicon accretion pipeline (cluster → triage → draft →
deterministic gates → human review → bless; row-stability law: rows cited by accepted
documents freeze, growth is append-only, edits are typed migration events; grammar-scale
posture: demand-sliced constraint subsetting before any grammar-opening amendment).

### §11.4 Deferred instruments (build on evidence demand)

- M4 conflict kinds (§6 table): `numeric_threshold_empty_intersection`,
  `strict_factual_contradiction`, `terminology_incoherence`, `condition_unsatisfiable`,
  `exception_resolved_conflict`; the Q0 self-check and raw-Q1 exception-stripped view (§6).
- Corpus growth set: 4–6 synthetic documents sharing populations/actions/conditions (reuse
  pressure), deterministic metamorphic variants (punctuation, kana/kanji, section order),
  indirect-rendering variants (synonyms, oblique deontics, convention terms, negated
  phrasing — the axis that dents faithfulness while verdicts stay intact), threshold- and
  factual-conflict cases.
- Component store + reuse/compactness/convergence exports (`component_reuse_graph.json`,
  `compactness_front.json`); `pipe.direct_rule_to_smt`, the deterministic
  architecture-ablation baseline (layered-minus-direct delta); path visualizations (§7.1).
- Model-free coverage experiment (`exp.m4_coverage`, experiment 2's measurement): set A
  builds mappings, set B runs `runtime_ai: false`; coverage, accuracy versus reference,
  mapping-set size, zero application-phase model calls.
- Claim-completeness instrument: a recorded classifier labels every SourceDocumentGraph
  region normative-candidate or not; every normative-candidate region ends claimed by an
  accepted rule or covered by a typed residual (`normative_region_unclaimed`, §7.4);
  calibrated against set A/B references, load-bearing from M6.
- LP explanation lane per the §6 LP profile; `registry/methods.yaml` seeded from the
  compendium (§14).
- Locked-measurement evaluator identity: the run manifest freezes test source, reference,
  lexicon, and metric-code hashes (`evaluator_lock.json` extends this identity with full
  semantics in M5).
- Informalization round-trip metamorphic (accepted IR → canonical CNL → route re-parse →
  normalized-hash compare) once from-IR rendering exists.

### §11.5 External resources: licensing + evidence rows

CKC is GPL-3.0-or-later (LICENSE; relicensed 2026-07-12 from an Apache-2.0 WITH
LLVM-exception text for a uniform governing license — the prior mixed posture was a cost,
never an impossibility). External resources (terminologies, lexicons, corpora, tools) enter
only behind a per-resource, per-version evidence row recorded BEFORE acquisition or mining:
rights holder, source URL + snapshot hash, as-of date, and the specific permissions for
acquisition, internal processing/mining, derivative authoring, committing derived rows, and
redistribution (`PermissionRecord` + `G-SOURCE-PERMISSION`), never inherited across resources
or versions. License compatibility answers permission alone — never compliance: the
obligations permission carries (notice retention, attribution, corresponding-source
conveyance) land in the evidence row and are recorded and met per resource. Copyleft as a
class is never a rejection ground; judge exact version, combination direction, resulting-work
license, and obligations (GPL-2.0-only stays GPLv3-incompatible; AGPL-3.0 combines but adds
network obligations). Standing verdicts: Attempto-family LGPL source (APE, AceRules, Codeco)
is direct-port-compatible with attribution — technical-fit verdicts stand, no port planned;
Clex is license-compatible as an EN-side candidate-mining seed behind its evidence row
(upstream commit 20960a5c header grant + COMLEX/LDC derivation), content import rejected on
row-level fit alone; SNOMED CT / MedDRA/J / LOINC stay gated by their own fee/terms grants
(§13.1). Adopted ACE precedents: the Clex/Ulex committed-core + per-corpus accretion
architecture and decree-authored forms; rejected: direct Clex content import, Ulex precedence
shadowing (collisions are hard gates here, never precedence).

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
  J-STAGE/JATS XML, and PMDA e-PI XML (structured sections — 禁忌/効能/用法 — and the future
  cross-source counterpart; permissions evidence-gated per §11). Licensed textbook EPUB/PDF joins as a
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
- Completeness: the §11.4 claim-completeness instrument is load-bearing here — real corpus slices
  carry reference labels for at most a few sources, so every normative-candidate region must
  end claimed or typed-residual (`normative_region_unclaimed`); a skipped recommendation is a
  visible diagnostic, never a silent loss.
- Terminology: MEDIS standard code tables (病名/HOT) as the first external systems behind the
  TerminologyBinding requirements; version-pinned snapshots; JLAC10/11 laboratory codes registered
  next; license-encumbered vocabularies (SNOMED CT, MedDRA/J, LOINC) stay registry-listed until
  licensing evidence exists; the same snapshots double as seed resources for the §11.3 lexicon
  accretion pipeline (candidate synonym surfaces, gloss bases, external codes — per-resource,
  per-version evidence rows behind `G-SOURCE-PERMISSION` BEFORE acquisition or mining, §11.5).
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
  primary, English glosses linked per span; corpus overview, rule browser (rules quoted in the promoted audit surface — §10.4 CNL if
  promoted per §11, Japanese primary), finding list with
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
extends `schemas|prompts` with CNL grammar and prompt entries (§10.4); M4 adds `methods`, the
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
The pre-reset ClinicalCNL full design and M3 plan (superseded 2026-07-12) resolve at commit
`ecc19d3` (`git show ecc19d3:SPEC.md` §10, `git show ecc19d3:.agent/roadmap.md`,
`git show ecc19d3:.agent/memory.md`) — mine them when a §11.3 deferred capability is
promoted, never re-derive.
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
