# Category 5: Automated Reasoning, Constraint Solving, and Collision Detection

## 1. SMT-LIB Encodings with Z3, cvc5, and Bitwuzla

### Purpose
SMT (Satisfiability Modulo Theories) decides validity of first-order formulas over background theories (integers, reals, bit-vectors, arrays, strings, datatypes, uninterpreted functions). Used as a backend decision engine for verification, autoformalization checks, and constraint discharge.

### Maintainer/Standards body
SMT-LIB standard maintained by Clark Barrett (Stanford), Pascal Fontaine (Liège), Cesare Tinelli (Iowa). Current standard: SMT-LIB v2.6 (2021-05-12 reference) and SMT-LIB v2.7 (initial reference 2025-02-05; clarifications and minor errata in r2025-07-07; further extended on 2026-03-18 with an exponentiation operator in the Ints theory and a new QF_EIA logic), adding algebraic datatype declarations and a prenex-polymorphism + higher-order maps theory (HO-Core), with v3 in development (full higher-order logic). Solvers: Z3 (Microsoft Research; Leonardo de Moura, Nikolaj Bjørner; current release 4.16.0, Feb. 19 2026), cvc5 (Stanford/Iowa; successor to CVC4, Barrett et al.; current release 1.3.4, May 7 2026), Bitwuzla (Aina Niemetz, Mathias Preiner; bit-vector specialist, successor to Boolector; ongoing 2026 development including TACAS 2026 "Bit-Precise Interpolation in Bitwuzla" by Niemetz and Preiner supporting single-interpolant and inductive interpolation-sequence queries — full interpolation for QF_BV with experimental floating-point and partial array/UF support).

### Conceptual model
Quantifier-free or quantified first-order formulas in S-expression syntax. Each formula is declared over a `set-logic` (e.g., `QF_UFLIA`, `AUFLIRA`, `QF_BV`, `QF_FP`, `QF_S`, `QF_DT`, and the newer `QF_EIA` for exponential integer arithmetic). Solvers combine a CDCL SAT core (`CDCL(T)` in Z3/cvc5; lemmas-on-demand in Bitwuzla) with theory propagators. Output: `sat`/`unsat`/`unknown` plus optional model, unsat core, or proof.

### Expressiveness/Semantics
QF_LIA: NP-complete. QF_LRA: P (polynomial via simplex). QF_BV: NP-complete via bit-blasting. Quantified fragments are typically undecidable; solvers use heuristic E-matching, MBQI, enumerative instantiation. Strings + LIA undecidable in general but practically solvable for many fragments. Datatypes decidable for QF; recursive functions handled via syntactic unfolding.

### Composability/Modularity
SMT-LIB scripts compose via `push`/`pop` incremental stacks, `define-fun`, `declare-datatypes`. Theory combination via Nelson–Oppen (disjoint signatures) or model-based combination. Standard input/output enables solver portfolios.

### Suitability for autoformalization to IR
Excellent target for LLM-generated IR: textual, syntactically rigid, widely represented in training data. Idempotency requires canonical naming, fixed sort declarations, and deterministic `set-logic` choice. Round-trip stability is high when formulas are stratified by theory; semantic convergence aided by `define-fun` macros and `:named` annotations for tracking provenance.

### Formal verification potential
Z3 produces proofs in its own format (and via `proof=true`); cvc5 produces LFSC and Alethe proofs (Carcara checker, Coq/Lean reconstruction via SMTCoq, lean-smt). Bitwuzla produces DRAT proofs for the SAT backbone. Unsat cores extractable via `get-unsat-core` on `(! e :named id)` annotations.

### Tooling/Ecosystem maturity
SMT-COMP held annually. SMT-COMP 2024: Bitwuzla dominated bit-vector and quantifier-free combined-theory divisions (42 of 82 division gold medals); cvc5 dominated strings, datatypes, quantified UFDT/UFDTLIA; Z3++ led nonlinear integer/real arithmetic; OpenSMT, Yices2, SMTInterpol competitive in linear arithmetic. SMT-COMP 2025 (20th edition, co-located with SAT'25; results published August 11, 2025) saw continued dominance of Bitwuzla on bit-vector/floating-point divisions and cvc5 on strings/datatypes/quantified divisions; benchmark and execution-data archives are deposited on Zenodo (records 16887742 and 16875980). Bindings: Python (`z3-solver`, `pysmt`, `cvc5-pythonic`), C/C++, Java, OCaml, Rust (`z3.rs`).

### Japan-specific considerations
Takamasa Okudono (NII, ERATO MMSD) co-authored "Mind the Gap: Bit-vector Interpolation recast over Linear Integer Arithmetic" (TACAS 2020). Hasuo group (NII) uses Z3 as the SMT backend for hybrid-system safety verification (RSS/GA-RSS). Mizuhito Ogawa lab (JAIST) works on SMT for non-linear arithmetic. No Japan-originated SMT-COMP-winning solver, but heavy domestic use.

### Interoperability
- Within Category 5: Backend for MaxSAT (Z3's `optimize`), Apalache (TLA+→Z3), Datalog with constraints (Soufflé→Z3), MiniZinc via OptiMathSAT, e-graph extraction with SMT-guided cost functions.
- Category 1: Used to discharge FHIR Clinical Reasoning predicates, DMN decision-table consistency.
- Category 2: Encodes Minds-extracted clinical predicates as theory atoms over patient ontology sorts.
- Category 3: SHACL → SMT translation possible for cardinality and datatype facets.
- Category 4: Z3 is the proof-tactic backend for `lean-smt`, Isabelle's Sledgehammer, Why3's SMT provers, F* via Z3.

### Limitations/Known issues
Nonlinear real arithmetic decidable but doubly-exponential. Quantifier handling brittle; incompleteness common. Floating-point reasoning slow even with SymFPU encodings. String theory implementations diverge across solvers. Proofs are large and solver-specific.

### Training data proxy
Very abundant: Z3 GitHub repo `Z3Prover/z3` has ~12.2k stars (May 2026); cvc5 `cvc5/cvc5` has 1.3k stars; Bitwuzla `bitwuzla/bitwuzla` has ~347 stars (May 2026). Thousands of SMT-LIB benchmarks (~140k in SMT-COMP standard test set). Heavy Stack Overflow presence; standard target in textbook automated-reasoning courses.

## 2. SAT/MaxSAT for Minimal Conflict and Repair-Set Search

### Purpose
SAT decides propositional satisfiability; MaxSAT finds an assignment satisfying the maximum (possibly weighted) number of soft clauses subject to hard clauses. Used for conflict localization (minimum unsat fragment) and repair-set synthesis (smallest set of constraints whose removal restores consistency).

### Maintainer/Standards body
Annual SAT Competition (organizers Heule, Iser, Järvisalo, Suda). Annual MaxSAT Evaluation. Reference solvers: CaDiCaL, Kissat, Gimsatul, IsaSAT (Armin Biere group, Univ. Freiburg), Glucose (Audemard, Simon), MiniSat (Eén, Sörensson, legacy). MaxSAT: RC2 (Ignatiev, Morgado, Marques-Silva), EvalMaxSAT (Avellaneda), MaxHS (Davies, Bacchus), Open-WBO (Martins, Manquinho), UWrMaxSat (Piotrów). DIMACS CNF and WCNF formats are de facto standards.

### Conceptual model
Propositional CNF over Boolean variables. MaxSAT adds soft clauses with weights; objective minimizes total weight of violated softs. Algorithms: core-guided (OLL, MSU3), implicit hitting set (MaxHS), linear search, branch-and-bound (MaxCDCL).

### Expressiveness/Semantics
SAT: NP-complete (Cook–Levin). MaxSAT: FP^NP[log]-complete; partial-weighted MaxSAT widely used. No quantifiers, no native integers (encoded via cardinality/PB constraints — totalizer, sequential counter, sorting networks).

### Composability/Modularity
DIMACS CNF/WCNF universal. Soft clauses tagged with weights or "group MUS" labels. Incremental SAT supported in CaDiCaL via IPASIR-2 interface. Cardinality constraints layered as separate encoders (PySAT iTotalizer).

### Suitability for autoformalization to IR
Lower-level than SMT; LLMs target SAT indirectly via higher-level languages (ASP, MiniZinc, SMT) that flatten to CNF. As a backend, deterministic given fixed solver seed; idempotency requires variable-naming convention.

### Formal verification potential
DRAT/LRAT proofs standard (proof checkers cake_lpr, drat-trim, verified in Coq/Isabelle). Unsat cores native. Verified solvers exist: IsaSAT (Isabelle/HOL-verified, Mathias Fleury), versat.

### Tooling/Ecosystem maturity
SAT Competition 2024: Kissat (Biere et al., Freiburg) swept gold in SAT, UNSAT, SAT+UNSAT main tracks (kissat-sc2024: PAR-2 = 2788.13, 306 solved out of exactly 400 benchmarks per Biere's official Zenodo deposit, DOI 10.5281/zenodo.15095752: "These are the 400 CNF files in DIMACS format from the main track of the SAT competition 2024"). SAT Competition 2025 (results released August 2025) saw AE-Kissat-MAB (Ding, Luo, Li et al.) win the Main Sequential Track overall with PAR-2 = 2264.73 and 327 solved, with Kissat-public and Kissat-VSA placing 2nd and 3rd; CaDiCaL-SC2025 (Biere group) won the Main Sequential UNSAT subtrack (PAR-2 = 2327.00, 161 solved); MallobSat (Schreiber, Rigi-Luperti, Biere; KIT/Freiburg) won the Parallel Track (PAR-2 = 394.56, 337 solved) — its first parallel-track gold. MaxSAT Evaluation 2024 winners: SPB-MaxSAT-c-Band, SPB-MaxSAT-c-FPS swept all four anytime categories; EvalMaxSAT and UWrMaxSat strong in the exact track using SCIP+CDCL portfolios. Bindings: PySAT (Ignatiev), C/C++/Rust.

### Japan-specific considerations
Miyuki Koshimura (Kyushu University) — QMaxSat MaxSAT solver, multiple Max-SAT Evaluation podiums. Hidetomo Nabeshima (Yamanashi) — SAT solver development. Naoyuki Tamura, Mutsunori Banbara (Kobe/Nagoya) — Sugar CSP-to-SAT translator, multiple SAT-based competition wins (CSP Competition 2008 1st place; MaxSAT 2012 2nd place in Partial Weighted Max-SAT crafted). Takehide Soh (Kobe) — sCOP/Scarab.

### Interoperability
- Backbone of MaxSAT, MUS enumeration (§3), Datalog#-modulo-theories, MiniZinc backends (PicatSAT), CP-SAT (OR-Tools layers CP over SAT), ASP solvers (clasp wraps a CDCL SAT engine).
- Category 1: Repair-set search over rule sets (DMN tables, clinical pathways) maps directly to MaxSAT.
- Category 3: OWL EL axiom pinpointing reduces to Horn group-MUS (HGMUS).
- Category 4: DRAT proofs feed Isabelle/Coq-verified checkers.

### Limitations/Known issues
No first-order quantification, no real arithmetic, no theory of arrays. PB/cardinality encodings blow up CNF size. Solver runtime is highly instance-sensitive; no theoretical worst-case bounds beyond NP/FP^NP.

### Training data proxy
Massive. MiniSat is canonical pedagogical reference. GitHub: kissat ~629 stars, CaDiCaL ~553, MiniSat clones thousands (May 2026). SAT/MaxSAT competition reports annually (CEUR-WS, Univ. Helsinki Tech Reports).

## 3. MUS/MCS Enumeration and UNSAT-Core Extraction

### Purpose
Given an unsatisfiable constraint set, enumerate Minimal Unsatisfiable Subsets (MUS — minimal cores explaining infeasibility) and Minimal Correction Subsets (MCS — minimal deletions restoring satisfiability). Foundation for conflict explanation and contradiction localization in guideline corpora.

### Maintainer/Standards body
No formal standards body. Reference algorithms: CAMUS (Liffiton & Sakallah), MARCO (Liffiton, Previti, Malik, Marques-Silva), DAA (Bailey, Stuckey), eMUS (Previti, Marques-Silva), MUSer2 (Belov, Marques-Silva), HGMUS (Ignatiev et al.) for Horn/EL. Implementations: MARCO (Python, Liffiton, https://sun.iwu.edu/~mliffito/marco/), PySAT modules, FLINT.

### Conceptual model
Hitting-set duality (Reiter 1987, Birnbaum/Lozinskii): MUSes and MCSes form dual minimal hypergraph transversals. MCS is minimal set whose removal makes formula satisfiable; equivalently complement of Maximal Satisfiable Subset (MSS). Algorithms: explicit dualization (CAMUS), implicit dualization with seed-and-shrink and "map" formula tracking explored regions (MARCO/eMUS), IHS (implicit hitting set, used in MaxHS).

### Expressiveness/Semantics
Works over any monotone constraint formalism with a satisfiability oracle: SAT, SMT, CSP, Horn (axiom pinpointing in EL ontologies). Worst-case exponential number of MUSes/MCSes. Group-MUS variant tags constraints with group labels to find minimal-set-of-groups cores — exactly the right abstraction for "which clinical-guideline rules together contradict?"

### Composability/Modularity
MARCO is solver-agnostic: needs SAT oracle + shrink procedure. Group labels allow modeling per-guideline-section provenance. Works with Z3 (via `(set-option :produce-unsat-cores true)`) and PicoMUS.

### Suitability for autoformalization to IR
Natural fit when IR clauses each carry stable identifiers — enumeration returns sets of identifiers, directly machine-readable. Idempotency depends on solver determinism and a canonical lexicographic preference among MUSes (otherwise enumeration order can vary).

### Formal verification potential
Each MUS verifiable by unsatisfiability proof of the subset plus satisfiability proof of each proper sub-subset (computationally expensive but certifiable). MCS verifiable by satisfiability witness + per-element regression. Marques-Silva 2020 surveys formal duality proofs.

### Tooling/Ecosystem maturity
Mature in SAT and SMT: Z3 `get-unsat-core`, cvc5 `--produce-unsat-cores`. Standalone: MARCO, MUSTool, optimal-MUS/OCUS (Gamba, Bogaerts, Guns 2023). HGMUS used for SNOMED-CT debugging benchmarks.

### Japan-specific considerations
Hidetomo Nabeshima (Yamanashi) — minimal-model generation foundations relevant to ASP/MUS computation (Koshimura, Nabeshima, Fujita, Hasegawa, FTP 2009). Katsumi Inoue (NII) — model-generation theorem proving foundations underlying ASP MUS-style reasoning. No standalone Japan-originated MUS enumerator with international distribution identified.

### Interoperability
- Direct consumer of any SAT/SMT solver from §1, §2.
- Bridges to ASP for non-monotonic conflict analysis (MUS over choice/weak-constraint encodings).
- Category 1: Localizes contradictions between FHIR Clinical Reasoning artifacts or BPMN+CDS rule sets.
- Category 3: Axiom pinpointing in OWL EL ontologies — standard application (SNOMED-CT debugging).
- Category 4: MUS certificates can be reconstructed in Coq/Isabelle.

### Limitations/Known issues
Worst-case exponential. "Map formula" memory bloat for large constraint sets. Selecting *which* MUS to report (preferred MUS) is itself an optimization problem (smallest, lexicographically minimal, weighted minimum).

### Training data proxy
Moderate. Active in SAT/SMT/CP research literature (CP, IJCAI, AAAI, SAT). MARCO has modest GitHub presence; less Stack Overflow exposure than Z3 itself. Survey papers: Marques-Silva & Mencía 2020.

## 4. Datalog / Datalog± / RDFox-Style Rule Materialization

### Purpose
Declarative deductive-database rule language for forward-chaining inference. Materializes the least fixed point of Horn-like rules over a base relation set. Used for ontological reasoning, program analysis, graph analytics.

### Maintainer/Standards body
No single standards body; ISO Datalog standardization effort dormant. Reference engines: Soufflé (souffle-lang.github.io; Scholz, Jordan; CAV 2016), RDFox (Oxford Semantic Technologies / Boris Motik et al.; commercial), LogicBlox (defunct), ddlog (VMware/Differential Datalog), DLV (deductive front-end, Calabria), Vadalog (Datalog± at Oxford/TU Wien, Gottlob, Bellomarini). Datalog± framework formalized by Calì, Gottlob, Lukasiewicz (2009+).

### Conceptual model
Function-free Horn clauses (head ← body₁,…,bodyₙ) over relational facts. Semi-naïve evaluation iteratively computes new facts from previously derived facts (delta relations). Stratification handles negation (stratified negation, well-founded semantics). Datalog± adds existential rules (tuple-generating dependencies, TGDs), equality-generating dependencies (EGDs), keys.

### Expressiveness/Semantics
Pure Datalog: P-complete data complexity, EXPTIME combined complexity, decidable. Datalog± with existentials: undecidable in general; decidable fragments include guarded TGDs (2-EXPTIME-complete), sticky, weakly-acyclic, warded. RDFox's RL profile + Datalog corresponds to OWL 2 RL with materialization.

### Composability/Modularity
Rules are modular by predicate. Soufflé supports components, ADTs, records. RDFox supports SWRL-like and SPARQL-CONSTRUCT rules, incremental maintenance under fact addition and deletion (DRed-based). Standard input: facts as CSV/TSV or RDF triples; rules as `.dl` (Soufflé) or `dlog` (RDFox).

### Suitability for autoformalization to IR
Excellent — Datalog is a very common LLM target (clean syntax, declarative). Semi-naïve evaluation is deterministic → fixpoint is canonical → strong idempotency for the *materialization* layer. Variation only in derivation order, not final extension. Datalog± enables explicit existentials for "unspecified consequence" semantics in clinical rules.

### Formal verification potential
Soundness/completeness of semi-naïve evaluation classical (Abiteboul, Hull, Vianu). No native proof certificate, but proof trees / why-provenance computable (Soufflé `--provenance` flag returns derivation tree per fact). Verified Datalog: Bembenek et al., formalized Datalog in Coq.

### Tooling/Ecosystem maturity
Soufflé compiles to parallel C++ (used in Doop pointer analysis, security analyzers). RDFox handles billion-triple KGs in main memory. Vadalog optimized for Datalog±. dlv2 ASP-Core-2 compliant.

### Japan-specific considerations
Less prominent in Japan than ASP. Naoki Kobayashi (Univ. of Tokyo) works on higher-order model checking and CHC/Horn-clause solving — Datalog-adjacent. Makoto Tatsuta (NII) — separation logic and static analysis. No widely-known Japan-originated industrial Datalog product identified.

### Interoperability
- Bridges naturally to e-graph relational engines (egglog combines Datalog + equality saturation — §9).
- Drives RL-profile OWL reasoners (§5).
- Category 1: Native fit for openEHR archetype querying via AQL → Datalog translation; FHIR Path/CQL has reductions to Datalog for non-recursive queries.
- Category 3: RDFox is a primary OWL 2 RL + SHACL companion reasoner.
- Category 4: Provenance trees can be exported to Lean/Isabelle.

### Limitations/Known issues
Datalog± existentials require careful fragment choice to retain decidability. Aggregation semantics non-uniform across engines. Recursive aggregation problematic (no canonical fixpoint without lattice semantics). Negation under stratification only; full negation requires well-founded or stable-model semantics (→ ASP).

### Training data proxy
Moderate. Soufflé `souffle-lang/souffle` has 1.1k GitHub stars and 240 forks per the souffle-lang organization repositories page (last updated May 4, 2026); RDFox is closed-source (commercial Oxford Semantic Technologies, founded by Motik/Horrocks/Nenov; product since 2017). Active research (PLDI, VLDB, PODS, SIGMOD). Used in academic compilers/PL community.

## 5. OWL Reasoning with ELK, HermiT, and RDFox

### Purpose
Decide entailment, classification (subsumption hierarchy), realization (instance types), and consistency for OWL 2 ontologies. Used for terminological reasoning over biomedical ontologies (SNOMED-CT, ICD-11, NCIt).

### Maintainer/Standards body
W3C OWL 2 standard (2009/2012). Reasoners: ELK (Yevgeny Kazakov, Markus Krötzsch, František Simančík; Univ. Ulm/Manchester/Oxford), HermiT (Birte Glimm, Ian Horrocks, Boris Motik; Oxford/Ulm), RDFox (Oxford Semantic Technologies), Pellet (legacy Clark & Parsia/Stardog), FaCT++ (legacy Manchester).

### Conceptual model
Description Logic (DL) ABox+TBox+RBox knowledge bases. Profiles trade expressivity for tractability:
- OWL 2 EL: P-time classification (existential restrictions, intersection, no negation/universals). Reasoner: ELK uses consequence-based / completion-rule procedure.
- OWL 2 QL: AC0 query answering (DL-Lite family).
- OWL 2 RL: P-time via Datalog/RDFox materialization.
- OWL 2 DL (full): SROIQ(D), N2EXPTIME-complete. HermiT uses hypertableau calculus.

### Expressiveness/Semantics
SROIQ(D): object/data properties, role hierarchies, inverse, transitive, functional, cardinality, nominals, datatypes. ELK is incomplete for non-EL constructs but is the fastest classifier for EL-shaped clinical ontologies (SNOMED-CT classifies in seconds).

### Composability/Modularity
OWL API (Java) is the de facto reasoner interface. Module extraction (locality-based) allows splitting large ontologies. MORe combines ELK + HermiT/RDFox modularly, delegating EL fragments to ELK. Imports via `owl:imports`; ontology alignment via SKOS/EQUI.

### Suitability for autoformalization to IR
Strong. OWL Functional-Style Syntax and Manchester Syntax are both LLM-friendly. Classification produces canonical class hierarchy → idempotent given fixed ontology. Realization deterministic for OWL EL.

### Formal verification potential
Classification soundness/completeness proved for each profile. Axiom pinpointing (justifications) via OWL API `BlackBoxExplanation` or HGMUS over EL Horn encodings. RDFox produces derivation traces.

### Tooling/Ecosystem maturity
Protégé as standard editor. ROBOT (OBO Foundry) automates reasoning workflows. OWL Reasoner Evaluation (ORE) workshop competition. ELK (current 0.6.0; liveontologies/elk-reasoner, supporting OWL API 4.x and 5.x) widely used in biomedical OBO Foundry pipelines.

### Japan-specific considerations
Riichiro Mizoguchi (formerly Osaka, now JAIST) and Kouji Kozaki — Hozo ontology editor and role theory foundations. Hideaki Takeda (NII) — Semantic Web research, OWL-Full reasoning (Koide & Takeda, ASWC 2006). Takahira Yamaguchi (Keio) — DODDLE-OWL. Takeshi Imai et al. (Univ. of Tokyo Medical School) — Japan Journal of Medical Informatics radiological-diagnosis ontology (DOI 10.14948/jami.25.395). Note: JAMI = Japan Association for Medical Informatics; a Japan-specific "JaMI" branded medical ontology was not located in primary sources.

### Interoperability
- RDFox bridges §4 (Datalog) and §5 (OWL RL).
- ELK pinpointing is the canonical group-MUS application (§3).
- Category 1: FHIR ValueSet expansion uses OWL/SKOS reasoning; SNOMED-CT post-coordination requires EL classification.
- Category 2: MEDIS / Minds terminologies normalized via OWL alignment.
- Category 3: SHACL adds closed-world constraints atop OWL's open-world; both consumed.
- Category 4: Coq/Isabelle formalizations of DL (e.g., Beckert, Schmitt) reconstruct EL proofs.

### Limitations/Known issues
OWL 2 DL reasoners can blow up on highly-quantified axioms or large nominal sets. ELK incompleteness for non-EL constructs. Open-world semantics frequently confuses clinical-rule authors expecting closed-world (→ SHACL or Datalog for closed-world overlay). No native aggregation, arithmetic, or temporal reasoning.

### Training data proxy
Abundant. OWL 2 W3C documents heavily indexed. Protégé tutorials proliferate. ELK GitHub modest; HermiT well-cited. SNOMED-CT EL classification is the canonical OWL EL benchmark.

## 6. ASP with Clingo for Defaults, Exceptions, and Nonmonotonic Rules

### Purpose
Answer Set Programming encodes combinatorial search and nonmonotonic knowledge representation as logic programs whose stable models are solutions. First-class support for defaults, exceptions, preferences, and abduction — directly relevant to clinical guideline modeling (default treatment / exception per comorbidity).

### Maintainer/Standards body
ASP-Core-2 language standard (Calimeri, Faber, Gebser et al., 2020). Reference systems: Clingo / gringo / clasp (Potassco group, Univ. of Potsdam; Torsten Schaub, Martin Gebser, Roland Kaminski, Benjamin Kaufmann); DLV2 (Univ. of Calabria; Leone, Ricca); WASP. ASP Competition organized biennially (LPNMR/KR co-located).

### Conceptual model
Disjunctive logic programs under stable model semantics (Gelfond & Lifschitz 1988). Rules with normal/choice/aggregate/weak-constraint heads. Negation-as-failure with stable-model fixpoint. Weak constraints `:~ body. [w@l,tuple]` express optimization (lexicographic by priority level). Modular extensions: clingo[DL] difference logic, clingo[LP] linear constraints, clingcon CSP, ASP-modulo-theories.

### Expressiveness/Semantics
Disjunctive ASP: Σ₂ᴾ-complete (second level of polynomial hierarchy). Normal programs: NP-complete. Weak constraints add optimization layer (FPΣ₂ᴾ[log]). Strong negation `-p` distinguished from default negation `not p`. Multi-shot solving via Python/Lua API.

### Composability/Modularity
Programs split via `#program` declarations (named subprograms). Multi-shot solving (Clingo 4+) supports incremental grounding and reactive reasoning. Reification of programs as facts enables meta-programming.

### Suitability for autoformalization to IR
Excellent for clinical rules with exceptions. LLMs produce ASP fluently (well-represented in training). Semantic convergence requires fixed predicate signatures and stable rule naming; multiple optimal answer sets handled via deterministic enumeration order and `#show` projection. The Gelfond-style "knowledge pattern" methodology (Chen et al.) has been used to encode entire chronic-disease guidelines (CHF, ~80-page guideline) in ASP.

### Formal verification potential
Stable-model semantics has clean fixpoint characterization (Gelfond–Lifschitz reduct). Soundness/completeness proven for clasp's CDCL-based answer-set search. No standard proof certificate, but unsat cores and weak-constraint optima certifiable. ILASP (Mark Law) supports inductive learning of ASP programs.

### Tooling/Ecosystem maturity
ASP Competition since 2007; Potassco consistently dominant since ~2011. The clingo repository (potassco/clingo) saw continued active development through April 2026. Python `clingo` library, telingo (temporal ASP), eclingo (epistemic), clinguin (UI). DLV2 strong in disjunctive optimization.

### Japan-specific considerations
Highly active. Katsumi Inoue (NII) — Learning from Interpretation Transition (LFIT), inductive ASP, abductive logic programming; cited as foundational by ILASP authors. Chiaki Sakama (Wakayama Univ.) — brave/cautious induction, ASP semantics. Mutsunori Banbara (Nagoya), Takehide Soh (Kobe), Naoyuki Tamura (Kobe Emeritus) — SAT-based and ASP-based hybrid solvers (clingcon, aspartame, Sugar). Hidetomo Nabeshima (Yamanashi). Recurring joint papers with Potassco (Schaub) on Hamiltonian-cycle reconfiguration, scheduling.

### Interoperability
- clingo[DL/LP] embeds difference/linear arithmetic (bridges to §1, §7).
- ASP encoding of MUS/MCS enumeration available (§3).
- Category 1: ASP encodings of GLARE and Asbru clinical-guideline languages exist (Terenziani group, Spiotta, Terenziani & Theseider Dupré, "Temporal Conformance Analysis and Explanation of Clinical Guidelines Execution: An Answer Set Programming Approach", IEEE TKDE 29(11):2567–2580, 2017); ASP-based conformance analysis of guideline execution traces.
- Category 2: PMDA exception handling (e.g., drug-drug interaction exceptions) fits ASP weak constraints naturally.
- Category 3: ASP can simulate Datalog±/OWL RL with negation-as-failure for closed-world overlays.
- Category 4: Lean/Isabelle have ASP semantics formalizations (limited); ILASP can produce rules verifiable by post-hoc proof.

### Limitations/Known issues
Grounding bottleneck — ASP grounds variables before solving; large instance sets blow up. Mitigations: lazy grounding (alpha, Spasic), tight programs. Nonmonotonicity confusing for users accustomed to classical logic. No native real arithmetic without extensions.

### Training data proxy
Strong in academic AI literature; less commercial Stack Overflow presence than SMT. Potassco GitHub `potassco/clingo` ~785 stars (May 2026). Annual ASP competition reports. Active medical-AI literature (Heart failure ASP advisory system, Genesereth/Gelfond's textbook examples).

## 7. Constraint Programming with MiniZinc and OR-Tools CP-SAT

### Purpose
High-level modeling language (MiniZinc) targeting heterogeneous constraint solvers (CP, MIP, SAT, SMT, LCG) for scheduling, resource allocation, configuration. Useful for planning clinical care pathways with global constraints (resources, precedence, comorbidity exclusions).

### Maintainer/Standards body
MiniZinc maintained by Monash University (Peter J. Stuckey, Mark Wallace, Nick Nethercote, Jip Dekker). FlatZinc is the solver-agnostic intermediate language. MiniZinc Challenge held annually (CP conference). OR-Tools CP-SAT (Google; Laurent Perron). Gecode (Schulte, Lagerkvist, Tack). Chuffed (Stuckey/Chu, lazy clause generation). Choco (École des Mines de Nantes).

### Conceptual model
Variables over finite domains (integers, sets, booleans, floats with restrictions). Global constraints (`alldifferent`, `cumulative`, `circuit`, `regular`, `table`) with custom propagators. Lazy Clause Generation (LCG) combines CP propagation with SAT-style learned clauses (Chuffed, CP-SAT, Pumpkin). Solvers: pure CP (Gecode), LCG (Chuffed, CP-SAT, Pumpkin), MIP (Gurobi, CPLEX, HiGHS), SMT (OptiMathSAT), local search (Yuck, OR-Tools CP-SAT LS).

### Expressiveness/Semantics
Finite-domain CP is NP-complete (decidable). Optimization complete in FPΣ₁ᴾ-style budget. MiniZinc supports user-defined predicates, sets, arrays, optionality. Global constraint catalog (Beldiceanu et al.) ~400 catalogued constraints.

### Composability/Modularity
Models split via `include` directives. `.mzn` model + `.dzn` data file separation. FlatZinc enables solver swapping. Predicate libraries (`globals.mzn`) per-solver-tuned.

### Suitability for autoformalization to IR
Strong target. MiniZinc syntax close to mathematical notation. LLMs produce MiniZinc reasonably well (large IJCAI/CP corpus). Solver result deterministic for proven-optimal; non-unique solutions require deterministic search annotation for idempotency.

### Formal verification potential
Solvers typically not proof-producing (exception: Pumpkin and some LCG solvers emit DRAT-style certificates). MIP solvers (SCIP) produce VIPR certificates. Most CP solvers verified empirically through MiniZinc Challenge benchmarks.

### Tooling/Ecosystem maturity
MiniZinc Challenge 2024 results — Atlantis (free-search PAR 14.00, best) ahead of yuck-free 709.50, optimathsat 885, gecode-fd 938, or_tools-ls-free 1095, chuffed-free 1572; Choco took gold in fixed search; Atlantis also took gold in local search. MiniZinc Challenge 2025 (announced at CP2025, Glasgow, August 10–15 2025) saw Google OR-Tools CP-SAT sweep the medals across the major categories, consistent with its post-2017 dominance of the FlatZinc category. Python `minizinc-python`, MiniZinc IDE.

### Japan-specific considerations
Naoyuki Tamura, Mutsunori Banbara, Takehide Soh, Tomoya Tanjo — Sugar / Azucar (SAT-based CSP); won 4 of 10 categories at 3rd International CSP Solver Competition 2008. Active in scheduling/timetabling. Hidetomo Nabeshima — University of Yamanashi. Hokkaido and JAIST contribute occasionally; no Japan-originated MiniZinc Challenge winner identified.

### Interoperability
- CP-SAT internally compiles to SAT (§2); OptiMathSAT to SMT (§1).
- ASP-modulo-CSP via clingcon (§6); aspartame translates XCSP/Sugar facts to ASP.
- Category 1: BPMN/ePath scheduling and resource constraints map directly.
- Category 4: TLA+/Apalache symbolic execution can offload to CP solvers for combinatorial subproblems.

### Limitations/Known issues
No real-valued reasoning beyond restricted float domains. Quantifiers absent. Model performance highly solver- and encoding-dependent. Search annotations are imperative artifacts breaking pure declarativity.

### Training data proxy
Moderate. MiniZinc handbook well-indexed. OR-Tools CP-SAT has heavy Google-blog and Stack Overflow presence (GitHub `google/or-tools` ~13.5k stars, May 2026). Annual MiniZinc Challenge reports.

## 8. TLA+ TLC / Apalache Model Checking

### Purpose
Specify and verify behavioral properties (safety, liveness, refinement) of concurrent and distributed systems. Used for distributed clinical-data pipelines, eventual-consistency reasoning, audit-trail correctness.

### Maintainer/Standards body
TLA+ designed by Leslie Lamport (Microsoft Research, retired); language and TLC model checker maintained at GitHub tlaplus/tlaplus. Apalache developed by Igor Konnov, Jure Kukovec, Thomas Pani (formerly Informal Systems, originally TU Wien). Apalache is de-facto funded by its current maintainers and contributors post the Informal Systems wind-down, with continued repository activity at apalache-mc/apalache through 2026. PRISM/Storm and other QComp tools partly bridge but operate on probabilistic models.

### Conceptual model
Specifications as temporal logic of actions: state machine = initial predicate Init + next-state relation Next (a disjunction of actions). Properties expressed in temporal logic with `□`, `◇`, fairness conditions. TLC explores reachable states explicitly (BFS), checking invariants and temporal properties. Apalache translates a bounded fragment to SMT (Z3) for symbolic bounded model checking and inductive-invariant checking.

### Expressiveness/Semantics
TLA+ is set-theoretic + first-order + temporal (Linear Time TLA). Decidability: undecidable in general; TLC explicit-state with finite model bounds; Apalache decidable up to bounded execution length k. Refinement via implementation+specification implication.

### Composability/Modularity
Modules with `EXTENDS` and `INSTANCE`. Refinement mappings link abstract and concrete specs. PlusCal (also Lamport) transpiles to TLA+ for sequential-imperative style.

### Suitability for autoformalization to IR
TLA+ syntax is mathematical and LLM-friendly but model-checker idiosyncratic (CONSTANT / VARIABLE separation, action enabling). Idempotency strong when specification has named invariants and configured constants in TLC `.cfg` file. Apalache requires type annotations via `Apalache!Snapshot` types.

### Formal verification potential
TLC produces counterexample traces; Apalache produces counterexample + SMT-extracted state sequences. TLAPS (TLA+ Proof System) supports interactive proofs discharged to Isabelle, Z3, Zenon. No DRAT-like universal certificate.

### Tooling/Ecosystem maturity
TLA+ Toolbox IDE, VSCode TLA+ plugin. Mature in industry (AWS, Azure Cosmos DB, MongoDB, Oracle have published TLA+ specs). Apalache available via Docker, JAR; integrated with Quint (engineer-friendly TLA+ frontend). 2024 ETAPS Test-of-Time Tool Award to PRISM (related), and Apalache active development continues with maintainer-driven funding through 2026.

### Japan-specific considerations
Limited Japanese published TLA+ literature. Hasuo group (NII) emphasizes category-theoretic / hybrid-systems formal methods rather than TLA+. JAIST Ogata group works in OTS/CafeOBJ and Maude for distributed-system verification (Raft, Paxos), which is functionally adjacent. No primary public document confirming production TLA+ adoption at NTT/Hitachi/NEC was located.

### Interoperability
- Apalache → Z3 (§1) for symbolic checking.
- TLA+ can drive ASP/CSP for combinatorial subproblems via external integration.
- Category 1: Specifies workflow refinement between BPMN/ePath models and executable implementations.
- Category 4: TLAPS bridges to Isabelle; Quint provides higher-level frontend that may also target Lean/Rocq in future. Refinement is the natural complement to dependent-type proofs in Lean.

### Limitations/Known issues
TLC explicit-state blowup on large state spaces. Apalache currently limited to bounded model checking and inductive invariants; not all TLA+ features supported (e.g., some recursive operator forms). Liveness checking expensive. No native probabilistic extension (use PRISM/Storm instead).

### Training data proxy
Moderate-to-high. Lamport's books, Murat Demirbas's lecture series, Hillel Wayne's "Learn TLA+" widely circulated. TLA+ subreddit, Zulip community. Apalache GitHub `apalache-mc/apalache` has 557 stars (May 2026).

## 9. E-Graphs and Equality Saturation for IR Canonicalization

### Purpose
Compactly represent equivalence classes of terms (e-classes) over rewrite rules, supporting non-destructive rewriting and optimal-term extraction. Foundation for IR canonicalization: multiple semantically equivalent guideline formalizations converge to a canonical representative.

### Maintainer/Standards body
No standards body. Reference implementation: `egg` (Rust; Max Willsey, Chandrakana Nandi, Yisu Remy Wang, Oliver Flatt, Zachary Tatlock, Pavel Panchekha; POPL 2021). `egglog` (PLDI 2023; Yihong Zhang, Wang, Flatt, Cao, Zucker, Rosenthal, Tatlock, Willsey) — Datalog + equality saturation hybrid. Earlier: SimplifyCC e-graphs (Nelson 1980), Z3's congruence-closure module.

### Conceptual model
E-graph = union-find over e-classes + congruence-closed e-nodes. Equality saturation: repeatedly apply rewrite rules (LHS → RHS) until no new equalities or budget exhausted, then extract minimal-cost term per cost function. E-class analyses propagate semantic lattice values (constants, types) bottom-up. egglog adds relational pattern matching and Datalog-style rules.

### Expressiveness/Semantics
Equational rewriting over many-sorted algebras. Undecidable for unrestricted rules (word problem); equality saturation explicitly bounded by iteration limit or e-graph size. Sound but incomplete (extraction always returns *some* equivalent term, never *the* minimum unless saturation reached).

### Composability/Modularity
egg's `Language` + `Analysis` + `Rewrite` traits allow domain-specific extensions. Multiple e-graphs can be intersected via shared canonicalization. egglog modules compose via Datalog-style rule sets.

### Suitability for autoformalization to IR
Excellent for the convergence/idempotency goal. Equality saturation explicitly computes a canonical congruence-closed representative — multiple LLM autoformalization runs producing semantically equivalent but syntactically distinct outputs will be unified within the e-graph. This is exactly the "semantic convergence" property required.

### Formal verification potential
Egg's union-find and rebuilding proven correct (Willsey et al. POPL 2021). Proof production: egg supports proof extraction (rewrite chains as derivation sequences). No standardized proof format. egglog inherits Datalog provenance.

### Tooling/Ecosystem maturity
egg ~1.7k GitHub stars (egraphs-good/egg, May 2026); widely adopted in compiler/PL research (Cranelift, Herbie floating-point synthesis, SPORES tensor algebra, TASO/Tensat ML graph optimization, DialEgg for MLIR). egglog active (~730 stars at egraphs-good/egglog); competing implementations include eggcc, slotted-egraphs.

### Japan-specific considerations
Takahito Aoto (Niigata) and Yoshihito Toyama (Tohoku/Niigata) — term rewriting, confluence, modularity (Aoto, Nishida, Schöpf "Equational Theories and Validity for Logically Constrained Term Rewriting", FSCD 2024 / arXiv:2405.01174; ACP — Automated Confluence Prover by Aoto, Yoshida, Toyama). Kentaro Kikuchi (Tohoku) — nominal confluence tools (Aoto & Kikuchi, "Nominal Confluence Tool", IJCAR 2016, LNCS 9706, pp. 173–182). No direct Japanese egg/egglog adoption identified; Japanese term-rewriting community is strong on confluence/termination side rather than e-graph applications.

### Interoperability
- egglog unifies Datalog (§4) + equality saturation in one engine.
- Backend extraction can use SMT (§1) cost functions or ASP (§6) for preferred-term selection.
- Category 1: Canonicalizes alternative encodings of the same clinical rule (e.g., FHIR vs. CQL vs. ePath representations).
- Category 3: Ontology alignment as equality rewrites; e-graphs unify SNOMED/ICD/JJ1017 codes referring to identical concepts.
- Category 4: Lean/Rocq tactic backends inspired by e-graphs (e.g., `Mathlib`'s `simp` semantics); egg-style normalization in Cranelift and the Rust compiler.

### Limitations/Known issues
E-graph size can explode with prolific rules. No native handling of binders (workarounds: slotted e-graphs, nominal). Contextual equality saturation (e.g., conditional rewrites valid under path conditions) only partially supported. Extraction is NP-hard for general cost functions.

### Training data proxy
Growing rapidly post-2021. egg POPL paper ~750+ citations. Active workshop (EGRAPHS 2024, 2025). Less Stack Overflow presence; primarily research code in Rust.

## 10. PRISM / Storm for Probabilistic Policy or Risk Models

### Purpose
Probabilistic model checking — verify quantitative properties of stochastic systems (DTMC, CTMC, MDP, Markov automata, POMDPs, stochastic games) against probabilistic temporal logic (PCTL, CSL, PCTL*). Used for medical risk modeling, treatment-policy synthesis under uncertainty, screening-strategy evaluation.

### Maintainer/Standards body
PRISM — University of Oxford / University of Birmingham (Marta Kwiatkowska, Gethin Norman, David Parker). Current release: PRISM 4.10 (January 2026) with the patch release PRISM 4.10.1 (April 2, 2026); PRISM-games 3.2.2 with UMB (Unified Markov Binary) format. ETAPS 2024 Test-of-Time Tool Award. Storm — RWTH Aachen (Joost-Pieter Katoen, Christian Dehnert, Sebastian Junges, Matthias Volk, Tim Quatmann). JANI as common modeling-language interchange format (Budde et al.). QComp competition compares probabilistic model checkers.

### Conceptual model
DTMC: discrete-time transition probability matrix. CTMC: exponential rate matrix. MDP: nondeterministic + probabilistic actions; policy/scheduler resolves nondeterminism. Markov automata add exponentially distributed delays. PCTL: P_op p [φ U≤k ψ] kind formulas; CSL adds continuous-time bounds; PCTL* adds nested path operators. Storm and PRISM use value iteration, interval iteration, Gauss-Seidel, policy iteration, topological decomposition, BDD/MTBDD symbolic engines.

### Expressiveness/Semantics
Probabilistic reachability on finite MDPs in P (via LP). PCTL model checking PSPACE-complete in general. Storm and PRISM handle ~10⁷–10⁹ state spaces with symbolic engines. POMDPs undecidable for many properties; Storm has heuristic POMDP support. Parametric Markov models via parameter synthesis (PARAM, Storm-pars).

### Composability/Modularity
PRISM language: modules with synchronization labels. JANI as universal exchange. Storm input: PRISM, JANI, GSPN, DFT, explicit, probabilistic programs. Properties in separate `.pctl` files.

### Suitability for autoformalization to IR
Modeling language is concrete and parameterizable. LLMs can produce PRISM/JANI from clinical risk descriptions (transition probabilities, reward structures). Idempotency depends on canonical state-variable ordering; symbolic engine ordering can affect performance but not result.

### Formal verification potential
Numerical results have controllable error bounds (interval iteration provides certified intervals). Counterexample generation via SMT or MILP (Storm). No DRAT-level certificate but witness paths exportable.

### Tooling/Ecosystem maturity
PRISM ~30+ years mature; Storm 8+ years and growing rapidly. QComp benchmark suite. Storm's stormpy Python API enables tool integration. PRISM-games handles stochastic two-player games.

### Japan-specific considerations
Ichiro Hasuo (NII Group MMM) — compositional probabilistic model checking with string diagrams of MDPs (Watanabe, Eberhart, Asada, Hasuo, CAV 2023; Pareto curves TACAS 2024); category-theoretic foundations. Kohei Suenaga (Kyoto) — probabilistic programs, hybrid-system verification (Hasuo, Oyabu, Eberhart, Suenaga, Cho, Katsumata, JLAMP Jan. 2024). Masaki Waga (Kyoto) — probabilistic black-box checking via active MDP learning. Note: Taisuke Sato (Tokyo Tech emeritus / AIST) developed a *different* tool also named PRISM — a probabilistic logic-programming language, distinct from the Kwiatkowska/Norman/Parker model checker.

### Interoperability
- Apalache (§8) and Storm both consume Z3 (§1).
- Reward-structure synthesis can feed MaxSAT (§2) or CP-SAT (§7) for policy optimization.
- Category 1: CDS-Hooks risk-score calculation can be backed by probabilistic models; ePath probabilistic variants.
- Category 2: MDPs over PMDA drug-event statistics for adverse-event prediction.
- Category 4: TLA+/Apalache for safety; PRISM/Storm for quantitative properties. Some probabilistic-program logics formalized in Coq/Isabelle (Iris-MarkovChains, Coquelicot stochastic).

### Limitations/Known issues
State-space explosion for high-dimensional patient state. Continuous-state stochastic hybrid systems not natively supported (use ProbReach, Modest). POMDPs intractable for exact verification. Probabilistic-CTL alone cannot express expected-utility comparisons across policies (needs reward extensions).

### Training data proxy
Moderate. PRISM tutorial well-indexed (prismmodelchecker.org). Storm documentation thorough. Less Stack Overflow than SMT/SAT. QComp benchmark reports. Active research at CAV, TACAS, QEST.

## 11. Prolog and s(CASP) for Goal-Directed Clinical-Rule Execution with Justification Trees

### Purpose
General-purpose logic programming with SLD/SLDNF resolution as the operational semantics, extended (via s(CASP)) to goal-directed predicate Answer Set Programming with constructive negation, abduction, and human-readable justification trees. Used as a top-down, query-driven complement to the bottom-up Datalog (§4) and grounded ASP (§6) engines: the same clinical rule base can be both materialised (for batch conformance) and queried goal-directed (for patient-specific "why?" explanations at the point of care).

### Maintainer/Standards body
ISO/IEC 13211-1:1995 (Prolog Part 1: General core) and ISO/IEC 13211-2:2000 (Modules), maintained by ISO/IEC JTC1 SC22 WG17. Reference open implementations: SWI-Prolog (Jan Wielemaker, CWI / VU Amsterdam; current stable 10.0 series, with 10.1.x development line as of April 2026; the de-facto research/industry standard with web-services, tabling, CHR, CLP, and Pengines, now featuring native GUI tools for Linux/Wayland or X11, macOS Cocoa, and Windows Win32 via SDL3/Cairo/Pango, a substantially faster WASM build, and 10–30% performance improvements in clause indexing and compilation); SICStus Prolog (RISE / SICS, commercial; Mats Carlsson; mature constraint libraries); Scryer Prolog (Markus Triska, modern ISO-conformant in Rust); GNU Prolog (Daniel Diaz); Tau Prolog (browser/Node JS, José Riaza); Trealla Prolog (Andrew Davison). XSB (Stony Brook; David Warren, Terrance Swift; tabled WAM). B-Prolog / Picat (Neng-Fa Zhou). Constraint Handling Rules (CHR) standardised informally via Thom Frühwirth's reference (KU Leuven, then Ulm; *Constraint Handling Rules*, Cambridge 2009). s(CASP): Joaquín Arias, Manuel Carro (UPM/IMDEA), Elmer Salazar, Gopal Gupta (UT Dallas) — ICLP 2018, TPLP 2018, ongoing through 2024. Active community via the Prolog Wiki (prolog.org), Association for Logic Programming (ALP), ICLP conference.

### Conceptual model
Horn clauses `Head :- Body` with negation-as-failure (`\+`) and cut (`!`); SLD-resolution refutation as goal evaluator. Operational extensions: SLG-resolution for tabling (XSB, SWI), Constraint Logic Programming (CLP(FD) finite domains, CLP(R)/CLP(Q) real/rational, CLP(B) Boolean) — solver interleaved with unification; CHR (forward-chaining, committed-choice multi-headed rules over a constraint store, Turing-complete); definite clause grammars (DCGs) as syntactic sugar for difference-list parsing. s(CASP) (Arias, Carro, Salazar, Gupta) is a goal-directed, top-down implementation of predicate ASP under stable model semantics with constructive negation (`not p(X)` over uninstantiated variables) — avoids the grounding bottleneck of §6 clingo and emits per-query justification trees (a partial proof restricted to literals relevant to the queried goal). Co-induction (`% coinductive`) supports cyclic dependencies (loops in arguments, regulatory cycles).

### Expressiveness/Semantics
Pure Prolog is Turing-complete (function symbols + recursion); SLD-resolution is sound and refutation-complete for pure Horn clauses, but SLDNF is unsound on non-stratified negation. Tabled SLG (Chen & Warren, JACM 1996) computes the well-founded model in polynomial time for Datalog-with-negation, restoring soundness. s(CASP) computes stable models goal-directed: PSPACE-hard in worst case but typically terminates fast on the small "relevant fragment" of a query — Arias et al. (TPLP 2018) report tractable runtimes on benchmark suites where grounded ASP exhausts memory. CHR is Turing-complete (Sneyers et al., TPLP 2010). CLP(FD): NP-complete (over finite domains); CLP(R): polynomial via simplex.

### Composability/Modularity
Modules per ISO 13211-2 with `use_module/1`, `module/2`. SWI-Prolog packs (`pack_install/1`) catalog (~600 packs as of 2025). DCGs compose grammars naturally. CHR rule sets compose monotonically with shared constraint stores. s(CASP) programs compose like ASP (predicate signatures stable across modules). Foreign-language interfaces: C (SWI's `PL_*` API), Java (JPL), Python (`pyswip`, `janus_swi`), .NET. Tabling and CLP add orthogonal layers without changing the source grammar.

### Suitability for autoformalization to IR
High for the *executable* face of a clinical rule IR. Prolog is exceptionally well-represented in LLM training data (decades of textbooks: Bratko, Sterling-Shapiro, Covington; CS curricula; SWISH notebooks) — top-tier models emit syntactically valid Prolog with high probability. s(CASP) is less represented (research code), but its surface syntax is near-identical to standard ASP, so few-shot prompting transfers cleanly from §6 ASP examples. Goal-directed evaluation gives a natural mapping for "should patient X receive Y?" CDS queries: the IR is a knowledge base, the question is a goal, the answer is a justification tree directly displayable to a clinician (s(CASP)'s `?- p(X), justify.` query mode). Idempotency across LLM runs is enhanced when the clause head signature and ordering are constrained (linter pass: canonical argument order, alphabetic clause order).

### Formal verification potential
SLD-resolution soundness/completeness is textbook (Lloyd, *Foundations of Logic Programming*, Springer 1987). Well-founded semantics for tabled SLG is the canonical model — soundness proven (Van Gelder, Ross, Schlipf, JACM 1991). s(CASP)'s soundness w.r.t. stable model semantics proven by Arias et al. (TPLP 2018). The justification tree IS the proof certificate — each successful s(CASP) query returns the literal-level derivation, which is independently checkable (Arias, Carro, Chen, Gupta — "Justifications for Goal-Directed Constraint Answer Set Programming", ICLP 2020 Technical Communications / EPTCS). For pure Prolog: Isabelle/HOL formalisation of WAM compiler correctness (Pusch, TPHOLs 1996); Bohrer & Crary's Typed WAM (TWAM), a certifying abstract machine for logic programs with Coq formalization (VSTTE 2018); ProB (Leuschel, Düsseldorf) is a B-method tool with Prolog substrate that has been used for guideline verification. Verification of Prolog *implementations*: SWI's interpreter not formally verified, but its 30-year deployment history and large CHR/CLP test suites give engineering-grade assurance.

### Tooling/Ecosystem maturity
SWI-Prolog: very mature, active (10.0 stable / 10.1.x dev as of April 2026). Built-in HTTP server, JSON, web-IDE (SWISH, swish.swi-prolog.org), tabling, CLP, CHR, RDF/OWL libraries (`semweb`), unit-test framework (PlUnit), profiler, GUI debugger. SWISH supports executable papers and was used to publish the Probabilistic Logic Programming tutorial (Riguzzi et al.). s(CASP): GitHub `SWI-Prolog/sCASP` (vendored into SWI 9.x+ as `library(scasp)`); active development, ICLP tutorial track. SICStus Prolog: commercial but long-lived (since 1985); used in aerospace (Mercury Mission Operations) and transportation scheduling. Scryer Prolog: modern, fast on CLP(FD); growing user base. ProB has direct medical pedigree (Asbru / GLIF interpreters). Logtalk (Paulo Moura) — object-oriented layer running on SWI/SICStus/Scryer/XSB. Most engines are POPLmark/TPTP-tested.

### Japan-specific considerations
**Foundational.** The Japanese Fifth Generation Computer Systems project (FGCS, 第五世代コンピュータ; 1982–1992) at ICOT (新世代コンピュータ技術開発機構; director Kazuhiro Fuchi 渕一博) made Prolog and concurrent logic programming national R&D priorities, producing Concurrent Prolog (Ehud Shapiro at Weizmann, co-developed with ICOT), GHC (Guarded Horn Clauses; Kazunori Ueda, ICOT), Flat GHC, and KL1 (Kernel Language 1; the FGCS final-stage language) running on the PIM (Parallel Inference Machine) hardware. Although the FGCS commercial goals went unmet, the project trained a generation of Japanese logic-programming researchers and produced lasting artifacts: ICOT released KL1, KLIC (KL1-to-C compiler), and the Multi-VPIM emulator into open source via the AITEC archive. Active descendants: Kazunori Ueda (Waseda) — LMNtal (Linked Multi-set Nonlinear hierarchical Term, *TPLP* 2009 and subsequent), a successor concurrent-LP language with a verified model checker (SLIM). Ken Satoh (NII) — PROLEG over Prolog (research06.md §1), the canonical Japanese legal-reasoning Prolog system; ongoing JURISIN/PROLALA tutorials. Hidetomo Nabeshima (Yamanashi) — abductive Prolog. Naoki Kobayashi (Univ. of Tokyo) — higher-order model checking and CHC/Horn-clause solving over Prolog-style relational specifications. Chiaki Sakama (Wakayama emeritus) — extensive Prolog/ASP semantics work. Mutsunori Banbara (Nagoya) — CLP and ASP. Annual JSAI (人工知能学会) sessions still include LP tracks.

### Interoperability
- Bridges naturally to §4 Datalog: pure Datalog is the function-free Horn fragment of Prolog; tabled SWI Prolog is a viable Datalog engine, and Soufflé-style rules can be transliterated.
- Bridges to §6 ASP: s(CASP) IS predicate ASP, goal-directed; ASP programs round-trip via clingo's textual format.
- Bridges to §12 Probabilistic Logic Programming: ProbLog, cplint, PRISM (Sato) all sit on top of Prolog and reuse its solver; sharing the same IR backbone is straightforward.
- Bridges to §10 PRISM/Storm: probabilistic Prolog frontends compile to factor graphs that can be re-targeted to Storm DTMCs.
- Bridges to §1 SMT: CLP(FD/Q/R) and PrologCHR can call SMT backends; SWI's `library(clp)` includes a Z3 binding (`pl-z3`, Steven Schäfer / Tom Schrijvers).
- Category 1: Arden Syntax MLM compilers exist in Prolog (historical, Erasmus MC). PROforma (OpenClinical) was implemented in Prolog (Tallis engine, John Fox / Cancer Research UK). Asbru's AsbruView and IDAN had Prolog-based execution engines.
- Category 6: defeasible-logic translation to Prolog is canonical (Antoniou-Maher; SPINdle generates Prolog-style rule lists); Event Calculus reasoners (RTEC, jREC) are Prolog or compile to it.
- Category 3: SWI-Prolog has a long-lived `semweb` library — RDF/OWL ingest, SPARQL, ClioPatria triple store; Prolog-driven SWRL execution is one path for hybrid rule + ontology reasoning.

### Limitations/Known issues
SLDNF unsound on non-stratified negation (use tabled SLG or s(CASP)). Cut (`!`) breaks declarative reading; many production rule sets are de-facto procedural. Termination is generally undecidable; Prolog programmers rely on idiom (occur-check disabled by default — a 1980s performance compromise that can lead to incorrect unification under recursive structures). The "left-to-right depth-first" search order makes naive Prolog brittle to clause ordering — different orderings yield different runtimes (or non-termination). For clinical guideline use, this is a real risk: an LLM-emitted Prolog program may be logically correct but operationally diverge on the patient case at hand. Mitigations: tabling, mode declarations, occurs-check enabled (`set_prolog_flag(occurs_check, true)`), or migration to s(CASP) for non-procedural semantics. Industrial CDS deployments running Prolog are rare (Java/CQL/DMN dominate); finding clinicians who can read and maintain Prolog rule bases is a staffing risk.

### Training data proxy
Strong. Prolog has been continuously taught since 1972 (Colmerauer, Marseille); Bratko's *Prolog Programming for Artificial Intelligence* (Pearson, 4th ed. 2011) and Sterling & Shapiro's *The Art of Prolog* (MIT Press, 2nd ed. 1994) are in nearly every LLM's training set. SWI-Prolog GitHub ~1.2k stars; Scryer ~2.4k stars (modern resurgence). Stack Overflow `prolog` tag: ~14,000 questions. SWISH (swish.swi-prolog.org) public notebook gallery. ICLP 1984–present; *Theory and Practice of Logic Programming* (CUP, since 2001). s(CASP) tutorials at ICLP 2020–2024 and SWI documentation. Smaller than SMT/ASP in *recent* arXiv volume but a deeper historical canon — LLMs reliably generate idiomatic Prolog, including correct CLP(FD) use.

## 12. Probabilistic Logic Programming: ProbLog, cplint, PRISM (Sato), DeepProbLog

### Purpose
Augment logic programs with probabilities on facts (or rules) so that the answer to a query is a probability distribution over derivations. Provides a *single* language for combining symbolic clinical knowledge ("antibiotic A is effective against pathogen B") with epistemic uncertainty ("with probability 0.07 the patient has a contraindicating allergy", "the lab test has sensitivity 0.92") and exact inference of marginal/MPE probabilities. This is the natural answer to the "no probabilistic strength" gap in defeasible logic (Category 6 §1) and to the "no native uncertainty" gap in classical Event Calculus (Category 6 §7).

### Maintainer/Standards body
No standards body; the field is academic-driven. Foundational distribution semantics: Taisuke Sato, "A statistical learning method for logic programs with distribution semantics," ICLP 1995 — the seminal paper. Sato & Yoshitaka Kameya, "Parameter Learning of Logic Programs for Symbolic-Statistical Modeling," *Journal of Artificial Intelligence Research* 15:391–454 (2001), and Sato & Kameya, "New Advances in Logic-Based Probabilistic Modeling by PRISM," in *Probabilistic Inductive Logic Programming* (LNCS 4911), Springer, pp. 118–155 (2008) — PRISM (PRogramming In Statistical Modeling), distinct from the Kwiatkowska/Norman/Parker model checker (§10). David Poole's Independent Choice Logic (ICL, AIJ 1997) is an equivalent earlier formulation. **ProbLog**: Luc De Raedt, Angelika Kimmig, Hannu Toivonen, "ProbLog: A probabilistic Prolog and its application in link discovery," IJCAI 2007. **ProbLog2** (current, 2.2.x series; latest 2.2.9 released September 23, 2025): Anton Dries, Angelika Kimmig, Wannes Meert, Joris Renkens, Guy Van den Broeck, Jonas Vlasselaer, Luc De Raedt, ECML/PKDD 2015 — knowledge-compilation-based exact inference via SDD/d-DNNF. Maintained by KU Leuven DTAI (`dtai.cs.kuleuven.be/problog`; GitHub `ML-KULeuven/problog`). **cplint** / `pita` / `mcintyre`: Fabrizio Riguzzi (University of Ferrara), `cplint` system from ~2007, integrated with SWI-Prolog SWISH (cplint.eu). Riguzzi's textbook *Foundations of Probabilistic Logic Programming* (River Publishers, 2nd ed. 2022) is the standard reference. **CP-logic** / **LPADs**: Joost Vennekens, Marc Denecker, Maurice Bruynooghe (KU Leuven). **DeepProbLog**: Robin Manhaeve, Sebastijan Dumančić, Kimmig, Thomas Demeester, De Raedt, "DeepProbLog: Neural Probabilistic Logic Programming," NeurIPS 2018 (extended *AIJ* 2021). **aProbLog** (algebraic): Kimmig, Van den Broeck, De Raedt, AAAI 2011 — semiring-parameterised inference. **DC-ProbLog** / Hybrid ProbLog: Davide Nitti et al. for continuous random variables. ICLP, PLP workshop (annual), StarAI workshop (statistical+relational AI) are the main venues.

### Conceptual model
Sato's distribution semantics: a probabilistic logic program is a pair `(F, R)` of probabilistic facts `p :: f` (each `f` independently true with probability `p`) and a definite or normal logic program `R`. Each total choice over `F` yields a sampled "world", and the probability of a query `q` is the sum of probabilities of worlds in which `R ∪ chosen_F ⊨ q`. ProbLog computes this as Weighted Model Counting (WMC) over the Boolean formula whose models are the worlds entailing `q` — typically compiled to a Sentential Decision Diagram (SDD; Adnan Darwiche, UCLA) or d-DNNF and evaluated bottom-up in linear time in the circuit size. CP-logic / LPADs allow probabilistic *choices in rule heads* `(h1:p1 ; h2:p2 ; ...) :- body.` (analogous to disjunctive ASP with probabilities). DeepProbLog inserts neural network predicates as `nn(net, X, Y, Domain) :: f(X, Y).` whose probabilities are network outputs; gradients flow through the SDD via algebraic-ProbLog semirings, enabling end-to-end training. Algebraic ProbLog generalises to arbitrary commutative semirings (max-plus → MPE/MAP; fuzzy aggregation; provenance polynomials).

### Expressiveness/Semantics
Inference complexity: WMC is #P-complete in general, but exact via SDDs is linear in SDD size (which can be exponentially smaller than the propositional theory). PROBLOG queries with finitely many proofs are decidable; with infinite proofs, stratification and tabling are needed. PRISM (Sato) requires the *exclusive-explanation* assumption for efficient EM learning (otherwise mutually exclusive worlds must be enforced manually); ProbLog relaxes this at the cost of compiling to a richer circuit. Approximate inference: k-best proofs (top-k explanations), Monte Carlo sampling (`mcintyre`), bounded approximation (Poole), lifted inference (Van den Broeck, AAAI 2011 — for first-order PLPs with large populations). Parameter learning: EM (Sato-Kameya FAM, Riguzzi's EMBLEM), gradient descent (DeepProbLog), Learning from Interpretations (LFI). MAP/MPE queries via algebraic ProbLog with max-product semiring.

### Composability/Modularity
ProbLog inherits Prolog's module system. ProbLog programs can include CLP constraints (DC-ProbLog, hybrid). Neural predicates in DeepProbLog are first-class — networks defined in PyTorch, registered to the ProbLog interpreter via `nn/4` declarations. cplint composes via SWI module system, with the rest of the SWI-Prolog ecosystem (`semweb`, `tabling`, CHR). aProbLog's semiring abstraction allows swapping inference modes without rewriting the program. Sato's PRISM is more monolithic — model file + parameter file — but composes via library predicates.

### Suitability for autoformalization to IR
Moderate-high, with caveats. LLM training-data presence is thinner than for plain Prolog or ASP — KU Leuven tutorials and cplint SWISH notebooks are the main public corpus. Few-shot prompting from a small ProbLog primer plus the target rule pattern is effective for GPT-5.5 / Claude Opus 4.7 / Gemini 3.1 Pro (Category 8). The IR target sweet spot is *clinical-fact + probability + rule*: `0.12 :: contraindication(penicillin, X) :- documented_allergy(X, beta_lactam).` Map directly to (a) FHIR `AllergyIntolerance.verificationStatus.criticality` weights, (b) GRADE certainty bands (High/Moderate/Low/Very Low → {0.95, 0.7, 0.4, 0.15} or learned anchors). Constrained decoding (Category 8 §4) over a JSON-Schema'd PLP IR with `{probability: float, fact: string}` envelopes gives idempotency across LLM runs.

### Formal verification potential
Distribution semantics has clean model-theoretic foundations (Sato 1995; Riguzzi textbook). SDD/d-DNNF inference is deterministic and proof-producing: the compiled circuit IS the explanation of the probability. ProbLog can output the d-DNNF (or its arithmetic-circuit form), which is then checkable by an independent circuit evaluator. Soundness of EM parameter learning under exclusive-explanation assumption proven (Sato-Kameya 2001). DeepProbLog gradients have not been formally certified, but the symbolic SDD layer is independent of the neural layer — symbolic verification is preserved when neural predicates are abstracted to their probability outputs. No mechanized proof in Lean/Coq of the full ProbLog stack (open problem). Cross-checking with §5 OWL reasoners or §6 ASP is possible by stripping probabilities (purely-logical residual must remain consistent).

### Tooling/Ecosystem maturity
**ProbLog** (KU Leuven): GitHub `ML-KULeuven/problog`, ~407 stars (May 2026); web playground `problog.cs.kuleuven.be`; Python and SWI bindings; SDD via Wmc/`SDD` library (Darwiche group); active through 2025 with NeSy workshop ties (latest release 2.2.9, September 23 2025). **cplint** (Riguzzi, UniFE): SWISH integration (cplint.eu); textbook code; PITA (Probabilistic Inference modulo Theories) integrates Z3. **PRISM (Sato)**: distributed via AIST archive; less actively maintained as standalone but the *semantics* are influential. **DeepProbLog**: GitHub `ML-KULeuven/deepproblog`; PyTorch-based; NeurIPS demos. **DeepStochLog** (Winters, AAAI 2022): probabilistic stochastic grammars over PLP. **PLP-on-ASP**: `pasp` (Riguzzi/cplint dialect on clingo). **smProbLog** (Totis, Kimmig, De Raedt, *TPLP* 2023): stable-model ProbLog for negation. **ProbEC** (Skarlatidis, Paliouras, Artikis et al., NCSR Demokritos, *TPLP* 2015): Probabilistic Event Calculus implemented over ProbLog — directly applicable to noisy clinical event streams (see Category 6 §7). Stable, growing; community concentrated at KU Leuven, UniFE, NCSR Demokritos, UT Dallas, UCLA.

### Japan-specific considerations
**Origin point.** Sato Taisuke (佐藤泰介, Tokyo Tech emeritus, currently AIST AIRC / Tokyo Tech honorary professor) is the original architect of distribution semantics — PRISM (1995–) predates ProbLog by 12 years and is acknowledged as the foundation in every ProbLog paper. Yoshitaka Kameya (亀谷由隆, formerly Tokyo Tech / Nagoya Institute of Technology, now Meijo University) co-developed PRISM and contributed the FAM-EM learning algorithm (Sato-Kameya, JAIR 2001). The Japanese AI community has retained interest in PLP through: Katsumi Inoue (NII) — abductive logic programming, LFIT (Learning from Interpretation Transition), foundational to PLP parameter learning; Chiaki Sakama (Wakayama emeritus) — probabilistic stable-model semantics, brave/cautious induction; Hidetomo Nabeshima (Yamanashi) — probabilistic abduction. JSAI (人工知能学会) and IPSJ (情報処理学会) host PLP sessions sporadically. PRISM has been applied to biological pathway probabilistic modelling by Sato's group (HMMs, PCFGs, Bayesian networks all expressible in <20 lines of PRISM code), and the same techniques transfer to clinical pathway probabilistic CDS — though no Japan-originated industrial clinical PLP product is publicly documented.

### Interoperability
- §11 Prolog/s(CASP): ProbLog and cplint are *built on* SWI-Prolog; non-probabilistic residual is a Prolog program; explanations are Prolog proof terms with weights.
- §6 ASP: smProbLog extends ProbLog to ASP with stable-model semantics; PASOCS (Tuckey, Russo, Broda) probabilistic ASP solver.
- §10 PRISM/Storm: ProbLog programs over finite-horizon, propositional fragments can be compiled to DTMCs/MDPs for PRISM (Kwiatkowska) — bidirectional model exchange via JANI is research-grade but feasible.
- §1 SMT: PITA (cplint) integrates Z3 for probabilistic inference over numeric constraints; key for continuous-valued clinical observations (eGFR, BMI, INR).
- §9 E-Graphs: equality saturation over PLP terms is unexplored but conceptually compatible.
- Category 1: GRADE certainty bands and CDS-Hooks risk-score outputs map to ProbLog probabilistic facts; FHIR ImmunizationEvaluation / RiskAssessment resources are natural sinks.
- Category 2: PMDA adverse-event statistics (JADER) are population-frequency data ready for ProbLog parameterisation.
- Category 6 §1 Defeasible Logic: ProbLog answers the "no probabilistic strength" pitfall — defeasible rules get explicit probabilities; superiority `>` can be reconstructed as conditional probability.
- Category 6 §7 Event Calculus: ProbEC (Skarlatidis et al., TPLP 2015) directly enables uncertain longitudinal CDS.
- Category 8 §9: ProbLog is a natural target for Program-Aided Language Models — the LLM emits PLP code, an interpreter computes the probability.

### Limitations/Known issues
Inference cost: WMC is #P; SDD compilation can blow up on programs with many disjunctive proofs (the "intersection-of-proofs" overhead). Approximate-inference quality is hard to certify for safety-critical CDS. Parameter elicitation: clinicians provide point probabilities (or GRADE bands) but rarely full joint distributions — independent-fact assumption can be wrong (correlated allergies, comorbidities). The distribution-semantics independence assumption requires explicit conditioning to capture correlated risk factors. LLM-generated PLP needs validation that probabilities sum to a coherent measure (no implicit double-counting). Continuous variables (eGFR, age) require Hybrid ProbLog / DC-ProbLog, which are less mature. DeepProbLog training is slow vs. pure neural baselines (offset by interpretability). Lifted inference for large patient populations is research-grade. No clinical regulatory precedent (FDA/PMDA SaMD) for PLP-based CDS as of 2026.

### Training data proxy
Moderate. ProbLog tutorial pages and the Riguzzi textbook are well-indexed. ProbLog GitHub ~407 stars (May 2026); DeepProbLog active at `ML-KULeuven/deepproblog`; cplint web (cplint.eu) and SWISH notebooks publicly browsable. *TPLP* ProbLog/cplint papers are open-access. ICLP/PLP-workshop archive on probabilistic-logic-programming.org. Stack Overflow PLP tag is small (<200 questions) — LLM emission benefits from few-shot prompting with curated examples from the De Raedt / Riguzzi tutorials. Sato's foundational papers (1995, 2001, 2008) are in every LLM training set. DeepProbLog at NeurIPS gives a recent visibility boost.
