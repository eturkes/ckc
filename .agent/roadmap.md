# CKC Roadmap

Phases contain top-level tasks; top-level tasks may contain indented subtasks.
See `.agent/prompt.md` for the session workflow that governs progression.

## Phase 0: Proof-Carrying Research Kernel

- [x] 0.1 Project scaffold
- [ ] 0.2 CKC schema v0 (SPEC 10)
  - [x] 0.2.1 Foundation: ID newtypes, semantic profiles (SPEC 9), shared enums, RFC 8785 canonical JSON
  - [x] 0.2.2 Source + terminology types: CorpusDocument, SourceSpan, ExtractedTable, Concept, TerminologyBinding
  - [ ] 0.2.3 Evidence + formalization types: PICOFrame, EtDFrame, EvidenceAtom, ClinicalClaim, Rule, Norm, Action
  - [ ] 0.2.4 Structured artifact types: DecisionTable, DecisionRow, WorkflowFragment, EventNarrative, PatientCase, ExecutionWitness
  - [ ] 0.2.5 Verification + assurance types: Conflict, ArgumentGraph, Certificate, AssuranceNode, AuditTrace
  - [ ] 0.2.6 JSON Schema generation and golden byte tests
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
