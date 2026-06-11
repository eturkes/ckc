# Formal Specification, Proof Assistants, and Proof Engineering

## 1. Lean 4 + Mathlib + Aesop/Tactic Automation

### Purpose
Dependently typed FP language + interactive theorem prover for software verification and mechanized math; one unified language for definitions, programs, proofs, metaprograms.

### Maintainer/Standards body
Lean FRO (founded 2023, led by Leonardo de Moura). Stable 4.29.1 (2026-04-14); 4.30.0 in RC (4.30.0-rc2, 2026-04-16); earlier 2026: 4.28.0 (02-17), 4.29.0 (03-27). Mathlib4: `leanprover-community`. Aesop: Jannis Limperg under `leanprover-community`.

### Conceptual model
Calculus of Constructions + inductive types, universe polymorphism, definitional proof irrelevance for `Prop`, quotient types. Proofs are terms; tactics are `MetaM`/`TacticM` metaprograms elaborating to proof terms; sources compile to `.olean`. Mathlib: `structure`/`class` hierarchies + `instance` resolution. Aesop: white-box best-first proof search, user-tagged rule sets (`@[aesop safe]`, `unsafe`, `norm`); `simp`: confluent rewriter. `grind` (4.20+): linarith + cutsat + commutative-ring (Nullstellensatz certificates) + congruence closure.

### Expressiveness/Semantics
Classical or constructive at user option (`Classical.em` axiom available). Arbitrary structures: rings, topological spaces, measure theory, category theory, schemes, perfectoid spaces. Decidability is a first-class type class. Computation: `rfl` (reduction), `#eval` (bytecode), `native_decide` (native).

### Composability/Modularity
Lake build + Reservoir package index; `import` modules; type-class inheritance. Mathlib: global namespace, prefix conventions, ~226 MB source distribution. **CSLib** (The Lean Computer Science Library, `leanprover/cslib` on GitHub + Reservoir, Apache-2.0) depends on Mathlib, extends to CS: bundled `ReductionSystem` + `LTS` (Labelled Transition System) structures auto-derive multistep reduction, reachable states, confluence; `HasContext` (single-hole term contexts); `Congruence` (constructor-preserved equivalence relations). Its architectural "Spine" (arXiv:2602.15078) is designed for reuse across formalization initiatives.

### Suitability for autoformalization to IR
Strongest LLM autoformalization target among proof assistants; bulk of recent research (DeepSeek-Prover-V2, Kimina-Prover, LeanDojo, Goedel-Prover, Herald, AlphaProof) targets Lean 4. Idempotency via `simp` normal forms, `Decidable` canonical Booleans, `grind` normalization. Regeneration after edits mostly stable across Mathlib versions via deprecation aliases.

### Formal verification potential
Trusted kernel ~6 KLOC C++ (reference kernel; Lean4Lean Lean-in-Lean reimplementation runs 20–50% slower); small TCB. Decision procedures: `linarith`, `omega`, `polyrith`, `positivity`, `decide`, `bv_decide`, `grind`. SMT bridge: `lean-smt` (cvc5; proof reconstruction via Ethos/reflective rules). No native sledgehammer; `exact?`, `apply?`, `aesop?`, LLM-based `LLMlean`/`copilot`.

### Tooling/Ecosystem maturity
vscode-lean4 (widgets; module-hierarchy view in 4.22+). Zulip primary chat; nightly Mathlib CI; doc-gen4 HTML docs; `lake exe cache get`. **CSLib** (cslib.io; GitHub `leanprover/cslib`, 556 stars, 145 forks, Apache-2.0; release v4.29.0, 2026-03-31): aims to be Mathlib-for-CS. Lead: Fabrizio Montesi (University of Southern Denmark, FORM Centre for Formal Methods and Future Computing). Steering: Clark Barrett (Stanford/AWS), Swarat Chaudhuri (Google DeepMind/UT Austin), Jim Grundy (AWS), Pushmeet Kohli (Google DeepMind), Leonardo de Moura (Lean FRO/AWS). Funders: Renaissance Philanthropy, AWS, Google DeepMind, FORM/SDU, Stanford CENTAUR, ERC (CHORDS 101124225), NSF (CCF-2220991). Formalized content: Milner's CCS (commutativity/distributivity of parallel composition; bisimilarity is a congruence); System F with subtyping (locally nameless); Hennessy-Milner Logic + Hennessy-Milner theorem (arXiv:2602.15409); behavioral equivalences (bisimilarity, weak bisimilarity, simulation, trace equivalence) parametric over any LTS; combinatory logic; linear logic; automata theory; algorithms with time-complexity bounds. `grind` first-class: 314/338 declarations used it from initial commit; mean saving 7.1 lines/theorem; bisimilarity proofs save 15–39 lines; System F ~45% fewer lines than comparable Rocq development (arXiv:2602.15078). 2026 roadmap: most algorithms of a typical undergrad algorithms/data-structures course + most models/logics of a theory-of-computation course, type-system specification interfaces, nominal transition system logics (pi-calculus). Area maintainers: Alexandre Rademaker (Atlas Computing/FGV), Sorrachai Yingchareonthawornchai (ETH Zurich), Christopher Henson (Drexel), Kim Morrison.

### Japan-specific considerations
TPP (Theorem Proving and Provers) annual JP workshop covers Lean alongside Coq/Isabelle. Jacques Garrigue (Nagoya U) teaches Lean. JSSST SIG-PPL runs PPL Workshop + Summer School with Lean tutorials. No dedicated JP-government-funded Lean library; ERATO MMSD (NII, Ichiro Hasuo, JST grant JPMJER1603, 2016-10–2025-03) trained researchers across the spectrum.

### Interoperability
Out: Lean→Dedukti/Lambdapi translators (Deducteam). Cat 1: native JSON/YAML parsers ingest FHIR Clinical Reasoning/CQL/DMN; CQL semantics embeddable via inductive types. Cat 2: ontology IDs → Lean constants; `Finset`/`List`/`Multiset` fit guideline-recommendation semantics. Cat 3: OWL DL fragments embed into decidable Lean fragments via `Decidable` typeclass; RDF triples → typed relations. Cat 4: Lean→Dedukti enables independent checking against Isabelle/Rocq encodings.

### Limitations/Known issues
Tactic-syntax churn across versions; frequent Mathlib bumps. Full Mathlib builds take hours (cache mitigates). Universe-polymorphism elaboration errors notoriously opaque. `native_decide` enlarges TCB (trusts C compiler). `simp`/`aesop` can be slow on large rule sets.

### Training data proxy
Largest LLM training-data footprint among proof assistants. CSLib adds 556 stars, three arXiv papers (2602.04846, 2602.15078, 2602.15409), Lean Together 2026 presentation materials; its AI-integration area targets training datasets + AI-assisted contribution tools. Mathlib: >1.9M formally verified lines, ~404,000 declarations (~130,800 definitions + ~273,800 theorems), 772 contributors per live `leanprover-community.github.io/mathlib_stats.html` (May 2026). Mathlib Initiative roadmap (2025-10–2026-09): review-cycle response <1 week for 90% of cycles by 2026-09; current median ~2 weeks, ~300 PRs backlog. Zulip archive (May 2026): 12,843 topics in "new members" channel, 6,722 mathlib4, 7,667 lean4, 9,924 general; Zulip case study: "hundreds of active participants". Benchmarks: miniF2F (488 problems), ProofNet (371), PutnamBench (672 Lean 4 / 640 Isabelle / 412 Coq formalizations of 640 Putnam theorems; 1,724 total). DeepSeek-Prover-V2-671B: 88.9% on miniF2F-test (Pass@8192); 49/658 PutnamBench. AlphaProof (DeepMind, 2024-07; Nature paper "Olympiad-level formal mathematical reasoning with reinforcement learning", Hubert et al., published 2025-11-12): silver-medal-equivalent at IMO 2024 (4/6 problems combined with AlphaGeometry 2), using ~80M autoformalized Lean problems (translated from ~1M natural-language problems) for RL.

## 2. Rocq Prover + Stdlib/MathComp/Iris as a Secondary Proof Ecosystem

### Purpose
Mature interactive proof assistant (formerly Coq): dependently typed programming + machine-checked proof. Renamed "The Rocq Prover" at the 9.0 release, 2025-03-12.

### Maintainer/Standards body
INRIA leads; community + Rocq Consortium govern. Releases: 9.0 (2025-03), 9.1 (2025-09), 9.1.1 (2026-02), 9.2.0 (2026-03). Coq Platform 2025.01.0 (2025-02) bridges legacy Coq 8.20.1.

### Conceptual model
Calculus of Inductive Constructions (CIC): predicative `Type` hierarchy, impredicative `Prop`/`SProp`, primitive projections, native integers/floats/arrays. Spec language Gallina; tactics Ltac (legacy) + Ltac2 (typed). Extraction to OCaml/Haskell/Scheme. SSReflect (small-scale reflection; default-distributed since Coq 8.7): concise proof style underpinning MathComp.

### Expressiveness/Semantics
Constructive by default; classical axioms (LEM, choice, functional extensionality, propositional extensionality) optional + orthogonal. HoTT/UniMath univalence opt-in. Iris: deep-embedded higher-order concurrent separation logic; 2025 Most Influential POPL Paper Award ("Iris: Monoids and Invariants as an Orthogonal Basis for Concurrent Reasoning", POPL 2015); 2023 Alonzo Church Award to Birkedal, Bizjak, Dreyer, Jourdan, Jung, Krebbers, Sieczkowski, Svendsen, Swasey, Turon (presented ICALP 2023, Paderborn).

### Composability/Modularity
ML-style functor modules + canonical structures + type classes + unification hints. Three coexisting patterns: packed classes (MathComp), type classes (`stdpp`/Iris), Hierarchy-Builder. 9.0 split Stdlib into `rocq-core` (Corelib) + `rocq-stdlib`. Opam; Dune.

### Suitability for autoformalization to IR
Less LLM-targeted than Lean. Competing styles (vanilla, SSReflect, Equations, MathComp) hurt convergence/idempotency. Stronger when fixed to MathComp+SSReflect: rigid bullet structure, `Move=>`/`apply:` discipline, canonical-form normalization via `rewrite`. Lambdapi can mediate as IR.

### Formal verification potential
Kernel ~10 KLOC OCaml; small TCB. `vm_compute`/`native_compute` enable proof-by-reflection (SMTCoq, CoqHammer's reconstructor, micromega/lia/lra/nra/psatz). CoqHammer calls Vampire/E/Z3, reconstructs proofs. Iris: program-logic reasoning for concurrent imperative code.

### Tooling/Ecosystem maturity
VsCoq language server (in Coq Platform 2025); CoqIDE; Proof General. opam-coq-archive index. Rocq Platform 2025.08.3 bundles MathComp, Iris, MathComp-Analysis, Equations, ELPI.

### Japan-specific considerations
AIST **Reynald Affeldt** maintains: **MathComp-Analysis** (co-maintained with Cyril Cohen et al.; tutorial "An Introduction to MathComp-Analysis" dated 2025-01-13, updated 2025-10-24 reflecting Coq→Rocq rename); **monae** (monadic-effect hierarchy in Rocq, GitHub `affeldt-aist/monae`); **infotheo** v0.9.7 (2026-03 — information theory + linear ECCs; Rocq 9.0–9.1, MathComp ≥2.4.0). Contributors: Jacques Garrigue (Nagoya), Kazuhiko Sakaguchi (Tsukuba), Takafumi Saikawa (Nagoya). Nagoya U runs Coq/SSReflect courses (Garrigue). Atsushi Igarashi's group (Kyoto U): HELMHOLTZ (Coq-based verification of Tezos/Michelson smart contracts). Historical: Affeldt–Kobayashi mail-server verification (ISSS 2003).

### Interoperability
Strong Lambdapi/Dedukti export. SMTCoq imports LFSC/CVC5 certificates; CoqHammer reconstructs from E/Vampire/Z3. Iris formalizes Hoare-style guideline-step specs. Cat 1: Iris+HeapLang or Iris-OCaml model imperative pipelines; CompCert-style C-extraction available. Lean interop via Mathport/Mathlib4 (historical); Isabelle via Dedukti (Deducteam translators).

### Limitations/Known issues
Steeper learning curve than Lean; SSReflect idiom unfamiliar to newcomers. Universe inconsistencies hard to debug. Multiple incompatible "standard libraries". Native compute not yet on OCaml 5 for all architectures (re-enabled in 9.2.0 for some x86 setups). LLM autoformalization quality lags Lean.

### Training data proxy
GitHub: 452 repos tagged "Rocq Prover" (May 2026); aggregate `coq` topic count requires JS rendering, not officially published. Rocq Zulip very active. Stdlib + MathComp: hundreds of KLOC. Iris repo large; annual PLDI/ICFP/POPL papers. CoqGym, PISA-coq, CoqStoq benchmarks exist, smaller than miniF2F.

## 3. Isabelle/HOL + AFP/Sledgehammer as an Independent Audit Target

### Purpose
Generic logical framework instantiated to HOL. Mature, industrially deployed (seL4, CakeML, AFP). Independent audit target for cross-checking Lean/Rocq formalizations.

### Maintainer/Standards body
TUM (Tobias Nipkow) + Cambridge (Larry Paulson) + Innsbruck (Makarius Wenzel: Isabelle/Isar/PIDE). Current: Isabelle2025-2 (2026-01), shipping with Isabelle2025/AFP-2025.

### Conceptual model
Pure (intuitionistic typed λ-calculus + meta-implication ⟹ + meta-universal ⋀) = meta-logic; HOL = object logic. **Isar**: structured human-readable proof language. Proofs kernel-checked via primitive inference rules. PIDE: asynchronous document-oriented prover interface.

### Expressiveness/Semantics
Classical HOL, rank-1 polymorphism + Haskell-style type classes. Axiom of choice built-in. Less expressive than CIC (no dependent types) but enormously more automation. Locales: parameterized theories with refinement.

### Composability/Modularity
Locales (inheritance + instantiation), type classes, sessions (build units). AFP entries = independent sessions with metadata. Theory imports form a DAG.

### Suitability for autoformalization to IR
Excellent semantic stability; no definitional-equality dependence → textually robust proofs. Isar's declarative `have ... show` closer to natural-language proof than Lean tactic mode, but Lean has more training data. Neural-Isabelle work (Magnushammer, PISA, Baldur, Thor) leverages Sledgehammer reconstruction. A "minimalist proof language" for neural theorem proving over Isabelle/HOL (arXiv 2507.18885): pass@1 69.1% on PISA.

### Formal verification potential
**Sledgehammer**: dispatches goals to ATPs (E, SPASS, Vampire) + SMT (Z3, CVC4/5, veriT), reconstructs via Metis or `smt`. User's guide: Jasmin Blanchette (LMU München, 2025-03-13). Isabelle 2025.4+: reconstructs paramodulation/instantiation evidence from Z3/cvc5 Alethe-format proofs (arXiv 2508.20738). Kernel ~few KLOC SML.

### Tooling/Ecosystem maturity
Isabelle/jEdit (default IDE), Isabelle/VSCode (Documentation/Symbols/Sledgehammer panels in 2025-2), `isabelle build` CLI. AFP stats (live, May 2026): 973 entries, 593 authors, ~315,100 lemmas, ~5.17M LoC; AFP-2025 release ≈5,300 kLOC vs Isabelle2025 distribution ≈900 kLOC (HOL-Analysis 176 kLOC, HOL-Library 68 kLOC, HOL 123 kLOC).

### Japan-specific considerations
**Christian Sternagel** (JAIST alumnus/former Erwin-Schrödinger Fellow at JAIST; now Innsbruck; IsaFoR/CeTA certified rewriting termination). **Kazuhiro Ogata** (JAIST): CafeOBJ/OTS algebraic-specification method (successor to Kokichi Futatsugi); proof-scores survey (CoRR abs/2504.14561, 2025); co-edited ICFEM 2024 (Hiroshima, 2024-12-02–06, LNCS 15394, with Dominique Méry, Meng Sun, Shaoying Liu). **Mizuhito Ogawa** (JAIST): raSAT SMT for polynomial constraints (Tung, Khanh, Ogawa, FMSD 51(3):462–499, 2017). FeliCa Networks "Mobile FeliCa" case study (Kurita & Araki, ICFEM 2016, Tokyo): notable JP industrial VDM-based deployment.

### Interoperability
Export to Dedukti/Lambdapi via Deducteam. Isabelle/HOLCF: domain theory. Isabelle/DOF (Document Ontology Framework, Brucker–Wolff) embeds SACM/GSN assurance-case ontologies into Isabelle documents — directly relevant to clinical-guideline traceability. Cat 1: Isabelle/UTP (Unifying Theories of Programming) covers Hoare/Z-style/refinement, can model BPMN-style step semantics. Cat 3: Isabelle/DOF + ROntology allow OWL-DL embedding. Cat 4: Alethe proof-certificate import, SMTCoq-style (ITP 2025, Lachnitt et al.).

### Limitations/Known issues
No dependent types; higher-rank dependency needs HOLZF or ad-hoc tricks. Locales can become unwieldy. AFP entries can go unmaintained over years. Lower LLM training presence than Lean.

### Training data proxy
973 AFP entries; long-running JAR/CADE/ITP paper pipeline. Sledgehammer's tactic-form output structured but variable. Magnushammer (ICLR 2024) + "A Minimalist Proof Language" (arXiv 2507.18885) demonstrate transformer-based proving over Isabelle.

## 4. TLA+ / PlusCal for Pipeline, Convergence, and Idempotency Properties

### Purpose
Leslie Lamport's formal spec language for distributed/concurrent systems; excels at temporal properties — exactly the convergence/idempotency needs of the autoformalization pipeline.

### Maintainer/Standards body
Originally Microsoft Research; now TLA+ Foundation (under Linux Foundation, est. 2023; Lamport active emeritus). Reference: "Specifying Systems" (Lamport, Addison-Wesley, 2003); current updates via "The TLA+ Hyperbook".

### Conceptual model
Untyped ZF set theory + FOL + linear temporal logic of actions (TLA). A spec = single temporal formula `Init ∧ □[Next]_vars ∧ Liveness` characterizing infinite behaviors. **PlusCal** (2009): pseudocode-like surface language transpiling to TLA+. **TLC**: explicit-state model checker (BFS over reachable states). **TLAPS** (2012): interactive proof system backed by Zenon, Isabelle/TLA+, SMT.

### Expressiveness/Semantics
Classical FO set theory; arbitrary math operators. Hyperproperties (refinement, fairness, eventual delivery, stuttering invariance) first-class. **Idempotency**: `□(action(s) ⇒ action(action(s)) = action(s))` or bisimulation against an action's fixed point. **Convergence**: liveness `◇□P` (eventually always P) or Stabilization.

### Composability/Modularity
Modules with `EXTENDS`/`INSTANCE`. No type system (deliberate). Refinement via module instantiation under substitution.

### Suitability for autoformalization to IR
Not a target IR for guideline content; the meta-IR for the pipeline producing guideline IR. Spec "every two runs of the autoformalizer over the same input converge to a state-equivalent IR": `□(input = input' ⇒ ◇(IR ≡ IR'))`. Idempotency: `Normalize(Normalize(x)) = Normalize(x)`.

### Formal verification potential
TLC: exhaustive finite-scope checking (millions of states routine). Apalache: symbolic, SMT-backed. TLAPS: deductive hierarchical proofs (steps `<1>1`, `<1>2`, etc.) with Zenon/Isabelle/SMT backends.

### Tooling/Ecosystem maturity
TLA Toolbox (Eclipse-based); VS Code TLA+ extension. Apalache (University of Lugano / Informal Systems). Distributed TLC. PlusCal translator. Production use: AWS (S3, DynamoDB, EBS), Microsoft Azure, MongoDB, ConsenSys, Elastic.

### Japan-specific considerations
**Ichiro Hasuo** group, NII (Research Center for Mathematical Trust in Software and Systems, MTSS): ERATO MMSD (2016-10–2025-03, JST grant JPMJER1603, ~250 papers) used temporal-logic verification heavily (Goal-Aware RSS; ISO 34502 formalization with Masaki Waga, Kyoto U). CAV 2023 Distinguished Paper "Exploiting Adjoints in Property Directed Reachability Analysis" (Mayuko Kori et al.). Limited TLA+ adoption in JP clinical software; AIST High Reliability Software Engineering group historically used TLA+ in workshops.

### Interoperability
TLA+ specs encode into Lambdapi (Alessio Coltellacci & Stephan Merz, "encoding of TLA+ set theory in Lambdapi", ABZ 2023). Cat 1: BPMN/ePath flows are naturally TLA+ state machines; DMN decision tables = `IF/THEN/ELSE` blocks. Cat 2: PMDA workflow as TLA+ refinement chain. Cat 3: ontology classes = constant set declarations. Cat 4: Apalache exports SMT proofs (Alethe-pipeline compatible).

### Limitations/Known issues
Untyped → specs can be ill-formed at runtime; Apalache adds optional typing. TLC scope explosion on large state spaces. TLAPS less automated than Lean/Isabelle. ASCII syntax unfriendly. PlusCal hides TLA+ details but limits expressiveness.

### Training data proxy
Hillel Wayne's "Learn TLA+" tutorial widely referenced. `github.com/tlaplus/Examples`: hundreds of specs. Smaller LLM corpus than Lean; no miniF2F-equivalent benchmark.

## 5. Alloy / Forge for Finite Relational Counterexamples

### Purpose
Lightweight spec language + bounded analyzer by Daniel Jackson (MIT). Finds counterexamples to relational specs within finite scope: "shake-out" testing of guideline-IR invariants; student-accessible specification.

### Maintainer/Standards body
Alloy: MIT Software Design Group (Jackson); current **Alloy 6** integrates Electrum's temporal logic (mutable signatures + LTL operators `always`/`eventually`/`after`/`'`). Forge: Brown University PLT (Tim Nelson, Shriram Krishnamurthi) + Northeastern; OOPSLA 2024 paper "Forge: A Tool and Language for Teaching Formal Methods" (PACMPL Vol 8).

### Conceptual model
FO relational logic over typed atoms (signatures `sig`, fields, constraints). Translates to SAT via Kodkod; solver typically MiniSat or Glucose. **Small-scope hypothesis** (Jackson, Software Abstractions, MIT Press 2012, p. 141): "most bugs have small counterexamples" → bounded analysis empirically near-complete. Forge layers Froglet (functional subset) → Relational Forge → Temporal Forge, each adding expressiveness.

### Expressiveness/Semantics
Relational FOL + LTL (Alloy 6 / Temporal Forge). No quantification over relations (decidable bounded fragment). Transitive closure (`^`, `*`) first-class. Predicates + assertions (`assert ... check`) drive analyses.

### Composability/Modularity
Modules (`open`) with parametric polymorphism over signatures. Forge adds language levels + Sterling visualizer with CnD ("Cope and Drag") custom diagrams (ECOOP 2025).

### Suitability for autoformalization to IR
Excellent for **counterexample-driven debugging** of IR invariants. LLMs can emit Alloy fragments to test "no guideline recommends both X and not-X" or "every administered-by edge points to a Practitioner". Idempotency: `assert idempotent { all x: IR | normalize[normalize[x]] = normalize[x] } check idempotent for 5`. Not a full-IR target (no recursive data, no real arithmetic).

### Formal verification potential
SAT-backed; bounded but exhaustive within scope; cannot prove unbounded properties. Counterexamples are concrete, visualizable instances.

### Tooling/Ecosystem maturity
Alloy Analyzer (Java/Swing). Sterling (web visualizer). Forge runs on Racket; VS Code + DrRacket. Forge used in Brown's "Logic for Systems" course since 2019.

### Japan-specific considerations
Used in JP formal-methods courses at JAIST (Kazuhiro Ogata; Mizuhito Ogawa). FeliCa Networks "Mobile FeliCa" IC-chip firmware case study (Kurita & Araki, "Promotion of Formal Approaches in Japanese Software Industry and a Best Practice of FeliCa's Case", ICFEM 2016, Tokyo) was VDM-based, but JP FM community broadly familiar with Alloy. No JP-specific Alloy library identified.

### Interoperability
Alloy → SMT translation exists (Pardinus). Models hand-translate to TLA+ for unbounded proof. Cat 1: DMN decision tables + FHIR resource cardinalities map to `sig`/multiplicity constraints. Cat 3: OWL DL partial translations (DL-Lite fragments). Cat 4: counterexamples seed Lean/Rocq lemma statements.

### Limitations/Known issues
Bounded incompleteness. Awkward arithmetic. Default visualization can be unhelpful (Forge partly addresses via Sterling + CnD). Performance degrades past scope ~10–15 atoms.

### Training data proxy
Moderate. Jackson's textbook (MIT Press 2012, revised) + Macedo/Cunha online "Formal Software Design with Alloy 6". Hillel Wayne blog. GitHub: hundreds of public Alloy models. Forge corpus small (mostly Brown classroom).

## 6. Why3 / WhyML for SMT-Backed Executable Specifications

### Purpose
Deductive verification platform dispatching VCs to many automated + interactive provers. WhyML: ML-like first-order language with contracts; direct verification target and intermediate language for verifying C/Java/Ada/Rust.

### Maintainer/Standards body
INRIA Saclay / LRI (Toccata team). Authors: Jean-Christophe Filliâtre, Andrei Paskevich, Claude Marché, Guillaume Melquiond, François Bobot. Current Why3 1.8+ (2024–2025).

### Conceptual model
Logic: polymorphic FOL + algebraic data types + inductive predicates. Programs: WhyML — records, mutable fields, pattern matching, exceptions, ghost code, type invariants, **static alias control** (no memory model needed). VCs generated, transformed via rich transformation library, dispatched to Alt-Ergo, CVC4/CVC5, Z3, E, Vampire, Eprover, Spass, Princess + interactive backends (Rocq/Coq, Isabelle, PVS).

### Expressiveness/Semantics
First-order; less expressive than Lean/Rocq. Strong fit for executable specs over arithmetic, arrays, simple algebraic structures. Inductive predicates allow recursion. Extracts to OCaml/C/CakeML.

### Composability/Modularity
Theory modules + module cloning + refinement. Stdlib: integers, reals, sets, maps, queues, hash tables, arrays.

### Suitability for autoformalization to IR
Strong fit for clinical-rule executable specs: guideline rule = WhyML `let` with `requires`/`ensures`. Idempotency VC: `forall x. normalize(normalize x) = normalize x`. SMT backends check LLM-generated specs quickly; convergence supported by canonical normalization passes (transformations).

### Formal verification potential
Strong SMT automation for FO arithmetic; counterexample reconstruction from SMT models. Applications: GMP arithmetic (WhyMP), ParcourSup, **Creusot** (Rust verification, PLDI 2022 Distinguished Paper); intermediate language for Frama-C (C verification).

### Tooling/Ecosystem maturity
Why3 IDE, CLI, emacs mode. Opam package. Active in French academic ecosystem.

### Japan-specific considerations
Limited direct JP contributions to core. JP researchers use Why3 as backend (e.g. via Frama-C industrially). Kohei Suenaga's group (Kyoto U): SMT-based hybrid-systems reasoning with related tooling.

### Interoperability
Native SMT-LIB output; Alethe proofs from veriT/cvc5 replayable in Lambdapi. Can drive Rocq/Isabelle backends. Cat 1: WhyML maps directly to CQL-like expressions (FHIRPath/CQL operators are first-order). Cat 2: PMDA-style data dictionaries → WhyML record types. Cat 3: SHACL constraints → WhyML `predicate` definitions with VCs. Cat 4: frequently dispatches to the interactive provers above.

### Limitations/Known issues
First-order only; no higher-order; weak for category-theoretic constructions. SMT heuristics inconsistent across solver versions. Limited LLM training presence.

### Training data proxy
Modest. ~1k GitHub repos using WhyML. Annual JFLA + TAP papers. Creusot (Rust) growing.

## 7. F* for Typed Verified Services

### Purpose
Proof-oriented language: dependent types + refinement types + SMT verification, built for verified low-level/cryptographic code (HACL*, EverCrypt) and high-assurance services. Compiles to OCaml/F#/C (via KaRaMeL)/Rust/Wasm.

### Maintainer/Standards body
Microsoft Research + INRIA Paris (Prosecco) + Carnegie Mellon. Authors: Nikhil Swamy and collaborators. Active 2025–2026 releases (most recent F* 2026.3.24, 2026-03-24; prior 2025.12.15).

### Conceptual model
Dependent type theory + layered effect system (`Tot`, `Pure`, `Ghost`, `Stack`, `ST`, `Steel`, `Pulse`). Refinement types `x:t{p x}`. VCs discharged primarily by Z3; proof reconstruction via Meta-F*. **Pulse**: current separation-logic DSL in F* (successor/companion to Steel); merged into main F* repo 2024–2025.

### Expressiveness/Semantics
Classical with explicit ghost code. Dependent + refinement types express arbitrary FO specs inside types; SMT keeps verification largely automatic.

### Composability/Modularity
Module system; `.fsti`/`.fst` interface/implementation split; abstract types. Meta-F* tactics.

### Suitability for autoformalization to IR
Guideline rule = F* function whose type encodes its spec. Idempotency = refinement on normalizer output type, auto-checked by Z3. Strong: SMT discharges most LLM-generated obligations without manual proof; fast inner validation loop. Smaller LLM training base than Lean.

### Formal verification potential
Deployments: **HACL*** (verified cryptography: Mozilla Firefox NSS, Linux kernel, mbedTLS, Tezos, ElectionGuard, Wireguard); **EverCrypt** (S&P 2020); **DICE\\*** (verified attestation). 2025 papers: "Secure Parsing... CBOR/CDDL/COSE" (CCS 2025); "Mechanically Verified GC for OCaml" (~24k LoC F*/Pulse, JAR 69:7, 2025).

### Tooling/Ecosystem maturity
VS Code extension; emacs mode; Z3 4.8.5/4.13 dependency. Opam packaging. Active development.

### Japan-specific considerations
Minimal direct JP contribution. Some JP cryptographers (NTT Labs crypto teams) consume HACL*/EverCrypt; no publicly authored JP F* libraries.

### Interoperability
KaRaMeL extracts to C; can target Rust. Cat 1: refinement types over FHIR resource fields. Cat 4: Steel/Pulse alternatives to Iris (Rocq); both encode separation logic.

### Limitations/Known issues
SMT-dependent (Z3 hangs → non-reproducible failures). TCB includes Z3. Steep effect-system learning curve. Pulse/Steel powerful but young.

### Training data proxy
Modest. "Towards Neural Synthesis for SMT-Assisted Proof-Oriented Programming" (ICSE 2025): 940 KLOC F* dataset for LLM training.
## 8. Dependent-Type and Refinement-Type IR Schemas

### Purpose
Dependent/refinement type systems (LiquidHaskell, Idris 2, Agda, refinement-reflection) make IR schema invariants part of the type, statically enforced. Cross-cutting technique, not a single tool.

### Maintainer/Standards body
- LiquidHaskell: Niki Vazou et al.; GHC plugin.
- Idris 2: Edwin Brady; open-source community.
- Agda: Ulf Norell; Chalmers/Gothenburg + Inria + community.
- Refinement Reflection: POPL 2018 (Vazou, Tondwalkar, Choudhury, Scott, Newton, Wadler, Jhala; "Refinement Reflection: Complete Verification with SMT", PACMPL Vol 2, POPL, Article 53).

### Conceptual model
Refinement types `{ v: T | p(v) }`, SMT-decidable predicates. Dependent types: types depend on values → indexed types (`Vec n A`). Invariants like "every Recommendation has a non-empty Evidence list of matching SnomedCT codes" become inhabitable types; ill-formed IR documents fail typecheck.

### Expressiveness/Semantics
Refinement: decidable (modulo SMT), first-order predicates over base types. Dependent: full Martin-Löf/CIC strength, undecidable verification. Refinement-Reflection lifts Haskell function definitions into the refinement logic for equational reasoning.

### Composability/Modularity
LiquidHaskell: GHC modules. Idris 2: interfaces + modules. Agda: parameterized modules.

### Suitability for autoformalization to IR
Sweet spot for IR schemas. LLM generates Idris/Agda data types with structural invariants in types; wrongly-shaped output fails to compile — hard syntactic gate improving convergence/idempotency.

### Formal verification potential
Vazou, Seidel, Jhala, Vytiniotis, Peyton-Jones, "Refinement Types for Haskell" (ICFP '14, Gothenburg): "LIQUIDHASKELL is able to prove 96% of all recursive functions terminating, while requiring a modest 1.7 lines of termination-annotations per 100 lines of code", evaluated on containers, hscolour, bytestring, text, vector-algorithms, xmonad (>10,000 LoC). Idris `Dec` / Agda `Decidable` give certified decision procedures.

### Tooling/Ecosystem maturity
LiquidHaskell plugin-active; Idris 2 stable; Agda 2.7+ stable. All have language servers.

### Japan-specific considerations
Atsushi Igarashi group (Kyoto University): fundamental refinement-type and gradual-typing work (ICFP/POPL/ECOOP; AITO Dahl-Nygaard Junior Prize 2011 for Featherweight Java). HELMHOLTZ (Igarashi group) verifies Tezos/Michelson smart contracts via refinement types.

### Interoperability
LiquidHaskell shares SMT backend with Why3/F*. Idris/Agda → Coq/Lean translation (partial). Cat 1: FHIR `Reference`/`cardinality` constraints → refinement predicates. Cat 3: SHACL shape closure rules → refinement types; OWL class restrictions ≈ Σ-types in Agda/Idris.

### Limitations/Known issues
Beyond SMT → manual proof. Idris 2 ecosystem small. Agda lacks Lean-grade tactic automation. Steep learning curve.

### Training data proxy
Small vs Lean. Agda: thousands of GitHub repos; Idris fewer. LiquidHaskell community moderate.

## 9. Proof by Reflection for Executable Guideline Rule Evaluators

### Purpose
Technique (not tool): encode decision procedures as verified executable functions inside a proof assistant, invoked via reduction (`vm_compute`, `native_decide`, `Eval`); goals become "this Boolean term computes to `true`", reducing proof checking to certified evaluation. Ideal for guideline-applicability checks needing rules both formally specified and executable.

### Maintainer/Standards body
Samuel Boutin (Inria, TACS 1997, "Using reflection to build efficient and certified decision procedures"). Implementations: Coq/Rocq, Lean 4, Agda.

### Conceptual model
Inductive formula datatype; `decide : Formula → bool`; prove `forall f, decide f = true → ⟦f⟧`. To prove `⟦f⟧`: reflect to `f`, prove `decide f = true` by reduction (`vm_compute` Coq/Rocq; `native_decide` Lean 4 — compiles to OCaml/C and runs; `Reflexivity` after `vm_compute` Agda).

### Expressiveness/Semantics
Any computable predicate. Trades term size for proof-search complexity. Kernel must trust reduction strategy: `vm_compute` = Coq-internal VM (trusted); `native_compute`/`native_decide` extend TCB to OCaml compiler+runtime.

### Composability/Modularity
Any inductive can become a reflected language. Library examples: `lia`, `lra`, `ring`, `field` (Coq/Rocq); `omega`, `decide`, `bv_decide`, `grind` (Lean); solve-by-Reflection patterns (Agda).

### Suitability for autoformalization to IR
Guideline-rule evaluator ≈ `decide : ClinicalContext → Recommendation → Bool`. With reflection, LLM produces only rule data (syntax tree), not proofs. Idempotency `decide ∘ normalize = decide` provable by reflection.

### Formal verification potential
SMTCoq checks LFSC certificates from CVC5 inside Rocq with full kernel guarantees. MirrorShard (Malecha, Chlipala, Braibant) adds verified hint databases. CoqHammer reconstructs SMT proofs. Lean 4: `lean-smt` (arXiv 2505.15796) reflection-based discharging of an SMT proof-rule fragment.

### Tooling/Ecosystem maturity
Native to Rocq, Lean, Agda; well-developed.

### Japan-specific considerations
Reynald Affeldt (AIST): reflection-based tactics in MathComp/SSReflect formalizations (information theory, ECC); his `monae` library uses reflection for equational reasoning over monadic effects.

### Interoperability
Pure technique; works inside any of the above proof assistants. Reflected proofs exportable via Dedukti/Lambdapi.

### Limitations/Known issues
`native_decide` enlarges TCB (OCaml compiler). Reduction time bottleneck on large terms. Reflection proofs opaque to humans.

### Training data proxy
Many tutorials (Software Foundations VFA `Decide` chapter, Chlipala CPDT). Less LLM training data than tactic-style proofs.

## 10. Checkable Proof Certificates and Derivation Traces

### Purpose
Small, independently-checkable proof artifacts (certificates) in a minimalist language so any party (auditor, regulator, downstream tool) re-verifies a derivation without trusting the original prover. Essential for trustworthy CDS where regulators audit reasoning chains.

### Maintainer/Standards body
- Dedukti / Lambdapi: Deducteam (Inria; Frédéric Blanqui); Lambdapi = modern interactive successor to Dedukti.
- LFSC: Stump et al. (University of Iowa); used by CVC4/CVC5.
- Alethe: Schurr, Fleury, Barbosa, Fontaine; emitted by veriT and cvc5 (spec at `verit.gitlabpages.uliege.be/alethe/specification.pdf`).
- SC-TPTP: TPTP community.
- ProofCert / Foundational Proof Certificates: Miller, Chihani.
- Carcara: Alethe checker/elaborator (Barbosa et al., TACAS 2023).

### Conceptual model
Logical framework (Edinburgh LF or λΠ-calculus modulo rewriting underlying Dedukti/Lambdapi) defines a small kernel calculus; each source logic (HOL, FOL, CIC, SMT theories) encoded as a signature of constants + rewrite rules; proof = typed λ-term; checking = type-checking. LFSC adds side conditions. Alethe = SMT-specific format with proof-rule schemata.

### Expressiveness/Semantics
λΠ-modulo is Turing-complete; encodes HOL, CIC, PVS, simple-type theory, Alethe SMT proofs. Trade-off: more permissive theories require more axioms (less trust).

### Composability/Modularity
Lambdapi: implicit arguments, unification hints, tactics — usable as proof IDE, not just checker. Translators: Coq→Dedukti (CoqInE), HOL-Light→Dedukti, Isabelle→Dedukti, Lean→Dedukti, PVS→Lambdapi (Personoj), MetaMath→Lambdapi.

### Suitability for autoformalization to IR
Excellent neutral IR for cross-proof-assistant verification of clinical-guideline derivations: LLM-generated Lean proof → Lambdapi → re-checked by independent Rocq-trained team. Idempotency: checker is deterministic and bitwise reproducible.

### Formal verification potential
Tiny TCB (typically 1–2 KLOC). Lambdapi = de facto pivot of the ICSPA ANR project exchanging B, Event-B, TLA+ proofs (Coltellacci & Merz, ABZ 2023; Grieu & Bodeveix, ABZ 2024/2025 — Rodin/Event-B certification in Lambdapi).

### Tooling/Ecosystem maturity
Lambdapi VS Code plugin; Deducteam's Personoj (PVS), HOL-Light translators, Carcara for Alethe. EuroProofNet COST action (2021–2025) drives interoperability.

### Japan-specific considerations
No major Japanese contribution to Dedukti/Lambdapi core; JAIST (Ogata, Ogawa) and AIST (Affeldt) well-positioned consumers. Deducteam regularly invites Japan participants to EuroProofNet workshops.

### Interoperability
Universal proof-pivot. Direct bridges from methods 1, 2, 3, 6 (via SMT proofs). Carcara elaborates coarse-grained Alethe into fine-grained steps for higher confidence. SMTCoq imports LFSC into Rocq. Isabelle Alethe replay (Lachnitt et al., ITP 2025) closes the cvc5→Isabelle loop.

### Limitations/Known issues
Translation gaps (universe polymorphism, classical axioms) need careful encoding. Lambdapi tactic ecosystem much smaller than Lean/Rocq's. `dkcheck`/`lambdapi check` performance good but lags native checkers on huge proofs.

### Training data proxy
Small. PxTP, FSCD, FroCoS papers. EuroProofNet spans hundreds of researchers.

## 11. CrossHair for Python Contract Verification by SMT-Backed Symbolic Execution

### Purpose
Static analysis verifying pure-Python functions against declared contracts (pre/postconditions, invariants) via symbolic execution: symbolic ("proxy") objects per argument, all reachable paths traced, SMT solver searches for a path-condition assignment violating the post-condition (counter-example). Niche between property-based testing (Hypothesis, pytest-randomly) and heavyweight verifiers (F*, LiquidHaskell): no separate spec language — typed Python + `assert`/docstring contracts.

### Maintainer/Standards body
Solo-founded/maintained by Phillip Schanely (GitHub `pschanely`); growing contributors since first public releases 2019–2020. PyPI: `crosshair-tool`. No standards body; contract syntaxes: PEP 316 (docstring `pre:`/`post:`), `icontract` (Marko Ristin), `deal` (Gram / `orsinium`), plain `assert`, `typing` annotations. `hypothesis-crosshair` package co-released with Hypothesis maintainers (David R. MacIver, Zac Hatfield-Dodds).

### Conceptual model
Concolic/pure-symbolic execution over the CPython AST. Per annotated arg `T`, builds `Proxy[T]` whose dunders (`__eq__`, `__lt__`, attribute access, indexing, iteration) record Z3 path-conditions. Containers lazily realised via "smart" symbolic lists/dicts/sets backed by uninterpreted-function arrays. Forks symbolic state at each branch; completed path → negated post-conditions sent to solver; `sat` → concrete counter-example (proxies replaced by solver witness); `unsat` → verified up to budget. Modes:
- `crosshair check` — verify annotated contracts in file/module/package.
- `crosshair watch` — file-watcher daemon, incremental re-run on edit.
- `crosshair diffbehavior fn_a fn_b` — symbolic differential testing; inputs where implementations diverge (refactor validation, regression localisation).
- `crosshair cover` — minimal concrete test suite achieving symbolic-branch coverage.

### Expressiveness/Semantics
Decidable Python fragment in QF_UFLIA + QF_S + bit-vectors + uninterpreted functions: bounded int/string/sequence ops, boolean logic, dict/set membership, dataclass field access, `enum`, `Optional`/union dispatch. Floats via bit-precise IEEE-754 encoding (slow). Sound modulo (i) path budget (timeouts → `Unknown`, never `Verified`), (ii) C-extension calls (opaque), (iii) non-determinism (`random`, time, network, filesystem — must be stubbed). Mutation, exception flow, generators modelled. No `multiprocessing`, native threads, or `async`/`await`.

### Composability/Modularity
Per-file/class/function selection via CLI selectors + `# crosshair: on/off` directives. Config in `pyproject.toml` `[tool.crosshair]`: path budgets, per-condition timeouts, excludes. Contract decorators orthogonal: one function can carry `icontract.require`, `deal.post`, docstring `pre:`, and a Hypothesis strategy simultaneously, all honoured. Pre-commit hook and GitHub Actions integrations in README.

### Suitability for autoformalization to IR
Strong target at the implementation layer: LLM emits Python function + PEP 316 docstring for a clinical recommendation; CrossHair is the inner-loop validator certifying it or returning a clinical-scenario counter-example. High round-trip stability — Python is the highest-frequency LLM-pretraining language; docstring contracts far easier to generate correctly than Lean/F* proof terms. Normaliser idempotency as `post: __return__ == normalize(__return__)`, discharged up to budget. Good convergence under regeneration: contract changes local; counter-examples re-prompt the LLM with concrete data, narrowing search.

### Formal verification potential
Bounded by decidable fragment + search budget; assurance tool, not kernel-verified prover. No proof certificate (`Verified` = search-completed claim, not checkable derivation) → alone insufficient for ISO 26262 / IEC 62304 evidence; but counter-examples are concrete runnable pytest-style reproducers auditors/regulators can re-execute. Pairs with `lean-smt` / F* / LiquidHaskell: prototype contract in CrossHair, then lift to refinement-typed/proof-assistant target once it survives counter-example search.

### Tooling/Ecosystem maturity
Active, multiple releases/year. VS Code extension (`CrossHair Python`) shows counter-examples as in-editor squiggles; community PyCharm plugin. Recent Hypothesis ships the `crosshair` backend (`@given(...)` + `settings(backend="crosshair")`) — existing property tests re-run symbolically with zero test-body change. Pre-commit hook published; GitHub Actions examples; `hypothesis-crosshair` on PyPI.

### Japan-specific considerations
No known Japanese core contributors. Japanese `icontract`/`deal` community (JSSST PPL, Python Boot Camp materials) and Hypothesis teams at Preferred Networks and LINE-Yahoo Japan benefit transitively (re-validate property tests under CrossHair backend without rewriting). JAIST Mizuhito Ogawa group (symbolic execution, SMT non-linear arithmetic) and NII Hasuo group (ERATO MMSD; cyber-physical symbolic verification) are natural collaborators for medical-device Python under AMED-funded CDS pipelines. Japan-Minds-derived recommendation predicates as Python contracts → directly checkable, lowering barrier to mechanised clinical validation (Category 2).

### Interoperability
- Within Cat 4: counter-examples lift to F*/Lean as failing-test obligations driving verified rewrite; refinement-type schemas (§8) compile to Python type hints + `assert`; Why3/WhyML executable specs (§6) cross-validated against Python reference via `diffbehavior`.
- Cat 1: FHIR Clinical Reasoning predicate prototyped in Python + docstring contract, verified before mechanising in CQL/DMN — "spec by example" toward standards-conformant deliverable.
- Cat 2: Minds-derived Japanese recommendations as Python predicates (JLAC/MEDIS codes as constants) become checkable contracts.
- Cat 3: SHACL shape closures + OWL class restrictions on FHIR resources as Python `assert`s; CrossHair finds violating RDF instances.
- Cat 5: shares Z3 backend (§5.1); contracts exceeding depth budget re-expressed in SMT-LIB or Apalache/TLA+ (§5.8).
- Cat 6: temporal clinical predicates (e.g., "two consecutive eGFR < 60 readings ≥ 90 days apart") as Python functions over typed event lists; boundary cases explored (off-by-one, empty histories, equal timestamps).
- Cat 8: execution-feedback leg of LLM agentic autoformalization loop — counter-examples re-prompt repair; the closed-loop pattern of Kimina-Prover / Goedel-Prover.
- Cat 9: counter-examples = adversarial unit-test inputs for clinical-validation suites and human-in-the-loop review.
- Cat 10: counter-example traces = auditable IEC 62304 evidence artifacts complementing assurance-case argumentation.

### Limitations/Known issues
Path explosion on loops with data-dependent bounds; recursion needs explicit `--max_iterations`. C-extensions (NumPy, Pandas, scikit-learn, PyTorch) opaque — dataframe-heavy clinical pipelines need mocked/stubbed implementations. Float reasoning (IEEE-754 bit-blasting) orders of magnitude slower than int/string; most common cause of `Unknown` in physiological-threshold rules. No `async`/`await`, native threads, or true parallelism. `Unknown` (budget-exhausted) common on real-world code and must not be conflated with `Verified` — project explicitly distinguishes the states. Single-process Python; scales poorly past ~hundreds of LoC per function. No proof certificate or independently-checkable derivation.

### Training data proxy
`pschanely/CrossHair`: moderate, growing community (low-thousands stars); `hypothesis-crosshair` separate package. Docs/tutorial corpus smaller than Lean/F*/Hypothesis. LLMs trained on general Python + `icontract`/`deal`/Hypothesis idioms produce CrossHair-targetable code without fine-tuning (contracts = decorated functions + docstrings). Modest Stack Overflow presence; support via GitHub issues + project Discord. PyCon US and EuroPython talks by Schanely (2020, 2022) = canonical introductions.

## 12. SAW, Crucible, and Cryptol for Implementation Verification Against Formal Specifications

### Purpose
Verify that production C, Java, Rust, Go, or WebAssembly implementations of CDS components faithfully implement formal specifications — the "last mile" between a proved property in Lean/Rocq/TLA+/SMT and a deployed binary. Cryptol: DSL for executable reference specifications. SAW (Software Analysis Workbench): proves Cryptol-spec ↔ implementation equivalence via symbolic execution. Crucible: language-agnostic symbolic-execution framework under SAW. §§1–10 verify the specification; this verifies the implementation.

### Maintainer/Standards body
Galois, Inc. (Portland, Oregon; founded 1999 by John Launchbury; Launchbury directed DARPA I2O 2014–2017, then returned as Chief Scientist; among the largest commercial Haskell users and most active US formal-methods companies). Releases: Cryptol 3.5.0 (January 28, 2026; `GaloisInc/cryptol`, ~1,200 stars, 129 forks, BSD-3-Clause, 87.7% Haskell); SAW 1.5.1 (May 22, 2026; `GaloisInc/saw-script`, 505 stars, 82 forks, BSD-3-Clause, 80.1% Haskell); Crucible (`GaloisInc/crucible`, 767 stars, 47 forks, BSD-3-Clause, 81.8% Haskell; user-facing tool Crux v0.12, January 29, 2026). Funding: NSA Laboratory for Advanced Cybersecurity Research; ONR Contract N68335-17-C-0452; DARPA (HACMS, SSITH/BESSPIN, CHARIOT/QSAFE). Related Galois tools: What4 (`GaloisInc/what4`; solver-agnostic Haskell library for Z3, Yices 2, cvc5, Bitwuzla, Boolector, STP, dReal — SMT/SAT layer beneath Crucible); Copilot (v4.7.1, May 8, 2026; `Copilot-Language/copilot`, 822 stars; stream-based runtime verification generating constant-memory constant-time C99 monitors from Haskell specs; NASA Contracts NNL08AD13T, 80LARC17C0004, NNL09AA00A; maintainers Alwyn Goodloe, Ivan Perez; not GitHub Copilot); Macaw (binary code discovery + symbolic execution for x86-64, PowerPC, ARM, RISC-V; lifts machine code into Crucible IR).

### Conceptual model
Pipeline: specify → verify → deploy. Cryptol: pure functional, Hindley-Milner + arithmetic size constraints, native arbitrary-width bit-vectors, sequence types; executable spec, `:check` (QuickCheck-style random testing) and `:prove` (Z3, Yices, cvc5 backends). Built for NSA as classified crypto-spec standard; public 2008. SAW: SAWScript composes verification tasks at scale; canonical workflow: Cryptol spec → compile production code to IR (LLVM bitcode via clang for C, JVM bytecode for Java, MIR for Rust) → SAWScript proves implementation computes the same function as the spec for all inputs in scope, via symbolic execution + SAT/SMT. Crucible: programs as CFGs explored forward-symbolically; path conditions dispatched to What4's solver portfolio. Frontends: `crucible-llvm`/`crux-llvm` (C/C++, LLVM bitcode 3.5–20.0), `crucible-jvm` (Java), `crux-mir` (Rust/MIR), `crucible-go`, `crucible-wasm`. Crux: user-facing tool on Crucible for bounded intricate code (crypto modules, serializer/deserializer pairs, protocol implementations).

### Expressiveness/Semantics
Cryptol: side-effect-free, total (modulo non-termination warnings), dependent-length sequence types (`[n]a`, `n` a type-level natural), constraints like `fin n, n >= 128`; modules, type synonyms, `newtype`, `where` clauses, pattern matching. SAWScript: imperative-looking but deterministic; `llvm_verify`, `jvm_verify`, `mir_verify` establish pre/post Hoare triples composed via lemma reuse (verified A = assumption when verifying caller B). Crucible: mutation, heap allocation, pointer arithmetic (C), exception flow (Java), ownership/borrowing (Rust/MIR), GC semantics. What4: QF_UFBV + arrays + uninterpreted functions; bitvector reasoning is the core strength (cryptographic heritage).

### Composability/Modularity
Cryptol modules via `import`; specs reference each other. SAWScript compositional verification: a verified function's spec becomes an override when verifying callers — critical for codebases with thousands of functions. Crucible frontends independently maintained, shared solver backends. What4 standalone on Hackage (`what4`). All share a common symbolic-value representation in the Galois Haskell ecosystem.

### Suitability for autoformalization to IR
Indirect but important: not autoformalization targets — they verify the CDS pipeline implementation, not guideline IR content. Role: once §§1–10 produce a proved IR compiled into a Rust/C/Java production service, SAW/Crucible prove faithful implementation. Amazon s2n case study (Chudnov, Collins, Cook, Dodds, Huffman, MacCarthaigh, Mertens, Mullen, Tasiran, Tomb, Walkingshaw, "Continuous Formal Verification of Amazon s2n", CAV 2018): Cryptol specs of HMAC + TLS handshake proved equivalent to production C, SAW in CI for continuous verification on every change — analogous to Knowledge CI/CD (Category 10 §9). Cryptol's DSL-for-specification design = proven archetype for a clinical-rules DSL: clean, executable, SMT-verifiable single source of truth for both proofs and implementations. Galois 2025–2026 QSAFE (DARPA CHARIOT) specified all CNSA 2.0 post-quantum algorithms (ML-KEM/CRYSTALS-Kyber, ML-DSA/CRYSTALS-Dilithium, SLH-DSA/SPHINCS+) in Cryptol — the Cryptol→SAW pipeline scales to production cryptographic standards.

### Formal verification potential
Strong. SAW proves functional equivalence up to bounded input scope (bit-precise for bit-vectors; unbounded for symbolic inputs within solver decidability); sound for the bounded fragment — verified function provably computes the spec's output for all in-scope inputs. No independently-checkable certificates (unlike DRAT for SAT or Alethe for SMT), but reproducible given pinned tool versions + solver configs. Crucible sound modulo frontend translation (LLVM/JVM/MIR → Crucible IR) and solver backend; LLVM frontend extensively validated against LLVM semantics. Copilot adds runtime monitors flagging temporal-property violations with mathematically guaranteed constant memory/time; generated C99 provably bisimilar to the Haskell spec via `copilot-verifier`.

### Tooling/Ecosystem maturity
Mature, active; all major tools had 2026 releases. SAW: LLVM 3.5–20.0 (all modern clang), JVM bytecode, Rust MIR. Cryptol: REPL, batch mode, "Literal Cryptol" for weaving specs into documents. Docs at `tools.galois.com/cryptol` and `tools.galois.com/saw`. s2n continuous verification (CAV 2018) = canonical industrial case; AWS still uses SAW for libcrypto. Galois also verified portions of libgcrypt and Bouncy Castle (Java). DARPA HACMS ($18M with Rockwell Collins, Data61, Boeing, University of Minnesota; 4.5 years): Ivory (Haskell EDSL for safe C) + Tower concurrency framework → SMACCMPilot verified quadcopter autopilot. DARPA SSITH/BESSPIN ($16.6M): hardware security via RISC-V FPGA. NRC HARDENS: formally verified reactor trip system via Rigorous Digital Engineering. Galois healthcare practice (galois.com/solutions/healthcare): secure health-data analytics (MPC, PSI, differential privacy), cyber-physical medical-device security (ARPA-H UPGRADE/SAFE-Dev hospital cybersecurity), ML safety for personalized medicine (automated insulin dosers, robotic surgery, virtual patient trials). Swanky (`GaloisInc/swanky`, MIT): Rust MPC libraries (garbled circuits, ZK proofs, oblivious transfer, PSI) for privacy-preserving clinical analysis. C2Rust (with Immunant): C→Rust migration for modernising legacy C CDS implementations.

### Japan-specific considerations
No Galois office or named contributors in Japan; limited direct engagement (primary users: US defense, aerospace, cloud — NSA, AWS, DARPA contractors). Adjacent: Kohei Suenaga (Kyoto University) SMT-based hybrid-system verification; Naoki Kobayashi (University of Tokyo) higher-order model checking with MoCHi targeting OCaml — Crucible's Haskell-native symbolic execution extends naturally there; NII MTSS group (Hasuo) CPS symbolic verification. No Japan-specific Cryptol/SAW case study or deployment identified. BSD-3-Clause removes IP barriers. JAIST Kazuhiro Ogata group + AIST formal methods: complementary algebraic-specification (CafeOBJ/Maude) and process-algebra spaces interoperating conceptually with Crucible's CFG approach.

### Interoperability
- Within Cat 4: SAW/Crucible verify implementations of specs in Lean (§1, extracted code), Why3/WhyML (§6, C extraction), F* (§7, KaRaMeL C or Rust extraction). `crux-mir` handles Rust verified under Creusot/Prusti contracts. CrossHair (§11) covers Python; Crucible covers C/Java/Rust/Go/WASM — together spanning major deployment languages. What4 shares solver backends with `lean-smt` (§1) and Sledgehammer (§3). Copilot runtime monitors complement §§1–11 static verification with temporal-property monitoring at deployment.
- Cat 1: FHIR Clinical Reasoning engines in Java (HAPI) or Rust SAW/Crucible-verified against CQL/ELM specs; FHIRPath evaluator correctness via Cryptol specification of operator semantics.
- Cat 5: What4 = alternative interface to the same Z3/cvc5/Bitwuzla backends (that category's §1); SAW solver portfolio overlaps.
- Cat 8: tool-calling agents (§5) invoke SAW/Crucible as verification backends behind MCP, closing the LLM-generated-code → formal-check loop.
- Cat 10: s2n pattern maps to Knowledge CI/CD (§9): SAW in CI per commit proving spec compliance. Copilot monitors map to Observability/Continuous Verification (§10) as formally-guaranteed runtime health checks complementing OTel tracing. SBOM/AIBOM (§8): BSD-3-Clause licensing + Haskell build chains integrate with SPDX 3.0 Software + AI profiles.

### Limitations/Known issues
Bounded by solver decidability — quantified properties over unbounded data structures need manual lemma decomposition/induction (SAWScript compositional overrides; not fully automated). Cryptol type system less expressive than Lean/Rocq (no dependent types beyond size arithmetic, no inductive types); excels at bit-level specs, not a general theorem prover. Crucible frontends inherit target-language complexity: LLVM = C undefined-behaviour minefield; JVM = reflection/class loading; MIR = Rust borrow semantics. No proof certificates — SAW results are tool-specific claims (contrast DRAT/Alethe). Haskell toolchain (GHC 9.6–9.12) adds build complexity outside Haskell shops. Community smaller/more specialised than Lean's or Z3's; modest Stack Overflow. Healthcare work to date = device security + data privacy, not guideline formalization — CDS guideline-pipeline verification is a novel use case without published precedent.

### Training data proxy
Moderate. Cryptol: ~1,200 stars; Programming Guide + reference manual public at `tools.galois.com`; public Cryptol examples: AES, SHA, ECDSA, CNSA 2.0 post-quantum algorithms. SAW: 505 stars; s2n CAV 2018 (Chudnov et al.) highly cited; SAWScript tutorial + manual at `tools.galois.com/saw`. Crucible: 767 stars; VSTTE 2016 (Dockins, Foltzer, Hendrix, Huffman, McNamee, Tomb, "Constructing Semantic Models of Programs with the Software Analysis Workbench"). Copilot: 822 stars; Pike, Wegmann, Niller, Goodloe, "Copilot: A Hard Real-Time Runtime Monitor", RV 2010. Less LLM training presence than Lean/Z3/FHIR but sufficient with in-context examples. Thin Stack Overflow/community-forum presence; support via Galois commercial consulting + GitHub issue trackers.
