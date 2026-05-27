# CKC Roadmap

Phases contain top-level tasks; top-level tasks may contain indented subtasks.
See `.agent/prompt.md` for the session workflow that governs progression.

## Phase 0: Proof-Carrying Research Kernel

- [x] 0.1 Project scaffold
- [ ] 0.2 CKC schema v0 (SPEC 10)
  - [ ] 0.2.1 Foundation: workspace deps (serde, sha2, schemars), ID newtypes for every SPEC 10 object, SemanticProfile enum (SPEC 9), shared enums (BindingStatus, CertificateClass, ConflictClassification, Severity, etc.), RFC 8785 canonical JSON serializer, content-hash fn (sha256 of canonical bytes). Gate: `cargo test -p ckc-core` passes; canonical JSON determinism and ID serde round-trip tests pass.
  - [ ] 0.2.2 Source and terminology types: CorpusDocument, SourceSpan, ExtractedTable, Concept, TerminologyBinding (exact fields per SPEC 10). Gate: serde round-trip tests; canonical JSON byte stability for one fixture instance of each type.
  - [ ] 0.2.3 Evidence and clinical formalization types: PICOFrame, EtDFrame, EvidenceAtom, Norm, Action, Rule, ClinicalClaim (exact fields per SPEC 10). Gate: serde round-trip; profiles field validates against SemanticProfile.
  - [ ] 0.2.4 Structured artifact types: DecisionTable, DecisionRow, WorkflowFragment, EventNarrative, PatientCase, ExecutionWitness (exact fields per SPEC 10). Gate: serde round-trip tests.
  - [ ] 0.2.5 Verification and assurance types: Conflict, ArgumentGraph, Certificate, AssuranceNode, AuditTrace (exact fields per SPEC 10). Gate: serde round-trip tests.
  - [ ] 0.2.6 JSON Schema generation and golden byte tests: generate JSON Schema from all Rust types (schemars), write golden canonical JSON fixtures covering every type, verify content hashes are deterministic across serialization round-trips. Gate: `cargo test -p ckc-core` passes; golden files committed; schema files committed.
- [ ] 0.3 Normal Form (SPEC 11)
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
