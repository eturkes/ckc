# Category 4: Formal Specification, Proof Assistants, and Proof Engineering

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
Dependent/refinement type systems (LiquidHaskell, Idris 2, Agda, refinement-reflection) make IR schema invariants part of the type, statically enforced. Cross-cutting technique, not a single tool; sits within the typed-functional substrate of §13.

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
Mature, active; all major tools had 2026 releases. SAW: LLVM 3.5–20.0 (all modern clang), JVM bytecode, Rust MIR. Cryptol: REPL, batch mode, "Literal Cryptol" for weaving specs into documents. Docs at `tools.galois.com/cryptol` and `tools.galois.com/saw`. s2n continuous verification (CAV 2018) = canonical industrial case; AWS still uses SAW for libcrypto. Galois also verified portions of libgcrypt and Bouncy Castle (Java). DARPA HACMS ($18M with Rockwell Collins, Data61, Boeing, University of Minnesota; 4.5 years): Ivory (Haskell EDSL for safe C) + Tower concurrency framework → SMACCMPilot verified quadcopter autopilot. DARPA SSITH/BESSPIN ($16.6M): hardware security via RISC-V FPGA. NRC HARDENS: formally verified reactor trip system via Rigorous Digital Engineering. Galois healthcare practice (galois.com/solutions/healthcare): secure health-data analytics (MPC, PSI, differential privacy), cyber-physical medical-device security (ARPA-H UPGRADE/SAFE-Dev hospital cybersecurity), ML safety for personalized medicine (automated insulin dosers, robotic surgery, virtual patient trials). Swanky (`GaloisInc/swanky`, MIT): Rust MPC libraries (garbled circuits, ZK proofs, oblivious transfer, PSI) for privacy-preserving clinical analysis. C2Rust (with Immunant): C→Rust migration for modernising legacy C CDS implementations (§14).

### Japan-specific considerations
No Galois office or named contributors in Japan; limited direct engagement (primary users: US defense, aerospace, cloud — NSA, AWS, DARPA contractors). Adjacent: Kohei Suenaga (Kyoto University) SMT-based hybrid-system verification; Naoki Kobayashi (University of Tokyo) higher-order model checking with MoCHi targeting OCaml — Crucible's Haskell-native symbolic execution extends naturally there; NII MTSS group (Hasuo) CPS symbolic verification. No Japan-specific Cryptol/SAW case study or deployment identified. BSD-3-Clause removes IP barriers. JAIST Kazuhiro Ogata group + AIST formal methods: complementary algebraic-specification (CafeOBJ/Maude) and process-algebra spaces interoperating conceptually with Crucible's CFG approach.

### Interoperability
- Within Cat 4: SAW/Crucible verify implementations of specs in Lean (§1, extracted code), Why3/WhyML (§6, C extraction), F* (§7, KaRaMeL C or Rust extraction). `crux-mir` handles Rust verified under Creusot/Prusti contracts (§14). CrossHair (§11) covers Python; Crucible covers C/Java/Rust/Go/WASM — together spanning major deployment languages. What4 shares solver backends with `lean-smt` (§1) and Sledgehammer (§3). Copilot runtime monitors complement §§1–11 static verification with temporal-property monitoring at deployment.
- Cat 1: FHIR Clinical Reasoning engines in Java (HAPI) or Rust SAW/Crucible-verified against CQL/ELM specs; FHIRPath evaluator correctness via Cryptol specification of operator semantics.
- Cat 5: What4 = alternative interface to the same Z3/cvc5/Bitwuzla backends (that category's §1); SAW solver portfolio overlaps.
- Cat 8: tool-calling agents (§5) invoke SAW/Crucible as verification backends behind MCP, closing the LLM-generated-code → formal-check loop.
- Cat 10: s2n pattern maps to Knowledge CI/CD (§9): SAW in CI per commit proving spec compliance. Copilot monitors map to Observability/Continuous Verification (§10) as formally-guaranteed runtime health checks complementing OTel tracing. SBOM/AIBOM (§8): BSD-3-Clause licensing + Haskell build chains integrate with SPDX 3.0 Software + AI profiles.

### Limitations/Known issues
Bounded by solver decidability — quantified properties over unbounded data structures need manual lemma decomposition/induction (SAWScript compositional overrides; not fully automated). Cryptol type system less expressive than Lean/Rocq (no dependent types beyond size arithmetic, no inductive types); excels at bit-level specs, not a general theorem prover. Crucible frontends inherit target-language complexity: LLVM = C undefined-behaviour minefield; JVM = reflection/class loading; MIR = Rust borrow semantics. No proof certificates — SAW results are tool-specific claims (contrast DRAT/Alethe). Haskell toolchain (GHC 9.6–9.12) adds build complexity outside Haskell shops. Community smaller/more specialised than Lean's or Z3's; modest Stack Overflow. Healthcare work to date = device security + data privacy, not guideline formalization — CDS guideline-pipeline verification is a novel use case without published precedent.

### Training data proxy
Moderate. Cryptol: ~1,200 stars; Programming Guide + reference manual public at `tools.galois.com`; public Cryptol examples: AES, SHA, ECDSA, CNSA 2.0 post-quantum algorithms. SAW: 505 stars; s2n CAV 2018 (Chudnov et al.) highly cited; SAWScript tutorial + manual at `tools.galois.com/saw`. Crucible: 767 stars; VSTTE 2016 (Dockins, Foltzer, Hendrix, Huffman, McNamee, Tomb, "Constructing Semantic Models of Programs with the Software Analysis Workbench"). Copilot: 822 stars; Pike, Wegmann, Niller, Goodloe, "Copilot: A Hard Real-Time Runtime Monitor", RV 2010. Less LLM training presence than Lean/Z3/FHIR but sufficient with in-context examples. Thin Stack Overflow/community-forum presence; support via Galois commercial consulting + GitHub issue trackers.
## 13. Typed Functional Programming as the Substrate for Verifiable CDS

### Purpose
Cross-cutting foundations entry, not a single tool — framing analogous to §8 (Dependent/Refinement-Type IR Schemas) and §9 (Proof by Reflection), both of which depend on this substrate. Nearly every system in §§1–12 — Lean 4, Rocq's Gallina, Isabelle/HOL's inner logic, Agda, Idris 2, F*, LiquidHaskell, Why3/WhyML, §9's reflected decision procedures — is structurally a typed lambda calculus extended with inductive data types, pattern matching, parametric polymorphism, and (where present) dependent/refinement types. The substrate view clarifies (i) why autoformalization targets these languages over imperative ones, (ii) why pure-functional rule evaluators uniquely suit regulated CDS, (iii) where Haskell-, OCaml-, F*-family ecosystems contribute beyond the catalogued proof-assistant entries: host languages for embedded guideline DSLs, refinement-typed FHIR adapters, equationally-reasoned rule pipelines.

### Maintainer/Standards body
- **Curry–Howard correspondence**: Haskell B. Curry, "Functionality in Combinatory Logic" (1934); William A. Howard, "The Formulae-as-Types Notion of Construction" (1969 manuscript, published 1980 in Seldin & Hindley eds., *To H. B. Curry: Essays on Combinatory Logic, Lambda Calculus and Formalism*).
- **ML lineage**: Robin Milner et al., Edinburgh, "A Theory of Type Polymorphism in Programming" (JCSS 1978); ML was originally the *meta-language* of the LCF prover (Milner, Gordon, Wadsworth, *Edinburgh LCF*, 1979) — a functional language invented because proof tactics required higher-order, statically-typed manipulation of theorems. Standard ML: Milner, Tofte, Harper, MacQueen, *The Definition of Standard ML (Revised)*, MIT Press 1997.
- **Haskell**: Haskell Language Committee; Haskell 1.0 Report (1990; Hudak, Peyton-Jones, Wadler, et al.); Haskell 2010 Report (Simon Marlow, ed.). De facto standard is **GHC Haskell**, GHC Team (Simon Peyton-Jones, Simon Marlow, Richard Eisenberg, Ben Gamari, Simon Hengel, Sebastian Graf, Adam Gundry, et al.), funded by Microsoft Research, Well-Typed, Tweag, IOG, and the Haskell Foundation (Haskell Foundation Inc., 501(c)(3), founded 2020).
- **OCaml**: Inria (Xavier Leroy, Damien Doligez, Jacques Garrigue, Didier Rémy, Gabriel Scherer, Nicolás Ojeda Bär); OCaml Software Foundation (2018–).
- **F***: Microsoft Research + Inria (Nikhil Swamy, Cătălin Hriţcu, Chantal Keller, Aseem Rastogi, Antoine Delignat-Lavaud, Jonathan Protzenko, Tahina Ramananandro, Aymeric Fromherz); catalogued in §7 as a language, here as ML-family member.
- **Liquid Types**: Patrick Rondon, Ming Kawaguchi, Ranjit Jhala, "Liquid Types" (PLDI 2008, Tucson).
- **Propositions as Types** (canonical exposition): Philip Wadler, *CACM* 58(12), December 2015.
- **Pedagogical canon**: Benjamin C. Pierce, *Types and Programming Languages* (MIT Press 2002), *Advanced Topics in Types and Programming Languages* (2005); Pierce et al., *Software Foundations* (online, Coq/Rocq-based, updated through 2024); Adam Chlipala, *Certified Programming with Dependent Types* (MIT Press 2013). These texts ground most current LLM training data on functional verification.

### Conceptual model
Three intertwined ideas:
- **Curry–Howard** (type = proposition, term = proof, β-reduction = proof normalisation): a total, type-checking `eligibleForStatin : PatientRecord → Maybe Recommendation` is, dependently typed, simultaneously an executable rule and a constructive proof of "every patient record yields either no recommendation or the named one, with the applicability proof recoverable".
- **ADTs + pattern matching**: clinical entities decompose as sums-of-products. `data LabResult = Numeric Double Unit Timestamp | Categorical Code Timestamp | Missing Reason Timestamp` enumerates the closed case set; the compiler checks every rule covers every branch; illegal states (a `Numeric` with no `Unit`) become unrepresentable.
- **Purity/referential transparency** (substitutability `a ≡ b ⇒ f a ≡ f b`): makes (a) proof-by-reflection (§9) work, (b) equational reasoning a sound rule-pipeline refactoring tool, (c) audit replay (Category 10) bitwise reproducible from logged inputs, (d) property-based testing (Category 9, QuickCheck lineage) meaningful — random inputs probe a mathematical function, not a stateful black box.

Layered on top: parametric polymorphism (System F); type classes/higher-kinded types (System Fω + qualified types, Wadler & Blott, POPL 1989); GADTs (Cheney & Hinze; Peyton-Jones et al., ICFP 2006) for typed ASTs and tagless-final EDSLs; monadic and algebraic-effect encodings (Moggi LICS 1989; Plotkin & Power FOSSACS 2002; Plotkin & Pretnar ESOP 2009) for explicit effect surfaces; refinement/dependent extensions (§8) for invariant-bearing schemas.

### Expressiveness/Semantics
Relevant lambda-cube fragment:
- Simply-typed λ-calculus + base types ≈ HOL (Isabelle/HOL inner logic).
- System F (rank-1 polymorphism) ≈ Hindley–Milner (ML, OCaml, early Haskell).
- System Fω + type families + GADTs ≈ modern GHC Haskell; together they encode a useful dependent-types fragment via singleton-encoded value reflection (Eisenberg & Weirich, "Dependently Typed Haskell" line of work; PHaskell prototype, ICFP 2021).
- Refinement-extended F ≈ F*, LiquidHaskell (§8).
- Full dependent types (Π, Σ) ≈ Idris 2, Agda, Lean 4, Rocq Gallina (§§1, 2, 8).

Equational reasoning is the load-bearing semantic property: in a pure core, `let x = e in body` is observably identical to substitution, so a rule like `if eGFR < 60 ∧ persistsFor 90Days then ChronicKidneyDiseaseG3 else …` can be rewritten under any context-equivalence-preserving transformation without changing the externally observed verdict — false for the equivalent Java method, where hidden mutable state, exceptions, or lazy-initialised singletons make `x` and `e` observably differ.

### Composability/Modularity
- **ML-style functors (parameterised modules)**: `module RuleEngine (T : TerminologyService) = struct … end` — rule engine parameterised over a terminology service.
- **Haskell type classes** (Wadler & Blott, POPL 1989): ad-hoc polymorphism; `class CodeSystem c where … instance CodeSystem SnomedCT where … instance CodeSystem JLAC where …` lets the same rule code operate over Western or Japanese terminologies with type-directed dispatch.
- **Backpack** (Edward Z. Yang, POPL 2014): Haskell module signatures; guideline modules type-check against terminology *interfaces* before any concrete code system is linked.
- **Tagless-final encoding** (Carette, Kiselyov, Shan, JFP 2009) and **free monads** (Swierstra, "Data types à la carte", JFP 2008): canonical techniques for embedding a guideline DSL whose single source expression can be (a) executed against a live FHIR server, (b) traced for audit, (c) reduced symbolically by an SMT backend, (d) emitted as a CQL/DMN artefact, (e) re-interpreted under a property-based test harness — the operational answer to one IR targeting Categories 1, 5, 9, 10 simultaneously.
- **Lens/optics** (van Laarhoven 2009; Edward Kmett's `lens`, `optics`): compositional read/write access to deeply-nested clinical records (FHIR Bundles, CDA documents) without imperative traversal code.

### Suitability for autoformalization to IR
Every LLM autoformalization target here — Lean 4, Rocq, Isabelle/Isar, Agda, Idris 2, F* — is typed functional. Two structural advantages for LLMs:
- **Small surface syntax, strong local checking**: ADTs + pattern matching let an LLM emit a rule as a closed expression whose well-formedness is checkable in a single typechecking pass; ill-typed output is rejected before any semantic check. Compiler-error signal-to-noise is much higher than for Python, where syntactic acceptance implies little.
- **EDSL-style emission — "emit data, not control flow"**: the most reliable LLM-emission pattern in the autoformalization literature (Category 8): produce a tagless-final term or free-monad ADT that *describes* the guideline, leaving evaluation, validation, and proof discharge to fixed, human-vetted interpreters that independently target FHIR (Category 1), SMT (Category 5), audit logs (Category 10). Structurally easier than emitting a fully proved Lean term.

Pure-functional outputs are normalisable (β-reduction, `simp` lemmas, GHC `-O2` rewrite rules): semantically-equivalent-but-syntactically-different re-prompts canonicalise to a single normal form, sharply improving stability across regenerations.

### Formal verification potential
By Curry–Howard, a closed term of type `T` in a sufficiently expressive functional core *is* a proof of `T` up to consistency of the type theory: a total `validatePrescription : Prescription → Either Error (Verified Prescription)` whose codomain encodes the verification obligations is both an executable validator and a proof object surviving extraction, kernel re-checking, and (via §10) certificate emission. Wadler's "Propositions as Types" (CACM 2015) is the canonical short exposition. Pierce's *Software Foundations* (VFA and PLF volumes) is the most widely-used FP→verification pedagogical bridge and, by GitHub stars and citation, the single most referenced training corpus for LLM-targeted proof generation. CDS-relevant artefacts:
- **CompCert** (Leroy, Inria; *CACM* 2009): verified C compiler with pure-functional passes, correctness mechanised in Coq; shows the substrate scales to industrial software with regulatory implications (RTCA DO-178C avionics qualification).
- **seL4** (Klein et al., SOSP 2009): OS-kernel functional-correctness proof, Haskell prototype refined to verified C; benchmark for "verified" in safety-critical contexts comparable to ISO 14971 medical-device risk.
- **Project Everest / miTLS / HACL\*** (Microsoft Research + Inria, 2016–): production verified TLS stack in F*; same substrate could host a verified CDS rule engine.

### Tooling/Ecosystem maturity
- **GHC Haskell** (current series 9.10/9.12): production-grade. Industrial users: Standard Chartered (entire quant library; Wei Hu, ICFP 2014 keynote), Meta's Sigma anti-abuse system (Jon Coens, Marlow, et al.; ~1M req/s in Haskell), Mercury Bank, Anduril, GitHub Semantic, IOG (Cardano).
- **HLS (Haskell Language Server)**: LSP with type-on-hover, code lenses, hlint integration; Haskell Foundation HLS team.
- **Cabal / Stack / Nix**: three coexisting build tools; Hackage hosts ~16 000 packages; Stackage provides curated LTS snapshots.
- **GHC plugins**: LiquidHaskell (§8), Plutus, type-checker plugins (`ghc-typelits-natnormalise`).
- **OCaml 5** (effect handlers, parallelism): Inria; Jane Street largest industrial user (millions of LoC; financial trading); MirageOS unikernels; Coq/Rocq itself.
- **F\* / Karamel / EverCrypt** (Project Everest): production in Windows kernel HTTPS stack and Linux kernel WireGuard implementation; CI under Microsoft Research.
- **Lean 4** (§1): implemented in itself, compiles to C, a complete functional language.

### Japan-specific considerations
- **Kazu Yamamoto** (IIJ-II — Internet Initiative Japan Innovation Institute): one of the most prolific Japanese production-Haskell engineers; author/maintainer of `tls`, `http2`, `http3`, `quic`, `dns`, `iproute`, and the Mighttpd2 web server — the network stack under a non-trivial fraction of Japan-hosted Haskell services. Mighttpd demonstrates high-throughput production Haskell is operationally viable in Japanese infrastructure, which matters for hosting clinical-rule services nationally.
- **Atsushi Igarashi** (Kyoto University): refinement types, gradual typing, Featherweight Java (AITO Dahl-Nygaard Junior Prize 2011); cited in §8.
- **Eijiro Sumii** (Tohoku University): polymorphism, type-system metatheory, bisimulation for higher-order languages; POPL/ICFP author.
- **Naoki Kobayashi** (University of Tokyo): higher-order model checking, intersection types, type-based verification (HORS, HFL model checking; ERATO MMSD collaborator); his MoCHi tool verifies higher-order OCaml programs against safety properties — directly applicable to verified OCaml-hosted CDS rule engines.
- **Susumu Katayama** (Miyazaki University): MagicHaskeller inductive program synthesis from types — precursor to LLM-driven program synthesis.
- **Reynald Affeldt** (AIST): MathComp/SSReflect formalisations; `monae` monad library — Japanese-hosted reusable functional verification infrastructure.
- **Yoshihiko Futamura** (Meiji Gakuin, emeritus): partial evaluation (Futamura projections, *Systems, Computers, Controls* 1971) — foundational for specialising guideline interpreters to fixed guidelines; direct CDS performance implications.
- **Community**: Haskell-jp (haskell.jp), regular Haskell-jp Mokumoku sessions, "関数プログラミング" community in Tokyo and Osaka; Japanese translations of Hutton's *Programming in Haskell* (2009/2017) and Bird's *Thinking Functionally with Haskell* (2014); native textbooks by Yamamoto, Kenji Yoshida (Septeni; `xuwei-k`), and others.
- **PPL Workshop** (Programming and Programming Languages, JSSST): annual venue for Japanese FP research; relevant for recruiting collaborators on CDS DSL work.
- **AMED relevance**: AMED-funded clinical-AI projects to date use predominantly Python and Java stacks; a typed-functional CDS layer is unrepresented in AMED-funded CDS — simultaneously a gap and an opportunity. Closest extant work: Igarashi's HELMHOLTZ (Tezos/Michelson smart-contract refinement-type verifier, not clinical) and Kobayashi's MoCHi (general higher-order verifier).

### Interoperability
- **Within Category 4**: underlies §§1, 2, 3, 7, 8, 9 — §1 Lean 4 is typed-functional with dependent types; §2 Gallina is a pure functional core (CIC); §3 Isabelle/HOL inner logic is essentially typed lambda calculus; §7 F* is ML-family; §8 LiquidHaskell + Idris + Agda explicitly functional; §9 reflection is sound only because the reduction semantics are pure-functional. §10 proof certificates serialise functional λ-terms in λΠ-calculus. §11 CrossHair, though Python-targeted, is structurally a symbolic *function* interpreter — tractable by treating user code as if pure.
- **Category 1 (Computable Guideline IR & CDS Standards)**: Haskell FHIR libraries (`fhir`, `hs-fhir` — community-maintained, smaller than Java HAPI FHIR but useful as typed reference implementation); **Servant** (Alp Mestanogullari, Julian Arni et al.) for type-level-routed FHIR REST APIs with type-checker-enforced request/response shapes; CQL-grammar parser-combinators straightforward in `megaparsec`/`parsec`; DMN decision tables as sum types with exhaustive matches.
- **Category 2 (Japan Clinical Guideline & Terminology)**: SNOMED CT, ICD-10, JLAC, MEDIS code spaces as `newtype`-wrapped identifiers with smart constructors enforcing well-formedness — illegal codes unrepresentable; Minds-derived recommendation predicates as Haskell ADTs directly consumable by LiquidHaskell (§8) and Lean 4 (§1) via straightforward syntactic mapping.
- **Category 3 (Ontologies/RDF/Terminology Engineering)**: `rdf4h`, `hsparql` SPARQL bindings; SHACL closure rules map onto refinement-type predicates (§8); OWL class restrictions correspond to Σ-types in Idris/Agda, reachable from Haskell via singleton-encoded `data`.
- **Category 5 (Automated Reasoning & Constraint Solving)**: **SBV** (Levent Erkök; "Symbolic Bit Vectors") — canonical Haskell binding for Z3, CVC5, Yices, MathSAT, Boolector, ABC — discharges a guideline-as-Haskell-function directly to SMT. Why3/WhyML (§6) is ML-family by construction. SMT-LIB itself is an s-expression-shaped functional language, parseable in tens of lines of Haskell.
- **Category 6 (Clinical Rule Semantics & Temporal Reasoning)**: Allen's interval algebra (Allen, *CACM* 1983) has a textbook encoding as a 13-constructor Haskell ADT with pattern-matching rule combinators; FRP libraries (Yampa, reflex-frp, threepenny-gui; Conal Elliott & Paul Hudak, ICFP 1997 for FRP origin) give a typed substrate for streaming clinical-event reasoning with explicit temporal semantics.
- **Category 7 (Information Retrieval/RAG)**: less direct; Hasktorch (Haskell LibTorch bindings) and Servant-based RAG service shells exist; primary contribution is wrapping untyped LLM I/O in typed contracts at the application boundary.
- **Category 8 (LLM/Agentic Autoformalization)**: LLM-emitted Lean/Rocq/Agda/F*/LiquidHaskell *is* typed-functional code; GHC's `-Wall -Werror` and Lean's elaborator serve the same "syntactic gate" pattern — only well-typed agent outputs survive the inner loop; tagless-final EDSL emission lets LLM-generated guideline data flow to multiple downstream interpreters in one round.
- **Category 9 (Evaluation/Clinical Validation)**: **QuickCheck** (Koen Claessen & John Hughes, ICFP 2000, Montréal; cited at `evaluation-clinical-validation.md:174`) — canonical Haskell-origin property-based testing of pure code; equational reasoning lets shrinkers minimise counter-examples without re-running expensive harness setup; SmallCheck (Runciman, Naylor, Lindblad, Haskell Symposium 2008) gives exhaustive bounded-depth alternatives.
- **Category 10 (Regulatory Compliance/Assurance)**: pure rule evaluators yield trivially reproducible audit trails — same input record + same compiled binary ⇒ bitwise-identical output, no hidden state to capture — eliminating the audit-non-determinism of stateful CDS evaluators; directly supports IEC 62304 §5.7 (verification) and §8 (configuration management) reproducibility evidence, FDA SaMD pre-market documentation, and Japan's PMDA Notification 0411 No. 1 (program-medical-device software-lifecycle) documentation. Futamura-style partial evaluation of a guideline interpreter to a fixed Minds-J 2023 guideline yields a residual specialised binary whose audit surface is just that guideline — relevant to per-guideline release certification.

### Limitations/Known issues
- **Industrial CDS ecosystems predominantly imperative**: HAPI FHIR (Java), Smart-on-FHIR client libraries (JavaScript/TypeScript), CDC FHIR validators (Java), most hospital EHR-side APIs (C# for US vendors, Java for Cerner/Oracle Health, varied for Japan vendors). FFI bridging (Haskell `inline-java`, OCaml `ctypes`, F* `KaRaMeL` to C) is real but adds operational complexity.
- **Hiring pool**: healthcare-informatics × FP is a small intersection; Japan recruiting feasible (IIJ-II, AIST, university spinouts) but smaller than Python/Java teams.
- **Laziness (Haskell-specific)**: GHC's default non-strict evaluation complicates space/time reasoning for streaming clinical event data; `StrictData`, `Strict`, bang patterns, `nothunks` mitigate but require discipline. OCaml/F*/SML are strict by default.
- **Module-system gaps**: Haskell's module system (even with Backpack) is weaker than SML's; OCaml functors more expressive but less integrated with type-class style; the mismatch hampers cross-ecosystem code reuse.
- **"Avoid success at all costs"** (Peyton-Jones's Haskell slogan): community prefers experimental language evolution over backward compatibility — relevant where ABI/API stability over a 10-year medical-device lifecycle matters. Pin to LTS Stackage or Nix-managed flake inputs.
- **Open refinement/dependent problems**: type-class coherence under refinement partially open (LiquidHaskell handles it; GHC does not natively); decidability boundaries of refinement predicate logics are SMT-dependent.
- **Floating-point semantics**: physiological-threshold rules over reals need IEEE-754-bit-precise reasoning (slow) or rational/interval surrogates (lossy); same issue noted in §11 CrossHair limitations.
- **Documentation quality varies**: GHC user guide excellent; LiquidHaskell, Idris 2, Agda uneven; F* improved markedly post-2022 but still steeper than mainstream FP onboarding.

### Training data proxy
- **Haskell**: Hackage ~16 000 packages; GitHub Haskell repositories in the low hundreds of thousands. Canonical pretraining-corpus texts: Hutton, *Programming in Haskell* (2nd ed. 2016, Cambridge); Bird, *Thinking Functionally with Haskell* (2014, Cambridge); Hudak, Hughes, Peyton-Jones, Wadler, "A History of Haskell: Being Lazy with Class" (HOPL III 2007); Marlow, *Parallel and Concurrent Programming in Haskell* (O'Reilly 2013); Lipovača, *Learn You a Haskell for Great Good!* (2011, No Starch); university course materials from Edinburgh, Cambridge, Glasgow, Utrecht, Chalmers, UPenn (CIS 194). Pierce, *Software Foundations* (online, Coq-based): the single largest functional-verification training corpus.
- **OCaml**: Madhavapeddy, Minsky, Hickey, *Real World OCaml* (O'Reilly, 2nd ed. 2022); Inria's OCaml documentation; Jane Street's open-source `core`/`async`.
- **F\***: Project Everest publications; F* tutorial (`fstar-lang.org`); smaller corpus than Haskell/OCaml but growing.
- **F#**: Scott Wlaschin, *Domain Modeling Made Functional* (Pragmatic Bookshelf 2018) — CDS-relevant because its worked examples are ADT domain-modelling and railway-oriented programming directly applicable to clinical-decision pipelines.
- **LLM coverage**: Haskell moderately represented (current models produce idiomatic code with reasonable type-class usage and monad-transformer stacks); OCaml sparser; F* and Idris sparse enough that in-context exemplars help substantially (Category 8 retrieval-augmented synthesis).
- **Conferences**: ICFP (annual; canonical venue), POPL, PLDI, Haskell Symposium, OCaml Workshop, ML Workshop, TyDe (Type-Driven Development), CPP (Certified Programs and Proofs). Japan-relevant: PPL (JSSST, annual); IFIP WG 2.8 Functional Programming meetings held in Japan (Kobe 2018, Tomakomai 2022).
## 14. Memory-Safe Systems Languages: Rust, Ada/SPARK, and the Production Substrate

### Purpose
Deployment substrate complementing §13: §13 = typed-functional core where guideline rules are reasoned about; this = systems substrate where CDS binaries, FHIR servers, ingestion pipelines, audit-log writers run, with compile-time elimination of the memory-safety vuln classes (use-after-free, double-free, buffer overrun, data race) historically dominating CVEs in clinical stacks on C/C++/hand-tuned JNI. Memory safety = deployment-time property, complementary to specification-time formal correctness (§§1–12) and substrate-time equational reasoning (§13); a verified rule in a memory-unsafe runtime is not a verified CDS system — the assurance argument leaks through the runtime. Substrate framing clarifies: (i) CISA's January 1, 2026 memory-safety roadmap mandate has direct implications for any CDS vendor on the Secure-by-Design Pledge; (ii) Safety-Critical Rust Consortium + Ferrocene's IEC 62304 Class C qualification are first-order for Class III SaMD architecture; (iii) where Rust, Ada/SPARK, and (varying degrees) Swift/Go/Java/C# fit alongside §13's ecosystem. Cross-cutting entry, parallel in framing to §13.

### Maintainer/Standards body
- **Rust**: Rust Foundation (501(c)(6), Delaware, founded February 2021); Rust Project leadership council (succeeds core team since December 2022 governance restructure). Linux 7.0 kernel builds anchor to Rust 1.93 (Debian stable toolchain). Frozen at 2024 edition; 6-week release cadence.
- **Safety-Critical Rust Consortium**: founded June 2024 under Rust Foundation; ten founders — AdaCore, Arm, Ferrous Systems, HighTec EDV-Systeme, Lynx Software Technologies, OxidOS, TECHFUND, TrustInSoft, Veecle, **Woven by Toyota** (Tokyo). Publishes in-development *Safety-Critical Rust Coding Guidelines* (GitHub `Safety-Critical-Rust-Consortium/safety-critical-rust-coding-guidelines`); driving 2026 Rust Project Goal for MC/DC (Modified Condition/Decision Coverage) — the DO-178C/DO-254 DAL A criterion, transferrable to airborne implantable medical devices.
- **Ferrocene** (qualified Rust toolchain): Ferrous Systems GmbH (Berlin); TÜV SÜD qualification body. **Ferrocene 26.02.0** (February 2026) covers Rust 1.91/1.92, adds ISO 26262 ASIL B certification for a curated `core` subset; retains TÜV SÜD qualifications ISO 26262 ASIL D (highest), IEC 61508 SIL 3, and **IEC 62304 Class C** (highest medical-device software-safety class; announced January 14, 2025, first such qualification for any Rust toolchain). Supports customer certification toward IEC 61508 SIL 4 and DO-178C DAL C. Source open (Apache-2.0/MIT) since 2024.
- **Ada / SPARK**: AdaCore (Paris/New York; founders Cyrille Comar, Robert Dewar d. 2015). Ada 2022 (ISO/IEC 8652:2023); SPARK 2014/2022. SPARK Pro = production verification toolchain; GNAT Community / GNAT FSF open source. Markets explicitly to medical devices (`adacore.com/industries/medical`).
- **Swift**: Apple; Core Team (Chris Lattner originated; led by Doug Gregor, Ben Cohen, et al.). Swift 6.2 ships strict-memory-safety mode aligned with NSA/CISA guidance, positioned for defense/regulated workloads; compiles to Android, Windows, embedded (Swift Embedded subset).
- **Go**: Google (Griesemer, Pike, Thompson). Memory-safe via GC; concurrency-safe only via runtime race detector — not static. Go 1.24 (2025): range-over-func iterators, `GOAUTH` supply-chain tightening.
- **Java / C# / Kotlin / JVM-family**: managed-runtime memory safety; implicit in HAPI FHIR (Java) and most EHR-vendor stacks; comparison baseline only here.
- **CISA / NSA**: joint info sheet *Memory Safe Languages: Reducing Vulnerabilities in Modern Software Development* (June 24, 2025) updates 2023 *The Case for Memory Safe Roadmaps*. CISA Secure-by-Design Pledge (296 signatories as of May 2026) sets **January 1, 2026 deadline** for signatories to publish a memory-safety roadmap. Sheet highlights **Ada and Rust** as the two languages combining memory safety with safety-critical performance/predictability.
- **DARPA TRACTOR** (Translating All C to Rust): DARPA Information Innovation Office, announced July 2024; T&E by MIT Lincoln Laboratory. Automated C→Rust translation matching skilled-human style; static/dynamic analysis + LLM synthesis. Academic contract awards from mid-2025.
- **Zig**: Zig Software Foundation (Andrew Kelley). Pre-1.0 as of May 2026; included only to clarify *Zig is not memory-safe* in the CISA sense (no borrow checker, no type-system lifetime enforcement; runtime safety in debug builds only, UB in release).

### Conceptual model
Four intertwined ideas, Rust canonical:
- **Ownership/affine types**: one owner per value; pass/return transfers ownership; owner's scope = lifetime. Affine type system (each binding used at most once; `Copy` structural exception). Deterministic drop at scope exit = RAII without GC pauses. Antecedent: Wadler, "Linear Types Can Change the World!" (TPL 1990); canonical mechanised soundness proof for safe Rust + well-bracketed `unsafe`: RustBelt (Jung, Jourdan, Krebbers, Dreyer, POPL 2018; Dreyer 2019 ICFP keynote).
- **Borrowing/lifetimes**: `&T` shared, `&mut T` mutable with statically-checked non-aliasing; mutable refs exclusive, no ref outlives referent — static "law of exclusivity" (Swift uses same term for its mixed static/dynamic enforcement). Aliasing model for `unsafe`: **Stacked Borrows** (Jung, Dang, Kang, Dreyer, POPL 2020) → **Tree Borrows** (Villani, Jung, et al., 2023–2025), ~54% fewer false-positive rejections of legitimate unsafe patterns while preserving compiler optimisations. **Miri** (Ralf Jung, Benjamin Kimock, Christian Poveda, Eduardo Sánchez Muñoz, Oli Scherer, Qian Wang; *Miri: Practical Undefined Behavior Detection for Rust*, POPL 2026): MIR-level interpreter implementing both models; de facto UB detector for `unsafe`.
- **Send/Sync**: thread-send and thread-share safety as auto-derived marker traits, compile-time checked; with ownership yields compile-time data-race freedom for safe Rust — distinguishing it from Go (GC-safe, runtime races), Java (memory-safe, race-prone), C++ (neither).
- **Pure-safety vs. proven-functional-correctness layering** (most important conceptual point): memory safety necessary, not sufficient, for CDS correctness — closes the security-vulnerability surface; functional correctness (§§1–12) closes the clinical-correctness surface. SPARK proves both simultaneously: Ada subset excluding pointer aliasing (or, since SPARK 2014, a Rust-style ownership pointer model) ∩ contract-derived proof obligations. Rust + Verus/Prusti/Creusot/Kani approach the same intersection from the other direction (memory-safe core, specification layered atop).

### Expressiveness/Semantics
Safe Rust ≈ subset of System F + ADTs + affine references accepted by the borrow checker:
- **ADTs + pattern matching**: structurally identical to §13. `enum LabResult { Numeric(f64, Unit, Timestamp), Categorical(Code, Timestamp), Missing(Reason, Timestamp) }`; `match` exhaustiveness same compile-time obligation. Operational differences: stack allocation by default, monomorphised generics, no laziness.
- **Traits**: Wadler-style ad-hoc polymorphism, isomorphic to Haskell type classes; coherence enforced (one `impl` per type-trait pair) with orphan rules. `trait CodeSystem` impls for `SnomedCT`, `JLAC`, `MEDIS` give type-directed dispatch to Japanese/Western terminologies.
- **Trait objects (`dyn Trait`)**: existential subtyping for dynamic dispatch (e.g., heterogeneous runtime rule collections).
- **`const` generics + GATs** (stable since Rust 1.65): analogue of GHC type families; type-level clinical-record schemas with unit-of-measure and reference-range constraints.
- **`unsafe` stratified escape hatch**: safety obligations in prose ("safety contract" of `unsafe fn` / blocks); stdlib encapsulates unsafe primitives so downstream code stays safe. **Rust Safety Tags RFC** (in flight 2026): machine-checkable safety annotations on every public stdlib `unsafe` API, verifiable by Clippy lints and rust-analyzer.
- **`async`/`.await`**: stackless coroutines compiled to state machines; `Future` trait + executor runtimes (Tokio, async-std, Smol, Embassy). Memory-safe under ownership; self-referential async state stable since Rust 1.39 (December 2019).

SPARK: Ada subset excluding (or constraining) pointer aliasing, exceptions, unbounded recursion; VCs discharged by GNATprove (Why3 + CVC5/Z3/Alt-Ergo backend, §6). Trades expressiveness for proof tractability. SPARK 2014 ownership-based pointers (Foughali et al., *Borrowing Safe Pointers from Rust in SPARK*, 2018) lifted SPARK from "no pointers" to "pointers under Rust-style aliasing discipline" with full proof discharge preserved.

### Composability/Modularity
- **Cargo + crates.io**: ~165,000 packages (May 2026); **RustSec Advisory Database** (~600 advisories, volunteer-maintained by Rust Security WG) + `cargo audit` for dependency scanning. Reproducible builds via `Cargo.lock`; offline via `cargo vendor`. Vs. Haskell (Cabal/Hackage/Stackage): monomorphisation + LTO → single static binaries with deterministic optimisation, suited to clinical release artefacts. Vs. Java (Maven/classpath): no runtime classloader, eliminating load-time dependency-override supply-chain attacks.
- **Workspaces + feature flags**: shared dependency resolution; compile-time configuration (e.g., Helios FHIR server `--features r4,r5,r6` selects resource model at build).
- **`#![no_std]` / `#![no_alloc]`**: stdlib stratified `core` (no alloc/IO) / `alloc` / `std`; same language targets embedded implantable monitors (`#![no_std]` + Embassy) and hospital servers. Ferrocene certified subset = `core` + curated `alloc` subset.
- **FFI**: `extern "C"` zero-cost C ABI; `bindgen` C-header→Rust; `cxx` (David Tolnay) typed Rust↔C++. Enables incremental migration from HAPI FHIR (Java/JNI) or Cerner-Oracle-Health backends without clean-room rewrites.
- **Ada `with`/`use` + child packages + generics**: predates ML's module system by a decade (Ada 83); closer to ML functors than type classes. SPARK preserves modularity, restricts expressiveness.

### Suitability for autoformalization to IR
Not the primary autoformalization target (§13 is); natural compilation target after:
- **Rust as target**: Lean 4 → Rust extractor (community-maintained; smaller surface than Coq/Rocq OCaml/Haskell extraction) emits idiomatic Rust from verified Lean terms. F* → KaRaMeL → C is production (HACL\*, Project Everest); KaRaMeL-to-Rust backend in development at Microsoft Research, 2024–2026. Pipeline: prove rule in Lean 4 (§1) → extract Rust → link into Rust-native FHIR server (Helios) → single static binary, reproducible build, IEC 62304-qualified toolchain (Ferrocene). Structurally cleaner than Lean → OCaml → dynamically-linked runtime for reproducibility and regulatory surface area.
- **Refinement-typed Rust** via Prusti/Creusot: specs live beside implementation; LLM emits annotated Rust — operationally easier than producing well-typed Lean. Verus adds *linear ghost types* for proof-relevant pointer-manipulation invariants.
- **Idempotency via determinism**: pure-safe-Rust `PatientRecord` → `Recommendation` is bitwise-deterministic on identical inputs (no hidden state, no allocator non-determinism in pure functions, no FP reordering under controlled codegen flags) — §13's property, achieved via ownership rather than purity.

### Formal verification potential
- **Verus** (Lattuada, Hance, Cho, Brun, Subasinghe, Zhou, Howell, Parno, Hawblitzel; OOPSLA 2023, "Verus: Verifying Rust Programs using Linear Ghost Types"; follow-up "Verus: A Practical Foundation for Systems Verification", Lattuada, Hance, Bosamiya, Brun, Cho, LeBlanc, Srinivasan, Achermann, Chajed, Hawblitzel, Howell, Lorch, Padon, Parno, SOSP 2024, Distinguished Artifact Award): SMT-backed; exploits ownership/borrow checking in proofs; linear ghost types; pointer-manipulating + concurrent Rust. Production-adjacent: Microsoft IronKV verified KV store, verified-bootloader work.
- **Prusti** (ETH Zürich; Astrauskas, Bílý, Fiala, Grannan, Matheja, Müller, Poli, Summers; *The Prusti Project: Formal Verification for Rust*, NFM 2022 — NASA Formal Methods 14th Symposium): deductive verifier on Viper separation-logic platform; `#[requires]`/`#[ensures]` attributes, loop invariants. Production at Swiss medical-imaging startups (per CHI 2025 case studies) and Boundary Layer.
- **Creusot** (Xavier Denis, INRIA): Rust contracts → Why3 VCs → SMT (Z3, CVC5, Alt-Ergo). Featured tutorial at **POPL 2026** (January 11, 2026). Used for a verified SAT solver. Its "prophecy encoding" of mutable references is more general than Verus' current support — broader idiom class verifiable, heavier proof obligations.
- **Kani** (AWS; Felipe Monteiro, Daniel Schwartz-Narbonne, et al.): bounded model checker, Rust MIR → Goto-C → CBMC. Strong on `unsafe`, weaker on unbounded loops. Open source MIT/Apache-2.0; used internally at AWS for parts of Firecracker microVM and Rust stdlib portions.
- **Rust Verification Tools (RVT) — Project Oak** (Google): static (KLEE, SeaHorn, SMACK) + dynamic verifiers behind unified Rust front end; smaller community, useful for ensemble verification.
- **Ada SPARK + GNATprove + Why3**: the mature stack. **Hillrom** ECG-algorithm migration C++ → Ada/SPARK (new + legacy; the whitepaper is the canonical, most-cited industrial medical case study); Thales (Astrée-verified avionics); SPARK 2014 *Heart-Pump* case study (Medical Design Briefs, *Formally Verifying Heart Pump Software with SPARK and Echo*); undisclosed cardiac-monitoring/infusion-pump customers per AdaCore's medical page.
- **Miri**: not a verifier; de facto UB detector for `unsafe`. Catches use-after-free, OOB access, uninitialised reads, alignment violations, data races, aliasing violations under Tree Borrows. `cargo +nightly miri test`.
- **Category-4 reachability**: Verus/Prusti-verified crates export the same proof-certificate shape (§10) as Lean 4 (§1) or Why3 (§6) — SMT discharge + checkable witnesses. TCB adds Rust compiler (large) + Verus/Prusti translator (medium) vs. Lean's small kernel — argues for Lean 4 as *specification* language, Rust as *implementation* language in dual-stack CDS architectures.

### Tooling/Ecosystem maturity
- **Toolchain**: `rustup`; **Ferrocene** for IEC 62304 / ISO 26262 / IEC 61508-qualified regulated use. `rust-analyzer` (LSP, originally Aleksey Kladov): VS Code, IntelliJ, Emacs, Helix, Zed. Clippy ~770 lints; `cargo fmt` (rustfmt) deterministic formatting.
- **Rust in Linux 7.0** (2026): experimental designation removed; mainline anchors Rust 1.93 / Debian stable. NVIDIA Nova GPU driver, Google Android `ashmem`, several NVMe and high-speed-networking drivers in production on hundreds of millions of devices. Policy: "Rust for new code, C for existing subsystems, no forced migrations" — hospital-server kernels and device firmware will run Rust drivers under CDS workloads. ~Two-thirds of historical Linux CVEs trace to memory-safety bugs the borrow checker would prevent.
- **Rust FHIR ecosystem (production, 2025–2026)**:
  - **Helios FHIR Server** (`HeliosSoftware/hfs`): HL7 FHIR R4, R4B, R5, R6, feature-flag selectable; clinical-analytics optimised.
  - **Haste Health Clinical Data Repository**: FHIR-native CDR; public benchmarks ~5× FHIRPath evaluation throughput vs. TypeScript reference.
  - **Kodjin FHIR Server**: commercial Rust microservice implementation; all FHIR versions.
  - **`fhir-sdk`, `fhir-rs`, `rust-fhir`** (crates.io): SDKs generated from FHIR StructureDefinitions; type-safe resource access.
- **Cryptography**: **RustCrypto** (community pure-Rust AES, RSA, ECDSA, Ed25519, BLAKE3, etc.); `ring` (BoringSSL-derived, Brian Smith); `rustls` (Rust-native TLS, Cure53-audited, used in Cloudflare quiche QUIC and Mozilla NSS-replacement work). CDS relevance: data-in-transit encryption with no memory-safety attack surface.
- **WebAssembly**: most mature WASM story of any language; `wasm32-wasip2` tier-2, ships every release. WasmEdge for edge AI inference; Bytecode Alliance `wasmtime` production server-side. CDS: edge-deploy verified rule binaries (rural Japan, ship-borne medicine, disaster-response telemedicine) as portable WASM, sub-millisecond cold start.
- **Async runtimes**: Tokio (canonical, Carl Lerche et al.), Smol, async-std, Embassy (no-std embedded). Tokio + axum + tower = production HTTP stack of Helios, Cloudflare pingora, AWS Firecracker, Discord API gateway.
- **AdaCore tooling**: GNAT Studio IDE, GNATprove, GNATcheck (coding standards), GNATtest, GNATcoverage (MC/DC). Certification Materials cover DO-178C, ISO 26262, EN 50128, ECSS, IEC 61508; explicitly list medical-device use cases.

### Japan-specific considerations
- **Tokyo Rust** (`tokyorust.org`): primary Japanese Rust community; runs Tokyo Rust meetup; positions Rust for Japan's manufacturing, automotive, critical-infrastructure sectors. Adjacent to but distinct from the Rust-jp Japanese-language community.
- **Woven by Toyota** (Tokyo): Safety-Critical Rust Consortium founding member; Arene automotive-OS platform commits heavily to Rust for in-vehicle safety-critical software — directly transferable to IEC 62304-certified hospital-device firmware (same Ferrocene qualification path).
- **TECHFUND** (Tokyo): Consortium founding member; blockchain/Web3 background, actively contributing to safety-critical Rust governance.
- **Preferred Networks (PFN, Tokyo)**: flagship Japanese ML company; **Preferred Robotics** subsidiary does autonomous mobile robots, where Rust + ROS 2 (feature-complete `rclrs` client library; ROS 2 Rust Working Group active as of March 2026 meeting) is increasingly the substrate.
- **AdaCore Japan / SPARK adoption**: Japanese-language presence. Hitachi and Kawasaki Heavy Industries use Ada/SPARK in rail signalling per AdaCore case studies — transferable to PMDA-regulated medical-device firmware. The Hillrom ECG SPARK study is the most-cited medical instance and is read in Japanese medical-device-software training literature.
- **MHLW / METI**: Japan SaMD market projected USD 22.93M (2025) → USD 96.20M (2033); MHLW + METI promote ICT-enhanced caregiving. "Regulated SaMD + AMED-funded clinical AI + memory-safe substrate" is unrepresented in Japanese clinical stacks (Python/Java-leaning) — same opportunity gap as §13, here for the *runtime* substrate.
- **Researchers**:
  - **Naoki Kobayashi** (U. Tokyo, cited §13): higher-order model checking; MoCHi targets OCaml, but linearity/ownership analysis techniques transfer to Rust verification.
  - **Tachio Terauchi** (Waseda): refinement and dependent refinement types; POPL/PLDI author; directly applicable to Prusti-style Rust verification.
  - **Hiroyuki Katsura, Yuki Nishida** et al. (Tohoku/Kyoto): Rust-related publications at PPL and JSSST PPL workshops, 2022–2026.
- **Conferences**: **Rust.Tokyo** (annual since 2019), Rust-jp events; **RustConf** (North America), **EuroRust**, **Rust Nation UK**, **Rust India**, **OxidizeConf** (embedded). PPL (JSSST, cited §13) covers Rust verification. **HILT** (ACM SIGAda High Integrity Language Technology, annual) for Ada/SPARK.

### Interoperability
- **Within Category 4**: deployment-time complement to §13. §1 Lean 4 → Rust extraction; §6 Why3 ← Creusot; §7 F\* → KaRaMeL → C (→ Rust in progress); §8 refinement-type IR schemas → Prusti/Creusot annotations; §9 proof by reflection executes equivalently in Lean and pure-safe Rust given same ADT representations; §10 proof certificates unchanged across substrates; §11 CrossHair Python-only, conceptually parallel to Kani.
- **Category 1 (Computable Guideline IR & CDS Standards)**: Helios, Haste Health, Kodjin, `fhir-sdk`/`fhir-rs`/`rust-fhir` supply Rust-native FHIR R4/R4B/R5/R6, replacing or complementing HAPI FHIR (Java). CQL evaluators in Rust pre-production (no canonical crate, May 2026); DMN decision tables encode as `enum` + exhaustive `match` as in Haskell.
- **Category 2 (Japan Guideline & Terminology)**: SNOMED CT, ICD-10, JLAC, MEDIS as Rust `newtype` wrappers over `String`/`u32` with smart constructors — illegal codes unrepresentable. Cross-codable to Haskell ADTs (§13) and Lean inductives (§1) via Serde JSON.
- **Category 3 (Ontologies/RDF)**: `oxigraph` (Rust SPARQL engine), `sophia` (Rust RDF). SHACL checking experimental in Rust; production typically calls Java TopBraid via JNI.
- **Category 5 (Automated Reasoning)**: SMT bindings `z3`, `cvc5-sys`, `yices2`; `egg` equality saturation. Verus/Prusti/Creusot/Kani all dispatch to SMT.
- **Category 6 (Rule Semantics & Temporal)**: Allen interval algebra = 13-variant `enum`. FRP in Rust (`flux-rs`, `frunk`, `bevy` ECS events) less mature than Yampa but production in robotics (ROS 2 Rust). Stream engines Materialize, Arroyo, Fluvio are Rust-implemented.
- **Category 7 (IR/RAG)**: `candle` (Hugging Face pure-Rust ML), `burn`, `tch-rs` (libtorch), Qdrant vector DB (Rust-native) — complete Rust RAG substrate; clinical RAG pipelines as single static binaries with cryptographically reproducible builds.
- **Category 8 (LLM/Agentic)**: LLM Rust emission well supported; exceptional compiler diagnostics raise round-trip success vs. C++/Python. Emit-Rust + `cargo check` + iterate loops are among the most reliable autoformalization-to-deployment patterns in production.
- **Category 9 (Evaluation/Validation)**: `proptest` (Rust QuickCheck analogue, Andrew Gallant et al.) property testing with shrinking; `cargo nextest` parallel tests; `cargo llvm-cov` coverage; `cargo mutants` mutation testing. **Loom** (David Tolnay, Carl Lerche): exhaustive concurrent-execution-permutation testing for `unsafe` sync primitives — relevant to multi-threaded CDS rule engines.
- **Category 10 (Regulatory/Assurance)**: Ferrocene IEC 62304 Class C (January 2025) = toolchain-level evidence Rust can implement an IEC 62304-compliant lifecycle. CISA January 1, 2026 roadmap deadline binds Pledge-signatory CDS vendors; the roadmap is a *regulatory*, not developer, artefact. SBOM via `cargo sbom`/`cyclonedx-rust-cargo`; AIBOM via `cargo bom`; SLSA provenance. §13's reproducibility argument carries: build with `--release -Ccodegen-units=1 -Cembed-bitcode=no` + pinned `Cargo.lock` = bit-for-bit reproducible across machines, supporting IEC 62304 §8 configuration-management evidence and FDA SaMD pre-market documentation.

### Limitations/Known issues
- **`unsafe` real and necessary**: ~25–30% of crates contain `unsafe` (Astrauskas et al., *How Do Programmers Use Unsafe Rust?*, OOPSLA 2020); stdlib, FFI, lock-free structures, OS abstractions, high-performance parsers depend on it. Memory safety is thus layered: safe Rust sound modulo correct `unsafe` encapsulation. Partial mitigations: Miri, MIRAI, Stacked/Tree Borrows analysis, RustSec. Xu, Chen, Yin, Hao et al. 2020, *Memory-Safety Challenge Considered Solved? An In-Depth Study with All Rust CVEs*: essentially every Rust memory-safety CVE traced to an `unsafe` block — confirming the threat model and that encapsulation is operationally working.
- **Conservative borrow checker**: self-referential structs, intrusive data structures, certain graph encodings need `Rc<RefCell<T>>` (runtime borrow check) or `unsafe`. Tree Borrows cuts false rejections ~54% vs. Stacked Borrows; boundary still audible to working programmers. Next-gen Polonius checker in long-term development.
- **Compile times**: >500-dependency graphs and release LTO are slow. Mitigations: `sccache`, `mold` linker, `cargo-chef` Docker layering. Developer-velocity issue, not regulatory.
- **Clinical ecosystem gaps**: Rust FHIR younger than HAPI FHIR (Java); CQL/ELM evaluators pre-production; SMART-on-FHIR clients limited. FFI migration feasible but adds operational complexity (same point as §13).
- **Japan hiring**: Rust easier than Haskell/Lean (Tokyo Rust meetup, Rust.Tokyo, Woven by Toyota / Preferred Networks recruitment) but smaller than Python/Java pools; bilingual Rust + clinical-domain engineers are a narrow intersection. SPARK/Ada hiring globally small; AdaCore training partially compensates; Japanese rolling-stock/avionics talent overlaps but is not directly recruitable into clinical software.
- **Zig is not memory-safe (CISA sense)**: pre-1.0 (May 2026); excellent C interop, transparent control flow, debug-build runtime safety, but no static borrow checker, no affine types, no compile-time data-race freedom — calling it memory-safe in a roadmap would be a documentation error. Same caveat for C++ Core Guidelines + lifetime profile: static lifetime *analysis*, cannot enforce full borrow discipline at compile time; **Project Verona** (Microsoft Research) explores adding one to C++ (memory regions + ownership + concurrent safety), still research as of 2026.
- **MC/DC in Rust toolchain**: DO-178C DAL A coverage criterion in development as 2026 Rust Project Goal (Consortium); until shipped, DAL A airborne/implantable use requires Ada/SPARK alongside or alternative coverage instrumentation. Less relevant for IEC 62304 Class C (Ferrocene already covers).
- **Floating-point semantics**: same caveat as §13 — physiological-threshold rules over IEEE-754 doubles inherit reordering/fast-math hazards. Rust `f64` is IEEE-compliant by default, but `-Ofast`-equivalent flags exist and must be excluded from clinical builds; `rust_decimal`/`bigdecimal` provide decimal alternatives for monetary and dosage arithmetic.
- **Supply chain**: ~165,000-package crates.io is a large surface. Mitigations: `cargo audit` (RustSec), `cargo vet` (Mozilla-developed, Google-used), `cargo deny` policy enforcement, in-development crates.io publish attestation. Same risk class as PyPI/npm with smaller typical dependency surface. 2025–2026 malicious-code incidents on crates.io were caught quickly by RustSec — response time, not incident existence, is the relevant metric.

### Training data proxy
- **Rust**: extremely strong LLM coverage. *Rust Reference*, *Rustonomicon* (unsafe-code reference), *The Rust Programming Language* (Klabnik & Nichols, 2nd ed. 2023, No Starch; free at `doc.rust-lang.org/book/`), *Rust for Rustaceans* (Jon Gjengset, 2021, No Starch), *Programming Rust* (Blandy, Orendorff, Tindall, 2nd ed. 2021, O'Reilly), *Zero to Production in Rust* (Luca Palmieri, 2022) all widely indexed. `rustdoc` output mirrored on `docs.rs`. GitHub Rust repos in the low millions.
- **Ada/SPARK**: smaller, high-quality corpus. *Programming in Ada 2012* (John Barnes, 2014, Cambridge); *Building High Integrity Applications with SPARK* (McCormick & Chapin, 2015, Cambridge); AdaCore's *Learn Ada* (`learn.adacore.com`). LLMs weaker on idiomatic Ada/SPARK than Rust; in-context exemplars help materially.
- **Swift**: moderate coverage; `docs.swift.org`; *The Swift Programming Language* (free, Apple).
- **LLM coverage rank** (memory-safe axis): Rust ≈ Java > C# > Go > Swift > Ada/SPARK > Zig; Zig sparse enough to need in-context examples for non-trivial work (like F\*/Idris, §13).
- **Benchmarks/corpora**: BigCloneBench (cross-language), HumanEval-X, MBPP-Rust (community), CRUX-Eval (Rust-specific, 2024), Verus/Prusti example corpora for verification. RustSec advisory DB doubles as a labelled memory-safety-vulnerability dataset for fine-tuning detection models.
- **Venues**: RustConf (canonical), EuroRust, Rust.Tokyo, Rust Nation UK, Rust India, OxidizeConf; HILT (ACM SIGAda); PLDI, POPL, OOPSLA, ICFP, OSDI, SOSP, ASPLOS, SIGCSE all carry Rust streams; PPL (Japan, cited §13).
