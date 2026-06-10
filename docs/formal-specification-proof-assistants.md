# Category 4: Formal Specification, Proof Assistants, and Proof Engineering

## 1. Lean 4 + Mathlib + Aesop/Tactic Automation

### Purpose
Dependently typed functional programming language and interactive theorem prover for both software verification and mechanized mathematics. Provides a unified language for definitions, programs, proofs, and metaprograms.

### Maintainer/Standards body
Lean FRO (Lean Focused Research Organization), founded 2023, led by Leonardo de Moura. Current stable release Lean 4.29.1 (April 14, 2026); v4.30.0 in release-candidate stage (4.30.0-rc2, April 16, 2026). Earlier 2026 releases include 4.28.0 (February 17, 2026) and 4.29.0 (March 27, 2026). Mathlib4 maintained by `leanprover-community`. Aesop maintained by Jannis Limperg under `leanprover-community`.

### Conceptual model
Calculus of Constructions with inductive types, universe polymorphism, definitional proof irrelevance for `Prop`, and quotient types. Proofs are terms; tactics are metaprograms in `MetaM`/`TacticM` monads that elaborate to proof terms. Source files compile to `.olean` files. Mathlib is structured around `structure`/`class` hierarchies with `instance` resolution. Aesop is a white-box best-first proof search engine with user-tagged rule sets (`@[aesop safe]`, `unsafe`, `norm`); `simp` provides a confluent rewriter. `grind` (Lean 4.20+) integrates linarith, cutsat, commutative-ring (Nullstellensatz certificates), and congruence closure.

### Expressiveness/Semantics
Classical or constructive at user option (`Classical.em` available as axiom). Supports arbitrary mathematical structures: rings, topological spaces, measure theory, category theory, schemes, perfectoid spaces. Decidability is a first-class type class. Computation by reduction (`rfl`), bytecode (`#eval`), or native (`native_decide`).

### Composability/Modularity
Lake build system + Reservoir package index. Module system with `import`. Type-class inheritance for hierarchies. Mathlib uses a global namespace with prefix conventions; its source distribution is on the order of 226 MB. **CSLib** (The Lean Computer Science Library, `leanprover/cslib` on GitHub and Reservoir, Apache-2.0) depends on Mathlib and extends it into computer science: it provides bundled `ReductionSystem` and `LTS` (Labelled Transition System) structures from which multistep reduction, reachable states, and confluence properties derive automatically, plus `HasContext` (single-hole term contexts) and `Congruence` (equivalence relations preserved by constructors). CSLib's architectural "Spine" (arXiv:2602.15078) is explicitly designed for reuse across formalization initiatives.

### Suitability for autoformalization to IR
Strongest target for LLM autoformalization among proof assistants: the bulk of recent autoformalization research (DeepSeek-Prover-V2, Kimina-Prover, LeanDojo, Goedel-Prover, Herald, AlphaProof) targets Lean 4. Idempotency is supported by `simp` normal forms, `Decidable` instances yielding canonical Booleans, and `grind`'s normalization. Syntactic regeneration after edits is mostly stable across Mathlib versions due to deprecation aliases.

### Formal verification potential
Trusted kernel ~6 KLOC C++ (the Lean 4 reference kernel; a Lean-in-Lean reimplementation, Lean4Lean, runs 20–50% slower); small TCB. Native decision procedures: `linarith`, `omega`, `polyrith`, `positivity`, `decide`, `bv_decide`, `grind`. SMT bridge via `lean-smt` (cvc5, with proof reconstruction via Ethos/reflective rules). No native sledgehammer but `exact?`, `apply?`, `aesop?`, and LLM-based `LLMlean`/`copilot` extensions exist.

### Tooling/Ecosystem maturity
VS Code extension (vscode-lean4) with widgets, module-hierarchy view (Lean 4.22+). Zulip is primary chat; Mathlib CI builds nightly. Doc-gen4 for HTML docs. Mathlib4 cache via `lake exe cache get`. **CSLib** (cslib.io; GitHub `leanprover/cslib`, 556 stars, 145 forks, Apache-2.0; current release v4.29.0, March 31, 2026) aims to be for computer science what Mathlib is for mathematics. Led by Fabrizio Montesi (University of Southern Denmark, FORM Centre for Formal Methods and Future Computing); steering committee includes Clark Barrett (Stanford/AWS), Swarat Chaudhuri (Google DeepMind/UT Austin), Jim Grundy (AWS), Pushmeet Kohli (Google DeepMind), and Leonardo de Moura (Lean FRO/AWS). Funded by Renaissance Philanthropy, AWS, Google DeepMind, FORM/SDU, Stanford CENTAUR, ERC (CHORDS grant 101124225), and NSF (CCF-2220991). Current formalized content: Milner's Calculus of Communicating Systems (CCS) with commutativity/distributivity of parallel composition and the result that bisimilarity is a congruence; System F with subtyping using locally nameless representation; Hennessy-Milner Logic with the Hennessy-Milner theorem (arXiv:2602.15409); behavioral equivalences (bisimilarity, weak bisimilarity, simulation, trace equivalence) parametric over any LTS; combinatory logic; linear logic; automata theory; algorithms with time-complexity bounds. CSLib makes the `grind` tactic a first-class citizen: 314 of 338 declarations used `grind` from the initial commit, with measured savings averaging 7.1 lines per theorem and bisimilarity proofs saving 15–39 lines; a System F formalization measured approximately 45% fewer lines than a comparable Rocq development (arXiv:2602.15078). The 2026 roadmap targets formalizations of most algorithms in a typical undergraduate algorithms/data-structures course and most models/logics in a typical theory-of-computation course, plus type-system specification interfaces and nominal transition system logics (pi-calculus). Area maintainers: Alexandre Rademaker (Atlas Computing/FGV), Sorrachai Yingchareonthawornchai (ETH Zurich), Christopher Henson (Drexel), Kim Morrison.

### Japan-specific considerations
TPP (Theorem Proving and Provers) annual workshop in Japan covers Lean alongside Coq/Isabelle. Jacques Garrigue (Nagoya University) teaches Lean. JSSST SIG-PPL runs PPL Workshop and Summer School with Lean tutorials. No dedicated Japanese-government-funded Lean library, but the ERATO MMSD project (NII, Ichiro Hasuo, JST grant JPMJER1603, October 2016 – March 2025) trained researchers across the spectrum.

### Interoperability
Bridges out: Lean → Dedukti/Lambdapi translators exist (Deducteam). Bridges to Category 1: native JSON/YAML parsers can ingest FHIR Clinical Reasoning/CQL/DMN; CQL semantics can be embedded via inductive types. Category 2: ontology IDs map to Lean constants; Mathlib's `Finset` / `List` / `Multiset` fit guideline-recommendation semantics. Category 3: OWL DL fragments embed into decidable Lean fragments via `Decidable` typeclass; RDF triples become typed relations. Inter-category in Cat 4: Lean → Dedukti enables independent checking against Isabelle/Rocq encodings.

### Limitations/Known issues
Tactic syntax churn between Lean versions; Mathlib bumps are frequent. Build times for full Mathlib are hours; cache mitigates. Universe-polymorphism elaboration errors are notoriously opaque. `native_decide` enlarges TCB (requires trusting the C compiler). Performance of `simp` and `aesop` on large rule sets can be problematic.

### Training data proxy
Largest LLM training-data footprint among proof assistants. CSLib adds 556 GitHub stars, three arXiv papers (2602.04846, 2602.15078, 2602.15409), and Lean Together 2026 presentation materials; its AI-integration area explicitly targets training datasets and AI-assisted contribution tools. Mathlib contains over 1.9 million lines of formally verified mathematics, with ~404,000 total declarations (~130,800 definitions + ~273,800 theorems) and 772 contributors per the live `leanprover-community.github.io/mathlib_stats.html` page (May 2026). The Mathlib Initiative roadmap (October 2025–September 2026) targets review-cycle response under one week for 90% of cycles by September 2026; current median wait time is ~two weeks against ~300 PRs in backlog. The Lean Community Zulip archive (May 2026) shows 12,843 topics in the "new members" channel alone, with 6,722 topics in mathlib4, 7,667 in lean4, and 9,924 in general; Zulip's case study characterises it as "hundreds of active participants". Benchmarks: miniF2F (488 problems), ProofNet (371 problems), PutnamBench (672 Lean 4 / 640 Isabelle / 412 Coq formalizations of 640 underlying Putnam theorems; 1,724 formalizations total). DeepSeek-Prover-V2-671B reaches 88.9% pass on miniF2F-test (Pass@8192) and solves 49/658 PutnamBench. AlphaProof (DeepMind, July 2024; Nature paper "Olympiad-level formal mathematical reasoning with reinforcement learning", Hubert et al., published November 12, 2025) achieved silver-medal-equivalent performance at IMO 2024 (4/6 problems combined with AlphaGeometry 2) using ~80M autoformalized formal Lean problems (translated from ~1M natural-language problems) for RL.

## 2. Rocq Prover + Stdlib/MathComp/Iris as a Secondary Proof Ecosystem

### Purpose
Mature interactive proof assistant (formerly Coq) for dependently typed programming and machine-checked proof. Renamed to "The Rocq Prover" with the 9.0 release on March 12, 2025.

### Maintainer/Standards body
INRIA leads development; community + Rocq Consortium govern. Current releases: Rocq 9.0 (March 2025), 9.1 (September 2025), 9.1.1 (February 2026), 9.2.0 (March 2026). Coq Platform 2025.01.0 (February 2025) bridges legacy Coq 8.20.1.

### Conceptual model
Calculus of Inductive Constructions (CIC), with predicative `Type` hierarchy, impredicative `Prop`/`SProp`, primitive projections, native integers/floats/arrays. Specification language Gallina; tactic languages Ltac (legacy) and Ltac2 (typed). Proof terms can be extracted to OCaml/Haskell/Scheme. SSReflect (small-scale reflection, default-distributed since Coq 8.7) provides a more concise proof style underpinning MathComp.

### Expressiveness/Semantics
Constructive by default; classical axioms (LEM, choice, functional extensionality, propositional extensionality) are optional and orthogonal. Supports HoTT/UniMath-style univalence as an opt-in flavor. Iris embeds higher-order concurrent separation logic into Rocq as a deep embedding (recipient of 2025 Most Influential POPL Paper Award for "Iris: Monoids and Invariants as an Orthogonal Basis for Concurrent Reasoning", POPL 2015, and the 2023 Alonzo Church Award awarded to Birkedal, Bizjak, Dreyer, Jourdan, Jung, Krebbers, Sieczkowski, Svendsen, Swasey, and Turon, presented at ICALP 2023, Paderborn).

### Composability/Modularity
ML-style functor module system + canonical structures + type classes + unification hints. Three coexisting design patterns: packed classes (MathComp), type classes (`stdpp`/Iris), and Hierarchy-Builder. Rocq 9.0 split Stdlib into `rocq-core` (Corelib) and `rocq-stdlib`. Opam package manager; Dune build.

### Suitability for autoformalization to IR
Less LLM-targeted than Lean. Multiple competing styles (vanilla, SSReflect, Equations, MathComp) hurt convergence/idempotency. Stronger when target is fixed to MathComp+SSReflect: rigid bullet structure, `Move=>`/`apply:` discipline, canonical-form normalization via `rewrite`. Lambdapi can serve as IR mediator.

### Formal verification potential
Kernel ~10 KLOC OCaml; small TCB. `vm_compute` and `native_compute` reductions enable proof-by-reflection (SMTCoq, CoqHammer's reconstructor, micromega/lia/lra/nra/psatz). Hammer plugin (CoqHammer) calls Vampire/E/Z3 and reconstructs proofs. Iris adds powerful program-logic reasoning for concurrent imperative code.

### Tooling/Ecosystem maturity
VsCoq language server (now in Coq Platform 2025); CoqIDE; Proof General. opam-coq-archive package index. Rocq Platform 2025.08.3 bundles MathComp, Iris, MathComp-Analysis, Equations, ELPI.

### Japan-specific considerations
AIST – **Reynald Affeldt** maintains: **MathComp-Analysis** (co-maintained with Cyril Cohen et al.; "An Introduction to MathComp-Analysis" tutorial dated January 13, 2025, with an updated version dated October 24, 2025 reflecting the Coq→Rocq rename); **monae** (hierarchy of monadic effects in Rocq, GitHub `affeldt-aist/monae`); **infotheo** v0.9.7 (March 2026 — Rocq formalization of information theory and linear ECCs, compatible with Rocq 9.0–9.1 and MathComp ≥2.4.0). Contributors include Jacques Garrigue (Nagoya), Kazuhiko Sakaguchi (Tsukuba), Takafumi Saikawa (Nagoya). Nagoya University runs Coq/SSReflect courses (Garrigue). Atsushi Igarashi's group at Kyoto University developed HELMHOLTZ (Coq-based verification of Tezos/Michelson smart contracts). Historical: Affeldt–Kobayashi mail-server verification (ISSS 2003).

### Interoperability
Strong Lambdapi/Dedukti export. SMTCoq imports LFSC/CVC5 certificates; CoqHammer reconstructs from E/Vampire/Z3. Iris formalizes Hoare-style guideline-step specifications. Bridges to Category 1: Iris+HeapLang or Iris-OCaml model imperative pipelines; CompCert-style C-extraction available. Cross-category interop with Lean is via Mathport / Mathlib4 (historical); with Isabelle via Dedukti (Deducteam translators).

### Limitations/Known issues
Steeper learning curve than Lean; SSReflect idiom unfamiliar to newcomers. Universe inconsistencies hard to debug. Multiple incompatible "standard libraries". Native compute not yet supported with OCaml 5 on all architectures (re-enabled in Rocq 9.2.0 for some x86 setups). LLM autoformalization quality lags Lean.

### Training data proxy
GitHub topic `coq` is large (452 repositories tagged "Rocq Prover" as of May 2026; aggregate `coq` topic count requires JS rendering and is not officially published). Rocq Zulip very active. Rocq Stdlib + MathComp library cover hundreds of KLOC. Iris repository large, with PLDI/ICFP/POPL papers annually. CoqGym, PISA-coq, and CoqStoq benchmarks exist but are smaller than miniF2F.

## 3. Isabelle/HOL + AFP/Sledgehammer as an Independent Audit Target

### Purpose
Generic logical framework instantiated to higher-order logic (HOL). Mature, industrially-deployed proof assistant (seL4, CakeML, AFP). Useful as an independent auditing target for cross-checking Lean/Rocq formalizations.

### Maintainer/Standards body
TUM (Tobias Nipkow) + University of Cambridge (Larry Paulson) + University of Innsbruck (Makarius Wenzel for Isabelle/Isar/PIDE). Current release Isabelle2025-2 (January 2026), shipping with Isabelle2025/AFP-2025.

### Conceptual model
Pure (intuitionistic typed λ-calculus + meta-implication ⟹ + meta-universal ⋀) is the meta-logic; HOL is an object logic on top. **Isar** is the structured human-readable proof language. Proofs are kernel-checked via primitive inference rules. PIDE is the asynchronous document-oriented prover interface.

### Expressiveness/Semantics
Classical higher-order logic with rank-1 polymorphism + type classes (Haskell-style). Axiom of choice built-in. Less expressive than CIC (no dependent types), but the trade-off yields enormously more automation. Locales provide parameterized theories with refinement.

### Composability/Modularity
Locale system (parameterized theories with inheritance and instantiation), type classes, sessions (build units). AFP entries are independent sessions with metadata. Theory imports form a DAG.

### Suitability for autoformalization to IR
Excellent semantic stability; the lack of definitional-equality dependence means proofs are more textually robust. Isar's declarative `have ... show` structure is closer to natural-language proof than Lean's tactic mode, but Lean has more training data. Recent neural-Isabelle work (Magnushammer, PISA, Baldur, Thor) leverages Sledgehammer for proof reconstruction. A "minimalist proof language" for neural theorem proving over Isabelle/HOL (arXiv 2507.18885) reports pass@1 of 69.1% on PISA.

### Formal verification potential
**Sledgehammer** dispatches goals to external ATPs (E, SPASS, Vampire) and SMT solvers (Z3, CVC4/5, veriT) and reconstructs proofs via Metis or the `smt` tactic. The user's guide is by Jasmin Blanchette (LMU München, dated March 13, 2025). Recent improvements (Isabelle 2025.4+) reconstruct paramodulation/instantiation evidence from Z3/cvc5 proofs in Alethe format (arXiv 2508.20738). Kernel ~few thousand lines SML.

### Tooling/Ecosystem maturity
Isabelle/jEdit (default IDE), Isabelle/VSCode (panels for Documentation/Symbols/Sledgehammer in 2025-2), command-line `isabelle build`. AFP statistics (live, May 2026): 973 entries, 593 authors, ~315,100 lemmas, ~5.17 million LoC; the AFP-2025 release ≈5,300 kLOC against Isabelle2025 distribution ≈900 kLOC (HOL-Analysis 176 kLOC, HOL-Library 68 kLOC, HOL 123 kLOC).

### Japan-specific considerations
**Christian Sternagel** (JAIST alumnus / former Erwin-Schrödinger Fellow at JAIST; now at University of Innsbruck; IsaFoR/CeTA for certified rewriting termination). **Kazuhiro Ogata** group (JAIST) — CafeOBJ/OTS algebraic-specification method (successor to Kokichi Futatsugi); proof-scores survey (CoRR abs/2504.14561, 2025); co-edited ICFEM 2024 (Hiroshima, December 2–6, 2024, LNCS 15394, with Dominique Méry, Meng Sun, Shaoying Liu). **Mizuhito Ogawa** (JAIST) — raSAT SMT for polynomial constraints (Tung, Khanh, Ogawa, FMSD 51(3):462–499, 2017). The FeliCa Networks "Mobile FeliCa" case study (Kurita & Araki, ICFEM 2016, Tokyo) is a notable Japanese industrial VDM-based formal-methods deployment.

### Interoperability
Isabelle export to Dedukti/Lambdapi via Deducteam. Isabelle/HOLCF for domain theory. Isabelle/DOF (Document Ontology Framework, Brucker–Wolff) embeds SACM/GSN assurance-case ontologies into Isabelle documents — directly relevant to clinical-guideline traceability. Bridges to Category 1: Isabelle/UTP (Unifying Theories of Programming) covers Hoare/Z-style/refinement, can model BPMN-style step semantics. Category 3: Isabelle/DOF + ROntology allow OWL-DL ontology embedding. Cross-Cat-4 interop: SMTCoq-style proof certificate import via Alethe (ITP 2025, Lachnitt et al.).

### Limitations/Known issues
No dependent types; encoding higher-rank dependency requires HOLZF or ad-hoc tricks. Locales can become unwieldy. AFP entries can become unmaintained over years. Lower LLM training presence than Lean.

### Training data proxy
973 AFP entries cited above; long-running JAR/CADE/ITP paper pipeline. Sledgehammer's pre-existing tactic-form output is structured but variable. Magnushammer (ICLR 2024) and "A Minimalist Proof Language" (arXiv 2507.18885) demonstrate transformer-based theorem proving over Isabelle.

## 4. TLA+ / PlusCal for Pipeline, Convergence, and Idempotency Properties

### Purpose
Formal specification language by Leslie Lamport, designed for distributed and concurrent systems. Excellent at expressing temporal properties — exactly the convergence and idempotency properties required for the autoformalization pipeline.

### Maintainer/Standards body
Originally Microsoft Research; now stewarded by the TLA+ Foundation (under the Linux Foundation, est. 2023; Lamport active emeritus). Reference text: "Specifying Systems" (Lamport, Addison-Wesley, 2003); current updates via "The TLA+ Hyperbook".

### Conceptual model
Untyped Zermelo–Fraenkel set theory + first-order logic + linear temporal logic of actions (TLA). A specification is a single temporal formula `Init ∧ □[Next]_vars ∧ Liveness` characterizing infinite behaviors. **PlusCal** is a pseudocode-like algorithmic surface language (introduced 2009) that transpiles to TLA+. **TLC** is the explicit-state model checker (BFS over reachable states). **TLAPS** is the interactive proof system (introduced 2012) backed by Zenon, Isabelle/TLA+, and SMT.

### Expressiveness/Semantics
Classical first-order set theory; supports arbitrary mathematical operators. Hyperproperties (refinement, fairness, eventual delivery, stuttering invariance) are first-class. **Idempotency**: expressible as `□(action(s) ⇒ action(action(s)) = action(s))` or as bisimulation against an action's fixed point. **Convergence**: liveness property `◇□P` (eventually always P) or formalized as Stabilization.

### Composability/Modularity
Modules with `EXTENDS`/`INSTANCE`. No type system (deliberate). Refinement via module instantiation under substitution.

### Suitability for autoformalization to IR
TLA+ is not a target IR for guideline content itself but the meta-IR for the pipeline that produces guideline IR. Specify: "every two runs of the autoformalizer over the same input converge to a state-equivalent IR" as `□(input = input' ⇒ ◇(IR ≡ IR'))`. Idempotency: `Normalize(Normalize(x)) = Normalize(x)`.

### Formal verification potential
TLC: exhaustive model-checking up to finite scopes (millions of states routine). Apalache: symbolic model checker (SMT-backed). TLAPS: deductive proof, hierarchical proofs (steps `<1>1`, `<1>2`, etc.), with Zenon/Isabelle/SMT backends.

### Tooling/Ecosystem maturity
TLA Toolbox (Eclipse-based); VS Code TLA+ extension. Apalache (University of Lugano / Informal Systems). Distributed TLC. PlusCal translator. Used in production by AWS (S3, DynamoDB, EBS), Microsoft Azure, MongoDB, ConsenSys, Elastic.

### Japan-specific considerations
**Ichiro Hasuo**'s group at NII (Research Center for Mathematical Trust in Software and Systems, MTSS) — ERATO MMSD project (October 2016 – March 2025, JST grant JPMJER1603, ~250 papers) used temporal-logic verification heavily (Goal-Aware RSS, ISO 34502 formalization with Masaki Waga, Kyoto University). CAV 2023 Distinguished Paper "Exploiting Adjoints in Property Directed Reachability Analysis" (Mayuko Kori et al.). Limited TLA+ direct adoption in Japanese clinical software; AIST's High Reliability Software Engineering group has historically used TLA+ in workshops.

### Interoperability
TLA+ specs encode into Lambdapi (Alessio Coltellacci & Stephan Merz — "encoding of TLA+ set theory in Lambdapi", ABZ 2023). Bridges to Category 1: BPMN/ePath process flows are naturally TLA+ state machines; DMN decision tables are TLA+ `IF/THEN/ELSE` blocks. Category 2: PMDA workflow can be modeled as a TLA+ refinement chain. Category 3 ontology classes appear as constant set declarations. Bridges within Cat 4: Apalache exports SMT proofs (compatible with Alethe pipelines).

### Limitations/Known issues
No types means specs can be ill-formed at runtime; Apalache adds optional typing. TLC scope explosion on large state spaces. TLAPS proofs less automated than Lean/Isabelle. ASCII syntax not user-friendly. PlusCal hides TLA+ details but limits expressiveness.

### Training data proxy
Hillel Wayne's "Learn TLA+" tutorial widely referenced. Examples repository (`github.com/tlaplus/Examples`) with hundreds of specs. Smaller LLM training corpus than Lean; no benchmark equivalent to miniF2F.

## 5. Alloy / Forge for Finite Relational Counterexamples

### Purpose
Lightweight specification language and bounded analyzer by Daniel Jackson (MIT). Designed to find counterexamples to relational specifications within a finite scope, useful for "shake-out" testing of guideline-IR invariants and for student-accessible specification.

### Maintainer/Standards body
Alloy: MIT Software Design Group (Daniel Jackson); current release **Alloy 6** integrates Electrum's temporal logic (mutable signatures + LTL operators `always`/`eventually`/`after`/`'`). Forge: Brown University PLT (Tim Nelson, Shriram Krishnamurthi) + Northeastern; documented in OOPSLA 2024 paper "Forge: A Tool and Language for Teaching Formal Methods" (PACMPL Vol 8).

### Conceptual model
First-order relational logic over typed atoms (signatures `sig`, fields, constraints). Translates to SAT via Kodkod; the SAT solver is typically MiniSat or Glucose. **Small-scope hypothesis** (Jackson, Software Abstractions, MIT Press 2012, p. 141): "most bugs have small counterexamples", so bounded analysis is empirically nearly complete. Forge layers Froglet (functional subset) → Relational Forge → Temporal Forge, each adding expressiveness.

### Expressiveness/Semantics
Relational first-order logic + LTL (in Alloy 6 / Temporal Forge). No quantification over relations (decidable bounded fragment). Transitive closure (`^`, `*`) is first-class. Predicates and assertions (`assert ... check`) drive analyses.

### Composability/Modularity
Modules (`open`) with parametric polymorphism over signatures. Forge adds language levels and Sterling visualizer with CnD ("Cope and Drag") for custom diagrams (ECOOP 2025).

### Suitability for autoformalization to IR
Excellent for **counterexample-driven debugging** of IR invariants. LLMs can emit Alloy fragments to test "no guideline recommends both X and not-X" or "every administered-by edge points to a Practitioner". Idempotency: `assert idempotent { all x: IR | normalize[normalize[x]] = normalize[x] } check idempotent for 5`. Not a target for the full IR (no recursive data, no real arithmetic).

### Formal verification potential
SAT-backed; bounded but exhaustive within scope. Cannot prove unbounded properties. Counterexamples are concrete, visualizable instances.

### Tooling/Ecosystem maturity
Alloy Analyzer (Java/Swing). Sterling (web visualizer). Forge runs on Racket; supports VS Code and DrRacket. Forge has been used in Brown's CS course "Logic for Systems" since 2019.

### Japan-specific considerations
Alloy used in Japanese formal-methods courses at JAIST (Kazuhiro Ogata; Mizuhito Ogawa). The FeliCa Networks "Mobile FeliCa" IC-chip firmware case study (Kurita & Araki, "Promotion of Formal Approaches in Japanese Software Industry and a Best Practice of FeliCa's Case", ICFEM 2016, Tokyo) was VDM-based, but the Japanese FM community is broadly familiar with Alloy. No Japan-specific Alloy library identified.

### Interoperability
Alloy → SMT translation exists (Pardinus). Alloy/Forge models can be hand-translated to TLA+ for unbounded proof. Bridges to Category 1: DMN decision tables and FHIR resource cardinalities map naturally to Alloy `sig`/multiplicity constraints. Category 3: OWL DL has known partial translations to Alloy (DL-Lite fragments). Cross-Cat-4: Alloy counterexamples can seed Lean/Rocq lemma statements.

### Limitations/Known issues
Bounded incompleteness. Awkward arithmetic. Visualization can be unhelpful by default (Forge addresses partly with Sterling + CnD). Performance can degrade past scope ~10–15 atoms.

### Training data proxy
Moderate. Daniel Jackson's textbook (MIT Press 2012, revised) plus Macedo/Cunha's online "Formal Software Design with Alloy 6" book. Hillel Wayne blog. GitHub: hundreds of public Alloy models. Forge corpus is small (mostly Brown classroom).

## 6. Why3 / WhyML for SMT-Backed Executable Specifications

### Purpose
Deductive program verification platform that dispatches verification conditions to a herd of automated and interactive provers. WhyML is an ML-like first-order programming language with contracts, designed both as a direct verification target and an intermediate language for verifying C/Java/Ada/Rust.

### Maintainer/Standards body
INRIA Saclay / LRI (Toccata team). Authors: Jean-Christophe Filliâtre, Andrei Paskevich, Claude Marché, Guillaume Melquiond, François Bobot. Current release Why3 1.8+ (2024–2025).

### Conceptual model
Logic component: polymorphic first-order logic with algebraic data types and inductive predicates. Program component: WhyML with records, mutable fields, pattern matching, exceptions, ghost code, type invariants, **static alias control** (no memory model needed). VCs are generated and transformed via a rich transformation library, then dispatched to: Alt-Ergo, CVC4/CVC5, Z3, E, Vampire, Eprover, Spass, Princess, plus interactive backends (Rocq/Coq, Isabelle, PVS).

### Expressiveness/Semantics
First-order; not as expressive as Lean/Rocq. Strong fit for executable specifications over arithmetic, arrays, simple algebraic structures. Inductive predicates allow recursion. Extracts to OCaml/C/CakeML.

### Composability/Modularity
Theory modules + module cloning + refinement. Standard library covers integers, reals, sets, maps, queues, hash tables, arrays.

### Suitability for autoformalization to IR
Strong fit for clinical-rule executable specs: a guideline rule can be a WhyML `let` with `requires`/`ensures`. Idempotency proved by VC `forall x. normalize(normalize x) = normalize x`. SMT backends mean LLM-generated specs are checked quickly; convergence is supported by the canonical normalization passes (transformations).

### Formal verification potential
SMT-backed automation strong for first-order arithmetic. Counterexample reconstruction from SMT models. Notable applications: GMP arithmetic (WhyMP), ParcourSup, **Creusot** (Rust verification, PLDI 2022 Distinguished Paper). Also used as an intermediate language by Frama-C (C verification).

### Tooling/Ecosystem maturity
Why3 IDE, command-line, emacs mode. Opam package. Active in French academic ecosystem.

### Japan-specific considerations
Limited direct Japanese contributions to Why3 core. Japanese researchers use Why3 as a backend (e.g., via Frama-C in industrial settings). Kohei Suenaga's group (Kyoto University) uses SMT-based reasoning for hybrid systems with related tooling.

### Interoperability
Native SMT-LIB output; Alethe proofs from veriT/cvc5 can be replayed in Lambdapi. Why3 can drive Rocq/Isabelle as backends. Bridges to Category 1: WhyML maps directly to CQL-like expressions (FHIRPath, CQL operators are first-order). Category 2: PMDA-style data dictionaries map to WhyML record types. Category 3: SHACL constraints become WhyML `predicate` definitions with VCs. Cross-Cat-4: Why3 frequently dispatches to interactive provers covered above.

### Limitations/Known issues
First-order only; no higher-order. Less expressive for category-theoretic constructions. SMT solver heuristics produce inconsistent results across versions. Limited LLM training presence.

### Training data proxy
Modest. ~1k GitHub repos using WhyML. Annual JFLA and TAP papers. Creusot (Rust) is growing.

## 7. F* for Typed Verified Services

### Purpose
Proof-oriented programming language with dependent types + refinement types + SMT-based verification, developed for verified low-level/cryptographic code (HACL*, EverCrypt) and high-assurance services. Compiles to OCaml/F#/C (via KaRaMeL)/Rust/Wasm.

### Maintainer/Standards body
Microsoft Research + INRIA Paris (Prosecco) + Carnegie Mellon. Authors: Nikhil Swamy and collaborators. Active 2025–2026 releases (most recent F* 2026.3.24, March 24, 2026; prior 2025.12.15).

### Conceptual model
Dependent type theory with a layered effect system (`Tot`, `Pure`, `Ghost`, `Stack`, `ST`, `Steel`, `Pulse`). Refinement types `x:t{p x}`. Verification conditions are discharged primarily by Z3 with proof reconstruction via Meta-F*. **Pulse** is the current separation-logic DSL embedded in F* (successor/companion to Steel); merged into the main F* repository in 2024–2025.

### Expressiveness/Semantics
Classical with explicit ghost code. Dependent types + refinements enable expressing arbitrary first-order specifications inside types; SMT-backed checking keeps verification largely automatic.

### Composability/Modularity
Module system; interface (`.fsti`) / implementation (`.fst`) split; abstract types. Tactics in Meta-F*.

### Suitability for autoformalization to IR
A guideline rule becomes an F* function whose type encodes its specification. Idempotency expressed as a refinement on the normalizer's output type, automatically checked by Z3. Strong because SMT discharges most LLM-generated obligations without manual proof; LLM autoformalization output can be validated in a fast inner loop. Smaller LLM training base than Lean.

### Formal verification potential
Notable deployments: **HACL*** (verified cryptography, in Mozilla Firefox NSS, Linux kernel, mbedTLS, Tezos, ElectionGuard, Wireguard). **EverCrypt** (S&P 2020). **DICE\\*** (verified attestation). 2025 papers include "Secure Parsing... CBOR/CDDL/COSE" (CCS 2025) and "Mechanically Verified GC for OCaml" (~24k LoC of F*/Pulse, JAR 69:7, 2025).

### Tooling/Ecosystem maturity
VS Code extension; emacs mode; Z3 4.8.5/4.13 dependency. Opam packaging. Active development.

### Japan-specific considerations
Minimal direct Japanese contribution. Some Japanese cryptographers (NTT Labs cryptography teams) consume HACL*/EverCrypt but do not author F* libraries publicly.

### Interoperability
KaRaMeL extracts to C; can target Rust. Bridges to Category 1: refinement types over FHIR resource fields. Cross-Cat-4: Steel/Pulse are alternatives to Iris (Rocq); both encode separation logic.

### Limitations/Known issues
SMT-dependent (Z3 hangs cause non-reproducible failures). TCB includes Z3. Steep learning curve for the effect system. Pulse/Steel are powerful but young.

### Training data proxy
Modest. "Towards Neural Synthesis for SMT-Assisted Proof-Oriented Programming" (ICSE 2025) reports a 940 KLOC F* dataset for LLM training.

## 8. Dependent-Type and Refinement-Type IR Schemas

### Purpose
Use of dependent or refinement type systems (LiquidHaskell, Idris 2, Agda, refinement-reflection) to make IR schema invariants part of the type and enforce them statically. Cross-cutting technique rather than a single tool; sits within the broader typed-functional substrate catalogued in §13.

### Maintainer/Standards body
- **LiquidHaskell**: Niki Vazou et al.; integrated as a GHC plugin.
- **Idris 2**: Edwin Brady; current open-source community.
- **Agda**: Ulf Norell; maintained by Chalmers/Gothenburg + Inria + community.
- **Refinement Reflection**: POPL 2018 (Vazou, Tondwalkar, Choudhury, Scott, Newton, Wadler, Jhala; "Refinement Reflection: Complete Verification with SMT", PACMPL Vol 2, POPL, Article 53).

### Conceptual model
Refinement types: `{ v: T | p(v) }`, SMT-decidable predicates. Dependent types: types may depend on values, enabling indexed types (e.g., `Vec n A` for length-`n` vectors). Schema invariants such as "every Recommendation has a non-empty Evidence list of matching SnomedCT codes" become inhabitable types; ill-formed IR documents do not typecheck.

### Expressiveness/Semantics
Refinement types: decidable verification (modulo SMT), limited to first-order predicates over base types. Dependent types: full Martin-Löf or CIC strength but verification becomes undecidable. Refinement-Reflection bridges: lifts Haskell function definitions into the refinement logic for equational reasoning.

### Composability/Modularity
LiquidHaskell uses GHC modules. Idris 2 has interfaces + modules. Agda has parameterized modules.

### Suitability for autoformalization to IR
Sweet spot for IR schemas. An LLM can generate Idris/Agda data type definitions where guideline structural invariants are part of the types. Convergence/idempotency improved because incorrectly-shaped output fails to compile, providing a hard syntactic gate.

### Formal verification potential
Vazou, Seidel, Jhala, Vytiniotis, and Peyton-Jones, "Refinement Types for Haskell" (ICFP '14, Gothenburg) report: "LIQUIDHASKELL is able to prove 96% of all recursive functions terminating, while requiring a modest 1.7 lines of termination-annotations per 100 lines of code", evaluated on containers, hscolour, bytestring, text, vector-algorithms, and xmonad (>10,000 LoC). Idris's `Dec` and Agda's `Decidable` provide certified decision procedures.

### Tooling/Ecosystem maturity
LiquidHaskell is plugin-active; Idris 2 stable; Agda 2.7+ stable. All have language-server support.

### Japan-specific considerations
**Atsushi Igarashi** group (Kyoto University) has produced fundamental work on refinement types and gradual typing (ICFP, POPL, ECOOP papers; AITO Dahl-Nygaard Junior Prize 2011 for Featherweight Java). HELMHOLTZ tool (Igarashi group) verifies Tezos/Michelson smart contracts using refinement types.

### Interoperability
LiquidHaskell shares an SMT backend with Why3/F*. Idris/Agda export to Coq/Lean via translation (partial). Bridges to Category 1: FHIR `Reference` and `cardinality` constraints become refinement predicates. Category 3: SHACL shape closure rules map onto refinement types; OWL class restrictions ≈ Σ-types in Agda/Idris.

### Limitations/Known issues
Refinement-type frontier: when goals exceed SMT, manual proof needed. Idris 2 ecosystem is small. Agda lacks tactic automation comparable to Lean's. Learning curve is steep.

### Training data proxy
Small relative to Lean. Agda has thousands of GitHub repos; Idris fewer. LiquidHaskell user community moderate.

## 9. Proof by Reflection for Executable Guideline Rule Evaluators

### Purpose
Technique (not a tool) for encoding decision procedures as verified executable functions inside a proof assistant, then invoking them via reduction (`vm_compute`, `native_decide`, `Eval`). Goals become "this Boolean term computes to `true`", reducing proof checking to certified evaluation. Ideal for guideline-applicability checks where rules must be both formally specified and executable.

### Maintainer/Standards body
Conceptually attributed to Samuel Boutin (Inria, TACS 1997, "Using reflection to build efficient and certified decision procedures"). Implementations across Coq/Rocq, Lean 4, Agda.

### Conceptual model
Define an inductive datatype of formulas/syntax. Define a decision procedure `decide : Formula → bool` and prove `forall f, decide f = true → ⟦f⟧`. To prove `⟦f⟧`, reflect to `f`, then prove `decide f = true` by reduction (`vm_compute` in Coq/Rocq; `native_decide` in Lean 4 — compiles to OCaml/C and runs; `Reflexivity` after `vm_compute` in Agda).

### Expressiveness/Semantics
Effectively any computable predicate. Trades term size for proof-search complexity. The kernel must trust the reduction strategy: `vm_compute` uses a Coq-internal VM (trusted), `native_compute`/`native_decide` use OCaml native code compilation (extends TCB to OCaml compiler and runtime).

### Composability/Modularity
Reusable: any inductive can become a reflected language. Library examples: `lia`, `lra`, `ring`, `field` in Coq/Rocq; `omega`, `decide`, `bv_decide`, `grind` in Lean; `solve-by-Reflection` patterns in Agda.

### Suitability for autoformalization to IR
A guideline-rule evaluator is essentially `decide : ClinicalContext → Recommendation → Bool`. With reflection, an LLM can be asked to produce only the rule data (the syntax tree), not proofs — the decision procedure handles the rest. Idempotency: `decide ∘ normalize = decide` provable by reflection.

### Formal verification potential
**SMTCoq** uses reflection to check LFSC certificates from CVC5 inside Rocq with full kernel guarantees. **MirrorShard** (Malecha, Chlipala, Braibant) extends reflection with verified hint databases. CoqHammer reconstructs SMT proofs. In Lean 4, `lean-smt` (arXiv 2505.15796) implements reflection-based discharging of a fragment of SMT proof rules.

### Tooling/Ecosystem maturity
Native to Rocq, Lean, Agda; well-developed.

### Japan-specific considerations
Reynald Affeldt (AIST) has used reflection-based tactics in MathComp/SSReflect formalizations (information theory, ECC). Affeldt's `monae` library uses reflection for equational reasoning over monadic effects.

### Interoperability
Pure-technique; works inside any of the above proof assistants. Reflected proofs can be exported via Dedukti/Lambdapi.

### Limitations/Known issues
`native_decide` enlarges TCB to include the OCaml compiler. Reduction time can be a bottleneck for large terms. Proofs by reflection are opaque to humans.

### Training data proxy
Many tutorial sources (Software Foundations VFA `Decide` chapter, Chlipala CPDT). Less LLM training data than tactic-style proofs.

## 10. Checkable Proof Certificates and Derivation Traces

### Purpose
Generate small, independently-checkable proof artifacts (certificates) in a minimalist language so any party (auditor, regulator, downstream tool) can re-verify a derivation without trusting the original prover. Essential for trustworthy CDS where regulators may need to audit reasoning chains.

### Maintainer/Standards body
- **Dedukti / Lambdapi**: Deducteam (Inria; Frédéric Blanqui). Lambdapi is the modern interactive successor to Dedukti.
- **LFSC**: Stump et al. (University of Iowa); used by CVC4/CVC5.
- **Alethe**: Schurr, Fleury, Barbosa, Fontaine; veriT and cvc5 emit Alethe (specification at `verit.gitlabpages.uliege.be/alethe/specification.pdf`).
- **SC-TPTP**: TPTP community.
- **ProofCert / Foundational Proof Certificates**: Miller, Chihani.
- **Carcara**: Alethe proof checker/elaborator (Barbosa et al., TACAS 2023).

### Conceptual model
A **logical framework** (Edinburgh LF or the λΠ-calculus modulo rewriting underlying Dedukti/Lambdapi) defines a small kernel calculus. Each source logic (HOL, FOL, CIC, SMT theories) is encoded as a signature of constants and rewrite rules. A proof becomes a typed λ-term; checking reduces to type-checking. LFSC adds side conditions. Alethe is an SMT-specific format with proof-rule schemata.

### Expressiveness/Semantics
λΠ-modulo (Dedukti/Lambdapi) is Turing-complete and can encode HOL, CIC, PVS, simple-type theory, Alethe-style SMT proofs. Trade-off: more permissive theories require more axioms (less trust).

### Composability/Modularity
Lambdapi supports implicit arguments, unification hints, tactics — making it suitable as a proof IDE, not just a checker. Multiple source-system translators exist: Coq→Dedukti (CoqInE), HOL-Light→Dedukti, Isabelle→Dedukti, Lean→Dedukti, PVS→Lambdapi (Personoj), MetaMath→Lambdapi.

### Suitability for autoformalization to IR
Excellent neutral IR for cross-proof-assistant verification of clinical-guideline derivations. An LLM-generated Lean proof can be exported to Lambdapi and re-checked by an independent Rocq-trained team. Idempotency: re-running the checker is deterministic and bitwise reproducible.

### Formal verification potential
Tiny TCB (typically 1–2 KLOC). Lambdapi is the de facto pivot for the ICSPA ANR project exchanging B, Event-B, and TLA+ proofs (Coltellacci & Merz, ABZ 2023; Grieu & Bodeveix, ABZ 2024/2025 — Rodin/Event-B certification in Lambdapi).

### Tooling/Ecosystem maturity
Lambdapi VS Code plugin; Deducteam's Personoj (PVS), HOL-Light translators, Carcara checker for Alethe. EuroProofNet COST action (2021–2025) drives interoperability.

### Japan-specific considerations
No major Japanese contribution to Dedukti/Lambdapi core, but JAIST (Ogata, Ogawa) and AIST (Affeldt) are well-positioned to consume the technology. The Deducteam Lambdapi maintainers regularly invite participants from Japan to EuroProofNet workshops.

### Interoperability
Universal proof-pivot. Direct bridges from each of methods 1, 2, 3, 6 (via SMT proofs) above. Carcara elaborates coarse-grained Alethe steps into fine-grained ones for higher-confidence checking. SMTCoq imports LFSC into Rocq. The Isabelle Alethe replay (Lachnitt et al., ITP 2025) closes the loop from cvc5 to Isabelle.

### Limitations/Known issues
Translation gaps (e.g., universe polymorphism, classical axioms) require careful encoding. Lambdapi tactic ecosystem is much smaller than Lean/Rocq's. Performance of `dkcheck`/`lambdapi check` is good but lags native checkers for huge proofs.

### Training data proxy
Small. PxTP workshop, FSCD papers, FroCoS. EuroProofNet community spans hundreds of researchers.

## 11. CrossHair for Python Contract Verification by SMT-Backed Symbolic Execution

### Purpose
Static analysis tool that verifies pure-Python functions against their declared contracts (preconditions, postconditions, invariants) by symbolic execution. Instead of running concrete test inputs, CrossHair substitutes symbolic ("proxy") objects for each argument, traces all reachable execution paths, and asks an SMT solver to find a path-condition assignment that violates the post-condition — i.e., a counter-example. Fills the niche between random/property-based test generators (Hypothesis, pytest-randomly) and heavyweight verifiers (F*, LiquidHaskell): no separate specification language is required, just typed Python plus `assert` statements or docstring contracts.

### Maintainer/Standards body
Solo-founded and primarily maintained by **Phillip Schanely** (GitHub `pschanely`), with a steadily growing contributor community since the first public releases in 2019–2020. PyPI package: `crosshair-tool`. No formal standards body; the contract syntaxes consumed are PEP 316 (docstring-embedded `pre:`/`post:`), `icontract` (Marko Ristin), `deal` (Gram / `orsinium`), plain Python `assert`, and `typing` annotations. Hypothesis integration is shipped as the separate `hypothesis-crosshair` package, co-released with the Hypothesis maintainers (David R. MacIver, Zac Hatfield-Dodds).

### Conceptual model
Concolic / pure-symbolic execution over the CPython AST. For each argument with a type annotation `T`, CrossHair constructs a `Proxy[T]` whose dunder operations (`__eq__`, `__lt__`, attribute access, indexing, iteration) record Z3 path-conditions rather than evaluating concretely. Container types are realised lazily via "smart" symbolic lists/dicts/sets backed by uninterpreted-function arrays. At every branch, CrossHair forks the symbolic state and queues both successors; when a path completes, post-conditions become Z3 obligations whose negation is sent to the solver. A `sat` result is reported as a concrete counter-example (the model is concretised by replacing each proxy with the solver's witness); `unsat` means the path is verified up to the configured budget. Operational modes:
- `crosshair check` — verify all annotated contracts in a file/module/package.
- `crosshair watch` — file-watcher daemon that re-runs incrementally on edit.
- `crosshair diffbehavior fn_a fn_b` — symbolic differential testing; returns inputs on which two implementations diverge (useful for refactor validation and regression localisation).
- `crosshair cover` — emits a minimal concrete test suite achieving symbolic-branch coverage.

### Expressiveness/Semantics
Captures the decidable fragment of Python expressible in QF_UFLIA + QF_S + bit-vectors + uninterpreted functions: bounded integer/string/sequence operations, boolean logic, dict/set membership, dataclass field access, `enum`, `Optional`/union dispatch. Floating point handled by bit-precise IEEE-754 encoding (slow). Verification is sound modulo (i) the path budget (timeouts return `Unknown`, never `Verified`), (ii) C-extension calls (opaque to the symbolic engine), and (iii) non-determinism (`random`, time, network, filesystem — must be stubbed). Mutation, exception flow, and generator semantics are modelled. No support for `multiprocessing`, native threads, or `async`/`await` concurrency.

### Composability/Modularity
Per-file, per-class, or per-function selection via CLI selectors and `# crosshair: on/off` directives. Configuration via `pyproject.toml` (`[tool.crosshair]`) for path budgets, per-condition timeouts, and exclude patterns. Contract decorators are orthogonal: a single function can carry an `icontract.require`, a `deal.post`, a docstring `pre:`, and a Hypothesis strategy simultaneously, and CrossHair will honour all of them. Pre-commit hook and GitHub Actions integrations are documented in the project README.

### Suitability for autoformalization to IR
Strong target for LLM autoformalization at the **implementation layer**: an LLM given a clinical-guideline recommendation can emit a Python function plus a PEP 316 docstring (`pre:`/`post:`), and CrossHair becomes the inner-loop validator that either certifies the implementation or returns a clinical-scenario counter-example. Round-trip stability is high — Python is the highest-frequency language in LLM pretraining corpora, and docstring contracts are far easier for an LLM to generate correctly than Lean/F* proof terms. Idempotency of a guideline normaliser can be expressed as `post: __return__ == normalize(__return__)` and discharged automatically up to the path budget. Convergence under repeated LLM regeneration is good: contract-only changes are local, and counter-examples re-prompt the LLM with concrete data, narrowing the search.

### Formal verification potential
Soundness is bounded by the decidable fragment plus the configured search budget; CrossHair is an **assurance tool**, not a kernel-verified prover. No proof certificate is emitted (a `Verified` result is a search-completed claim, not a checkable derivation), so it does not by itself meet ISO 26262 / IEC 62304 evidence requirements — but counter-examples are concrete reproducers (a runnable `pytest`-style failing input), exactly the form an auditor or regulator can re-execute. Pairs naturally with `lean-smt` / F* / LiquidHaskell when stronger guarantees are required: prototype the contract in CrossHair, then lift it to a refinement-typed or proof-assistant target once it survives counter-example search.

### Tooling/Ecosystem maturity
Active development with multiple releases per year. VS Code extension (`CrossHair Python`) surfaces counter-examples as in-editor squiggles. PyCharm plugin community-maintained. Recent Hypothesis releases ship the `crosshair` backend (`@given(...)` with `settings(backend="crosshair")`), so existing Hypothesis property-based tests can be re-executed under symbolic execution with zero code change to the test body. Pre-commit hook published; GitHub Actions example workflows documented. `hypothesis-crosshair` package on PyPI.

### Japan-specific considerations
No known Japanese core contributors to CrossHair itself. However, the Japanese `icontract` / `deal` user community (visible in JSSST PPL and Python Boot Camp materials) and Hypothesis-using teams at Preferred Networks and LINE-Yahoo Japan benefit transitively: existing property-based tests can be re-validated under the CrossHair backend without rewriting. JAIST's Mizuhito Ogawa group (symbolic execution, SMT for non-linear arithmetic) and NII's Hasuo group (ERATO MMSD; symbolic verification of cyber-physical systems) work in adjacent areas and are natural collaborators for extending CrossHair-style verification to medical-device Python code under AMED-funded CDS pipelines. Japan-Minds-derived recommendation predicates encoded as Python contracts become directly checkable, which lowers the barrier to mechanised clinical validation in Japanese settings (Category 2).

### Interoperability
- **Within Category 4**: Counter-examples from CrossHair lift to F* / Lean as failing-test obligations that drive a fully verified rewrite. Refinement-type schemas (§8) compile to Python type hints + `assert`, directly checkable by CrossHair. Why3 / WhyML executable specifications (§6) can be cross-validated against a Python reference implementation via CrossHair `diffbehavior`.
- **Category 1**: A FHIR Clinical Reasoning predicate prototyped in Python with a docstring contract can be verified by CrossHair before being mechanised in CQL or DMN — useful for "spec by example" workflow on the way to a standards-conformant deliverable.
- **Category 2**: Minds-derived Japanese clinical recommendations expressed as Python predicates (with JLAC / MEDIS terminology codes as constants) become CrossHair-checkable contracts.
- **Category 3**: SHACL shape closures and OWL class restrictions on FHIR resources can be expressed as Python-side `assert` statements; CrossHair finds RDF instances that violate the closure.
- **Category 5**: Shares Z3 as the SMT backend (§5.1); contracts that exceed CrossHair's symbolic-execution depth budget can be re-expressed directly in SMT-LIB or in Apalache / TLA+ (§5.8) for fuller exploration.
- **Category 6**: Temporal clinical predicates (e.g., "two consecutive eGFR < 60 readings ≥ 90 days apart") encode naturally as Python functions over typed event lists; CrossHair explores their boundary cases (off-by-one, empty histories, equal timestamps).
- **Category 8**: Forms the "execution-feedback" leg of an LLM agentic autoformalization loop — counter-examples re-prompt the LLM to repair the contract or the implementation, exactly the closed-loop pattern advocated by Kimina-Prover / Goedel-Prover for proof assistants.
- **Category 9**: Counter-examples are directly usable as adversarial unit-test inputs for clinical-validation suites and human-in-the-loop review.
- **Category 10**: Counter-example traces serve as auditable evidence artifacts in IEC 62304 software-of-medical-device documentation, complementing the assurance-case argumentation from Category 10.

### Limitations/Known issues
Path explosion on loops with data-dependent bounds; recursive functions need explicit `--max_iterations`. C-extension calls (NumPy, Pandas, scikit-learn, PyTorch) are opaque — symbolic execution cannot enter them, so dataframe-heavy clinical pipelines fall back to mocked or stubbed implementations. Floating-point reasoning is supported via IEEE-754 bit-blasting but is orders of magnitude slower than integer/string reasoning, and is the most common cause of `Unknown` results in physiological-threshold rules. No support for `async`/`await` concurrency, native threads, or true parallelism. `Unknown` (budget-exhausted) results are common on real-world code and must **not** be conflated with `Verified` — the project explicitly distinguishes the two states. Single-process Python only; analysis time scales poorly past ~hundreds of LoC per function. No proof certificate or independently-checkable derivation is produced.

### Training data proxy
The `pschanely/CrossHair` GitHub repository has a moderate but steadily growing user community (low-thousands stars range), with `hypothesis-crosshair` a separate dedicated package. Documentation and tutorial corpus is smaller than that of Lean / F* / Hypothesis. LLMs trained on general Python plus `icontract` / `deal` / Hypothesis idioms can produce CrossHair-targetable code without CrossHair-specific fine-tuning, because contracts are syntactically just decorated functions and docstrings. Stack Overflow presence is modest; primary support channels are the GitHub issue tracker and the project's Discord. PyCon US and EuroPython conference talks by Schanely (2020, 2022) serve as the canonical introductions.

## 12. SAW, Crucible, and Cryptol for Implementation Verification Against Formal Specifications

### Purpose
Verify that production C, Java, Rust, Go, or WebAssembly implementations of CDS pipeline components faithfully implement their formal specifications — the "last mile" between a proved property in Lean/Rocq/TLA+/SMT and a deployed binary. Cryptol provides a domain-specific functional language for writing executable reference specifications; SAW (Software Analysis Workbench) proves equivalence between a Cryptol specification and a production implementation via symbolic execution; Crucible is the language-agnostic symbolic execution framework underlying SAW. Together they close the gap that §§1–10 leave open: those entries verify the *specification*; this entry verifies the *implementation*.

### Maintainer/Standards body
**Galois, Inc.** (Portland, Oregon; founded 1999 by John Launchbury). Launchbury served as Director of DARPA's Information Innovation Office (I2O) 2014–2017 before returning to Galois as Chief Scientist. Galois is one of the largest commercial users of Haskell and one of the most active US formal-methods companies. Current releases: **Cryptol 3.5.0** (January 28, 2026; GitHub `GaloisInc/cryptol`, ~1,200 stars, 129 forks, BSD-3-Clause; 87.7% Haskell); **SAW 1.5.1** (May 22, 2026; GitHub `GaloisInc/saw-script`, 505 stars, 82 forks, BSD-3-Clause; 80.1% Haskell); **Crucible** (GitHub `GaloisInc/crucible`, 767 stars, 47 forks, BSD-3-Clause; 81.8% Haskell; user-facing tool Crux v0.12, January 29, 2026). Primary funding: NSA Laboratory for Advanced Cybersecurity Research, Office of Naval Research (Contract N68335-17-C-0452), DARPA (HACMS, SSITH/BESSPIN, CHARIOT/QSAFE programs). Related Galois tools: **What4** (GitHub `GaloisInc/what4`; solver-agnostic Haskell library interfacing Z3, Yices 2, cvc5, Bitwuzla, Boolector, STP, dReal — the SMT/SAT plumbing layer beneath Crucible); **Copilot** (v4.7.1, May 8, 2026; GitHub `Copilot-Language/copilot`, 822 stars; stream-based runtime verification framework generating constant-memory, constant-time C99 monitors from Haskell specifications; funded by NASA Contracts NNL08AD13T, 80LARC17C0004, and NNL09AA00A; maintained by Alwyn Goodloe and Ivan Perez; not to be confused with GitHub Copilot); **Macaw** (binary code discovery and symbolic execution for x86-64, PowerPC, ARM, RISC-V; lifts machine code into Crucible IR).

### Conceptual model
Three components form a pipeline: *specify → verify → deploy*. **Cryptol** is a pure functional language with Hindley-Milner polymorphism extended with arithmetic size constraints, native arbitrary-width bit-vectors, and sequence types. A Cryptol program is an executable specification — it can be `:check`-ed (QuickCheck-style random testing) and `:prove`-d (dispatching to Z3, Yices, cvc5 as SMT backend). Originally developed for the NSA as a classified standard for specifying cryptographic algorithms, made public in 2008. **SAW** provides a scripting language (**SAWScript**) for composing verification tasks at scale. The canonical workflow: write a Cryptol specification of an algorithm or subsystem → compile the production C/Java/Rust code to the appropriate IR (LLVM bitcode for C via clang, JVM bytecode for Java, MIR for Rust) → use SAWScript to state and prove that the implementation computes the same function as the Cryptol spec, for all inputs within a given scope, via symbolic execution backed by SAT/SMT. **Crucible** is the language-agnostic symbolic execution engine: programs are expressed as control-flow graphs (CFGs) that Crucible explores forward-symbolically, constructing path conditions dispatched to What4's solver portfolio. Frontends: `crucible-llvm` / `crux-llvm` (C/C++ via LLVM bitcode, versions 3.5–20.0), `crucible-jvm` (Java), `crux-mir` (Rust via MIR), `crucible-go` (Go), `crucible-wasm` (WebAssembly). **Crux** is the user-facing verification tool built on Crucible, targeting bounded intricate code (cryptographic modules, serializer/deserializer pairs, protocol implementations).

### Expressiveness/Semantics
Cryptol: side-effect-free, total (modulo non-termination warnings), with dependent-length sequence types (e.g., `[n]a` for a sequence of `n` elements of type `a` where `n` is a type-level natural). Constraints like `fin n, n >= 128` express size requirements. Modules, type synonyms, `newtype`, `where` clauses, pattern matching. SAWScript: imperative-looking but deterministic; `llvm_verify`, `jvm_verify`, `mir_verify` commands establish pre/post-condition Hoare triples and compose them via lemma reuse (verified function A becomes an assumption when verifying caller B). Crucible: handles mutation, heap allocation, pointer arithmetic (C), exception flow (Java), ownership/borrowing (Rust/MIR), and garbage collection semantics. What4 encodes path conditions in QF_UFBV + arrays + uninterpreted functions; bitvector reasoning is the core strength given the cryptographic heritage.

### Composability/Modularity
Cryptol modules compose via `import`; specifications can reference each other. SAWScript supports compositional verification: verify a function, then use its specification as an override when verifying its callers — this is critical for scaling to codebases with thousands of functions. Crucible frontends are independently maintained and compose with shared solver backends. The What4 library is separately usable as a standalone Haskell SMT/SAT interface (Hackage package `what4`). All tools integrate via the Galois Haskell ecosystem and share a common symbolic-value representation.

### Suitability for autoformalization to IR
Indirect but important. SAW/Crucible are not themselves autoformalization targets — they verify the *implementation* of the CDS pipeline, not the clinical-guideline IR content. Their role in the CDS architecture is: once the autoformalization pipeline has produced a formally specified and proved IR (via §§1–10), and that IR is compiled into a production service written in Rust/C/Java, SAW/Crucible prove that the service faithfully implements the specification. The Amazon s2n case study (Chudnov, Collins, Cook, Dodds, Huffman, MacCarthaigh, Mertens, Mullen, Tasiran, Tomb, Walkingshaw, "Continuous Formal Verification of Amazon s2n", CAV 2018) demonstrated this pattern for TLS: Cryptol specifications of HMAC and TLS handshake were proved equivalent to the production C implementation, with SAW integrated into the CI pipeline for *continuous* formal verification on every code change — directly analogous to the "Knowledge CI/CD" pattern described in Category 10 §9. Cryptol's DSL-for-specification design is also a proven archetype for any domain-specific specification language the CDS project might create for clinical rules: Cryptol demonstrates that a clean, executable, SMT-verifiable DSL can serve as the single source of truth from which both proofs and implementations flow. Galois's 2025–2026 QSAFE project (under DARPA CHARIOT) formally specified all CNSA 2.0 post-quantum algorithms (ML-KEM/CRYSTALS-Kyber, ML-DSA/CRYSTALS-Dilithium, SLH-DSA/SPHINCS+) in Cryptol, demonstrating that the Cryptol→SAW pipeline scales to production cryptographic standards.

### Formal verification potential
Strong. SAW proves functional equivalence between specification and implementation up to a bounded input scope (bit-precise for bit-vectors; unbounded for symbolic inputs within solver decidability). For the bounded fragment, results are sound — a verified function *provably* computes the same output as the Cryptol spec for all inputs in scope. SAW does not produce independently-checkable proof certificates (unlike DRAT for SAT or Alethe for SMT), but the verification result is reproducible given pinned tool versions and solver configurations. Crucible's symbolic execution is sound modulo the frontend translation (LLVM/JVM/MIR to Crucible IR) and the solver backend; the LLVM frontend has been extensively validated against the LLVM semantics. Copilot (runtime verification) adds a complementary layer: formally specified monitors that observe the deployed system at runtime and flag violations of temporal properties, with mathematically guaranteed constant-memory and constant-time execution — the generated C99 code is provably bisimilar to the Haskell specification via `copilot-verifier`.

### Tooling/Ecosystem maturity
Mature and actively maintained. All major tools saw 2026 releases. SAW supports LLVM 3.5–20.0 (covering all modern clang versions), Java via JVM bytecode, and Rust via MIR. Cryptol ships with a REPL, batch mode, and "Literal Cryptol" for weaving specifications into documents. Galois publishes comprehensive documentation at `tools.galois.com/cryptol` and `tools.galois.com/saw`. The Amazon s2n continuous-verification deployment (CAV 2018) remains the canonical industrial case study for SAW; AWS continues to use SAW for libcrypto verification. Galois has also verified portions of libgcrypt and the Bouncy Castle Java cryptographic library. The DARPA HACMS program ($18M joint project with Rockwell Collins, Data61, Boeing, University of Minnesota; 4.5 years) used Galois's Ivory language (Haskell EDSL for safe C) and Tower concurrency framework to build the SMACCMPilot verified quadcopter autopilot. DARPA SSITH/BESSPIN ($16.6M) applied Galois tools to hardware security via RISC-V FPGA implementations. The NRC HARDENS project demonstrated a formally verified reactor trip system using Galois's Rigorous Digital Engineering methodology. Galois maintains a healthcare practice (galois.com/solutions/healthcare) covering secure health-data analytics (MPC, PSI, differential privacy), cyber-physical medical-device security (the ARPA-H UPGRADE/SAFE-Dev program for hospital cybersecurity), and ML safety for personalized medicine (automated insulin dosers, robotic surgery, virtual patient trials). **Swanky** (GitHub `GaloisInc/swanky`, MIT license) provides Rust libraries for secure multi-party computation (garbled circuits, zero-knowledge proofs, oblivious transfer, private set intersection) relevant to privacy-preserving clinical data analysis. **C2Rust** (developed with Immunant) automates C-to-Rust migration — relevant for modernising legacy C-based CDS implementations into memory-safe Rust (§14).

### Japan-specific considerations
No Galois office or named contributors in Japan. Limited direct Japanese engagement with Cryptol/SAW — the primary user communities are US defense, aerospace, and cloud infrastructure (NSA, AWS, DARPA contractors). However, Japanese researchers in adjacent areas could benefit: Kohei Suenaga (Kyoto University) works on SMT-based hybrid-system verification using related symbolic-execution techniques; Naoki Kobayashi (University of Tokyo) works on higher-order model checking with MoCHi targeting OCaml — Crucible's Haskell-native symbolic execution extends naturally to such contexts; the NII MTSS group (Hasuo) applies symbolic verification to cyber-physical systems. No Japan-specific Cryptol or SAW case study or deployment was identified. Galois tools are BSD-3-Clause licensed, removing IP barriers to Japanese adoption. JAIST's Kazuhiro Ogata group and AIST's formal-methods activities operate in complementary algebraic-specification (CafeOBJ/Maude) and process-algebra spaces that interoperate conceptually with Crucible's CFG-based approach.

### Interoperability
- **Within Category 4**: SAW/Crucible verify implementations of specifications written in Lean (§1, via extracted code), Why3/WhyML (§6, via C extraction), F* (§7, via KaRaMeL C extraction or Rust extraction). Crucible's `crux-mir` handles Rust code verified under Creusot/Prusti contracts (§14). CrossHair (§11) covers Python; Crucible covers C, Java, Rust, Go, WASM — together they span the major deployment languages. What4 shares solver backends with `lean-smt` (§1) and Sledgehammer (§3). Copilot's runtime monitors complement the static verification of §§1–11 with temporal-property monitoring at deployment time.
- **Category 1**: FHIR Clinical Reasoning engine implementations in Java (HAPI) or Rust can be SAW/Crucible-verified against their CQL/ELM specifications; FHIRPath evaluator correctness is amenable to Cryptol specification of the operator semantics.
- **Category 5**: What4 is an alternative solver interface to the same Z3/cvc5/Bitwuzla backends catalogued in §1 of that category; SAW's solver portfolio overlaps.
- **Category 8**: Tool-calling agents (§5) can invoke SAW/Crucible as verification backends behind MCP, closing the loop between LLM-generated code and formal correctness checking.
- **Category 10**: The Amazon s2n continuous-verification pattern maps directly to Knowledge CI/CD (§9): SAW runs in CI on every commit, proving that code changes preserve specification compliance. Copilot monitors map to the Observability/Continuous Verification entry (§10) as formally-guaranteed runtime health checks complementing OTel-style tracing. SBOM/AIBOM (§8): Galois tools' BSD-3-Clause licensing and Haskell-based build chains integrate cleanly with SPDX 3.0 Software + AI profiles.

### Limitations/Known issues
SAW verification is bounded by solver decidability — quantified properties over unbounded data structures require manual lemma decomposition or induction (handled via SAWScript's compositional override mechanism, not fully automated). Cryptol's type system is less expressive than Lean/Rocq (no dependent types beyond size arithmetic, no inductive types); it excels at bit-level specifications but is not a general-purpose theorem prover. Crucible frontends inherit the semantic complexity of their target languages: the LLVM frontend must model C's undefined-behaviour minefield; the JVM frontend must handle reflection and class loading; the MIR frontend tracks Rust borrow semantics. No proof certificates — SAW verification results are tool-specific claims, not independently checkable derivations (contrast with DRAT/Alethe). The Haskell toolchain dependency (GHC 9.6–9.12) adds build complexity for teams not already in the Haskell ecosystem. Galois tools are well-documented but the user community is smaller and more specialised than Lean's or Z3's; Stack Overflow presence is modest. Healthcare applications to date focus on device security and data privacy rather than clinical-guideline formalization — the CDS-specific application of SAW/Crucible to guideline-pipeline verification is a novel use case without published precedent.

### Training data proxy
Moderate. Cryptol: `GaloisInc/cryptol` ~1,200 GitHub stars; the Cryptol Programming Guide and reference manual are public at `tools.galois.com`; specifications of AES, SHA, ECDSA, and CNSA 2.0 post-quantum algorithms are public Cryptol examples. SAW: 505 GitHub stars; the s2n verification is published at CAV 2018 (Chudnov et al.) with high citation count; SAWScript tutorial and reference manual at `tools.galois.com/saw`. Crucible: 767 GitHub stars; academic papers at VSTTE 2016 (Dockins, Foltzer, Hendrix, Huffman, McNamee, Tomb, "Constructing Semantic Models of Programs with the Software Analysis Workbench"). Copilot: 822 GitHub stars; Pike, Wegmann, Niller, Goodloe, "Copilot: A Hard Real-Time Runtime Monitor", RV 2010. Smaller LLM training-data presence than Lean/Z3/FHIR but sufficient for code generation with in-context examples. Stack Overflow and community-forum presence is thin; primary support via Galois's commercial consulting and GitHub issue trackers.

## 13. Typed Functional Programming as the Substrate for Verifiable CDS

### Purpose
Names the cross-cutting foundation that the rest of this category implicitly assumes: nearly every system in §§1–12 is, structurally, an applied typed-functional language. Lean 4, Rocq's Gallina, Isabelle/HOL's inner logic, Agda, Idris 2, F*, LiquidHaskell, Why3/WhyML, and the reflected decision procedures of §9 are all variants of the typed lambda calculus extended with inductive data types, pattern matching, parametric polymorphism, and (where present) dependent or refinement types. Treating "typed functional programming" as a substrate rather than as a list of tools clarifies (i) why autoformalization targets these languages rather than imperative ones, (ii) why pure-functional rule evaluators are uniquely well-suited to regulated CDS, and (iii) where Haskell-, OCaml-, and F*-family ecosystems contribute beyond the proof-assistant entries already catalogued — chiefly as host languages for embedded guideline DSLs, refinement-typed FHIR adapters, and equationally-reasoned rule pipelines. Cross-cutting foundations entry, not a single tool — analogous in framing to §8 (Dependent/Refinement-Type IR Schemas) and §9 (Proof by Reflection), both of which depend on this substrate.

### Maintainer/Standards body
- **Curry–Howard correspondence**: Haskell B. Curry, "Functionality in Combinatory Logic" (1934); William A. Howard, "The Formulae-as-Types Notion of Construction" (1969 manuscript, published 1980 in Seldin & Hindley eds., *To H. B. Curry: Essays on Combinatory Logic, Lambda Calculus and Formalism*).
- **ML lineage**: Robin Milner et al., Edinburgh, "A Theory of Type Polymorphism in Programming" (JCSS 1978); ML was originally the *meta-language* of the LCF theorem prover (Milner, Gordon, Wadsworth, *Edinburgh LCF*, 1979) — i.e., a functional language was invented because proof tactics required higher-order, statically-typed manipulation of theorems. Standard ML defined by Milner, Tofte, Harper, MacQueen, *The Definition of Standard ML (Revised)*, MIT Press 1997.
- **Haskell**: Haskell Language Committee. Haskell 1.0 Report (1990; Hudak, Peyton-Jones, Wadler, et al.); Haskell 2010 Report (Simon Marlow, ed.). De facto standard is **GHC Haskell**, maintained by the GHC Team (Simon Peyton-Jones, Simon Marlow, Richard Eisenberg, Ben Gamari, Simon Hengel, Sebastian Graf, Adam Gundry, et al.) with funding from Microsoft Research, Well-Typed, Tweag, IOG, and the Haskell Foundation (Haskell Foundation Inc., 501(c)(3), founded 2020).
- **OCaml**: Inria (Xavier Leroy, Damien Doligez, Jacques Garrigue, Didier Rémy, Gabriel Scherer, Nicolás Ojeda Bär). OCaml Software Foundation (2018–).
- **F***: Microsoft Research + Inria. Nikhil Swamy, Cătălin Hriţcu, Chantal Keller, Aseem Rastogi, Antoine Delignat-Lavaud, Jonathan Protzenko, Tahina Ramananandro, Aymeric Fromherz. Already catalogued in §7 as a language; here named as a member of the ML family.
- **Liquid Types**: Patrick Rondon, Ming Kawaguchi, Ranjit Jhala, "Liquid Types" (PLDI 2008, Tucson).
- **Propositions as Types** (canonical exposition): Philip Wadler, "Propositions as Types", *CACM* 58(12), December 2015.
- **Pedagogical canon**: Benjamin C. Pierce, *Types and Programming Languages* (MIT Press 2002) and *Advanced Topics in Types and Programming Languages* (2005); Pierce et al., *Software Foundations* (online textbook, Coq/Rocq-based, updated through 2024); Adam Chlipala, *Certified Programming with Dependent Types* (MIT Press 2013). These are the texts on which most current LLM training data about functional verification is grounded.

### Conceptual model
Three intertwined ideas:
- **Curry–Howard**: a type is a proposition, a term of that type is a proof, β-reduction is proof normalisation. A total, type-checking function `eligibleForStatin : PatientRecord → Maybe Recommendation` is, in a dependently-typed setting, simultaneously an executable rule and a constructive proof of "for every patient record, either no recommendation applies or the named recommendation does, and the proof of applicability is recoverable".
- **Algebraic data types (ADTs) + pattern matching**: clinical entities decompose naturally as sums-of-products. `data LabResult = Numeric Double Unit Timestamp | Categorical Code Timestamp | Missing Reason Timestamp` enumerates the closed set of cases; the compiler checks that every rule covers every branch. Inhabited illegal states (e.g., a `Numeric` with no `Unit`) become unrepresentable.
- **Pure functions and referential transparency**: `f x = f x` for every observable `x`; substitutability `a ≡ b ⇒ f a ≡ f b`. This is the algebraic property that (a) makes proof-by-reflection (§9) work, (b) makes equational reasoning over rule pipelines a sound refactoring tool, (c) makes audit replay (Category 10) bitwise reproducible from logged inputs, and (d) makes property-based testing (Category 9, QuickCheck lineage) meaningful — random inputs probe a mathematical function, not a stateful black box.

Layered on top: parametric polymorphism (System F), higher-kinded types and type classes (System Fω + qualified types, Wadler & Blott, POPL 1989), generalised algebraic data types (GADTs, Cheney & Hinze; Peyton-Jones et al., ICFP 2006) for typed ASTs and tagless-final EDSLs, monadic and algebraic-effect encodings (Moggi LICS 1989; Plotkin & Power FOSSACS 2002; Plotkin & Pretnar ESOP 2009) for explicit effect surfaces, and refinement / dependent extensions (§8) for invariant-bearing schemas.

### Expressiveness/Semantics
The relevant fragment of the lambda cube:
- **Simply-typed λ-calculus + base types** ≈ HOL (Isabelle/HOL inner logic).
- **System F (rank-1 polymorphism)** ≈ Hindley–Milner (ML, OCaml, early Haskell).
- **System Fω + type families + GADTs** ≈ modern GHC Haskell. Type families and GADTs together encode a useful fragment of dependent types via singleton-encoded value reflection (Eisenberg & Weirich, "Dependently Typed Haskell" line of work; PHaskell prototype, ICFP 2021).
- **Refinement-extended F** ≈ F*, LiquidHaskell (already §8).
- **Full dependent types (Π, Σ)** ≈ Idris 2, Agda, Lean 4, Rocq Gallina (already §§1, 2, 8).

Equational reasoning is the load-bearing semantic property: in a pure functional core, `let x = e in body` is observably identical to substituting `e` for `x` in `body`, which means a guideline rule expressed as `if eGFR < 60 ∧ persistsFor 90Days then ChronicKidneyDiseaseG3 else …` can be refactored by rewriting under any context-equivalence preserving transformation without changing the rule's externally observed verdict — a property that does *not* hold for the equivalent Java method, because hidden mutable state, exceptions, or lazy-initialised singletons can make `x` and `e` observably different.

### Composability/Modularity
- **ML-style functors (parameterised modules)**: a rule engine parameterised over a terminology service is `module RuleEngine (T : TerminologyService) = struct … end`.
- **Haskell type classes** (Wadler & Blott, POPL 1989): ad-hoc polymorphism. `class CodeSystem c where … instance CodeSystem SnomedCT where … instance CodeSystem JLAC where …` lets the same rule code operate over Western or Japanese terminologies with type-directed dispatch.
- **Backpack** (Edward Z. Yang, POPL 2014): Haskell module signatures. Permits guideline modules to be type-checked against terminology *interfaces* before any concrete code system is linked.
- **Tagless-final encoding** (Carette, Kiselyov, Shan, JFP 2009) and **free monads** (Swierstra, "Data types à la carte", JFP 2008): two canonical techniques for embedding a guideline DSL such that its abstract syntax can be (a) executed against a live FHIR server, (b) traced for audit, (c) reduced symbolically by an SMT backend, (d) emitted as a CQL/DMN artefact, and (e) re-interpreted under a property-based test harness — all from the same source expression. This composability is the operational answer to "how can one IR target Categories 1, 5, 9, and 10 simultaneously?"
- **Lens/optics libraries** (van Laarhoven 2009; Edward Kmett's `lens`, `optics` libraries): compositional read/write access to deeply-nested clinical records (FHIR Bundles, CDA documents) without imperative traversal code.

### Suitability for autoformalization to IR
The target language of LLM autoformalization in this category is, in every case, a typed functional language — Lean 4, Rocq, Isabelle/Isar, Agda, Idris 2, F*. Two structural properties make the substrate good for LLMs:
- **Small surface syntax with strong local checking**. ADTs + pattern matching let an LLM emit a guideline rule as a closed expression whose well-formedness is checkable in a single typechecking pass; ill-typed output is rejected before any semantic check is needed. The signal-to-noise ratio of compiler errors is much higher than for, say, Python (where syntactic acceptance does not imply much).
- **EDSL-style emission**. The most reliable LLM-emission pattern observed in the autoformalization literature (Category 8) is "emit data, not control flow": ask the model to produce a tagless-final term or a free-monad ADT that *describes* the guideline, then leave the evaluation, validation, and proof-discharge to fixed, human-vetted interpreters. This is structurally easier for the LLM than emitting a fully proved Lean term, and the interpreter pipeline can independently target FHIR (Category 1), SMT (Category 5), and audit logs (Category 10).

Convergence: pure-functional outputs are normalisable (β-reduction, `simp` lemmas, GHC `-O2` rewrite rules), which means re-prompts of the LLM that emit semantically-equivalent-but-syntactically-different rules can be canonicalised to a single normal form, sharply improving stability across regenerations.

### Formal verification potential
By Curry–Howard, a closed term of type `T` in a sufficiently expressive functional core *is* a proof of `T` up to the consistency of the type theory. A total function `validatePrescription : Prescription → Either Error (Verified Prescription)` whose codomain encodes the verification obligations is therefore both an executable validator and a proof object that survives extraction, kernel re-checking, and (via §10) certificate emission.

Wadler's "Propositions as Types" (CACM 2015) is the canonical short exposition. Pierce's *Software Foundations* (VFA and PLF volumes) provides the most widely-used pedagogical bridge from "functional programming" to "verification" — and is, by GitHub stars and citation, the single most referenced training corpus for LLM-targeted proof generation. Concrete CDS-relevant verification artefacts:
- **CompCert** (Leroy, Inria; *CACM* 2009): a verified C compiler whose passes are pure functional and whose correctness theorem is mechanised in Coq. Demonstrates that the substrate scales to industrial software with regulatory implications (RTCA DO-178C avionics qualification).
- **seL4** (Klein et al., SOSP 2009): functional-correctness proof of an OS kernel, Haskell prototype refined to verified C. Sets the benchmark for what "verified" means in safety-critical contexts comparable to ISO 14971 medical-device risk.
- **Project Everest / miTLS / HACL\*** (Microsoft Research + Inria, 2016–): production verified TLS stack written in F*. Same substrate as could host a verified CDS rule engine.

### Tooling/Ecosystem maturity
- **GHC Haskell** (current series 9.10/9.12): production-grade. Industrial users include Standard Chartered (entire quant library; Wei Hu, ICFP 2014 keynote), Meta's Sigma anti-abuse system (Jon Coens, Marlow, et al., Sigma processes ~1M req/s in Haskell), Mercury Bank, Anduril, GitHub Semantic, IOG (Cardano).
- **HLS (Haskell Language Server)**: LSP server with type-on-hover, code lenses, hlint integration. Maintained by the Haskell Foundation HLS team.
- **Cabal / Stack / Nix**: three coexisting build tools. Hackage hosts ~16 000 packages; Stackage provides curated LTS snapshots.
- **GHC plugins** including LiquidHaskell (§8), Plutus, type-checker plugins (`ghc-typelits-natnormalise`).
- **OCaml 5** (effect handlers, parallelism): Inria; Jane Street is the largest industrial user (millions of LoC; financial trading), MirageOS (unikernels), Coq/Rocq itself.
- **F\* / Karamel / EverCrypt** (Project Everest): production deployments in Windows kernel HTTPS stack, Linux kernel WireGuard implementation. CI under Microsoft Research.
- **Lean 4** (already §1) is implemented in itself, compiles to C, and is a complete functional language.

### Japan-specific considerations
- **Kazu Yamamoto** (IIJ-II — Internet Initiative Japan Innovation Institute): one of the most prolific Japanese Haskell production engineers. Author/maintainer of `tls`, `http2`, `http3`, `quic`, `dns`, `iproute`, and the Mighttpd2 web server — the network stack that underlies a non-trivial fraction of Japan-hosted Haskell services. His Mighttpd work demonstrates that high-throughput production Haskell is operationally viable in Japanese infrastructure environments, which matters for hosting clinical-rule services nationally.
- **Atsushi Igarashi** (Kyoto University): refinement types, gradual typing, Featherweight Java (AITO Dahl-Nygaard Junior Prize 2011). Already cited in §8.
- **Eijiro Sumii** (Tohoku University): polymorphism, type-system metatheory, bisimulation for higher-order languages. POPL/ICFP author.
- **Naoki Kobayashi** (University of Tokyo): higher-order model checking, intersection types, type-based program verification (HORS, HFL model checking; ERATO MMSD collaborator). His MoCHi tool verifies higher-order OCaml programs against safety properties — directly applicable to verified OCaml-hosted CDS rule engines.
- **Susumu Katayama** (Miyazaki University): MagicHaskeller inductive program synthesis from types — a precursor to modern LLM-driven program synthesis.
- **Reynald Affeldt** (AIST): MathComp/SSReflect formalisations; `monae` monad library — Japanese-hosted reusable functional verification infrastructure.
- **Yoshihiko Futamura** (Meiji Gakuin, emeritus): partial evaluation (the Futamura projections, *Systems, Computers, Controls* 1971) — a foundational technique for specialising guideline interpreters to fixed guidelines, with direct CDS performance implications.
- **Community**: Haskell-jp (haskell.jp), regular Haskell-jp Mokumoku sessions, the "関数プログラミング" community in Tokyo and Osaka. Japanese translations of Hutton's *Programming in Haskell* (2009/2017) and Bird's *Thinking Functionally with Haskell* (2014) exist; native textbooks by Yamamoto, Kenji Yoshida (Septeni; `xuwei-k`), and others.
- **PPL Workshop** (Programming and Programming Languages, JSSST): annual venue where Japanese functional-programming research is presented; relevant for recruiting collaborators on CDS DSL work.
- **AMED relevance**: AMED-funded clinical-AI projects to date have used predominantly Python and Java stacks. A typed-functional CDS layer is currently unrepresented in AMED-funded CDS, which is simultaneously a gap and an opportunity — the closest extant work is Igarashi's HELMHOLTZ (Tezos/Michelson smart-contract refinement-type verifier, not clinical) and Kobayashi's MoCHi (general higher-order verifier).

### Interoperability
- **Within Category 4**: This entry *underlies* §§1, 2, 3, 7, 8, 9. §1 Lean 4 is a typed-functional language with dependent types; §2 Rocq's Gallina is a pure functional core (CIC); §3 Isabelle/HOL's inner logic is essentially typed lambda calculus; §7 F* is ML-family; §8 LiquidHaskell + Idris + Agda are explicitly functional; §9 proof-by-reflection is only sound because the reduction semantics are those of a pure functional core. §10 proof certificates serialise functional λ-terms in λΠ-calculus. §11 CrossHair, although Python-targeted, is structurally a symbolic *function* interpreter — its tractability comes from treating user code as if it were pure.
- **Category 1 (Computable Guideline IR & CDS Standards)**: Haskell FHIR libraries (`fhir`, `hs-fhir` — community-maintained, smaller than the Java HAPI FHIR ecosystem but useful as a typed reference implementation); **Servant** (Alp Mestanogullari, Julian Arni et al.) for type-level-routed FHIR REST APIs where each endpoint's request/response shape is enforced by the type checker; parser-combinator implementations of CQL grammar are straightforward in `megaparsec` / `parsec`; DMN decision tables encode as sum types with exhaustive pattern matches.
- **Category 2 (Japan Clinical Guideline & Terminology)**: SNOMED CT, ICD-10, JLAC, MEDIS code spaces become `newtype`-wrapped identifiers with smart constructors enforcing well-formedness — illegal codes unrepresentable. Minds-derived recommendation predicates expressed as Haskell ADTs are directly consumable by both LiquidHaskell (§8) and Lean 4 (§1) via straightforward syntactic mapping.
- **Category 3 (Ontologies / RDF / Terminology Engineering)**: Haskell RDF libraries (`rdf4h`, `hsparql`) provide SPARQL query bindings; SHACL closure rules map onto refinement-type predicates (§8); OWL class restrictions correspond to Σ-types in Idris/Agda, also reachable from Haskell via singleton-encoded `data` definitions.
- **Category 5 (Automated Reasoning & Constraint Solving)**: **SBV** (Levent Erkök; "Symbolic Bit Vectors") is the canonical Haskell binding for Z3, CVC5, Yices, MathSAT, Boolector, ABC — letting a guideline expressed as a Haskell function be discharged directly to an SMT solver. Why3/WhyML (§6) is ML-family by construction. SMT-LIB itself is an s-expression-shaped functional language, parseable in tens of lines of Haskell.
- **Category 6 (Clinical Rule Semantics & Temporal Reasoning)**: Allen's interval algebra has a textbook encoding as a 13-constructor Haskell ADT with pattern-matching rule combinators (Allen, *CACM* 1983). Functional reactive programming libraries (Yampa, reflex-frp, threepenny-gui; Conal Elliott & Paul Hudak, ICFP 1997 for FRP origin) provide a typed substrate for streaming clinical event reasoning with explicit temporal semantics.
- **Category 7 (Information Retrieval / RAG)**: Less direct, but Hasktorch (Haskell bindings to LibTorch) and Servant-based RAG service shells exist; the primary contribution of this substrate is to wrap untyped LLM I/O in typed contracts at the application boundary.
- **Category 8 (Language Models / Agentic Autoformalization)**: LLM-emitted code in Lean / Rocq / Agda / F* / LiquidHaskell *is* code in a typed functional language. GHC's `-Wall -Werror` and Lean's elaborator both serve as the same "syntactic gate" pattern — only well-typed agent outputs survive the inner loop. Tagless-final EDSL emission (described above) is the structural pattern that lets LLM-generated guideline data flow to multiple downstream interpreters in one round.
- **Category 9 (Evaluation / Clinical Validation)**: **QuickCheck** (Koen Claessen & John Hughes, ICFP 2000, Montréal) — already cited at `evaluation-clinical-validation.md:174` — is the canonical Haskell-origin technique for property-based testing of pure functional code. Equational reasoning lets shrinkers minimise counter-examples without re-running expensive harness setup. SmallCheck (Runciman, Naylor, Lindblad, Haskell Symposium 2008) provides exhaustive bounded-depth alternatives.
- **Category 10 (Regulatory Compliance / Assurance)**: Pure rule evaluators produce **trivially reproducible audit trails**: given the same input record and the same compiled binary, output is bitwise identical, no hidden state needs to be captured. This eliminates an entire class of audit-non-determinism issues that plague stateful CDS evaluators and directly supports IEC 62304 §5.7 (verification) and §8 (configuration management) reproducibility evidence, FDA SaMD pre-market documentation, and Japan's PMDA Notification 0411 No. 1 (program-medical-device software-lifecycle) documentation. Futamura-style partial evaluation (above) of a guideline interpreter to a fixed Minds-J 2023 guideline yields a residual specialised binary whose audit surface is just that one guideline — directly relevant to per-guideline release certification.

### Limitations/Known issues
- **Industrial CDS ecosystems are predominantly imperative**: HAPI FHIR (Java), Smart-on-FHIR client libraries (JavaScript/TypeScript), CDC FHIR validators (Java), most hospital EHR-side APIs (C# for US vendors, Java for Cerner/Oracle Health, varied for Japan vendors). FFI bridging (Haskell `inline-java`, OCaml `ctypes`, F* `KaRaMeL` to C) is real but adds operational complexity.
- **Hiring pool**: Healthcare-informatics-with-functional-programming is a small intersection. Recruiting in Japan is feasible (IIJ-II, AIST, university spinouts) but smaller than for Python or Java teams.
- **Laziness (Haskell-specific)**: GHC's default non-strict evaluation complicates space and time reasoning for streaming clinical event data. Strict-by-default extensions (`StrictData`, `Strict`, bang patterns, `nothunks`) mitigate this but require disciplined use. OCaml / F* / SML are strict by default and avoid this issue.
- **Module-system gaps**: Haskell's module system (even with Backpack) is weaker than SML's; OCaml functors are more expressive but less integrated with the type-class style. The mismatch between Haskell type classes and ML functors hampers cross-ecosystem code reuse.
- **"Avoid success at all costs"**: Peyton-Jones's well-known Haskell slogan reflects a community preference for experimental language evolution over backward compatibility — relevant to long-lived regulated software where ABI/API stability across a 10-year medical-device lifecycle matters. Pin to LTS Stackage or Nix-managed flake inputs.
- **Refinement / dependent type fragments still have unsolved problems**: type-class coherence under refinement is partially open (LiquidHaskell handles this; GHC does not natively); decidability boundaries of refinement predicate logics are SMT-dependent.
- **Floating-point semantics**: physiological-threshold rules involve real numbers; rigorous functional encodings need IEEE-754-bit-precise reasoning (slow) or rational/interval surrogates (lossy). Same issue noted in §11 CrossHair limitations.
- **Documentation quality varies**: GHC user guide is excellent; LiquidHaskell, Idris 2, Agda documentation is uneven; F* has improved markedly post-2022 but is still steeper than mainstream FP onboarding.

### Training data proxy
- **Haskell**: Hackage hosts ~16 000 packages; GitHub Haskell repositories number in the low hundreds of thousands. Canonical texts known to be present in LLM pretraining corpora: Hutton, *Programming in Haskell* (2nd ed. 2016, Cambridge); Bird, *Thinking Functionally with Haskell* (2014, Cambridge); Hudak, Hughes, Peyton-Jones, Wadler, "A History of Haskell: Being Lazy with Class" (HOPL III 2007); Marlow, *Parallel and Concurrent Programming in Haskell* (O'Reilly 2013); Lipovača, *Learn You a Haskell for Great Good!* (2011, No Starch). University course materials from Edinburgh, Cambridge, Glasgow, Utrecht, Chalmers, UPenn (CIS 194). Pierce, *Software Foundations* (online, Coq-based) is the single largest functional-verification training corpus.
- **OCaml**: Madhavapeddy, Minsky, Hickey, *Real World OCaml* (O'Reilly, 2nd ed. 2022). Inria's OCaml documentation. Jane Street's open-source `core`/`async` libraries.
- **F\***: Project Everest publications. F\* tutorial (`fstar-lang.org`). Smaller corpus than Haskell/OCaml but growing.
- **F#**: Scott Wlaschin, *Domain Modeling Made Functional* (Pragmatic Bookshelf 2018) — particularly relevant for CDS because the worked examples are domain-modelling with ADTs and railway-oriented programming directly applicable to clinical-decision pipelines.
- **LLM coverage**: Haskell is moderately represented (sufficient for current-generation models to produce idiomatic code with reasonable type-class usage and monad transformer stacks). OCaml is sparser. F* and Idris are sparse enough that LLMs benefit substantially from in-context exemplars (Category 8 retrieval-augmented synthesis).
- **Conferences**: ICFP (annual; the canonical venue), POPL, PLDI, Haskell Symposium, OCaml Workshop, ML Workshop, TyDe (Type-Driven Development), CPP (Certified Programs and Proofs). Japan-relevant: PPL (JSSST Programming and Programming Languages Workshop, annual); IFIP WG 2.8 Functional Programming meetings have been held in Japan (e.g., Kobe 2018, Tomakomai 2022).

## 14. Memory-Safe Systems Languages: Rust, Ada/SPARK, and the Production Substrate

### Purpose
Names the cross-cutting *deployment* substrate that complements §13: where §13 catalogues the typed-functional core in which guideline rules can be *reasoned about*, this entry catalogues the systems-programming substrate in which the resulting CDS binaries, FHIR servers, ingestion pipelines, and audit-log writers actually *run* — and runs them with compile-time elimination of the memory-safety vulnerability classes (use-after-free, double-free, buffer overruns, data races) that have historically dominated CVEs in clinical software stacks built atop C, C++, or hand-tuned JNI. Memory safety is a *deployment-time* property, separate from but complementary to the *specification-time* property of formal correctness (§§1–12) and the *substrate-time* property of equational reasoning (§13). A verified guideline rule loaded into a memory-unsafe runtime is not a verified CDS system; the assurance argument leaks through the runtime. Treating memory-safe systems languages as a substrate rather than as a single tool clarifies (i) why CISA's January 1, 2026 memory-safety roadmap mandate has direct implications for any CDS vendor on the Secure-by-Design Pledge, (ii) why the Safety-Critical Rust Consortium and Ferrocene's IEC 62304 Class C qualification are now first-order considerations for Class III SaMD architecture, and (iii) where Rust, Ada/SPARK, and (to varying degrees) Swift, Go, Java, and C# fit alongside the typed-functional ecosystem of §13. Cross-cutting deployment-substrate entry, parallel in framing to §13.

### Maintainer/Standards body
- **Rust**: Rust Foundation (501(c)(6), Delaware, founded February 2021); Rust Project leadership council (succeeds the former core team since the December 2022 governance restructure). Linux 7.0 kernel builds anchor to Rust 1.93 (Debian stable toolchain). The language is frozen at the 2024 edition with a steady 6-week release cadence.
- **Safety-Critical Rust Consortium**: founded June 2024 under the Rust Foundation umbrella, with ten founding organisations — AdaCore, Arm, Ferrous Systems, HighTec EDV-Systeme, Lynx Software Technologies, OxidOS, TECHFUND, TrustInSoft, Veecle, and **Woven by Toyota** (Tokyo). Publishes the in-development *Safety-Critical Rust Coding Guidelines* on GitHub (`Safety-Critical-Rust-Consortium/safety-critical-rust-coding-guidelines`) and is driving a 2026 Rust Project Goal for MC/DC (Modified Condition/Decision Coverage) support — the coverage criterion DO-178C/DO-254 requires for DAL A avionics, transferrable to airborne implantable medical devices.
- **Ferrocene** (qualified Rust toolchain): Ferrous Systems GmbH (Berlin); TÜV SÜD qualification body. **Ferrocene 26.02.0** (February 2026) covers Rust 1.91/1.92, adds ISO 26262 ASIL B certification for a curated subset of the Rust `core` library, and retains the toolchain's existing TÜV SÜD qualifications: ISO 26262 ASIL D (highest classification), IEC 61508 SIL 3, and — most directly relevant here — **IEC 62304 Class C** (the highest medical-device software-safety class), the latter announced January 14, 2025 as the first such qualification for any Rust toolchain. Ferrocene supports customer certification efforts toward IEC 61508 SIL 4 and DO-178C DAL C. The Ferrocene source has been open-sourced (Apache-2.0 / MIT) since 2024.
- **Ada / SPARK**: AdaCore (Paris/New York; Cyrille Comar, Robert Dewar founders, the latter d. 2015). Ada 2022 standard (ISO/IEC 8652:2023); SPARK 2014/2022. SPARK Pro is the production verification toolchain; the open-source GNAT Community / GNAT FSF lines remain available. AdaCore explicitly markets to the medical-device industry (`adacore.com/industries/medical`).
- **Swift**: Apple Inc.; Swift Core Team (Chris Lattner originated, currently led by Doug Gregor, Ben Cohen, et al.). Swift 6.2 ships a strict-memory-safety mode explicitly aligned with NSA/CISA guidance, with positioning for defense and regulated workloads; Swift now compiles to Android, Windows, and embedded targets via the Swift Embedded subset.
- **Go**: Google (Robert Griesemer, Rob Pike, Ken Thompson); Go team. Memory-safe via garbage collection; concurrency-safe via a runtime race detector — not a static guarantee. Go 1.24 (2025) introduced range-over-func iterators and tightened the supply-chain story via `GOAUTH`.
- **Java / C# / Kotlin / JVM-family**: managed-runtime memory safety. Already implicit in HAPI FHIR (Java) and most EHR-vendor stacks; relevant here only as the comparison baseline.
- **CISA / NSA joint guidance**: Cybersecurity and Infrastructure Security Agency (CISA); National Security Agency Cybersecurity Directorate. Joint information sheet *Memory Safe Languages: Reducing Vulnerabilities in Modern Software Development* (June 24, 2025) updates the 2023 *The Case for Memory Safe Roadmaps*. The CISA Secure-by-Design Pledge (296 signatories as of May 2026) sets a **January 1, 2026 deadline** for signatory organisations to publish a memory-safety roadmap. The information sheet explicitly highlights **Ada and Rust** as the two languages combining memory safety with the performance and predictability needed in safety-critical contexts.
- **DARPA TRACTOR** (Translating All C to Rust): DARPA Information Innovation Office, announced July 2024; T&E by MIT Lincoln Laboratory. Targets automated C-to-Rust translation matching skilled-human Rust style; combines static/dynamic analysis with LLM-driven synthesis. Contract awards to academic teams from mid-2025.
- **Zig**: Zig Software Foundation (Andrew Kelley founder). Approaching but not yet at 1.0 as of May 2026 — included here only to clarify that *Zig is not memory-safe* in the CISA sense (no borrow checker, no type-system-enforced lifetime; runtime safety only in debug builds, undefined behaviour in release).

### Conceptual model
Four intertwined ideas, with Rust as the canonical embodiment:

- **Ownership and affine types**: every value has exactly one owner; passing or returning a value transfers ownership; the owner's scope determines the value's lifetime. Formally, Rust is an affine type system (each binding used at most once, with `Copy` providing a structural exception). Drop is deterministic — destructors run at scope exit — which gives RAII without GC pauses. Wadler's "Linear Types Can Change the World!" (TPL 1990) is the theoretical antecedent; RustBelt (Jung, Jourdan, Krebbers, Dreyer, POPL 2018; Dreyer's 2019 ICFP keynote) is the canonical mechanised soundness proof for safe Rust + a well-bracketed `unsafe` boundary.
- **Borrowing and lifetimes**: shared (`&T`) and mutable (`&mut T`) references with statically-checked non-aliasing. The borrow checker enforces that mutable references are exclusive and that no reference outlives its referent — the static enforcement of the "law of exclusivity" (Swift uses the same term for its analogous, mixed static/dynamic enforcement). The aliasing model under which `unsafe` code must operate has evolved from **Stacked Borrows** (Jung, Dang, Kang, Dreyer, POPL 2020) to **Tree Borrows** (Villani, Jung, et al., 2023–2025), with Tree Borrows reducing false-positive rejections of legitimate unsafe patterns by ~54% while preserving the optimisations the compiler depends on. **Miri** (Ralf Jung, Benjamin Kimock, Christian Poveda, Eduardo Sánchez Muñoz, Oli Scherer, Qian Wang; *Miri: Practical Undefined Behavior Detection for Rust*, POPL 2026) is the Rust MIR-level interpreter implementing both models and the de facto undefined-behaviour detector for `unsafe` code.
- **Send / Sync and compile-time data-race freedom**: Rust elevates "is it safe to send this value across threads?" (`Send`) and "is it safe to share this value across threads?" (`Sync`) to auto-derived marker traits checked at compile time. Combined with ownership, this yields compile-time data-race freedom for safe Rust — the property that distinguishes Rust from Go (GC-safe, races at runtime), Java (memory-safe, race-prone), and C++ (neither).
- **Pure-safety vs. proven-functional-correctness layering**: memory safety is necessary but not sufficient for CDS correctness. Memory safety closes the *security-vulnerability* surface; functional correctness (§§1–12) closes the *clinical-correctness* surface. SPARK proves both simultaneously: SPARK is the intersection of an Ada subset that excludes aliasing-via-pointers (or, since SPARK 2014, admits a Rust-style ownership-based pointer model) with the proof obligations derived from contracts. Rust + Verus / Prusti / Creusot / Kani approach the same intersection from the other direction (start with a memory-safe core, layer specification and discharge atop). This layering is the most important conceptual point of this section.

### Expressiveness/Semantics
Rust's safe fragment is, by construction, the subset of (essentially) System F + algebraic data types + affine references where the borrow checker accepts the programme. Within that fragment:

- **ADTs + pattern matching**: structurally identical to §13. `enum LabResult { Numeric(f64, Unit, Timestamp), Categorical(Code, Timestamp), Missing(Reason, Timestamp) }` is the same data structure Haskell expresses, and `match` exhaustiveness is the same compile-time obligation. Differences are operational: stack allocation by default, monomorphisation of generics, no laziness.
- **Trait system**: Rust traits (Wadler-style ad-hoc polymorphism) are isomorphic to Haskell type classes, with coherence enforced (one `impl` per type-trait pair) and orphan rules to make this tractable. `trait CodeSystem` with implementations for `SnomedCT`, `JLAC`, `MEDIS` lets the same rule code dispatch type-directedly to Japanese or Western terminologies.
- **Trait objects (`dyn Trait`)**: existential subtyping for dynamic dispatch when monomorphisation is undesirable (e.g., heterogeneous rule collections at runtime).
- **`const` generics + GATs (Generic Associated Types, stable since Rust 1.65)**: the Rust analogue of GHC's type families. Enables type-level encoding of clinical-record schemas with unit-of-measure and reference-range constraints.
- **`unsafe` as a stratified escape hatch**: safe Rust ⊃ `unsafe` Rust; safety obligations on `unsafe` code are written in prose (the "safety contract" of `unsafe fn` and `unsafe { … }` blocks). The standard library carefully encapsulates unsafe primitives so downstream code can be entirely safe. The **Rust Safety Tags RFC** (in flight as of 2026) extends this with machine-checkable annotations: every public `unsafe` API in the standard library gets tagged with safety properties verifiable by Clippy lints and rust-analyzer.
- **`async` / `.await`**: Rust's async model uses stackless coroutines compiled to state machines, with the `Future` trait + executor runtimes (Tokio, async-std, Smol, Embassy). Memory-safe under the same ownership rules; borrow-checker interaction with self-referential async state has been stable since Rust 1.39 (December 2019).

SPARK operates on a subset of Ada that excludes (or constrains) aliasing through pointers, excludes exceptions, and excludes recursion that cannot be bounded — yielding programmes whose verification conditions can be discharged by GNATprove (Why3 + CVC5 / Z3 / Alt-Ergo backend, §6). The trade-off is expressiveness for proof tractability. SPARK 2014's introduction of an ownership-based pointer model (Foughali et al., *Borrowing Safe Pointers from Rust in SPARK*, 2018) lifted SPARK from "no pointers" to "pointers under Rust-style aliasing discipline" while preserving full proof discharge.

### Composability/Modularity
- **Cargo + crates.io**: monomorphisation-aware build system; the crates.io registry hosts ~165,000 packages as of May 2026, with the **RustSec Advisory Database** (~600 advisories, volunteer-maintained by the Rust Security Working Group) and `cargo audit` providing dependency-vulnerability scanning. Reproducible builds via `Cargo.lock`; offline builds via `cargo vendor`. The contrast with the Haskell ecosystem (Cabal + Hackage + Stackage) is that Rust's monomorphisation + LTO produces single static binaries with deterministic optimisation — directly suitable for clinical-binary release artefacts. The contrast with Java (Maven, classpath) is the absence of a runtime classloader, eliminating an entire class of supply-chain attacks where a malicious dependency overrides a benign one at load time.
- **Workspaces and feature flags**: Cargo workspaces compose multiple crates with shared dependency resolution; feature flags express compile-time configuration (e.g., FHIR R4 vs. R5 vs. R6 support in the Helios FHIR server, where `--features r4,r5,r6` selects the resource model at build time).
- **`#![no_std]` and `#![no_alloc]`**: the standard library is stratified as `core` (no allocation, no I/O), `alloc` (allocation, no I/O), `std` (full). This stratification is what lets the same Rust language target embedded medical devices (implantable monitors using `#![no_std]` + Embassy) and full-fat hospital servers. Ferrocene's certified subset is `core` plus a curated `alloc` subset.
- **Rust foreign-function interface**: `extern "C"` provides zero-cost C ABI compatibility; `bindgen` automates C header-to-Rust generation; `cxx` (David Tolnay) provides typed Rust ↔ C++ interop. Crucial for incremental migration from HAPI FHIR (Java, via JNI) or Cerner-Oracle-Health backends without clean-room rewrites.
- **Ada `with` / `use` clauses + child packages + generics**: Ada's module system predates ML's by a decade (Ada 83) and is closer in spirit to ML functors than to Haskell type classes. SPARK preserves Ada's modularity while restricting expressiveness.

### Suitability for autoformalization to IR
Memory-safe systems languages are *not* the primary autoformalization target — that role belongs to the typed-functional substrate of §13. They are, however, the natural compilation target after autoformalization:

- **Rust as compilation target**: Lean 4 → Rust extractor (community-maintained, smaller surface than Coq/Rocq's OCaml/Haskell extraction) emits idiomatic Rust from a verified Lean term. F* → KaRaMeL → C is production-deployed (HACL\*, Project Everest); a KaRaMeL-to-Rust backend (in development at Microsoft Research, 2024–2026) targets Rust for the same use case. The clinical-deployment story is therefore: prove the rule in Lean 4 (§1), extract to Rust, link into a Rust-native FHIR server (e.g., Helios), deploy as a single static binary with reproducible build under an IEC 62304-qualified toolchain (Ferrocene). This pipeline is structurally cleaner than Lean → OCaml → dynamically-linked Linux runtime, both for reproducibility and for the regulatory surface area.
- **Refinement-typed Rust** via Prusti and Creusot lets verification specifications live alongside implementation in a single language; the LLM's job is then to emit annotated Rust rather than a separate proof artefact. This is operationally easier than producing well-typed Lean. Verus extends this with *linear ghost types* specifically to capture proof-relevant invariants about pointer-manipulating code.
- **Idempotency via determinism**: a pure-safe-Rust function from `PatientRecord` to `Recommendation` is bitwise-deterministic on identical inputs — no hidden state, no allocator non-determinism in pure functions, no floating-point reordering under controlled codegen flags. This is the same property §13 attributes to the functional substrate, achieved here through ownership rather than purity.

### Formal verification potential
- **Rust + Verus** (Andrea Lattuada, Travis Hance, Chanhee Cho, Matthias Brun, Isitha Subasinghe, Yi Zhou, Jon Howell, Bryan Parno, Chris Hawblitzel; OOPSLA 2023, "Verus: Verifying Rust Programs using Linear Ghost Types"; with a follow-up "Verus: A Practical Foundation for Systems Verification" by Lattuada, Hance, Bosamiya, Brun, Cho, LeBlanc, Srinivasan, Achermann, Chajed, Hawblitzel, Howell, Lorch, Padon, and Parno at SOSP 2024, Distinguished Artifact Award): SMT-backed verifier exploiting Rust's ownership and borrow-checking in proofs; linear ghost types; supports pointer-manipulating and concurrent Rust. Production-adjacent uses in Microsoft's IronKV verified key-value store and the Verus-verified bootloader work.
- **Rust + Prusti** (ETH Zürich; Vytautas Astrauskas, Aurel Bílý, Jonáš Fiala, Zachary Grannan, Christoph Matheja, Peter Müller, Federico Poli, Alexander J. Summers; *The Prusti Project: Formal Verification for Rust*, NFM 2022 — NASA Formal Methods, 14th International Symposium): deductive verifier built on the Viper separation-logic platform. Pre/postconditions in `#[requires]` / `#[ensures]` attributes; loop invariants. Production use at Swiss medical-imaging startups (per their CHI 2025 case studies) and at Boundary Layer.
- **Rust + Creusot** (Xavier Denis, INRIA): translates Rust contracts to Why3 verification conditions, then dispatches to SMT (Z3, CVC5, Alt-Ergo). Featured tutorial at **POPL 2026** (January 11, 2026). Used to develop a verified SAT solver. Its "prophecy encoding" of mutable references is more general than Verus' current support — meaning Creusot can verify a broader class of Rust idioms, though with heavier proof obligations.
- **Rust + Kani** (Amazon Web Services; Felipe Monteiro, Daniel Schwartz-Narbonne, et al.): bounded model checker translating Rust MIR → Goto-C → CBMC. Strong on `unsafe` code verification, weaker on unbounded-loop properties. Open source, MIT/Apache-2.0. Used internally at AWS for parts of Firecracker microVM and for verifying portions of the Rust standard library.
- **Rust Verification Tools (RVT) — Project Oak**: Google's collection of static (KLEE, SeaHorn, SMACK) and dynamic verifiers wrapped behind a unified Rust front end. Smaller community than Verus/Prusti/Creusot/Kani but useful for ensemble verification.
- **Ada SPARK + GNATprove + Why3**: the mature formal-verification stack. Used by **Hillrom** for migration of ECG-algorithm code from C++ to Ada/SPARK, both new and legacy translation — one of the most-cited industrial medical case studies (the Hillrom whitepaper is the canonical reference); by Thales (Astrée-verified avionics); by the SPARK 2014 *Heart-Pump* case study (Medical Design Briefs, *Formally Verifying Heart Pump Software with SPARK and Echo*); and by undisclosed customers in cardiac monitoring and infusion-pump contexts per AdaCore's medical-industry page.
- **Miri**: not a verifier, but the de facto UB detector for `unsafe` Rust. Catches use-after-free, out-of-bounds access, uninitialised-memory reads, alignment violations, data races, and aliasing violations under Tree Borrows. Run as `cargo +nightly miri test` to validate `unsafe` blocks across hundreds of CDS-relevant invariants.
- **Reachability to the rest of Category 4**: a Rust crate verified by Verus or Prusti exports the same proof-certificate shape (§10) as Lean 4 (§1) or Why3 (§6) does — discharge obligations to SMT solvers; emit checkable witnesses. The trusted computing base (TCB) for Rust formal verification adds the Rust compiler (large) and the Verus/Prusti translator (medium), versus Lean's small kernel — a meaningful TCB difference that argues for Lean 4 as the *specification* language and Rust as the *implementation* language in dual-stack CDS architectures.

### Tooling/Ecosystem maturity
- **Toolchain**: `rustup` for toolchain management; **Ferrocene** for IEC 62304 / ISO 26262 / IEC 61508-qualified toolchain on the regulated side. `rust-analyzer` (LSP server, originally Aleksey Kladov) is the de facto IDE backend, with VS Code, IntelliJ, Emacs, Helix, and Zed integrations. Clippy (~770 lints) is the canonical static-analysis layer; `cargo fmt` (rustfmt) provides deterministic formatting.
- **Rust in Linux Kernel 7.0** (2026): Linux 7.0 removed Rust's experimental designation; mainline kernel builds anchor to Rust 1.93 / Debian stable. NVIDIA's Nova GPU driver, Google's Android `ashmem` subsystem, and several NVMe and high-speed-networking drivers run in Rust in production on hundreds of millions of devices. Kernel policy is now "Rust for new code, C for existing subsystems, no forced migrations" — directly relevant because hospital-server kernels and embedded-device firmware will run Rust drivers under existing CDS workloads. Roughly two-thirds of historical Linux CVEs trace to memory-safety bugs that the Rust borrow checker would prevent at compile time.
- **Rust FHIR ecosystem (production, 2025–2026)**:
  - **Helios FHIR Server** (`HeliosSoftware/hfs`): Rust implementation of HL7 FHIR R4, R4B, R5, and R6, feature-flag selectable; optimised for clinical-analytics workloads.
  - **Haste Health Clinical Data Repository**: FHIR-native CDR in Rust; public benchmarks report ~5× FHIRPath evaluation throughput vs. TypeScript reference implementations.
  - **Kodjin FHIR Server**: commercial Rust microservice FHIR implementation; supports all FHIR versions.
  - **`fhir-sdk`, `fhir-rs`, `rust-fhir`** (crates.io): Rust SDKs generated from FHIR StructureDefinitions, providing type-safe access to FHIR resources.
- **Cryptography**: **RustCrypto** (community-maintained suite of pure-Rust AES, RSA, ECDSA, Ed25519, BLAKE3, etc.); `ring` (BoringSSL-derived, Brian Smith); `rustls` (Rust-native TLS, audited by Cure53, used in Cloudflare's quiche QUIC stack and Mozilla's NSS replacement work). Direct CDS relevance: clinical-data-in-transit encryption with no memory-safety attack surface.
- **WebAssembly**: Rust has the most mature WebAssembly story of any language — `wasm32-wasip2` is a tier-2 platform shipping with every Rust release. WasmEdge supports edge-deployed AI inference; the Bytecode Alliance's `wasmtime` is the production server-side runtime. CDS implication: deploy verified rule binaries to edge devices in clinics with constrained network connectivity (rural Japan, ship-borne medicine, disaster-response telemedicine) as portable WASM modules with sub-millisecond cold start.
- **`async` runtimes**: Tokio (canonical, Carl Lerche et al.), Smol, async-std, Embassy (no-std embedded). Tokio + axum + tower is the production HTTP-server stack used by Helios, Cloudflare's pingora, AWS Firecracker, and Discord's API gateway.
- **AdaCore tooling**: GNAT Studio IDE, GNATprove (SPARK proof discharge), GNATcheck (coding-standards conformance), GNATtest (unit testing), GNATcoverage (MC/DC coverage). AdaCore Certification Materials cover DO-178C, ISO 26262, EN 50128, ECSS, and IEC 61508 and explicitly list medical-device use cases.

### Japan-specific considerations
- **Tokyo Rust** (`tokyorust.org`): Japan's primary Rust community organisation; runs the Tokyo Rust meetup, builds a support network for Japanese Rustaceans, and explicitly positions Rust as a natural fit for Japan's manufacturing, automotive, and critical-infrastructure sectors. Adjacent to but distinct from the broader Rust-jp Japanese-language community.
- **Woven by Toyota** (Tokyo): founding member of the Safety-Critical Rust Consortium. Woven's Arene automotive-OS platform commits heavily to Rust for in-vehicle safety-critical software — a directly transferable case study for hospital-device firmware certified to IEC 62304 (Ferrocene's qualification path is the same).
- **TECHFUND** (Tokyo): founding member of the Safety-Critical Rust Consortium. Background in blockchain / Web3 but actively contributing to safety-critical Rust governance.
- **Preferred Networks (PFN, Tokyo)**: Japan's flagship ML company; **Preferred Robotics** subsidiary works on autonomous mobile robots, a domain where Rust + ROS 2 (via the feature-complete `rclrs` Rust client library, with active development in the ROS 2 Rust Working Group as of the March 2026 meeting) is increasingly the chosen substrate.
- **AdaCore Japan / Japanese SPARK adoption**: AdaCore maintains a Japanese-language presence. Japanese rolling-stock and rail-signalling manufacturers (Hitachi, Kawasaki Heavy Industries) use Ada/SPARK in safety-critical signalling per AdaCore case studies — directly transferable to PMDA-regulated medical-device firmware. The Hillrom ECG SPARK case study is the most-cited medical-domain instance and is read directly in the Japanese medical-device-software training literature.
- **MHLW / METI digital-health context**: Japan's Software-as-a-Medical-Device market is projected to grow from USD 22.93M (2025) to USD 96.20M (2033); MHLW + METI promote ICT-enhanced caregiving. The intersection of "regulated SaMD" + "AMED-funded clinical AI" + "memory-safe substrate" is currently unrepresented in Japanese clinical-software stacks (which lean Python and Java), constituting the same opportunity-gap noted in §13 — only here for the *runtime* substrate rather than the specification substrate.
- **Japanese researchers in Rust-related verification**:
  - **Naoki Kobayashi** (University of Tokyo, already cited §13): higher-order model checking; MoCHi targets OCaml today, but the linearity and ownership analysis techniques transfer to Rust verification.
  - **Tachio Terauchi** (Waseda University): refinement types and dependent refinement types for functional verification; POPL/PLDI author. Refinement-type work directly applicable to Prusti-style Rust verification.
  - **Hiroyuki Katsura, Yuki Nishida** et al. (Tohoku / Kyoto): Rust-related publications at PPL and JSSST PPL workshops, 2022–2026.
- **Conferences**: **Rust.Tokyo** (annual since 2019), Rust-jp community events; **RustConf** (annual, North America), **EuroRust**, **Rust Nation UK**, **Rust India**, **OxidizeConf** (embedded Rust). PPL (JSSST PPL Workshop, already cited §13) covers Rust-related verification work alongside the typed-functional content. **HILT** (High Integrity Language Technology, ACM SIGAda annual) is the closest analogue for Ada/SPARK.

### Interoperability
- **Within Category 4**: This entry pairs with §13 as the *deployment-time* complement to the *substrate-time* entry there. §1 (Lean 4) → Rust extraction; §6 (Why3) ← Creusot translation; §7 (F\*) → KaRaMeL → C (and in-progress → Rust); §8 (refinement-type IR schemas) → Prusti / Creusot annotations; §9 (proof by reflection) executes equivalently in Lean and in pure-safe Rust given the same ADT representations; §10 (proof certificates) are emitted unchanged across substrates; §11 (CrossHair) is Python-only but conceptually parallel to Kani for Rust.
- **Category 1 (Computable Guideline IR & CDS Standards)**: Helios, Haste Health, Kodjin, and the `fhir-sdk` / `fhir-rs` / `rust-fhir` crates supply Rust-native FHIR R4 / R4B / R5 / R6 implementations — replacing or complementing HAPI FHIR (Java) for memory-safe deployment. CQL evaluators in Rust are pre-production (no canonical crate as of May 2026); DMN decision tables encode as `enum` + exhaustive `match` exactly as in Haskell.
- **Category 2 (Japan Clinical Guideline & Terminology)**: SNOMED CT, ICD-10, JLAC, MEDIS code spaces become Rust `newtype` wrappers around `String` or `u32` with smart constructors enforcing well-formedness — illegal codes unrepresentable. Cross-codable to Haskell ADTs (§13) and Lean inductive types (§1) via straightforward Serde JSON serialisation.
- **Category 3 (Ontologies / RDF / Terminology Engineering)**: `oxigraph` (Rust SPARQL engine) and `sophia` (Rust RDF library) provide native triple-store and SPARQL query support. SHACL closure-rule checking has experimental Rust support; production users still typically call Java-based TopBraid via JNI.
- **Category 5 (Automated Reasoning & Constraint Solving)**: Rust SMT bindings exist for Z3 (`z3` crate), CVC5 (`cvc5-sys`), Yices (`yices2`); `egg` for equality-saturation rewriting. Verus, Prusti, Creusot, and Kani all dispatch to SMT under the hood.
- **Category 6 (Clinical Rule Semantics & Temporal Reasoning)**: Allen's interval algebra encodes as a 13-variant `enum`. Functional reactive programming has Rust implementations (`flux-rs`, `frunk`, `bevy` ECS events) — less mature than Haskell's Yampa but production-deployed in robotics (ROS 2 Rust). Stream-processing engines Materialize, Arroyo, and Fluvio are Rust-implemented.
- **Category 7 (Information Retrieval / RAG)**: `candle` (Hugging Face's pure-Rust ML framework), `burn`, `tch-rs` (libtorch bindings), and the Qdrant vector DB (Rust-native) provide a complete Rust-native RAG substrate. Direct relevance: clinical-document retrieval-augmented LLM pipelines deployable as single static binaries with cryptographically reproducible builds.
- **Category 8 (Language Models / Agentic Autoformalization)**: LLM emission of Rust is well supported by current-generation models; the Rust compiler's diagnostic quality is exceptional, which raises the round-trip success rate compared to C++ or Python. Agentic loops that emit Rust + run `cargo check` + iterate are among the most reliable autoformalization-to-deployment patterns observed in production.
- **Category 9 (Evaluation / Clinical Validation)**: `proptest` (the Rust QuickCheck analogue, Andrew Gallant et al.) supports property-based testing with shrinking. `cargo nextest` provides parallel test execution; coverage via `cargo llvm-cov`; mutation testing via `cargo mutants`. **Loom** (David Tolnay, Carl Lerche) provides exhaustive concurrent-execution-permutation testing for `unsafe` synchronisation primitives — directly relevant to multi-threaded CDS rule engines.
- **Category 10 (Regulatory Compliance / Assurance)**: Ferrocene's IEC 62304 Class C qualification (January 2025) provides toolchain-level evidence that an IEC 62304-compliant medical-device software lifecycle can use Rust as the implementation language. CISA's January 1, 2026 memory-safety-roadmap deadline directly affects any CDS vendor on the Secure-by-Design Pledge; the roadmap document is a *regulatory* artefact, not a developer artefact. SBOM emission via `cargo sbom` / `cyclonedx-rust-cargo`; AIBOM via `cargo bom`; provenance under the SLSA framework. The reproducibility argument from §13 carries through: a Rust binary built with `--release -Ccodegen-units=1 -Cembed-bitcode=no` and a pinned `Cargo.lock` is bit-for-bit reproducible across machines, directly supporting IEC 62304 §8 configuration-management evidence and FDA SaMD pre-market documentation.

### Limitations/Known issues
- **`unsafe` is real and necessary**: roughly 25–30% of Rust crates contain some `unsafe` (Astrauskas et al., *How Do Programmers Use Unsafe Rust?*, OOPSLA 2020). The standard library, FFI bindings, lock-free data structures, OS abstractions, and high-performance parsers all depend on it. Memory safety in Rust is therefore a *layered* claim: safe Rust is sound modulo correct encapsulation by underlying `unsafe`. Tools (Miri, MIRAI, Stacked/Tree Borrows analysis, RustSec advisories) are partial mitigations. The Xu, Chen, Yin, Hao et al. 2020 study *Memory-Safety Challenge Considered Solved? An In-Depth Study with All Rust CVEs* found that essentially every Rust memory-safety CVE traced to an `unsafe` block — confirming both the threat model and that the encapsulation strategy is operationally working.
- **Borrow checker is conservative**: legitimate patterns (self-referential structs, intrusive data structures, certain graph encodings) require `Rc<RefCell<T>>` (runtime borrow check) or `unsafe`. Tree Borrows reduces the false-rejection rate by ~54% over Stacked Borrows, but the boundary remains audible to working programmers. The next-generation Polonius borrow checker is in long-term development.
- **Compile times**: large Rust crate graphs (>500 dependencies) compile slowly; release-build LTO is slow. Mitigations: `sccache`, the `mold` linker, `cargo-chef` for Docker layering. Not a regulatory issue but a developer-velocity issue that affects iteration on long-running clinical rule sets.
- **Ecosystem maturity gaps for clinical**: the Rust FHIR ecosystem is younger than HAPI FHIR (Java); CQL and ELM evaluators in Rust are pre-production; SMART-on-FHIR client libraries are limited. Incremental migration via FFI is feasible but adds operational complexity (the same point as §13).
- **Hiring pool in Japan**: Rust hiring is easier than Haskell or Lean hiring in Japan (Tokyo Rust meetup, Rust.Tokyo conference, Woven by Toyota / Preferred Networks recruitment), but smaller than Python or Java pools. Bilingual Rust + clinical-domain engineers are a narrow intersection. SPARK / Ada hiring is globally small; AdaCore's training partially compensates, and Japanese rolling-stock and avionics talent pools overlap but are not directly recruitable into clinical software.
- **Zig is not memory-safe in the CISA sense**: included for completeness — Zig (pre-1.0 as of May 2026) provides excellent C interop, transparent control flow, and runtime safety in debug builds, but has no static borrow checker, no affine type system, and no compile-time data-race freedom. Treating Zig as memory-safe in the CISA roadmap sense would be a documentation error. The same caveat applies to C++ Core Guidelines + lifetime profile, which provides static lifetime *analysis* but cannot enforce a full borrow discipline at compile time; **Project Verona** at Microsoft Research explores adding one to C++ (memory regions + ownership + concurrent safety), but it remains research as of 2026.
- **MC/DC support in the Rust toolchain**: the DO-178C DAL A coverage criterion is in development as a 2026 Rust Project Goal under the Safety-Critical Rust Consortium; until shipped, DAL A airborne/implantable medical-device use of Rust requires Ada/SPARK alongside it or alternative coverage instrumentation. Less relevant for IEC 62304 Class C (which Ferrocene already covers) than for DO-178C-classed implantable devices.
- **Floating-point semantics**: the same caveat as §13 — physiological-threshold rules over IEEE-754 doubles inherit reordering / fast-math hazards. Rust's `f64` arithmetic is IEEE-compliant by default, but `-Ofast`-equivalent flags exist and must be excluded from clinical builds; `rust_decimal` and `bigdecimal` provide decimal alternatives for monetary and dosage arithmetic.
- **Supply-chain attack surface**: crates.io's ~165,000-package ecosystem is a large attack surface. Mitigations: `cargo audit` against RustSec, `cargo vet` for crate-auditing (developed at Mozilla, used at Google), `cargo deny` for policy enforcement, and the in-development crates.io publish-attestation work. Same risk class as PyPI or npm, but with a smaller typical dependency surface. The 2025–2026 malicious-code incidents on crates.io were caught quickly by RustSec; the response time is the relevant metric, not the existence of incidents.

### Training data proxy
- **Rust**: extremely strong LLM coverage. The *Rust Reference*, the *Rustonomicon* (unsafe-code reference), *The Rust Programming Language* (Klabnik & Nichols, 2nd ed. 2023, No Starch; also free at `doc.rust-lang.org/book/`), *Rust for Rustaceans* (Jon Gjengset, 2021, No Starch), *Programming Rust* (Blandy, Orendorff, Tindall, 2nd ed. 2021, O'Reilly), *Zero to Production in Rust* (Luca Palmieri, 2022) are all widely indexed in LLM training data. crates.io documentation auto-generated by `rustdoc` is mirror-distributed on `docs.rs`. GitHub Rust repositories number in the low millions.
- **Ada / SPARK**: smaller corpus but high-quality. *Programming in Ada 2012* (John Barnes, 2014, Cambridge); *Building High Integrity Applications with SPARK* (McCormick & Chapin, 2015, Cambridge); AdaCore's *Learn Ada* (`learn.adacore.com`) online textbook. LLMs are weaker on idiomatic Ada/SPARK than on Rust; in-context exemplars help materially.
- **Swift**: moderate LLM coverage; Apple-canonical documentation (`docs.swift.org`); *The Swift Programming Language* book (free, Apple).
- **LLM coverage rank** (memory-safe-language axis): Rust ≈ Java > C# > Go > Swift > Ada/SPARK > Zig, where Zig is sparse enough to need in-context examples for non-trivial work (similar to F\* / Idris in §13).
- **Benchmarks and corpora**: BigCloneBench (cross-language), HumanEval-X (multi-language), MBPP-Rust (community-contributed), CRUX-Eval (Rust-specific, 2024), and the Verus / Prusti example corpora for verification benchmarks. The RustSec advisory database doubles as a labelled dataset of memory-safety vulnerability patterns suitable for fine-tuning vulnerability-detection models.
- **Conferences and venues**: **RustConf** (annual; canonical), **EuroRust**, **Rust.Tokyo**, **Rust Nation UK**, **Rust India**, **OxidizeConf** (embedded Rust); **HILT** (ACM SIGAda High Integrity Language Technology, annual); academic venues PLDI, POPL, OOPSLA, ICFP, OSDI, SOSP, ASPLOS, SIGCSE all carry Rust-related publication streams. Japan-relevant: PPL (already cited §13).