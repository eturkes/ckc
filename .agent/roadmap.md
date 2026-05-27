# CKC Roadmap

Phases contain top-level tasks; top-level tasks may contain indented subtasks.
See `.agent/prompt.md` for the session workflow that governs progression.

## Phase 0: Proof-Carrying Research Kernel

- [x] 0.1 Project scaffold
- [x] 0.2 CKC schema v0 (SPEC 10)
  - [x] 0.2.1 Foundation: workspace deps (serde, sha2, schemars), ID newtypes for every SPEC 10 object, SemanticProfile enum (SPEC 9), shared enums (BindingStatus, CertificateClass, ConflictClassification, Severity, etc.), RFC 8785 canonical JSON serializer, content-hash fn (sha256 of canonical bytes). Gate: `cargo test -p ckc-core` passes; canonical JSON determinism and ID serde round-trip tests pass.
  - [x] 0.2.2 Source and terminology types: CorpusDocument, SourceSpan, ExtractedTable, Concept, TerminologyBinding (exact fields per SPEC 10). Gate: serde round-trip tests; canonical JSON byte stability for one fixture instance of each type.
  - [x] 0.2.3 Evidence and clinical formalization types: PICOFrame, EtDFrame, EvidenceAtom, Norm, Action, Rule, ClinicalClaim (exact fields per SPEC 10). Gate: serde round-trip; profiles field validates against SemanticProfile.
  - [x] 0.2.4 Structured artifact types: DecisionTable, DecisionRow, WorkflowFragment, EventNarrative, PatientCase, ExecutionWitness (exact fields per SPEC 10). Gate: serde round-trip tests.
  - [x] 0.2.5 Verification and assurance types: Conflict, ArgumentGraph, Certificate, AssuranceNode, AuditTrace (exact fields per SPEC 10). Gate: serde round-trip tests.
  - [x] 0.2.6 JSON Schema generation and golden byte tests: generate JSON Schema from all Rust types (schemars), write golden canonical JSON fixtures covering every type, verify content hashes are deterministic across serialization round-trips. Gate: `cargo test -p ckc-core` passes; golden files committed; schema files committed.
- [ ] 0.3 Normal Form (SPEC 11)
  - [x] 0.3.1 NF pipeline scaffold and text normalization (passes 1-2): create `nf.rs` module in ckc-core. Define `NfContext` (rewrite log, diagnostic accumulator) and `Normalize` trait dispatching on SemanticProfile. Pass 1: preserve raw_text/raw fields verbatim. Pass 2: Unicode NFKC, Japanese punctuation normalization (full-width ↔ half-width), whitespace collapse on derived text fields (nfkc_text, search_text, display_text, gloss_ja, gloss_en, labels). Add `unicode-normalization` workspace dep. Gate: `cargo test -p ckc-core` passes; unit tests confirm Japanese text normalization and raw field preservation.
  - [x] 0.3.2 Structural normalization (passes 3-5): Pass 3: alpha-normalize variable names in Rule antecedent/consequent/context `Value` trees using stable canonical renaming. Pass 4: sort `source_span_ids`, `evidence_atoms`, commutative `and`/`or` operands, and unordered set fields by canonical comparison. Pass 5: preserve order for temporal sequences, priority chains, decision-table rows (order-sensitive hit policies), and workflow transitions. Gate: two Rules differing only in commutative antecedent order produce identical NF canonical bytes and digest.
  - [x] 0.3.3 Domain normalization stubs (passes 6-8): Pass 6: normalize Action constructors (canonical action_type casing, sort parameter keys). Pass 7: normalize quantities (numeric precision, unit string to canonical UCUM form). Pass 8: terminology normalization trait stub (identity transform until 0.6 e-graph integration). Gate: Action/quantity fixtures normalize deterministically; pass 8 compiles as identity.
  - [x] 0.3.4 Complex structure canonicalization (passes 9-11): Pass 9: Japanese clinical modality lexicon stub (minimal phrase → deontic-projection map for toy scenarios, preserving original_modality_phrase_ja). Pass 10: canonicalize DecisionTable — sort rows for non-order-sensitive hit policies, normalize cell `Value` trees. Pass 11: canonicalize ArgumentGraph (stable node/edge ordering by IDs) and WorkflowFragment (stable state/transition ordering by IDs). Gate: DecisionTable with shuffled rows (UNIQUE policy) yields identical NF; ArgumentGraph with shuffled edges yields identical NF.
  - [ ] 0.3.5 Stable IDs, diagnostic ordering, and idempotency proofs (passes 12-13 + invariants): Pass 12: generate deterministic stable IDs from normalized content hash + source anchor IDs. Pass 13: sort accumulated diagnostics by (stage, code, span). Property-based tests (`proptest`): NF(NF(x)) == NF(x) for Rule, ClinicalClaim, DecisionTable, ArgumentGraph. Golden NF fixtures for all types with committed expected bytes. Gate: `cargo test -p ckc-core` passes; proptest idempotency passes; golden NF files committed.
- [ ] 0.4 Content-addressed store
- [ ] 0.5 Toy fixtures (SPEC 20 Phase 0 scenarios)
- [ ] 0.6 Terminology and e-graph
- [ ] 0.7 Retrieval fixture
- [ ] 0.8 Compiler targets (SPEC 14)
- [ ] 0.9 Verification and certificates (SPEC 13)
- [ ] 0.10 Conflict detection (SPEC 15)
- [ ] 0.11 CLI commands (SPEC 18)
- [ ] 0.12 Report and UI
- [ ] 0.13 Replay and determinism
