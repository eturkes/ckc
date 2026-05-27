# CKC Roadmap

Progress tracker across sessions. Mark tasks `[x]` when done. Derived from
SPEC.md Section 20. Only add implementation-level subtasks here that help
orient a fresh session; phase-level descriptions live in SPEC.md.

## Phase 0: Proof-Carrying Research Kernel

Target: `ckc demo toy-research-kernel --replay --out runs/toy`

### 0.1 Project scaffold
- [x] Directory structure per SPEC Section 19
- [x] Rust workspace with all crates
- [x] License, gitignore, toolchain config
- [x] Agent memory/scratchpad system
- [x] Roadmap and future-session prompt
- [x] Git initialization and initial commit

### 0.2 CKC schema v0
- [ ] Core types in `ckc-core`: SourceSpan, Concept, TerminologyBinding,
      ClinicalClaim, Rule, Norm, Action, Conflict, Certificate, etc.
- [ ] JSON Schema generation or equivalence tests
- [ ] Schema version identifier

### 0.3 CKC Normal Form
- [ ] Deterministic rewrite pipeline (SPEC Section 11 passes)
- [ ] RFC 8785 canonical JSON serialization
- [ ] sha256 content hashing
- [ ] Property tests: NF(NF(x)) == NF(x)
- [ ] Golden tests for canonical JSON bytes

### 0.4 Content-addressed store
- [ ] Filesystem CAS in `ckc-store`
- [ ] Artifact metadata (schema version, producer, hashes, replay command)
- [ ] DuckDB integration for report joins

### 0.5 Toy fixtures
- [ ] Sepsis/beta-lactam recommendation + contraindication
- [ ] Japanese spelling variants (βラクタム / ベータラクタム / β-ラクタム)
- [ ] Decision table with overlapping rows + gap witness
- [ ] Event Calculus allergy persistence narrative
- [ ] Source span and table fixture JSON

### 0.6 Terminology and e-graph
- [ ] e-graph synonym fixture in `ckc-term`
- [ ] Spelling/brand/generic variant normalization
- [ ] egglog integration or standalone e-graph

### 0.7 Retrieval fixture
- [ ] Japanese analyzer fixture (Kuromoji/Sudachi/MeCab)
- [ ] BM25 retrieval baseline fixture
- [ ] Retrieval output artifact

### 0.8 Compiler targets
- [ ] CKC -> SMT-LIB (Z3 deontic, interval, decision-table)
- [ ] CKC -> Z3 Optimize/MaxSMT repair
- [ ] CKC -> cvc5 proof artifact (if feasible)
- [ ] CKC -> Clingo ASP (defeasible, argumentation, Event Calculus)
- [ ] CKC -> Datalog/Souffle
- [ ] CKC -> RDF/Turtle + SHACL shapes
- [ ] CKC -> Lean theorem
- [ ] CKC -> TLA+/Alloy stubs (generated meta-spec)
- [ ] CKC -> DMN/FEEL (decision table export)

### 0.9 Verification and certificates
- [ ] Z3 witness/unsat core collection
- [ ] MaxSMT repair output
- [ ] Clingo model collection
- [ ] SHACL validation report
- [ ] Lean compilation (zero sorry/admit)
- [ ] Certificate graph in `ckc-cert`
- [ ] Certificate class assignment (C0-C7)

### 0.10 Conflict detection
- [ ] Norm conflict witness (recommend-for vs recommend-against)
- [ ] Decision table overlap/gap detection
- [ ] Temporal/Event Calculus conflict
- [ ] Minimal conflict sets

### 0.11 CLI skeleton
- [ ] `ckc` binary with clap subcommands
- [ ] `ckc demo toy-research-kernel --replay --out <dir>`
- [ ] Structured diagnostic output

### 0.12 Report and UI
- [ ] Bilingual report JSON
- [ ] Static UI card (SvelteKit)
- [ ] Assurance seed artifact

### 0.13 Replay and determinism
- [ ] Replay manifest
- [ ] Hash comparison across repeated runs
- [ ] `ckc replay <manifest>` command

## Phase 1: Extraction and Span Registry
(subtasks to be added when Phase 0 completes)

## Phase 2: Terminology and Retrieval Substrate
(subtasks to be added when Phase 1 completes)

## Phase 3: Candidate Formalization and Semantic Firewall
(subtasks to be added when Phase 2 completes)

## Phase 4: Compiler Portfolio
(subtasks to be added when Phase 3 completes)

## Phase 5: Corpus-Scale Conflict Detection
(subtasks to be added when Phase 4 completes)

## Phase 6: Bilingual UI and Manuscript Package
(subtasks to be added when Phase 5 completes)

## Phase 7: Future CDS/SaMD Bridge
(subtasks to be added when Phase 6 completes)
