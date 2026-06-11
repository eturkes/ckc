# Category 5: Automated Reasoning, Constraint Solving, and Collision Detection

## 1. SMT-LIB Encodings with Z3, cvc5, and Bitwuzla

### Purpose
SMT decides first-order formulas over background theories (integers, reals, bit-vectors, arrays, strings, datatypes, UF). Backend decision engine for verification, autoformalization checks, constraint discharge.

### Maintainer/Standards body
SMT-LIB standard: Clark Barrett (Stanford), Pascal Fontaine (Liège), Cesare Tinelli (Iowa). Versions: v2.6 (2021-05-12 reference); v2.7 (initial reference 2025-02-05; clarifications/minor errata r2025-07-07; extended 2026-03-18 with Ints exponentiation operator + new QF_EIA logic), adding algebraic datatype declarations and prenex-polymorphism + higher-order maps theory (HO-Core); v3 in development (full higher-order logic). Solvers: Z3 (Microsoft Research; Leonardo de Moura, Nikolaj Bjørner; 4.16.0, Feb 19 2026); cvc5 (Stanford/Iowa; CVC4 successor, Barrett et al.; 1.3.4, May 7 2026); Bitwuzla (Aina Niemetz, Mathias Preiner; bit-vector specialist, Boolector successor; ongoing 2026 work incl. TACAS 2026 "Bit-Precise Interpolation in Bitwuzla" by Niemetz/Preiner — single-interpolant + inductive interpolation-sequence queries; full QF_BV interpolation, experimental floating-point, partial array/UF).

### Conceptual model
QF or quantified first-order formulas in S-expressions under `set-logic` (`QF_UFLIA`, `AUFLIRA`, `QF_BV`, `QF_FP`, `QF_S`, `QF_DT`, newer `QF_EIA` exponential integer arithmetic). CDCL SAT core (`CDCL(T)` in Z3/cvc5; lemmas-on-demand in Bitwuzla) + theory propagators. Output: sat/unsat/unknown + optional model, unsat core, proof.

### Expressiveness/Semantics
QF_LIA NP-complete; QF_LRA P (simplex); QF_BV NP-complete via bit-blasting. Quantified fragments typically undecidable — heuristic E-matching, MBQI, enumerative instantiation. Strings+LIA undecidable in general, practically solvable for many fragments. QF datatypes decidable; recursive functions via syntactic unfolding.

### Composability/Modularity
`push`/`pop` incremental stacks, `define-fun`, `declare-datatypes`. Theory combination: Nelson–Oppen (disjoint signatures) or model-based. Standard I/O enables solver portfolios.

### Suitability for autoformalization to IR
Excellent LLM-generated IR target: textual, syntactically rigid, abundant training data. Idempotency needs canonical naming, fixed sort declarations, deterministic `set-logic`. Round-trip stability high when theory-stratified; `define-fun` macros + `:named` annotations track provenance.

### Formal verification potential
Z3: own proof format (via `proof=true`). cvc5: LFSC + Alethe proofs (Carcara checker; Coq/Lean reconstruction via SMTCoq, lean-smt). Bitwuzla: DRAT for SAT backbone. Unsat cores via `get-unsat-core` on `(! e :named id)` annotations.

### Tooling/Ecosystem maturity
SMT-COMP annual. 2024: Bitwuzla dominated bit-vector + quantifier-free combined-theory divisions (42/82 division golds); cvc5 dominated strings, datatypes, quantified UFDT/UFDTLIA; Z3++ led nonlinear int/real arithmetic; OpenSMT, Yices2, SMTInterpol competitive in linear arithmetic. 2025 (20th edition, co-located SAT'25; results Aug 11 2025): Bitwuzla continued dominance on BV/FP divisions, cvc5 on strings/datatypes/quantified; benchmark + execution-data archives on Zenodo (records 16887742, 16875980). Bindings: Python (`z3-solver`, `pysmt`, `cvc5-pythonic`), C/C++, Java, OCaml, Rust (`z3.rs`).

### Japan-specific considerations
Takamasa Okudono (NII, ERATO MMSD) co-authored "Mind the Gap: Bit-vector Interpolation recast over Linear Integer Arithmetic" (TACAS 2020). Hasuo group (NII): Z3 backend for hybrid-system safety verification (RSS/GA-RSS). Mizuhito Ogawa lab (JAIST): SMT for non-linear arithmetic. No Japan-originated SMT-COMP-winning solver; heavy domestic use.

### Interoperability
- Within Category 5: backend for MaxSAT (Z3 `optimize`), Apalache (TLA+→Z3), Datalog with constraints (Soufflé→Z3), MiniZinc via OptiMathSAT, SMT-guided e-graph extraction cost functions.
- Category 1: discharges FHIR Clinical Reasoning predicates, DMN decision-table consistency.
- Category 2: encodes Minds-extracted clinical predicates as theory atoms over patient ontology sorts.
- Category 3: SHACL → SMT translation possible for cardinality/datatype facets.
- Category 4: proof-tactic backend for `lean-smt`, Isabelle Sledgehammer, Why3 SMT provers, F* via Z3.

### Limitations/Known issues
Nonlinear real arithmetic decidable but doubly-exponential. Quantifier handling brittle; incompleteness common. Floating-point slow even with SymFPU. String theory implementations diverge across solvers. Proofs large, solver-specific.

### Training data proxy
Very abundant: `Z3Prover/z3` ~12.2k stars (May 2026); `cvc5/cvc5` 1.3k; `bitwuzla/bitwuzla` ~347 (May 2026). Thousands of SMT-LIB benchmarks (~140k in SMT-COMP standard test set). Heavy Stack Overflow presence; standard in automated-reasoning courses.

## 2. SAT/MaxSAT for Minimal Conflict and Repair-Set Search

### Purpose
SAT: propositional satisfiability. MaxSAT: maximize (weighted) satisfied soft clauses under hard clauses. Used for conflict localization (minimum unsat fragment) and repair-set synthesis (smallest constraint set whose removal restores consistency).

### Maintainer/Standards body
Annual SAT Competition (organizers Heule, Iser, Järvisalo, Suda); annual MaxSAT Evaluation. SAT solvers: CaDiCaL, Kissat, Gimsatul, IsaSAT (Armin Biere group, Univ. Freiburg), Glucose (Audemard, Simon), MiniSat (Eén, Sörensson; legacy). MaxSAT: RC2 (Ignatiev, Morgado, Marques-Silva), EvalMaxSAT (Avellaneda), MaxHS (Davies, Bacchus), Open-WBO (Martins, Manquinho), UWrMaxSat (Piotrów). DIMACS CNF/WCNF de facto standards.

### Conceptual model
Propositional CNF; MaxSAT adds weighted soft clauses, minimizes violated weight. Algorithms: core-guided (OLL, MSU3), implicit hitting set (MaxHS), linear search, branch-and-bound (MaxCDCL).

### Expressiveness/Semantics
SAT NP-complete (Cook–Levin). MaxSAT FP^NP[log]-complete; partial-weighted widely used. No quantifiers, no native integers — cardinality/PB encodings (totalizer, sequential counter, sorting networks).

### Composability/Modularity
DIMACS CNF/WCNF universal. Softs tagged with weights or group-MUS labels. Incremental SAT in CaDiCaL via IPASIR-2. Cardinality constraints as separate encoders (PySAT iTotalizer).

### Suitability for autoformalization to IR
Lower-level than SMT; LLMs target SAT indirectly via ASP/MiniZinc/SMT flattening to CNF. Deterministic backend given fixed solver seed; idempotency needs variable-naming convention.

### Formal verification potential
DRAT/LRAT proofs standard (checkers cake_lpr, drat-trim; verified in Coq/Isabelle). Native unsat cores. Verified solvers: IsaSAT (Isabelle/HOL, Mathias Fleury), versat.

### Tooling/Ecosystem maturity
SAT Competition 2024: Kissat (Biere et al., Freiburg) swept gold in SAT, UNSAT, SAT+UNSAT main tracks (kissat-sc2024: PAR-2 = 2788.13, 306 solved of exactly 400 benchmarks per Biere's official Zenodo deposit, DOI 10.5281/zenodo.15095752: "These are the 400 CNF files in DIMACS format from the main track of the SAT competition 2024"). 2025 (results Aug 2025): AE-Kissat-MAB (Ding, Luo, Li et al.) won Main Sequential overall (PAR-2 = 2264.73, 327 solved); Kissat-public, Kissat-VSA 2nd/3rd; CaDiCaL-SC2025 (Biere group) won Main Sequential UNSAT (PAR-2 = 2327.00, 161 solved); MallobSat (Schreiber, Rigi-Luperti, Biere; KIT/Freiburg) won Parallel Track (PAR-2 = 394.56, 337 solved) — its first parallel-track gold. MaxSAT Evaluation 2024: SPB-MaxSAT-c-Band + SPB-MaxSAT-c-FPS swept all four anytime categories; EvalMaxSAT, UWrMaxSat strong in exact track (SCIP+CDCL portfolios). Bindings: PySAT (Ignatiev), C/C++/Rust.

### Japan-specific considerations
Miyuki Koshimura (Kyushu Univ.): QMaxSat, multiple MaxSAT Evaluation podiums. Hidetomo Nabeshima (Yamanashi): SAT solver development. Naoyuki Tamura, Mutsunori Banbara (Kobe/Nagoya): Sugar CSP-to-SAT translator, multiple SAT-based competition wins (CSP Competition 2008 1st; MaxSAT 2012 2nd, Partial Weighted Max-SAT crafted). Takehide Soh (Kobe): sCOP/Scarab.

### Interoperability
- Backbone of MaxSAT, MUS enumeration (§3), Datalog#-modulo-theories, MiniZinc backends (PicatSAT), CP-SAT (OR-Tools layers CP over SAT), ASP (clasp wraps CDCL SAT engine).
- Category 1: repair-set search over rule sets (DMN tables, clinical pathways) maps directly to MaxSAT.
- Category 3: OWL EL axiom pinpointing reduces to Horn group-MUS (HGMUS).
- Category 4: DRAT proofs feed Isabelle/Coq-verified checkers.

### Limitations/Known issues
No first-order quantification, real arithmetic, or arrays. PB/cardinality encodings blow up CNF size. Runtime highly instance-sensitive; no worst-case bounds beyond NP/FP^NP.

### Training data proxy
Massive. MiniSat canonical pedagogical reference. GitHub: kissat ~629 stars, CaDiCaL ~553, MiniSat clones thousands (May 2026). Annual competition reports (CEUR-WS, Univ. Helsinki Tech Reports).

## 3. MUS/MCS Enumeration and UNSAT-Core Extraction

### Purpose
Given an unsat constraint set, enumerate Minimal Unsatisfiable Subsets (MUS — minimal cores explaining infeasibility) and Minimal Correction Subsets (MCS — minimal deletions restoring satisfiability). Foundation for conflict explanation and contradiction localization in guideline corpora.

### Maintainer/Standards body
No formal standards body. Algorithms: CAMUS (Liffiton & Sakallah), MARCO (Liffiton, Previti, Malik, Marques-Silva), DAA (Bailey, Stuckey), eMUS (Previti, Marques-Silva), MUSer2 (Belov, Marques-Silva), HGMUS (Ignatiev et al.; Horn/EL). Implementations: MARCO (Python, Liffiton, https://sun.iwu.edu/~mliffito/marco/), PySAT modules, FLINT.

### Conceptual model
Hitting-set duality (Reiter 1987, Birnbaum/Lozinskii): MUSes/MCSes are dual minimal hypergraph transversals. MCS = complement of Maximal Satisfiable Subset (MSS). Algorithms: explicit dualization (CAMUS), implicit dualization with seed-and-shrink + "map" formula tracking explored regions (MARCO/eMUS), IHS (implicit hitting set, MaxHS).

### Expressiveness/Semantics
Any monotone constraint formalism with sat oracle: SAT, SMT, CSP, Horn (EL axiom pinpointing). Worst-case exponentially many MUSes/MCSes. Group-MUS tags constraints with group labels for minimal-set-of-groups cores — exactly the right abstraction for "which clinical-guideline rules together contradict?"

### Composability/Modularity
MARCO solver-agnostic: SAT oracle + shrink procedure. Group labels model per-guideline-section provenance. Works with Z3 (`(set-option :produce-unsat-cores true)`) and PicoMUS.

### Suitability for autoformalization to IR
Natural fit when IR clauses carry stable identifiers — enumeration returns identifier sets, directly machine-readable. Idempotency needs solver determinism + canonical lexicographic MUS preference (else enumeration order varies).

### Formal verification potential
MUS verifiable by unsat proof of subset + sat proof of each proper sub-subset (expensive but certifiable). MCS verifiable by sat witness + per-element regression. Marques-Silva 2020 surveys formal duality proofs.

### Tooling/Ecosystem maturity
Mature in SAT/SMT: Z3 `get-unsat-core`, cvc5 `--produce-unsat-cores`. Standalone: MARCO, MUSTool, optimal-MUS/OCUS (Gamba, Bogaerts, Guns 2023). HGMUS used for SNOMED-CT debugging benchmarks.

### Japan-specific considerations
Hidetomo Nabeshima (Yamanashi): minimal-model generation foundations relevant to ASP/MUS computation (Koshimura, Nabeshima, Fujita, Hasegawa, FTP 2009). Katsumi Inoue (NII): model-generation theorem proving underlying ASP MUS-style reasoning. No internationally distributed Japan-originated MUS enumerator identified.

### Interoperability
- Direct consumer of any §1/§2 solver.
- Bridges to ASP for non-monotonic conflict analysis (MUS over choice/weak-constraint encodings).
- Category 1: localizes contradictions between FHIR Clinical Reasoning artifacts or BPMN+CDS rule sets.
- Category 3: axiom pinpointing in OWL EL ontologies — standard application (SNOMED-CT debugging).
- Category 4: MUS certificates reconstructible in Coq/Isabelle.

### Limitations/Known issues
Worst-case exponential. Map-formula memory bloat on large constraint sets. Selecting which MUS to report (preferred: smallest, lexicographically minimal, weighted minimum) is itself an optimization problem.

### Training data proxy
Moderate. Active at CP, IJCAI, AAAI, SAT. MARCO modest GitHub presence; less Stack Overflow exposure than Z3. Survey: Marques-Silva & Mencía 2020.

## 4. Datalog / Datalog± / RDFox-Style Rule Materialization

### Purpose
Declarative deductive-database rule language, forward-chaining: materializes least fixed point of Horn-like rules over base relations. Ontological reasoning, program analysis, graph analytics.

### Maintainer/Standards body
No single body; ISO Datalog standardization dormant. Engines: Soufflé (souffle-lang.github.io; Scholz, Jordan; CAV 2016), RDFox (Oxford Semantic Technologies / Boris Motik et al.; commercial), LogicBlox (defunct), ddlog (VMware/Differential Datalog), DLV (Calabria), Vadalog (Datalog±, Oxford/TU Wien; Gottlob, Bellomarini). Datalog± formalized by Calì, Gottlob, Lukasiewicz (2009+).

### Conceptual model
Function-free Horn clauses (head ← body₁,…,bodyₙ) over relational facts; semi-naïve evaluation via delta relations. Stratified negation, well-founded semantics. Datalog± adds existential rules (TGDs), EGDs, keys.

### Expressiveness/Semantics
Pure Datalog: P-complete data complexity, EXPTIME combined, decidable. Datalog± existentials undecidable in general; decidable fragments: guarded TGDs (2-EXPTIME-complete), sticky, weakly-acyclic, warded. RDFox RL profile + Datalog corresponds to OWL 2 RL materialization.

### Composability/Modularity
Modular by predicate. Soufflé: components, ADTs, records. RDFox: SWRL-like + SPARQL-CONSTRUCT rules, incremental maintenance under fact addition/deletion (DRed-based). Facts as CSV/TSV or RDF triples; rules `.dl` (Soufflé) / `dlog` (RDFox).

### Suitability for autoformalization to IR
Excellent — very common LLM target (clean declarative syntax). Semi-naïve evaluation deterministic → canonical fixpoint → strong idempotency for the materialization layer (variation only in derivation order, not final extension). Datalog± existentials enable "unspecified consequence" semantics in clinical rules.

### Formal verification potential
Semi-naïve soundness/completeness classical (Abiteboul, Hull, Vianu). No native proof certificate; proof trees / why-provenance computable (Soufflé `--provenance` returns derivation tree per fact). Verified Datalog: Bembenek et al., formalized in Coq.

### Tooling/Ecosystem maturity
Soufflé compiles to parallel C++ (Doop pointer analysis, security analyzers). RDFox handles billion-triple KGs in main memory. Vadalog optimized for Datalog±. dlv2 ASP-Core-2 compliant.

### Japan-specific considerations
Less prominent than ASP. Naoki Kobayashi (Univ. of Tokyo): higher-order model checking, CHC/Horn-clause solving — Datalog-adjacent. Makoto Tatsuta (NII): separation logic, static analysis. No widely-known Japan-originated industrial Datalog product identified.

### Interoperability
- Bridges to e-graph relational engines (egglog = Datalog + equality saturation — §9).
- Drives RL-profile OWL reasoners (§5).
- Category 1: openEHR archetype querying via AQL → Datalog; FHIR Path/CQL reduce to Datalog for non-recursive queries.
- Category 3: RDFox primary OWL 2 RL + SHACL companion reasoner.
- Category 4: provenance trees exportable to Lean/Isabelle.

### Limitations/Known issues
Datalog± existentials need careful fragment choice for decidability. Aggregation semantics non-uniform across engines; recursive aggregation lacks canonical fixpoint without lattice semantics. Negation stratified-only; full negation needs well-founded or stable-model semantics (→ ASP).

### Training data proxy
Moderate. `souffle-lang/souffle` 1.1k stars, 240 forks (org repos page last updated May 4, 2026); RDFox closed-source (commercial, founded Motik/Horrocks/Nenov; product since 2017). Active at PLDI, VLDB, PODS, SIGMOD; academic compilers/PL use.

## 5. OWL Reasoning with ELK, HermiT, and RDFox

### Purpose
Entailment, classification (subsumption hierarchy), realization (instance types), consistency for OWL 2 ontologies. Terminological reasoning over biomedical ontologies (SNOMED-CT, ICD-11, NCIt).

### Maintainer/Standards body
W3C OWL 2 (2009/2012). Reasoners: ELK (Yevgeny Kazakov, Markus Krötzsch, František Simančík; Ulm/Manchester/Oxford), HermiT (Birte Glimm, Ian Horrocks, Boris Motik; Oxford/Ulm), RDFox (Oxford Semantic Technologies), Pellet (legacy Clark & Parsia/Stardog), FaCT++ (legacy Manchester).

### Conceptual model
DL ABox+TBox+RBox knowledge bases. Profiles:
- OWL 2 EL: P-time classification (existential restrictions, intersection; no negation/universals); ELK uses consequence-based/completion-rule procedure.
- OWL 2 QL: AC0 query answering (DL-Lite family).
- OWL 2 RL: P-time via Datalog/RDFox materialization.
- OWL 2 DL (full): SROIQ(D), N2EXPTIME-complete; HermiT hypertableau calculus.

### Expressiveness/Semantics
SROIQ(D): object/data properties, role hierarchies, inverse, transitive, functional, cardinality, nominals, datatypes. ELK incomplete for non-EL constructs but fastest classifier for EL-shaped clinical ontologies (SNOMED-CT classifies in seconds).

### Composability/Modularity
OWL API (Java) de facto reasoner interface. Locality-based module extraction splits large ontologies. MORe combines ELK + HermiT/RDFox, delegating EL fragments to ELK. `owl:imports`; alignment via SKOS/EQUI.

### Suitability for autoformalization to IR
Strong. Functional-Style and Manchester syntaxes both LLM-friendly. Classification produces canonical class hierarchy → idempotent given fixed ontology. Realization deterministic for OWL EL.

### Formal verification potential
Per-profile classification soundness/completeness proved. Axiom pinpointing (justifications) via OWL API `BlackBoxExplanation` or HGMUS over EL Horn encodings. RDFox produces derivation traces.

### Tooling/Ecosystem maturity
Protégé standard editor. ROBOT (OBO Foundry) automates reasoning workflows. ORE workshop competition. ELK 0.6.0 (liveontologies/elk-reasoner; OWL API 4.x/5.x) widely used in biomedical OBO Foundry pipelines.

### Japan-specific considerations
Riichiro Mizoguchi (formerly Osaka, now JAIST) + Kouji Kozaki: Hozo ontology editor, role theory. Hideaki Takeda (NII): Semantic Web, OWL-Full reasoning (Koide & Takeda, ASWC 2006). Takahira Yamaguchi (Keio): DODDLE-OWL. Takeshi Imai et al. (Univ. of Tokyo Medical School): Japan Journal of Medical Informatics radiological-diagnosis ontology (DOI 10.14948/jami.25.395). JAMI = Japan Association for Medical Informatics; a Japan-specific "JaMI"-branded medical ontology was not located in primary sources.

### Interoperability
- RDFox bridges §4 (Datalog) and §5 (OWL RL).
- ELK pinpointing = canonical group-MUS application (§3).
- Category 1: FHIR ValueSet expansion uses OWL/SKOS reasoning; SNOMED-CT post-coordination requires EL classification.
- Category 2: MEDIS / Minds terminologies normalized via OWL alignment.
- Category 3: SHACL adds closed-world constraints atop OWL open-world; both consumed.
- Category 4: Coq/Isabelle DL formalizations (e.g., Beckert, Schmitt) reconstruct EL proofs.

### Limitations/Known issues
OWL 2 DL reasoners blow up on highly-quantified axioms or large nominal sets. ELK incomplete outside EL. Open-world semantics confuses clinical-rule authors expecting closed-world (→ SHACL or Datalog overlay). No native aggregation, arithmetic, temporal reasoning.

### Training data proxy
Abundant. W3C OWL 2 documents heavily indexed; Protégé tutorials proliferate. ELK GitHub modest; HermiT well-cited. SNOMED-CT EL classification = canonical OWL EL benchmark.

## 6. ASP with Clingo for Defaults, Exceptions, and Nonmonotonic Rules

### Purpose
Combinatorial search + nonmonotonic KR as logic programs whose stable models are solutions. First-class defaults, exceptions, preferences, abduction — directly relevant to clinical guideline modeling (default treatment / exception per comorbidity).

### Maintainer/Standards body
ASP-Core-2 standard (Calimeri, Faber, Gebser et al., 2020). Systems: Clingo/gringo/clasp (Potassco, Univ. of Potsdam; Torsten Schaub, Martin Gebser, Roland Kaminski, Benjamin Kaufmann); DLV2 (Calabria; Leone, Ricca); WASP. ASP Competition biennial (LPNMR/KR co-located).

### Conceptual model
Disjunctive logic programs under stable model semantics (Gelfond & Lifschitz 1988). Normal/choice/aggregate/weak-constraint heads; negation-as-failure with stable-model fixpoint. Weak constraints `:~ body. [w@l,tuple]` express optimization (lexicographic by priority level). Extensions: clingo[DL] difference logic, clingo[LP] linear constraints, clingcon CSP, ASP-modulo-theories.

### Expressiveness/Semantics
Disjunctive ASP Σ₂ᴾ-complete; normal programs NP-complete; weak constraints add optimization (FPΣ₂ᴾ[log]). Strong negation `-p` vs default `not p`. Multi-shot solving via Python/Lua API.

### Composability/Modularity
`#program` named subprograms. Multi-shot (Clingo 4+): incremental grounding, reactive reasoning. Reification of programs as facts enables meta-programming.

### Suitability for autoformalization to IR
Excellent for clinical rules with exceptions; LLMs produce ASP fluently. Semantic convergence requires fixed predicate signatures + stable rule naming; multiple optimal answer sets handled via deterministic enumeration order + `#show` projection. Gelfond-style "knowledge pattern" methodology (Chen et al.) encoded entire chronic-disease guidelines (CHF, ~80-page guideline) in ASP.

### Formal verification potential
Stable-model semantics: clean fixpoint characterization (Gelfond–Lifschitz reduct). clasp CDCL answer-set search soundness/completeness proven. No standard proof certificate; unsat cores + weak-constraint optima certifiable. ILASP (Mark Law): inductive learning of ASP programs.

### Tooling/Ecosystem maturity
ASP Competition since 2007; Potassco dominant since ~2011. potassco/clingo active development through April 2026. Python `clingo`, telingo (temporal), eclingo (epistemic), clinguin (UI). DLV2 strong in disjunctive optimization.

### Japan-specific considerations
Highly active. Katsumi Inoue (NII): Learning from Interpretation Transition (LFIT), inductive ASP, abductive logic programming; cited as foundational by ILASP authors. Chiaki Sakama (Wakayama): brave/cautious induction, ASP semantics. Mutsunori Banbara (Nagoya), Takehide Soh (Kobe), Naoyuki Tamura (Kobe Emeritus): SAT-based + ASP-based hybrid solvers (clingcon, aspartame, Sugar). Hidetomo Nabeshima (Yamanashi). Recurring joint Potassco (Schaub) papers: Hamiltonian-cycle reconfiguration, scheduling.

### Interoperability
- clingo[DL/LP] embeds difference/linear arithmetic (→ §1, §7).
- ASP encodings of MUS/MCS enumeration available (§3).
- Category 1: ASP encodings of GLARE and Asbru exist (Spiotta, Terenziani & Theseider Dupré, "Temporal Conformance Analysis and Explanation of Clinical Guidelines Execution: An Answer Set Programming Approach", IEEE TKDE 29(11):2567–2580, 2017); ASP-based conformance analysis of guideline execution traces.
- Category 2: PMDA exception handling (e.g., drug-drug interaction exceptions) fits weak constraints.
- Category 3: ASP simulates Datalog±/OWL RL with negation-as-failure closed-world overlays.
- Category 4: limited Lean/Isabelle ASP semantics formalizations; ILASP rules verifiable by post-hoc proof.

### Limitations/Known issues
Grounding bottleneck — variables grounded before solving; large instances blow up. Mitigations: lazy grounding (alpha, Spasic), tight programs. Nonmonotonicity confuses classical-logic users. No native real arithmetic without extensions.

### Training data proxy
Strong academic AI literature; less commercial Stack Overflow presence than SMT. `potassco/clingo` ~785 stars (May 2026). Annual competition reports. Active medical-AI literature (heart-failure ASP advisory system, Genesereth/Gelfond textbook examples).

## 7. Constraint Programming with MiniZinc and OR-Tools CP-SAT

### Purpose
High-level modeling language (MiniZinc) targeting heterogeneous solvers (CP, MIP, SAT, SMT, LCG) for scheduling, resource allocation, configuration. Planning clinical care pathways with global constraints (resources, precedence, comorbidity exclusions).

### Maintainer/Standards body
MiniZinc — Monash (Peter J. Stuckey, Mark Wallace, Nick Nethercote, Jip Dekker). FlatZinc = solver-agnostic intermediate language. Annual MiniZinc Challenge (CP conference). OR-Tools CP-SAT (Google; Laurent Perron). Gecode (Schulte, Lagerkvist, Tack). Chuffed (Stuckey/Chu, lazy clause generation). Choco (École des Mines de Nantes).

### Conceptual model
Finite-domain variables (integers, sets, booleans, restricted floats). Global constraints (`alldifferent`, `cumulative`, `circuit`, `regular`, `table`) with custom propagators. LCG combines CP propagation with SAT learned clauses (Chuffed, CP-SAT, Pumpkin). Solver classes: pure CP (Gecode), LCG (Chuffed, CP-SAT, Pumpkin), MIP (Gurobi, CPLEX, HiGHS), SMT (OptiMathSAT), local search (Yuck, OR-Tools CP-SAT LS).

### Expressiveness/Semantics
Finite-domain CP NP-complete (decidable); optimization complete in FPΣ₁ᴾ-style budget. User-defined predicates, sets, arrays, optionality. Global constraint catalog (Beldiceanu et al.) ~400 constraints.

### Composability/Modularity
`include` directives; `.mzn` model + `.dzn` data separation; FlatZinc enables solver swapping; per-solver-tuned predicate libraries (`globals.mzn`).

### Suitability for autoformalization to IR
Strong target; syntax close to mathematical notation; LLMs produce MiniZinc reasonably well (large IJCAI/CP corpus). Result deterministic for proven-optimal; non-unique solutions need deterministic search annotation for idempotency.

### Formal verification potential
Solvers typically not proof-producing (exception: Pumpkin and some LCG solvers emit DRAT-style certificates). MIP solvers (SCIP) produce VIPR certificates. Most CP solvers validated empirically via MiniZinc Challenge benchmarks.

### Tooling/Ecosystem maturity
Challenge 2024: Atlantis best free-search (PAR 14.00) ahead of yuck-free 709.50, optimathsat 885, gecode-fd 938, or_tools-ls-free 1095, chuffed-free 1572; Choco gold in fixed search; Atlantis gold in local search. Challenge 2025 (announced CP2025, Glasgow, Aug 10–15 2025): Google OR-Tools CP-SAT swept medals across major categories, consistent with post-2017 FlatZinc dominance. `minizinc-python`, MiniZinc IDE.

### Japan-specific considerations
Naoyuki Tamura, Mutsunori Banbara, Takehide Soh, Tomoya Tanjo: Sugar/Azucar (SAT-based CSP); won 4 of 10 categories at 3rd International CSP Solver Competition 2008. Active in scheduling/timetabling. Hidetomo Nabeshima (Yamanashi). Hokkaido/JAIST contribute occasionally; no Japan-originated MiniZinc Challenge winner identified.

### Interoperability
- CP-SAT internally compiles to SAT (§2); OptiMathSAT to SMT (§1).
- ASP-modulo-CSP via clingcon (§6); aspartame translates XCSP/Sugar facts to ASP.
- Category 1: BPMN/ePath scheduling + resource constraints map directly.
- Category 4: TLA+/Apalache symbolic execution can offload combinatorial subproblems to CP solvers.

### Limitations/Known issues
No real-valued reasoning beyond restricted float domains; no quantifiers. Performance highly solver- and encoding-dependent. Search annotations are imperative artifacts breaking pure declarativity.

### Training data proxy
Moderate. MiniZinc handbook well-indexed. CP-SAT heavy Google-blog + Stack Overflow presence (`google/or-tools` ~13.5k stars, May 2026). Annual Challenge reports.

## 8. TLA+ TLC / Apalache Model Checking

### Purpose
Specify/verify behavioral properties (safety, liveness, refinement) of concurrent and distributed systems. Distributed clinical-data pipelines, eventual-consistency reasoning, audit-trail correctness.

### Maintainer/Standards body
TLA+ by Leslie Lamport (Microsoft Research, retired); language + TLC maintained at tlaplus/tlaplus. Apalache by Igor Konnov, Jure Kukovec, Thomas Pani (formerly Informal Systems, originally TU Wien); funded by current maintainers/contributors post Informal Systems wind-down, repository active at apalache-mc/apalache through 2026. PRISM/Storm + QComp tools partly bridge but operate on probabilistic models.

### Conceptual model
Temporal logic of actions: state machine = Init predicate + Next relation (disjunction of actions). Properties in temporal logic with `□`, `◇`, fairness. TLC explores reachable states explicitly (BFS), checking invariants + temporal properties. Apalache translates a bounded fragment to SMT (Z3) for symbolic bounded model checking + inductive-invariant checking.

### Expressiveness/Semantics
Set-theoretic + first-order + temporal (Linear Time TLA). Undecidable in general; TLC explicit-state with finite bounds; Apalache decidable up to bounded execution length k. Refinement via implementation+specification implication.

### Composability/Modularity
Modules with `EXTENDS`/`INSTANCE`. Refinement mappings link abstract/concrete specs. PlusCal (also Lamport) transpiles to TLA+ for sequential-imperative style.

### Suitability for autoformalization to IR
Mathematical, LLM-friendly syntax but model-checker idiosyncratic (CONSTANT/VARIABLE separation, action enabling). Idempotency strong with named invariants + configured constants in TLC `.cfg`. Apalache requires type annotations via `Apalache!Snapshot` types.

### Formal verification potential
TLC: counterexample traces. Apalache: counterexample + SMT-extracted state sequences. TLAPS (TLA+ Proof System): interactive proofs discharged to Isabelle, Z3, Zenon. No DRAT-like universal certificate.

### Tooling/Ecosystem maturity
TLA+ Toolbox IDE, VSCode plugin. Industry-mature (AWS, Azure Cosmos DB, MongoDB, Oracle published TLA+ specs). Apalache via Docker, JAR; integrated with Quint (engineer-friendly frontend). 2024 ETAPS Test-of-Time Tool Award to PRISM (related); Apalache maintainer-funded development continues through 2026.

### Japan-specific considerations
Limited Japanese TLA+ literature. Hasuo group (NII): category-theoretic/hybrid-systems formal methods instead. JAIST Ogata group: OTS/CafeOBJ, Maude for distributed-system verification (Raft, Paxos) — functionally adjacent. No primary public document confirming production TLA+ at NTT/Hitachi/NEC.

### Interoperability
- Apalache → Z3 (§1) for symbolic checking.
- TLA+ can drive ASP/CSP for combinatorial subproblems via external integration.
- Category 1: specifies workflow refinement between BPMN/ePath models and executable implementations.
- Category 4: TLAPS bridges to Isabelle; Quint may also target Lean/Rocq in future. Refinement is the natural complement to dependent-type proofs in Lean.

### Limitations/Known issues
TLC explicit-state blowup on large state spaces. Apalache limited to bounded model checking + inductive invariants; some TLA+ features unsupported (e.g., some recursive operator forms). Liveness checking expensive. No native probabilistic extension (use PRISM/Storm).

### Training data proxy
Moderate-to-high. Lamport's books, Murat Demirbas lectures, Hillel Wayne's "Learn TLA+" widely circulated. Subreddit, Zulip community. `apalache-mc/apalache` 557 stars (May 2026).

## 9. E-Graphs and Equality Saturation for IR Canonicalization

### Purpose
Compactly represent equivalence classes of terms (e-classes) over rewrite rules; non-destructive rewriting + optimal-term extraction. Foundation for IR canonicalization: multiple semantically equivalent guideline formalizations converge to a canonical representative.

### Maintainer/Standards body
No standards body. `egg` (Rust; Max Willsey, Chandrakana Nandi, Yisu Remy Wang, Oliver Flatt, Zachary Tatlock, Pavel Panchekha; POPL 2021). `egglog` (PLDI 2023; Yihong Zhang, Wang, Flatt, Cao, Zucker, Rosenthal, Tatlock, Willsey) — Datalog + equality saturation hybrid. Earlier: SimplifyCC e-graphs (Nelson 1980), Z3 congruence-closure module.

### Conceptual model
E-graph = union-find over e-classes + congruence-closed e-nodes. Equality saturation: apply rewrites (LHS → RHS) until no new equalities or budget exhausted, then extract minimal-cost term per cost function. E-class analyses propagate semantic lattice values (constants, types) bottom-up. egglog adds relational pattern matching + Datalog-style rules.

### Expressiveness/Semantics
Equational rewriting over many-sorted algebras. Undecidable for unrestricted rules (word problem); saturation bounded by iteration limit or e-graph size. Sound but incomplete (extraction returns *some* equivalent term; *the* minimum only at saturation).

### Composability/Modularity
egg `Language` + `Analysis` + `Rewrite` traits for domain extensions. E-graphs intersect via shared canonicalization. egglog modules compose via Datalog-style rule sets.

### Suitability for autoformalization to IR
Excellent for the convergence/idempotency goal: saturation computes a canonical congruence-closed representative — semantically equivalent but syntactically distinct LLM autoformalization outputs unify in the e-graph. Exactly the required "semantic convergence" property.

### Formal verification potential
Union-find + rebuilding proven correct (Willsey et al. POPL 2021). egg supports proof extraction (rewrite chains as derivation sequences); no standardized proof format. egglog inherits Datalog provenance.

### Tooling/Ecosystem maturity
egg ~1.7k stars (egraphs-good/egg, May 2026); adopted in compiler/PL research (Cranelift, Herbie floating-point synthesis, SPORES tensor algebra, TASO/Tensat ML graph optimization, DialEgg for MLIR). egglog active (~730 stars, egraphs-good/egglog); competing implementations eggcc, slotted-egraphs.

### Japan-specific considerations
Takahito Aoto (Niigata), Yoshihito Toyama (Tohoku/Niigata): term rewriting, confluence, modularity (Aoto, Nishida, Schöpf "Equational Theories and Validity for Logically Constrained Term Rewriting", FSCD 2024 / arXiv:2405.01174; ACP — Automated Confluence Prover by Aoto, Yoshida, Toyama). Kentaro Kikuchi (Tohoku): nominal confluence tools (Aoto & Kikuchi, "Nominal Confluence Tool", IJCAR 2016, LNCS 9706, pp. 173–182). No direct Japanese egg/egglog adoption; Japanese term-rewriting community strong on confluence/termination, not e-graph applications.

### Interoperability
- egglog unifies Datalog (§4) + equality saturation.
- Extraction can use SMT (§1) cost functions or ASP (§6) preferred-term selection.
- Category 1: canonicalizes alternative encodings of the same clinical rule (FHIR vs. CQL vs. ePath).
- Category 3: ontology alignment as equality rewrites; unifies SNOMED/ICD/JJ1017 codes for identical concepts.
- Category 4: Lean/Rocq tactic backends inspired by e-graphs (e.g., `Mathlib`'s `simp` semantics); egg-style normalization in Cranelift and the Rust compiler.

### Limitations/Known issues
Size explosion with prolific rules. No native binders (workarounds: slotted e-graphs, nominal). Contextual equality saturation (conditional rewrites under path conditions) only partially supported. Extraction NP-hard for general cost functions.

### Training data proxy
Growing rapidly post-2021. egg POPL paper ~750+ citations. EGRAPHS workshop (2024, 2025). Little Stack Overflow; primarily research Rust code.

## 10. PRISM / Storm for Probabilistic Policy or Risk Models

### Purpose
Probabilistic model checking: quantitative properties of stochastic systems (DTMC, CTMC, MDP, Markov automata, POMDPs, stochastic games) against probabilistic temporal logic (PCTL, CSL, PCTL*). Medical risk modeling, treatment-policy synthesis under uncertainty, screening-strategy evaluation.

### Maintainer/Standards body
PRISM — Oxford/Birmingham (Marta Kwiatkowska, Gethin Norman, David Parker). PRISM 4.10 (January 2026), patch 4.10.1 (April 2, 2026); PRISM-games 3.2.2 with UMB (Unified Markov Binary) format. ETAPS 2024 Test-of-Time Tool Award. Storm — RWTH Aachen (Joost-Pieter Katoen, Christian Dehnert, Sebastian Junges, Matthias Volk, Tim Quatmann). JANI common interchange format (Budde et al.). QComp competition.

### Conceptual model
DTMC: discrete-time transition probability matrix. CTMC: exponential rate matrix. MDP: nondeterministic + probabilistic actions, policy/scheduler resolves nondeterminism. Markov automata add exponentially distributed delays. PCTL: P_op p [φ U≤k ψ]; CSL adds continuous-time bounds; PCTL* nested path operators. Engines: value iteration, interval iteration, Gauss-Seidel, policy iteration, topological decomposition, BDD/MTBDD symbolic.

### Expressiveness/Semantics
Probabilistic reachability on finite MDPs in P (via LP). PCTL model checking PSPACE-complete. ~10⁷–10⁹ state spaces with symbolic engines. POMDPs undecidable for many properties; Storm heuristic POMDP support. Parametric Markov models via parameter synthesis (PARAM, Storm-pars).

### Composability/Modularity
PRISM language: modules with synchronization labels. JANI universal exchange. Storm inputs: PRISM, JANI, GSPN, DFT, explicit, probabilistic programs. Properties in separate `.pctl` files.

### Suitability for autoformalization to IR
Concrete, parameterizable modeling language. LLMs can produce PRISM/JANI from clinical risk descriptions (transition probabilities, reward structures). Idempotency needs canonical state-variable ordering; symbolic-engine ordering affects performance, not result.

### Formal verification potential
Numerical results have controllable error bounds (interval iteration gives certified intervals). Counterexamples via SMT or MILP (Storm). No DRAT-level certificate; witness paths exportable.

### Tooling/Ecosystem maturity
PRISM ~30+ years mature; Storm 8+ years, growing rapidly. QComp benchmark suite. stormpy Python API. PRISM-games handles stochastic two-player games.

### Japan-specific considerations
Ichiro Hasuo (NII Group MMM): compositional probabilistic model checking with string diagrams of MDPs (Watanabe, Eberhart, Asada, Hasuo, CAV 2023; Pareto curves TACAS 2024); category-theoretic foundations. Kohei Suenaga (Kyoto): probabilistic programs, hybrid-system verification (Hasuo, Oyabu, Eberhart, Suenaga, Cho, Katsumata, JLAMP Jan. 2024). Masaki Waga (Kyoto): probabilistic black-box checking via active MDP learning. Taisuke Sato (Tokyo Tech emeritus / AIST) developed a *different* tool also named PRISM — probabilistic logic-programming language, distinct from the Kwiatkowska/Norman/Parker checker.

### Interoperability
- Apalache (§8) and Storm both consume Z3 (§1).
- Reward-structure synthesis feeds MaxSAT (§2) or CP-SAT (§7) policy optimization.
- Category 1: CDS-Hooks risk-score calculation backed by probabilistic models; ePath probabilistic variants.
- Category 2: MDPs over PMDA drug-event statistics for adverse-event prediction.
- Category 4: TLA+/Apalache for safety; PRISM/Storm for quantitative. Probabilistic-program logics formalized in Coq/Isabelle (Iris-MarkovChains, Coquelicot stochastic).

### Limitations/Known issues
State-space explosion for high-dimensional patient state. Continuous-state stochastic hybrid systems unsupported natively (use ProbReach, Modest). POMDPs intractable for exact verification. PCTL alone cannot express cross-policy expected-utility comparisons (needs reward extensions).

### Training data proxy
Moderate. PRISM tutorial well-indexed (prismmodelchecker.org); Storm documentation thorough; less Stack Overflow than SMT/SAT. QComp reports. Active at CAV, TACAS, QEST.

## 11. Prolog and s(CASP) for Goal-Directed Clinical-Rule Execution with Justification Trees

### Purpose
Logic programming with SLD/SLDNF resolution; s(CASP) extends to goal-directed predicate ASP with constructive negation, abduction, human-readable justification trees. Top-down query-driven complement to bottom-up Datalog (§4) and grounded ASP (§6): same clinical rule base both materialised (batch conformance) and queried goal-directed (patient-specific point-of-care "why?" explanations).

### Maintainer/Standards body
ISO/IEC 13211-1:1995 (core) + 13211-2:2000 (modules), ISO/IEC JTC1 SC22 WG17. Implementations: SWI-Prolog (Jan Wielemaker, CWI / VU Amsterdam; stable 10.0 series, 10.1.x dev line as of April 2026; de-facto research/industry standard — web services, tabling, CHR, CLP, Pengines; native GUI for Linux/Wayland or X11, macOS Cocoa, Windows Win32 via SDL3/Cairo/Pango; substantially faster WASM build; 10–30% faster clause indexing/compilation); SICStus (RISE/SICS, commercial; Mats Carlsson; mature constraint libraries); Scryer (Markus Triska, modern ISO-conformant Rust); GNU Prolog (Daniel Diaz); Tau (browser/Node JS, José Riaza); Trealla (Andrew Davison); XSB (Stony Brook; David Warren, Terrance Swift; tabled WAM); B-Prolog/Picat (Neng-Fa Zhou). CHR standardised informally via Thom Frühwirth (KU Leuven, then Ulm; *Constraint Handling Rules*, Cambridge 2009). s(CASP): Joaquín Arias, Manuel Carro (UPM/IMDEA), Elmer Salazar, Gopal Gupta (UT Dallas) — ICLP 2018, TPLP 2018, ongoing through 2024. Community: Prolog Wiki (prolog.org), ALP, ICLP.

### Conceptual model
Horn clauses `Head :- Body` with NAF (`\+`) and cut (`!`); SLD-resolution refutation. Extensions: SLG-resolution tabling (XSB, SWI); CLP(FD)/CLP(R)/CLP(Q)/CLP(B) — solver interleaved with unification; CHR (forward-chaining, committed-choice multi-headed rules over a constraint store, Turing-complete); DCGs (difference-list parsing sugar). s(CASP): goal-directed top-down predicate ASP under stable model semantics with constructive negation (`not p(X)` over uninstantiated variables) — avoids §6 grounding bottleneck, emits per-query justification trees (partial proof restricted to query-relevant literals). Co-induction (`% coinductive`) supports cyclic dependencies (loops in arguments, regulatory cycles).

### Expressiveness/Semantics
Pure Prolog Turing-complete; SLD sound + refutation-complete for pure Horn; SLDNF unsound on non-stratified negation. Tabled SLG (Chen & Warren, JACM 1996) computes well-founded model in polynomial time for Datalog-with-negation, restoring soundness. s(CASP): PSPACE-hard worst case but typically fast on the small query-relevant fragment — Arias et al. (TPLP 2018) report tractable runtimes where grounded ASP exhausts memory. CHR Turing-complete (Sneyers et al., TPLP 2010). CLP(FD) NP-complete; CLP(R) polynomial (simplex).

### Composability/Modularity
ISO 13211-2 modules (`use_module/1`, `module/2`). SWI packs (`pack_install/1`, ~600 packs as of 2025). DCGs compose grammars; CHR rule sets compose monotonically over shared stores; s(CASP) composes like ASP (stable predicate signatures). FLI: C (`PL_*`), Java (JPL), Python (`pyswip`, `janus_swi`), .NET. Tabling/CLP are orthogonal layers, source grammar unchanged.

### Suitability for autoformalization to IR
High for the *executable* face of a clinical rule IR. Prolog exceptionally well-represented in training data (Bratko, Sterling-Shapiro, Covington textbooks; CS curricula; SWISH notebooks) — top-tier models emit syntactically valid Prolog with high probability. s(CASP) less represented (research code) but surface syntax near-identical to ASP, so few-shot transfers from §6 examples. Goal-directed evaluation maps "should patient X receive Y?" CDS queries: IR = knowledge base, question = goal, answer = justification tree displayable to a clinician (`?- p(X), justify.` query mode). Idempotency enhanced by constrained clause-head signature/ordering (linter pass: canonical argument order, alphabetic clause order).

### Formal verification potential
SLD soundness/completeness textbook (Lloyd, *Foundations of Logic Programming*, Springer 1987). Well-founded semantics soundness proven (Van Gelder, Ross, Schlipf, JACM 1991). s(CASP) soundness w.r.t. stable models proven (Arias et al., TPLP 2018). The justification tree IS the proof certificate — literal-level derivation, independently checkable (Arias, Carro, Chen, Gupta, "Justifications for Goal-Directed Constraint Answer Set Programming", ICLP 2020 Technical Communications / EPTCS). Pure Prolog: WAM compiler correctness in Isabelle/HOL (Pusch, TPHOLs 1996); TWAM certifying abstract machine with Coq formalization (Bohrer & Crary, VSTTE 2018); ProB (Leuschel, Düsseldorf) — B-method tool with Prolog substrate used for guideline verification. SWI interpreter not formally verified; 30-year deployment + large CHR/CLP test suites give engineering-grade assurance.

### Tooling/Ecosystem maturity
SWI-Prolog very mature, active (10.0 stable / 10.1.x dev, April 2026): built-in HTTP server, JSON, SWISH web-IDE (swish.swi-prolog.org), tabling, CLP, CHR, `semweb` RDF/OWL, PlUnit, profiler, GUI debugger. SWISH supports executable papers, incl. Probabilistic Logic Programming tutorial (Riguzzi et al.). s(CASP): GitHub `SWI-Prolog/sCASP`, vendored into SWI 9.x+ as `library(scasp)`; active, ICLP tutorial track. SICStus since 1985; aerospace (Mercury Mission Operations), transportation scheduling. Scryer modern, fast on CLP(FD), growing. ProB has direct medical pedigree (Asbru/GLIF interpreters). Logtalk (Paulo Moura): OO layer on SWI/SICStus/Scryer/XSB. Most engines POPLmark/TPTP-tested.

### Japan-specific considerations
**Foundational.** FGCS project (第五世代コンピュータ; 1982–1992) at ICOT (新世代コンピュータ技術開発機構; director Kazuhiro Fuchi 渕一博) made Prolog + concurrent logic programming national R&D priorities: Concurrent Prolog (Ehud Shapiro at Weizmann, co-developed with ICOT), GHC (Guarded Horn Clauses; Kazunori Ueda, ICOT), Flat GHC, KL1 (Kernel Language 1, final-stage language) on PIM (Parallel Inference Machine) hardware. Commercial goals unmet but trained a generation of Japanese LP researchers; ICOT open-sourced KL1, KLIC (KL1-to-C compiler), Multi-VPIM emulator via AITEC archive. Descendants: Ueda (Waseda) — LMNtal (Linked Multi-set Nonlinear hierarchical Term, *TPLP* 2009+), concurrent-LP successor with verified model checker (SLIM). Ken Satoh (NII) — PROLEG over Prolog (research06.md §1), canonical Japanese legal-reasoning Prolog system; JURISIN/PROLALA tutorials. Hidetomo Nabeshima (Yamanashi) — abductive Prolog. Naoki Kobayashi (Univ. of Tokyo) — higher-order model checking, CHC/Horn-clause solving over Prolog-style relational specs. Chiaki Sakama (Wakayama emeritus) — Prolog/ASP semantics. Mutsunori Banbara (Nagoya) — CLP and ASP. Annual JSAI (人工知能学会) sessions still include LP tracks.

### Interoperability
- §4 Datalog: pure Datalog = function-free Horn fragment of Prolog; tabled SWI is a viable Datalog engine; Soufflé-style rules transliterate.
- §6 ASP: s(CASP) IS predicate ASP, goal-directed; programs round-trip via clingo's textual format.
- §12 PLP: ProbLog, cplint, PRISM (Sato) sit on Prolog and reuse its solver; shared IR backbone straightforward.
- §10 PRISM/Storm: probabilistic Prolog frontends compile to factor graphs re-targetable to Storm DTMCs.
- §1 SMT: CLP(FD/Q/R) and PrologCHR can call SMT backends; SWI `library(clp)` includes a Z3 binding (`pl-z3`, Steven Schäfer / Tom Schrijvers).
- Category 1: Arden Syntax MLM compilers in Prolog (historical, Erasmus MC); PROforma (OpenClinical) implemented in Prolog (Tallis engine, John Fox / Cancer Research UK); Asbru's AsbruView and IDAN had Prolog-based execution engines.
- Category 6: defeasible-logic→Prolog translation canonical (Antoniou-Maher; SPINdle generates Prolog-style rule lists); Event Calculus reasoners (RTEC, jREC) are Prolog or compile to it.
- Category 3: SWI `semweb` library — RDF/OWL ingest, SPARQL, ClioPatria triple store; Prolog-driven SWRL execution for hybrid rule + ontology reasoning.

### Limitations/Known issues
SLDNF unsound on non-stratified negation (use tabled SLG or s(CASP)). Cut breaks declarative reading; many production rule sets de-facto procedural. Termination undecidable; occur-check disabled by default (1980s performance compromise → incorrect unification under recursive structures). Left-to-right depth-first search makes naive Prolog brittle to clause ordering — orderings change runtimes or termination; clinical risk: LLM-emitted Prolog may be logically correct but operationally diverge on the patient case at hand. Mitigations: tabling, mode declarations, `set_prolog_flag(occurs_check, true)`, or migration to s(CASP) for non-procedural semantics. Industrial CDS deployments running Prolog rare (Java/CQL/DMN dominate); clinicians who can read/maintain Prolog rule bases are a staffing risk.

### Training data proxy
Strong. Taught continuously since 1972 (Colmerauer, Marseille); Bratko *Prolog Programming for Artificial Intelligence* (Pearson, 4th ed. 2011) and Sterling & Shapiro *The Art of Prolog* (MIT Press, 2nd ed. 1994) in nearly every training set. SWI-Prolog ~1.2k stars; Scryer ~2.4k (modern resurgence). Stack Overflow `prolog` tag ~14,000 questions. SWISH public notebook gallery. ICLP 1984–present; *TPLP* (CUP, since 2001). s(CASP) tutorials ICLP 2020–2024 + SWI docs. Smaller recent arXiv volume than SMT/ASP but deeper historical canon — LLMs reliably generate idiomatic Prolog, including correct CLP(FD).

## 12. Probabilistic Logic Programming: ProbLog, cplint, PRISM (Sato), DeepProbLog

### Purpose
Probabilities on facts (or rules); query answers are probability distributions over derivations. A *single* language combining symbolic clinical knowledge ("antibiotic A is effective against pathogen B") with epistemic uncertainty ("with probability 0.07 the patient has a contraindicating allergy", "lab test sensitivity 0.92") and exact marginal/MPE inference. Natural answer to the "no probabilistic strength" gap in defeasible logic (Category 6 §1) and the "no native uncertainty" gap in classical Event Calculus (Category 6 §7).

### Maintainer/Standards body
Academic-driven; no standards body. Distribution semantics: Taisuke Sato, "A statistical learning method for logic programs with distribution semantics," ICLP 1995 (seminal). Sato & Yoshitaka Kameya, "Parameter Learning of Logic Programs for Symbolic-Statistical Modeling," *JAIR* 15:391–454 (2001); Sato & Kameya, "New Advances in Logic-Based Probabilistic Modeling by PRISM," in *Probabilistic Inductive Logic Programming* (LNCS 4911), Springer, pp. 118–155 (2008) — PRISM (PRogramming In Statistical Modeling), distinct from the §10 model checker. David Poole's Independent Choice Logic (ICL, AIJ 1997): equivalent earlier formulation. **ProbLog**: Luc De Raedt, Angelika Kimmig, Hannu Toivonen, "ProbLog: A probabilistic Prolog and its application in link discovery," IJCAI 2007. **ProbLog2** (current 2.2.x; 2.2.9 released September 23, 2025): Anton Dries, Kimmig, Wannes Meert, Joris Renkens, Guy Van den Broeck, Jonas Vlasselaer, De Raedt, ECML/PKDD 2015 — knowledge-compilation exact inference via SDD/d-DNNF; KU Leuven DTAI (dtai.cs.kuleuven.be/problog; GitHub `ML-KULeuven/problog`). **cplint**/`pita`/`mcintyre`: Fabrizio Riguzzi (Univ. Ferrara), from ~2007, SWI SWISH-integrated (cplint.eu); textbook *Foundations of Probabilistic Logic Programming* (River Publishers, 2nd ed. 2022) standard reference. **CP-logic**/**LPADs**: Joost Vennekens, Marc Denecker, Maurice Bruynooghe (KU Leuven). **DeepProbLog**: Robin Manhaeve, Sebastijan Dumančić, Kimmig, Thomas Demeester, De Raedt, "DeepProbLog: Neural Probabilistic Logic Programming," NeurIPS 2018 (extended *AIJ* 2021). **aProbLog** (algebraic): Kimmig, Van den Broeck, De Raedt, AAAI 2011 — semiring-parameterised inference. **DC-ProbLog**/Hybrid ProbLog: Davide Nitti et al. (continuous random variables). Venues: ICLP, annual PLP workshop, StarAI.

### Conceptual model
Distribution semantics: program = pair `(F, R)` of probabilistic facts `p :: f` (each independently true with probability p) and definite/normal program `R`. Total choice over F → sampled "world"; P(q) = sum of probabilities of worlds where `R ∪ chosen_F ⊨ q`. ProbLog computes via Weighted Model Counting (WMC) over the Boolean formula whose models are entailing worlds — compiled to Sentential Decision Diagram (SDD; Adnan Darwiche, UCLA) or d-DNNF, evaluated bottom-up in linear time in circuit size. CP-logic/LPADs: probabilistic rule-head choices `(h1:p1 ; h2:p2 ; ...) :- body.` (disjunctive ASP with probabilities analogue). DeepProbLog: neural predicates `nn(net, X, Y, Domain) :: f(X, Y).` with network-output probabilities; gradients flow through the SDD via aProbLog semirings, enabling end-to-end training. aProbLog generalises to arbitrary commutative semirings (max-plus → MPE/MAP; fuzzy aggregation; provenance polynomials).

### Expressiveness/Semantics
WMC #P-complete in general; exact via SDD linear in SDD size (possibly exponentially smaller than the propositional theory). Queries with finitely many proofs decidable; infinite proofs need stratification + tabling. PRISM (Sato) requires exclusive-explanation assumption for efficient EM (else mutual exclusivity enforced manually); ProbLog relaxes it at cost of richer circuit. Approximate: k-best proofs, Monte Carlo (`mcintyre`), bounded approximation (Poole), lifted inference (Van den Broeck, AAAI 2011 — first-order PLPs with large populations). Learning: EM (Sato-Kameya FAM, Riguzzi's EMBLEM), gradient descent (DeepProbLog), Learning from Interpretations (LFI). MAP/MPE via aProbLog max-product semiring.

### Composability/Modularity
Inherits Prolog modules. CLP constraints possible (DC-ProbLog, hybrid). DeepProbLog neural predicates first-class — PyTorch networks registered via `nn/4` declarations. cplint composes via SWI modules + ecosystem (`semweb`, tabling, CHR). aProbLog semiring abstraction swaps inference modes without rewriting. Sato's PRISM more monolithic (model file + parameter file), composes via library predicates.

### Suitability for autoformalization to IR
Moderate-high with caveats. Training-data presence thinner than plain Prolog/ASP — KU Leuven tutorials + cplint SWISH notebooks are the main public corpus. Few-shot from a small ProbLog primer + target rule pattern effective for GPT-5.5 / Claude Opus 4.7 / Gemini 3.1 Pro (Category 8). IR sweet spot — clinical-fact + probability + rule: `0.12 :: contraindication(penicillin, X) :- documented_allergy(X, beta_lactam).` Maps to (a) FHIR `AllergyIntolerance.verificationStatus.criticality` weights, (b) GRADE certainty bands (High/Moderate/Low/Very Low → {0.95, 0.7, 0.4, 0.15} or learned anchors). Constrained decoding (Category 8 §4) over a JSON-Schema'd PLP IR with `{probability: float, fact: string}` envelopes gives cross-run idempotency.

### Formal verification potential
Clean model-theoretic foundations (Sato 1995; Riguzzi textbook). SDD/d-DNNF inference deterministic + proof-producing: the compiled circuit IS the explanation; ProbLog can output the d-DNNF (or arithmetic-circuit form), checkable by an independent circuit evaluator. EM soundness under exclusive-explanation proven (Sato-Kameya 2001). DeepProbLog gradients not formally certified, but symbolic SDD layer is independent of the neural layer — symbolic verification preserved when neural predicates abstracted to probability outputs. No Lean/Coq mechanization of the full ProbLog stack (open problem). Cross-check vs §5 OWL or §6 ASP by stripping probabilities (purely-logical residual must remain consistent).

### Tooling/Ecosystem maturity
**ProbLog**: `ML-KULeuven/problog` ~407 stars (May 2026); web playground problog.cs.kuleuven.be; Python + SWI bindings; SDD via Wmc/`SDD` library (Darwiche group); active through 2025, NeSy workshop ties (2.2.9, September 23 2025). **cplint**: SWISH integration (cplint.eu); textbook code; PITA (Probabilistic Inference modulo Theories) integrates Z3. **PRISM (Sato)**: distributed via AIST archive; less actively maintained standalone, semantics influential. **DeepProbLog**: `ML-KULeuven/deepproblog`; PyTorch-based; NeurIPS demos. **DeepStochLog** (Winters, AAAI 2022): probabilistic stochastic grammars over PLP. **PLP-on-ASP**: `pasp` (Riguzzi/cplint dialect on clingo). **smProbLog** (Totis, Kimmig, De Raedt, *TPLP* 2023): stable-model ProbLog for negation. **ProbEC** (Skarlatidis, Paliouras, Artikis et al., NCSR Demokritos, *TPLP* 2015): Probabilistic Event Calculus over ProbLog — directly applicable to noisy clinical event streams (Category 6 §7). Stable, growing; community at KU Leuven, UniFE, NCSR Demokritos, UT Dallas, UCLA.

### Japan-specific considerations
**Origin point.** Sato Taisuke (佐藤泰介, Tokyo Tech emeritus, currently AIST AIRC / Tokyo Tech honorary professor) is the original architect of distribution semantics — PRISM (1995–) predates ProbLog by 12 years, acknowledged as foundation in every ProbLog paper. Yoshitaka Kameya (亀谷由隆, formerly Tokyo Tech / Nagoya Institute of Technology, now Meijo Univ.) co-developed PRISM, contributed FAM-EM learning (Sato-Kameya, JAIR 2001). Continued PLP interest: Katsumi Inoue (NII) — abductive LP, LFIT, foundational to PLP parameter learning; Chiaki Sakama (Wakayama emeritus) — probabilistic stable-model semantics, brave/cautious induction; Hidetomo Nabeshima (Yamanashi) — probabilistic abduction. JSAI (人工知能学会) and IPSJ (情報処理学会) host sporadic PLP sessions. PRISM applied to biological-pathway probabilistic modelling by Sato's group (HMMs, PCFGs, Bayesian networks each expressible in <20 lines of PRISM); techniques transfer to clinical-pathway probabilistic CDS — no Japan-originated industrial clinical PLP product publicly documented.

### Interoperability
- §11 Prolog/s(CASP): ProbLog and cplint *built on* SWI-Prolog; non-probabilistic residual is a Prolog program; explanations are weighted Prolog proof terms.
- §6 ASP: smProbLog extends ProbLog to stable-model semantics; PASOCS (Tuckey, Russo, Broda) probabilistic ASP solver.
- §10 PRISM/Storm: finite-horizon propositional ProbLog fragments compile to DTMCs/MDPs for PRISM (Kwiatkowska) — bidirectional exchange via JANI research-grade but feasible.
- §1 SMT: PITA (cplint) integrates Z3 for probabilistic inference over numeric constraints — key for continuous clinical observations (eGFR, BMI, INR).
- §9 E-Graphs: equality saturation over PLP terms unexplored, conceptually compatible.
- Category 1: GRADE certainty bands + CDS-Hooks risk scores map to probabilistic facts; FHIR ImmunizationEvaluation / RiskAssessment resources are natural sinks.
- Category 2: PMDA adverse-event statistics (JADER) are population-frequency data ready for parameterisation.
- Category 6 §1: defeasible rules get explicit probabilities; superiority `>` reconstructible as conditional probability.
- Category 6 §7: ProbEC (Skarlatidis et al., TPLP 2015) enables uncertain longitudinal CDS.
- Category 8 §9: natural Program-Aided Language Model target — LLM emits PLP code, interpreter computes the probability.

### Limitations/Known issues
WMC #P; SDD compilation blows up on programs with many disjunctive proofs ("intersection-of-proofs" overhead). Approximate-inference quality hard to certify for safety-critical CDS. Parameter elicitation: clinicians give point probabilities/GRADE bands, not full joint distributions — independent-fact assumption can be wrong (correlated allergies, comorbidities); distribution-semantics independence requires explicit conditioning for correlated risk factors. LLM-generated PLP needs validation that probabilities form a coherent measure (no implicit double-counting). Continuous variables (eGFR, age) require Hybrid ProbLog / DC-ProbLog — less mature. DeepProbLog training slow vs. pure neural baselines (offset by interpretability). Lifted inference for large patient populations research-grade. No clinical regulatory precedent (FDA/PMDA SaMD) for PLP-based CDS as of 2026.

### Training data proxy
Moderate. ProbLog tutorial pages + Riguzzi textbook well-indexed. ProbLog ~407 stars (May 2026); DeepProbLog active at `ML-KULeuven/deepproblog`; cplint.eu + SWISH notebooks publicly browsable. *TPLP* ProbLog/cplint papers open-access. ICLP/PLP-workshop archive on probabilistic-logic-programming.org. Stack Overflow PLP tag small (<200 questions) — LLM emission benefits from few-shot with curated De Raedt / Riguzzi tutorial examples. Sato's foundational papers (1995, 2001, 2008) in every training set. DeepProbLog at NeurIPS gives recent visibility boost.
