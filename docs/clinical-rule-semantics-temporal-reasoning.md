# Category 6: Clinical Rule Semantics, Temporal Reasoning, and Argumentation

## 1. Defeasible Logic for Exceptions and Weak/Strong Recommendations

### Purpose
Encode rules-with-exceptions and graded recommendation strength (strict vs defeasible rules, defeaters, superiority relation); clinical "should/may/consider" maps to formal weak/strong defeats.

### Maintainer/Standards body
No standards body. Foundational: Donald Nute (Univ. of Georgia, 1980s–90s). Dominant variant (DL, DDL) maintained by Guido Governatori (formerly Data61/CSIRO, now Central Queensland University), with Antonio Rotolo (Bologna), Francesco Olivieri (Griffith), Michael Maher.

### Conceptual model
Rule theory `(F, R→, R⇒, R~>, >)`: facts, strict rules `→`, defeasible rules `⇒`, defeaters `~>`, binary superiority `>` over rules. Proof tags `+Δ, –Δ` (definite), `+∂, –∂` (defeasible). Ambiguity-blocking vs ambiguity-propagating variants. Modal/temporal extensions add OBL, PERM, FORB (Defeasible Deontic Logic, DDL).

### Expressiveness/Semantics
Skeptical, non-monotonic, sub-classical. Propositional DL linear-time decidable (Maher, *TPLP* 1(6):691–711, 2001). Captures contrary-to-duty, reparation chains, hierarchical normative positions.

### Composability/Modularity
Rule sets compose monotonically syntactically; semantic interaction governed by `>`. Modular addition of modalities (OBL, INT, BEL) via Governatori-Rotolo conversion rules. Theory union may require recomputing superiority closure.

### Suitability for autoformalization to IR
High. If-then-with-exceptions-and-priorities syntax aligns with guideline prose; LLM-targeted IR can emit rules with explicit `defeasible` / `strict` / `defeater` tags and `>` annotations. Bounded rule grammar gives stabler cross-run lexicalization than full FOL.

### Formal verification potential
Decidable; polynomial propositional entailment enables exhaustive contradiction detection across two guideline rule bases. Modal/temporal DDL costlier but decidable for fragments.

### Tooling/Ecosystem maturity
- `SPINdle` (Lam & Governatori, "The Making of SPINdle," RuleML 2009) — Java, scales to >1M rules, XML I/O, standard + modal; latest release v2.2.4 on SourceForge.
- `Delores` (Maher et al.), `DR-DEVICE` (Bassiliades et al., RDF/Jena).
- `Regorous` (Governatori) — commercial DDL compliance checker; Amantea & Governatori, "Automated Compliance Checking for Medical Rosters," *Studies in Health Technology and Informatics* 327:723–727 (MIE 2025) — direct clinical compliance precedent.
- Modest GitHub footprint; SPINdle last active ~2015.

### Japan-specific considerations
PROLEG (Ken Satoh, NII): closely related exception-based reasoning for Japanese civil-code "Yoken-jijitsu-ron" (presupposed ultimate facts theory) over Prolog with rule/exception pairs; not formally identical to Nute-DL but interoperable in spirit. Satoh, Fungwacharakorn et al. (JURISIN, ICAIL 2021–2025) extend PROLEG: NL interfaces, LLM-driven normative extraction (Vienna VCLA talk, Nov 2024). Minds (JCQHC) guideline methodology uses GRADE narrative, not DL; no Japanese clinical DL toolchain identified.

### Interoperability
- IR/ontology: maps cleanly to RDF/OWL rule layer; SHACL violations can seed defeaters.
- DMN: defeasible rules ≈ decision tables with override hit policies; lossy.
- FHIR/CQL: CQL expressions populate rule antecedents; CQL lacks native exception priority.
- ASP/Datalog: DL translatable to stable-model ASP (Antoniou et al.); ideal back-end.
- Prolog / s(CASP) (Category 5 §11): goal-directed defeasible reasoning with justification trees is the canonical execution path; SPINdle exports Prolog-like rule lists; d-Prolog (Nute) was the original implementation. s(CASP) constructive negation handles the `not-applicable` distinction.
- Probabilistic Logic Programming (Category 5 §12): ProbLog / cplint / CP-logic add weighted facts atop defeasible rules (fixes the no-probabilistic-strength pitfall below); conditional-probability tables on rule heads encode GRADE certainty bands.
- Lean/Isabelle: DL meta-theory partially formalized (Maher); rule application not natively certified.

### Limitations/Known issues
Superiority elicitation brittle; ambiguity-blocking vs ambiguity-propagating semantics diverge on the same theory. No native probabilistic strength — `>` is ordinal; for quantitative weighting (GRADE bands as probabilities) overlay Probabilistic Logic Programming (Category 5 §12). Quantification limited (mostly propositional / function-free).

### Training data proxy
Moderate. Governatori corpus (governatori.net, ResearchGate) in LLM training. Conferences: RuleML+RR, JURIX, ICAIL, DEON, COMMA. SPINdle paper substantially cited though no precise count retrievable from indexed databases.

## 2. Deontic Logic for Obligations, Prohibitions, and Permissions

### Purpose
Formalize normative guideline content — *obligatory* (O), *permitted* (P), *forbidden* (F) — distinct from what is *true*. Handles contrary-to-duty (CTD) scenarios common in clinical exceptions ("if drug X contraindicated, then …").

### Maintainer/Standards body
None official. SDL: von Wright (1951). Dyadic deontic logic: Hansson, Lewis, Åqvist. Input/Output (I/O) logic: David Makinson & Leendert van der Torre, *J. Philosophical Logic* 29(4):383–408 (2000); 30(2):155–185 (2001); 32(4):391–416 (2003). Jones-Sergot agency. Governatori for computational DDL.

### Conceptual model
SDL: modal KD over `O`, with `Pp ≡ ¬O¬p`, `Fp ≡ O¬p`. Dyadic `O(B|A)`: "in context A, B is obligatory". I/O: norms as ordered pairs `(a, x)`; out₁..out₄ operators differ on reusability and reflexivity. Jones-Sergot adds agency operator `E`.

### Expressiveness/Semantics
Possible-worlds Kripke semantics for SDL. I/O has *no* truth-functional embedding — norms are pre-logical; suits norm bases that should escape logical paradoxes (Ross, free-choice, gentle-murderer, Chisholm).

### Composability/Modularity
I/O highly modular: norm sets are simple relations; combination = set union; consistency via constrained out operators. SDL composition tends to trigger CTD paradoxes (deontic explosion).

### Suitability for autoformalization to IR
High for tagged IR: each recommendation lexicalizes as `(context, action, modality)` tuple; typed triple store with `O/P/F` markers compiles to SDL boxes or I/O pairs. Restricting to dyadic norms `(antecedent, deontic-consequent)` improves cross-run idempotency.

### Formal verification potential
SDL decidable (PSPACE-complete for normal modal logics over propositional base). I/O decidability depends on base logic; constrained variants decidable. Cross-guideline conflict detection = inconsistency in `out(G ∪ G′, A)`. Empirical study (Lomotan et al., *Qual Saf Health Care* 2010 / PMC2982946): wide clinician variability interpreting deontic terms ("must" median = 100, "may" = 37 on 0–100 obligation scale) — motivates formal disambiguation.

### Tooling/Ecosystem maturity
LogiKey (Benzmüller, Parent, van der Torre, 2018+) embeds deontic logics in Isabelle/HOL via shallow semantical embedding — verified deontic reasoning. SDLib, MleanCoP-DL prototypes. Limited stand-alone solvers; usually combined with DL or ASP.

### Japan-specific considerations
Ken Satoh (NII) chairs PROLALA and JURISIN; PROLEG exception structure aligns with dyadic deontic readings. Chiaki Sakama (Wakayama) and Toshiko Wakaki (Shibaura, emerita): normative-flavoured logic programming. No Japan-specific clinical deontic toolchain identified.

### Interoperability
- HL7 FHIR + CQL: deontic tags annotate CQL "ActivityDefinition" recommendations; round-trip needs custom extension.
- DMN/BPMN: tasks taggable `mandatory`/`optional`; lossy.
- OWL: deontic role hierarchy expressible; quantified deontic logic not in OWL 2 DL.
- Isabelle/HOL (LogiKey), Lean: shallow embedding feasible — the certified path.
- SAT/SMT: SDL via QBF or modal-SAT translation.

### Limitations/Known issues
Classical paradoxes (Ross, Chisholm, Forrester) bite SDL; most recur in pragmatic clinical text. Prima facie vs all-things-considered obligations unresolved without combining with defeasible logic.

### Training data proxy
Strong philosophical literature (Stanford Encyclopedia; Gabbay-Horty-Parent-van der Meyden-van der Torre *Handbook of Deontic Logic and Normative Systems*, 2013–2021). DEON biennial since 1991. Lower code/GitHub footprint than ASP or ML.

## 3. Dung-Style Abstract Argumentation Frameworks

### Purpose
Attack-graph semantics for resolving conflicts among arguments (e.g., conflicting guideline recommendations) without commitment to internal argument structure.

### Maintainer/Standards body
Foundational: Phan Minh Dung, "On the acceptability of arguments and its fundamental role in nonmonotonic reasoning, logic programming, and n-person games," *Artificial Intelligence* 77(2):321–357 (1995). Community-curated via ICCMA (International Competition on Computational Models of Argumentation), biennial since ICCMA'15 (originally organised by Matthias Thimm, then Univ. Koblenz-Landau, now FernUniversität in Hagen, and Serena Villata, INRIA Sophia Antipolis; reported in Thimm & Villata, *Artificial Intelligence* 252:267–294, 2017). ICCMA'25 (sixth edition) concluded November 2025; results presented at the Arg&App workshop co-located with KR 2025 in Melbourne.

### Conceptual model
AF = `(A, R)`: A arguments, R ⊆ A × A attacks. Semantics: conflict-free, admissible, complete, grounded (unique, skeptical), preferred (maximal admissible), stable (attacks every outsider), semi-stable, ideal, eager, stage.

### Expressiveness/Semantics
Pure graph theory; directed graph + extension predicate. Complexity P (grounded) → coNP / Σ₂ᵖ (preferred, semi-stable skeptical acceptance).

### Composability/Modularity
Trivially compositional at graph level. Standard ICCMA APX/TGF/i23 formats. Extensions: bipolar AFs (support), value-based AFs (Bench-Capon), preference-based AFs, weighted AFs.

### Suitability for autoformalization to IR
Medium-low as primary IR (loses internal rule structure) — best as *downstream* semantic layer over a structured argumentation IR (ASPIC+/ABA/Carneades). Highly idempotent: argument identifiers + attack edges form a canonical graph.

### Formal verification potential
Credulous/skeptical acceptance are standard decision problems; certified solvers exist. Contradiction detection = empty stable extension or asymmetric attacks across guideline sources.

### Tooling/Ecosystem maturity
- `ASPARTIX` / `ASPARTIX-V` (TU Wien, Dvořák/Wallner/Woltran) — ASP encodings; multiple ICCMA editions.
- `μ-toksia` (Niskanen & Järvisalo, University of Helsinki) — SAT-based; first in all 21 main-track categories and several dynamic tracks at ICCMA 2019; Niskanen & Järvisalo, "µ-toksia: An Efficient Abstract Argumentation Reasoner," KR 2020, pp. 800–804.
- `ConArg`, `pyglaf`, CoQuiAAS. ICCMA benchmark sets public.

### Japan-specific considerations
Chiaki Sakama & Tjitze Rienstra, "Representing argumentation frameworks in answer set programming," *Fundamenta Informaticae* 155(3):261–292 (2017). Katsumi Inoue (NII) — abductive/inductive argumentation links. No clinical instantiation in Japan.

### Interoperability
- OWL/SHACL: arguments → OWL individuals; attacks → object property; coarse.
- ASP/Datalog: native (ASPARTIX); ideal back-end with `clingo`.
- SAT/QBF: ICCMA solvers use SAT or QBF for Σ₂ᵖ tasks.
- Lean/Isabelle/Rocq: AF semantics formalized (e.g., Caminada labelling in Isabelle).

### Limitations/Known issues
"Abstract" — no semantics for argument content; alone detects attack patterns, not logical contradictions. Self-defeating arguments problematic. Re-grounding the same guideline twice may yield different argument IDs unless naming policy is fixed.

### Training data proxy
Very strong: 30+ years of literature; COMMA biennial, ICCMA biennial, AAAI/IJCAI/KR/ECAI yearly tracks. Dung 1995 has over 1,400 citations per ACM Digital Library (substantially more on Google Scholar) — among the most-cited nonmonotonic-reasoning papers. Substantial open-source GitHub code.

## 4. ASPIC+ / Carneades for Explainable Guideline Conflict Arguments

### Purpose
Structured argumentation: arguments as inference trees from strict + defeasible rules over a base logic, evaluated via Dung semantics. Carneades adds explicit *proof standards* and *burden of proof* — valuable for graded clinical evidence (GRADE strong vs weak).

### Maintainer/Standards body
ASPIC+: Henry Prakken (Utrecht/Groningen), Sanjay Modgil (KCL); tutorial: Modgil & Prakken, *Argument & Computation* 5(1):31–62 (2014). Carneades: Thomas Gordon (Fraunhofer FOKUS), Henry Prakken, Douglas Walton, "The Carneades model of argument and burden of proof," *Artificial Intelligence* 171(10–15):875–896 (2007). LKIF Legal Knowledge Interchange Format associated with Carneades.

### Conceptual model
ASPIC+: language L with contrariness function; strict rules `Rs` (premises guarantee conclusion), defeasible rules `Rd` (presumptive); arguments = inference trees; three attack types — *undermining* (premise), *rebutting* (defeasible conclusion), *undercutting* (defeasible inference); preferences resolve attacks to defeats. Carneades: argument graphs with statement nodes, argument nodes (pro/con), proof standards (scintilla, preponderance, clear-and-convincing, beyond-reasonable-doubt), audiences with assumptions and weights.

### Expressiveness/Semantics
ASPIC+ inherits Dung semantics; rationality postulates (closure, consistency) guaranteed under well-formedness. Carneades supports stage-based evaluation; later versions (CAES) align with Dung; Carneades 4 implemented in Go.

### Composability/Modularity
Both modular: rules + premises + preferences are separate artefacts. ASPIC+ unifies ABA, classical-logic argumentation, defeasible logic as instantiations.

### Suitability for autoformalization to IR
Very high. IR: `Argument(id, premises, rule, conclusion, type∈{strict,defeasible})` + `Pref(rule_i > rule_j)`. Carneades argumentation-scheme catalogue — 96 named schemes per Walton, Reed & Macagno, *Argumentation Schemes* (Cambridge University Press, 2008) — directly recognizable in guideline language (e.g., *argument from expert opinion*, *argument from established rule*); scheme templates increase idempotency.

### Formal verification potential
Reduces to AF reasoning (NP / Σ₂ᵖ). Inter-guideline conflict surfaces as mutual defeat without a single preferred extension containing both. Rationality postulates theorem-prover-checkable (Heyninck & Straßer, *Argument & Computation* 12(1):3–47, 2021).

### Tooling/Ecosystem maturity
- `Carneades 4` (Go), https://github.com/carneades; v4.3 (July 2017) with native Constraint Handling Rules (CHR) inference engine in Go (removes Prolog dependency); last commit activity October 2024.
- ASPIC+ engines: `EPR` (Edinburgh), `TOAST` (Snaith & Reed), Python `pyASPIC` prototypes.
- Less mature than ASPARTIX for raw AF, but uniquely supports explanation export.

### Japan-specific considerations
Direct precedent: Oliveira, Novais et al. (JSPS KAKENHI Grant JP18K18115 funding for Oliveira), "Argumentation for Reasoning with Conflicting Clinical Guidelines and Preferences," AAAI 2018 workshop — uses ASPIC-G (ASPIC+ with Goals; Modgil-Prakken 2014, Oliveira et al. 2018) and ABA+ for patient-centric reasoning over conflicting guidelines. Carneades scheme catalog interoperates conceptually with PROLEG rule/exception view (Satoh, NII).

### Interoperability
- AIF (Argument Interchange Format, Reed/Rahwan): standard JSON/RDF argument exchange.
- LKIF: OWL-based, used by Carneades.
- CQL/FHIR: CQL "Library + ActivityDefinition" can populate premises.
- DMN: decision tables map to strict rules; loses defeasibility.
- ASP/Datalog: ASPIC+ encodings exist (Caminada-Sá-Alcântara).

### Limitations/Known issues
Preference elicitation; computational cost on large rule bases; multiple literature definitions (Prakken 2010 vs Modgil & Prakken 2014). Carneades versions diverge; cite version carefully.

### Training data proxy
Strong: COMMA, JURIX, ICAIL, AAAI. Modgil-Prakken 2014 tutorial canonical, well-represented in LLM training. GitHub modest (dozens of repos).

## 5. Assumption-Based Argumentation for Provenance-Labeled Disputes

### Purpose
Reduce argumentation to a deductive system over rules + named *assumptions* and *contraries*; attacks are *exactly* negation-by-contrary of an assumption. Natural fit for provenance: each assumption tagged with source guideline / evidence quality.

### Maintainer/Standards body
Bondarenko, Dung, Kowalski, Toni, "An abstract, argumentation-theoretic approach to default reasoning," *Artificial Intelligence* 93(1–2):63–101 (1997). Maintained by Francesca Toni's group (Imperial College London). Tutorial: Toni, "A tutorial on assumption-based argumentation," *Argument & Computation* 5(1):89–117 (2014).

### Conceptual model
ABA framework `⟨L, R, A, ¯⟩`: deductive system `(L, R)`, assumptions `A ⊆ L`, contrary mapping `¯: A → L`. Arguments = deductions from assumptions; attacks = contrary-attacks on assumptions. ABA+ (Čyras & Toni, KR 2016) adds preferences over assumptions. Provenance labels encoded as additional assumptions, e.g., `from(g_i)`.

### Expressiveness/Semantics
Flat ABA: expressively equivalent to Dung AF (semantics-preserving translation). Non-flat ABA captures autoepistemic logic, default logic, normal/extended LP. Six-valued labelling semantics (Schulz & Toni 2017) for finer-grained reasoning.

### Composability/Modularity
Highly modular: union of `R` and `A` sets; provenance preserved through assumption labels. Suits combining heterogeneous guideline sources with each source's assumptions identifiable.

### Suitability for autoformalization to IR
High. IR primitives: rule, assumption, contrary, provenance-tag — stable, lexically narrow. ABA syntactically closer to logic programming than ASPIC+, so LLM-emitted IR is more idempotent.

### Formal verification potential
Same complexity as AF. Dispute derivation procedures (Dung, Kowalski & Toni, "Dialectic proof procedures for assumption-based, admissible argumentation," *Artificial Intelligence* 170(2):114–159, 2006; Dung-Mancarella-Toni 2007) yield proof-tree explanations. ABA+ preferences support patient-specific overrides with traceable provenance.

### Tooling/Ecosystem maturity
- `ABAplus` (Imperial) — Java/Python with ABA+ preferences.
- `ASPforABA` (Lehtonen, Wallner & Järvisalo, ASP-based) — ICCMA 2023 ABA-track winner ahead of ACBAR, ASTRA, CRUSTABRI, FLEXABLE.
- `CaSAPI` (legacy). ICCMA 2023 added structured tracks featuring ABA; ABA tracks continued in ICCMA'25.

### Japan-specific considerations
- Toshiko Wakaki (Shibaura Institute of Technology, emerita): "Assumption-based argumentation for extended disjunctive logic programming and its relation to nonmonotonic reasoning," *Argument & Computation* 15(3):309–353 (2024); "Assumption-Based Argumentation Equipped with Preferences and its Application to Decision Making, Practical Reasoning, and Epistemic Reasoning," *Computational Intelligence* 33(4):706–736 (2017).
- Sakama-Inoue (Wakayama/NII) paraconsistent stable semantics connects to ABA via disjunctive LP.
- Kakas/Toni roots — strong link to Japan's logic-programming community.

### Interoperability
- ASP/Datalog: native (ASPforABA encodes ABA in clingo).
- OWL/SHACL: assumptions ≈ punned annotated triples; contraries as SHACL constraints.
- Prolog: original implementation language; integrates with PROLEG.
- AIF: ABA → AIF exporters exist.

### Limitations/Known issues
Non-flat ABA complexity higher; ABA+ preference semantics has several variants. Tooling less mature than ASPARTIX for raw AF.

### Training data proxy
Moderate-strong: Toni 2014 tutorial widely cited; COMMA, KR, IJCAI, AAAI, JELIA. GitHub presence smaller than Dung-AF tooling.

## 6. Paraconsistent Logic for Operating Over Inconsistent Guideline Sets

### Purpose
Draw useful conclusions from a contradictory KB (e.g., two guidelines disagreeing on a drug) without trivializing (avoid *ex falso quodlibet*).

### Maintainer/Standards body
No standards body. Nuel Belnap, "A useful four-valued logic" (1977) — values `T, F, Both, Neither`. Newton da Costa C-systems (1960s). Graham Priest *LP* (Logic of Paradox) 1979. Carnielli-Coniglio Logics of Formal Inconsistency (LFI) with consistency operator `°`.

### Conceptual model
Belnap-Dunn `FDE`: 4-valued lattice (`{∅, {T}, {F}, {T,F}}` under information and truth orderings). LP: 3-valued (`T, F, B`), designated values `{T, B}`. LFI: classical recapture under `°φ` (consistency assumption).

### Expressiveness/Semantics
FDE paraconsistent + paracomplete; LP paraconsistent only. FDE: no validities containing only `→`; classical equivalences fail (e.g., disjunctive syllogism). LFI restores classical reasoning where consistency assumed.

### Composability/Modularity
KB union safe: union of two consistent KBs may be classically inconsistent yet still yields meaningful FDE/LP entailment. Natural for merging multi-source guidelines where contradiction is expected.

### Suitability for autoformalization to IR
Medium. IR must carry a four-valued evidence label per atom — convenient for "evidence supports / refutes / both / unknown". Small fixed value lattice aids LLM idempotency. Less prose-aligned than rule logics.

### Formal verification potential
FDE has cut-free sequent calculi; entailment coNP-complete. Paraconsistent description logics — Bienvenu, Bourgaux & Kozhemiachenko, "Queries With Exact Truth Values in Paraconsistent Description Logics" (KR 2024, pp. 145–155; arXiv:2408.07283) — inconsistency-tolerant OWL-style queries with tractable data complexity for Horn DLs. Sakama-Inoue paraconsistent stable semantics gives ASP-style operationalization.

### Tooling/Ecosystem maturity
Limited engineering tooling. `paraconsistent.dl` prototypes, Prolog-based BDI agents (Wagner), Coniglio's *LFI* implementations. Bienvenu et al. (2024): paraconsistent DL query algorithms via reduction to classical DL — deployable on Pellet/HermiT after preprocessing.

### Japan-specific considerations
- Chiaki Sakama & Katsumi Inoue, "Paraconsistent stable semantics for extended disjunctive programs," *J. Logic and Computation* 5(3):265–285 (1995) — foundational, still cited.
- Sakama (Wakayama), "Extended well-founded semantics for paraconsistent logic programs," FGCS 1992.
- Japanese-French Laboratory for Informatics (CNRS–NII IRL 3527, Tokyo): Bienvenu holds a joint JFLI position (her affiliation in the 2024 paper) — active local NII network; follow-on: Bienvenu, Bourgaux, Inoue & Jean, "A Rule-Based Approach to Specifying Preferences over Conflicting Facts and Querying Inconsistent Knowledge Bases" (KR 2025; arXiv:2508.07742).

### Interoperability
- OWL: paraconsistent DL semantics layer over standard OWL 2 KBs.
- SHACL: validation reports naturally produce `B` (both) when conflicting constraints fire.
- ASP/Datalog: encodable via 4-valued atoms (Sakama-Inoue).
- Lean/Isabelle/Rocq: FDE and LP formalized for meta-theory; not standard libraries.
- SMT: no native support; encode via reified truth values.

### Limitations/Known issues
Loses disjunctive syllogism and modus ponens-variants depending on connective choice — inferences may seem counter-intuitive to clinicians. Multiple competing systems (LP, RM3, LFI) without consensus. Communicating four-valued answers to clinicians is a UX problem.

### Training data proxy
Moderate. SEP entries strong; Priest, *Introduction to Non-Classical Logic* (Cambridge University Press, 2nd ed. 2008) widely available. WoLLIC, JELIA, Logica. Limited GitHub code.

## 7. Event Calculus for Longitudinal Clinical Events

### Purpose
Reason about time-varying *fluents* (e.g., "patient on warfarin", "INR > 3") under a narrative of timestamped *events* (drug start, lab draw, dose change): projection, postdiction, persistence.

### Maintainer/Standards body
Robert Kowalski & Marek Sergot, "A logic-based calculus of events," *New Generation Computing* 4(1):67–95 (1986). Variants: Murray Shanahan (Imperial), Erik Mueller (DEC, *Commonsense Reasoning* MIT/Morgan-Kaufmann 2006), Alexander Artikis (CEC/RTEC, NCSR Demokritos), Antonis Kakas (REC, Cyprus).

### Conceptual model
Predicates `Happens(e, t)`, `Initiates(e, f, t)`, `Terminates(e, f, t)`, `HoldsAt(f, t)`, `Initially(f)`. Persistence axiom: fluent holds until terminated. Encodable as Horn clauses with negation-as-failure → Prolog/ASP. Discrete (DEC) and continuous variants.

### Expressiveness/Semantics
First-order; semantics via circumscription (Shanahan) or stable model (Mueller, Kim et al.). Solves the frame problem locally. Domain-independent core ≈ 6 axioms.

### Composability/Modularity
Events and fluents sortally typed and decoupled; narratives compose by union; fluent termination governs interaction. Strong modularity for new events (lab, observation, prescription).

### Suitability for autoformalization to IR
High for longitudinal CDS: clinical events → `Happens(e, t)`; conditions → fluents. Stable LLM emission under fixed event-type vocabulary. Pairs with FHIR Observation/MedicationAdministration resources (timestamped). Precedent: Bromuri, Brugues de la Torre, Dubosson & Schumacher, "Indexing the Event Calculus with Kd-trees to Monitor Diabetes" (arXiv:1710.01275) — EC on continuous glucose monitoring streams (288 events/day).

### Formal verification potential
Executable in Prolog (s(CASP), ASP via `clingo`). Reasoning about all patient histories satisfying a guideline = constraint-LP query. Inter-guideline conflict on the same fluent timeline detectable as inconsistent `HoldsAt`.

### Tooling/Ecosystem maturity
- Mueller's DEC reasoner (SAT-based, 2006).
- `RTEC` (Artikis, NCSR Demokritos) — runtime EC over event streams.
- `jREC` (Java), `cached-EC`.
- s(CASP) (Arias, Carro, Gupta): goal-directed EC under stable model semantics (Category 5 §11).
- Probabilistic Event Calculus (ProbEC / OSL_EC; Skarlatidis, Artikis, Filipou & Paliouras, "A probabilistic logic programming event calculus," *TPLP* 15(2):213–245, 2015) — EC axioms as ProbLog facts/rules over noisy event streams; addresses sensor uncertainty, missed events in clinical telemetry; ProbLog stack (Category 5 §12).
- ProB (B-method) supports EC for verification.

### Japan-specific considerations
Katsumi Inoue (NII): abductive event calculus, systems-biology applications (Fariñas del Cerro & Inoue eds., *Logical Modeling of Biological Systems*, Wiley/ISTE 2014). No direct JAMI/Minds EC tooling.

### Interoperability
- FHIR: Observation/MedicationStatement/Procedure resources timestamped — direct EC event sources.
- openEHR: ENTRY archetypes lower to EC fluents.
- ASP/Datalog: native (clingo + temporal extensions).
- Prolog / s(CASP) (Category 5 §11): native execution substrate — EC axioms compile to Horn clauses one-to-one; SWI-Prolog tabling handles narrative-long persistence queries.
- Probabilistic Logic Programming (Category 5 §12): ProbEC inherits ProbLog SDD-based exact inference — certifiable probability bounds on `HoldsAt` under uncertain event observations.
- Lean/Isabelle: EC axiomatizations formalized in HOL and Isabelle/HOL (Mueller, Shanahan).
- TLA+: similar event-state philosophy; bridge possible, non-trivial.

### Limitations/Known issues
Naïve `HoldsAt` queries over long narratives slow (quadratic in events × fluents); Artikis indexing addresses this. Concurrency and triggered events require CEC variant. No native uncertainty — for noisy clinical event streams (missed observations, sensor errors, late EHR documentation) overlay Probabilistic Event Calculus on ProbLog (Category 5 §12): marginal `HoldsAt` probabilities at the cost of #P-hard worst-case inference.

### Training data proxy
Strong: Kowalski-Sergot 1986 widely cited; Mueller textbook in LLM training; Shanahan, *Solving the Frame Problem* (MIT 1997). CommonSense, IJCAI, LPNMR. GitHub modest but stable.

## 8. Allen Interval Algebra and Temporal Constraint Networks

### Purpose
Qualitative and quantitative reasoning about durations and orderings (e.g., "antibiotic must precede surgery by 1–4 hours", "monitor INR weekly for ≥ 4 weeks").

### Maintainer/Standards body
James F. Allen, "Maintaining knowledge about temporal intervals," *Communications of the ACM* 26(11):832–843 (1983) — 13 base relations. Point algebra: Vilain & Kautz 1986. TCSP/STN/STNU: Dechter, Meiri, Pearl, "Temporal constraint networks," *Artificial Intelligence* 49(1–3):61–95 (1991). STNU (with uncertainty): Morris, Muscettola, Vidal 2001.

### Conceptual model
13 jointly exhaustive, pairwise disjoint relations: `before, meets, overlaps, starts, during, finishes, equals` + inverses. QCN (qualitative constraint network) = graph over interval variables. STN: points + binary constraints `xⱼ − xᵢ ≤ c`; consistency by all-pairs shortest path (Floyd-Warshall). STNU: contingent edges for uncontrollable durations; dynamic controllability.

### Expressiveness/Semantics
IA satisfiability NP-complete; STN P-time; STNU dynamic controllability P-time (Morris 2014). Maximal tractable fragments (ORD-Horn, Nebel & Bürckert, *J. ACM* 42(1):43–66, 1995) capture most clinical needs.

### Composability/Modularity
QCNs and STNs compose by node + constraint union. Sub-network analysis supported.

### Suitability for autoformalization to IR
Very high for temporal IR: every "X within Y hours of Z" or "between event A and event B" is a constraint edge. Bounded vocabulary (13 relations) gives strong idempotency.

### Formal verification potential
STN consistency = no negative cycle; certified algorithms in Coq/Isabelle. IA via path consistency; Nebel-Bürckert ORD-Horn gives polynomial procedure. Cross-guideline temporal conflict reduces to STN/STNU inconsistency.

### Tooling/Ecosystem maturity
- `GQR` (qualitative reasoner, Westphal/Wölfl), `SparQ` (spatial-temporal calculi), `PyTemporal`, Java `Temporal` libs.
- STN: numerous (CMU CSPACE, NASA Europa, OpenSTNU).
- ASP encodings: Janhunen & Sioutis, "Allen's Interval Algebra Makes the Difference," in *Declarative Programming and Knowledge Management* (LNCS 12057, 2020; arXiv:1909.01128).

### Japan-specific considerations
Naoyuki Tamura (Kobe) & Mutsunori Banbara (Nagoya): Sugar SAT-based CSP encoder (Tamura, Taga, Kitagawa, Banbara, "Compiling finite linear CSP into SAT," *Constraints* 14(2):254–272, 2009); Aspartame (Banbara, Gebser, Inoue, Ostrowski, Peano, Schaub, Soh, Tamura, Weise, LPNMR 2015 LNAI 9345 pp. 112–126). Becker, Cabalar, Diéguez, Hahn, Romero, Schaub, "Compiling Metric Temporal Answer Set Programming" (LPNMR 2024, LNCS 15245 pp. 15–29) extends ASP with metric temporal operators per Cabalar, Diéguez, Schaub & Schuhmann, *TPLP* 20(5):783–798 (2020) — directly applicable to STN encoding.

### Interoperability
- HL7 FHIR/openEHR: Timing element and Period datatype map to STN edges.
- CQL: temporal operators (`during`, `before`, `after`) align with Allen.
- OWL-Time ontology (W3C): qualitative reasoning bridges OWL ontologies.
- SMT: difference logic (DL) theory in Z3/CVC5 handles STN natively.
- Lean/Isabelle: STN algorithms formalized.

### Limitations/Known issues
Pure qualitative IA ignores durations; pure STN ignores qualitative disjunctions. STNU controllability has subtle semantics. Clinical "approximately" creates fuzzy intervals not natively supported.

### Training data proxy
Very strong: textbook material; TIME (annual), IJCAI, AAAI, KR, ICAPS. Abundant GitHub code.

## 9. LTL/MTL/STL over Patient Timelines

### Purpose
Specify and check temporal properties of patient trajectories: "if hyperkalemia, then dialysis within 6h"; "INR must remain in [2,3] over 14 days"; online monitoring of streaming labs/vitals.

### Maintainer/Standards body
LTL: Amir Pnueli, "The temporal logic of programs," FOCS 1977. MTL: Koymans, "Specifying real-time properties with metric temporal logic," *Real-Time Systems* 2(4):255–299 (1990). STL: Maler & Ničković, "Monitoring temporal properties of continuous signals," FORMATS 2004.

### Conceptual model
LTL: discrete linear time, operators `X, F, G, U, R`. MTL: time-bounded `F_[a,b]`, `G_[a,b]`. STL: real-valued predicates over signals, quantitative robustness `ρ(φ, w, t)`. Online vs offline monitoring; past-only fragments admit DFA monitors.

### Expressiveness/Semantics
LTL model checking PSPACE-complete; satisfiability PSPACE. MTL satisfiability undecidable in general, decidable for fragments (MITL, Alur-Feder-Henzinger). STL quantitative robustness semantics supports gradient-based falsification.

### Composability/Modularity
Spec conjunction direct. Assume/guarantee modularity (Pnueli & Rosner). Monitors compose pointwise.

### Suitability for autoformalization to IR
High for *quantified* temporal recommendations. LTL/MTL benefits from controlled formula templates (Dwyer-Avrunin-Corbett specification patterns). STL adds real-valued thresholds matching numeric lab data.

### Formal verification potential
Mature: SPIN (LTL), NuSMV (LTL/CTL), PRISM (probabilistic), Uppaal (TCTL), Breach/S-TaLiRo (STL falsification), RTAMT, Reelay, MoonLight (online STL monitoring). Compositional verification well-studied. Clinical precedent: Bufo, Bartocci, Sanguinetti, Borelli, Lucangelo, Bortolussi, "Temporal Logic Based Monitoring of Assisted Ventilation in Intensive Care Patients" (ISoLA 2014, LNCS 8803); Lamp, Silvetti, Breton, Nenzi & Feng, "A Logic-Based Learning Approach to Explore Diabetes Patient Behaviors" (CMSB 2019, LNCS 11773). Recent: Cumulative-Time STL (CT-STL; Chen, Zhang, Roy, Bartocci, Smolka, Stoller & Lin, EMSOFT 2025; arXiv:2504.10325) for cumulative-duration clinical specifications; Temporal Ensemble Logic (TEL; Li et al., TIME 2025, LIPIcs vol. 355) for clinical-trial representation.

### Tooling/Ecosystem maturity
Very high. Industrial-grade model checkers; runtime monitors deployed in automotive/avionics. STL increasingly used in medical CPS (artificial pancreas, ventilation, ICU monitoring).

### Japan-specific considerations
- Ichiro Hasuo (NII) led JST ERATO MMSD (Metamathematics for Systems Design), JPMJER1603, Oct 2016–March 2025 (concluded with founding of Imiron Co. Ltd. via JST START); positioning paper: Hasuo, "Metamathematics for Systems Design," *New Generation Computing* 35(3):271–305 (2017); homepage https://www.jst.go.jp/erato/hasuo/en/.
- Akazaki & Hasuo, "Time Robustness in MTL and Expressivity in Hybrid System Falsification," CAV 2015, LNCS 9207 pp. 356–374 — introduces `AvSTL`.
- Sato, An, Zhang, Hasuo, "Optimization-Based Model Checking and Trace Synthesis for Complex STL Specifications" (CAV 2024, LNCS 14683; arXiv:2408.06983).
- Primary application focus automotive (ISO 34502), not healthcare; methods transfer.
- Ishii, Yonezaki, Goldsztejn — interval-analysis STL monitoring (IEICE).

### Interoperability
- FHIR: Observation streams feed STL monitors directly.
- openEHR: longitudinal EHR queries lower to LTL formulae.
- TLA+: temporal logic of actions; alternative spec language with strong tooling (TLC, Apalache).
- Lean/Rocq/Isabelle: LTL/MTL semantics formalized; verified monitors (Schneider et al., RV 2020).
- SMT: bounded model checking via Z3/CVC5.
- ASP/Datalog: telingo (temporal ASP, Cabalar et al.).

### Limitations/Known issues
Full MTL undecidable; STL robustness sensitive to signal noise; specification authoring hard for clinicians without templates. State-space explosion for large patient cohorts.

### Training data proxy
Very strong. Pnueli in every PL/verification textbook; STL prominent in CPS/CAV/HSCC literature. Active GitHub (S-TaLiRo, Breach, RTAMT, MoonLight).

## 10. Multi-Criteria Decision Analysis for Preference-Sensitive Recommendations

### Purpose
Quantify trade-offs across heterogeneous criteria (efficacy, safety, cost, QoL, patient preference) for preference-sensitive recommendations; operationalize GRADE Evidence-to-Decision (EtD) and Shared Decision-Making.

### Maintainer/Standards body
No single body. Foundational: AHP (Saaty 1980), MAUT/MAVT (Keeney & Raiffa 1976), TOPSIS (Hwang & Yoon 1981), ELECTRE (Roy 1968+), PROMETHEE (Brans & Vincke 1985). GRADE working group (Guyatt et al.) maintains EtD. Healthcare MCDA: Marsh et al., "Multiple Criteria Decision Analysis for Health Care Decision Making — Emerging Good Practices: Report 2 of the ISPOR MCDA Emerging Good Practices Task Force," *Value in Health* 19(2):125–137 (2016).

### Conceptual model
Alternatives × criteria performance matrix; weighting (subjective via AHP pairwise comparison, or swing-weighting); aggregation (weighted sum / utility / outranking / distance to ideal). Outranking (ELECTRE/PROMETHEE) uses concordance/discordance to build a partial preorder; rank-reversal phenomena documented.

### Expressiveness/Semantics
Numeric, not logical. MAUT axiomatized (Keeney-Raiffa); AHP has Saaty eigenvector justification; ELECTRE/PROMETHEE preference thresholds. Fuzzy MCDA accommodates uncertainty.

### Composability/Modularity
Criteria hierarchies decompose recursively (AHP). Patient-specific weights can override population weights. GRADE EtD fixed schema: problem, desirable effects, undesirable effects, certainty of evidence, values, balance of effects, resources, equity, acceptability, feasibility.

### Suitability for autoformalization to IR
Medium. IR carries recommendation metadata: criteria list, alternative list, performance matrix, weights. Idempotency requires fixed criteria ontology (e.g., GRADE EtD schema). Not a logic; pairs with deontic/argumentation layers for *what* to do once ranked.

### Formal verification potential
Low intrinsic: MCDA gives a ranking, not a proof. Some axiomatic guarantees (independence, transitivity) checkable on weight elicitation. Sensitivity analysis standard but not model-checking-style verification.

### Tooling/Ecosystem maturity
- R: `topsis`, `MCDA`, `ahpsurvey`, `PROMETHEE`, `RMCDA` (Najafi & Mirzaei, arXiv:2502.08677, 2025).
- Python: `pymcdm`, `scikit-criteria`.
- MAGIQ, Hiview, Web-HIPRE (legacy).
- GRADEpro GDT — supports EtD; SHARE-IT project (Agoritsas et al., *BMJ* 2015;350:g7624 and follow-ups) integrates decision aids with digital guidelines and GRADE evidence summaries.

### Japan-specific considerations
Minds (Medical Information Network Distribution Service, operated by Japan Council for Quality Health Care, JCQHC) uses GRADE in guideline development; "Minds Guide for Developing Clinical Practice Guidelines Ver. 2.0" (PubMed 30620850, 2018), superseded by "Minds Manual for Guideline Development 2020 ver. 3.0" (Minds Manual Development Committee, JCQHC, 2021). MID-NET® (PMDA) and JADER (Japanese Adverse Drug Event Report) datasets — Fujiwara, Kawasaki & Yamada, *PLoS ONE* 11(4):e0154425 (2016) — feed evidence into EtD desirable/undesirable-effects rows. No Japanese MCDA-CDS integration tool identified; Japanese guideline formalization dominated by narrative methodology, not executable formalisms (identified gap).

### Interoperability
- FHIR: Patient `goal`, `Consent`, `RiskAssessment` resources hold patient preferences/weights.
- openEHR: `EVALUATION` archetypes record preference data.
- DMN: weighted-sum decision tables map MCDA aggregation.
- OWL/SHACL: criteria/alternatives ontology; SHACL validation of completeness.
- SMT/ASP: outranking encodable but seldom done.

### Limitations/Known issues
Weight elicitation fragile; rank-reversal across methods (especially TOPSIS, AHP); criteria not always preferentially independent. Patient-level customization scales poorly. MCDA outputs are not justifications — wrap in an argumentation layer (ASPIC+/Carneades) for clinician trust.

### Training data proxy
Very strong in operations research / HTA literature; weaker in CS conferences. Textbooks: Belton & Stewart, *Multiple Criteria Decision Analysis: An Integrated Approach* (Kluwer 2002); Greco, Ehrgott & Figueira eds., *Multiple Criteria Decision Analysis: State of the Art Surveys*, 2nd ed. (Springer 2016). GitHub modest but growing (pymcdm, RMCDA, scikit-criteria).
