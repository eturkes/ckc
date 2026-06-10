# Category 6: Clinical Rule Semantics, Temporal Reasoning, and Argumentation

## 1. Defeasible Logic for Exceptions and Weak/Strong Recommendations

### Purpose
Encode rules-with-exceptions and graded recommendation strengths (strict vs defeasible rules, defeaters, superiority relation) so that clinical "should/may/consider" maps to formal weak/strong defeats.

### Maintainer/Standards body
No standards body. Foundational: Donald Nute (Univ. of Georgia, 1980s–90s). Active maintainer of the dominant variant (DL, DDL): Guido Governatori (formerly Data61/CSIRO, now Central Queensland University), with Antonio Rotolo (Bologna), Francesco Olivieri (Griffith), Michael Maher.

### Conceptual model
Rule theory `(F, R→, R⇒, R~>, >)` of facts, strict rules `→`, defeasible rules `⇒`, defeaters `~>`, and a binary superiority `>` over rules. Proof tags: `+Δ, –Δ` (definite), `+∂, –∂` (defeasible). Ambiguity-blocking vs ambiguity-propagating variants. Modal/temporal extensions add OBL, PERM, FORB operators (Defeasible Deontic Logic, DDL).

### Expressiveness/Semantics
Skeptical, non-monotonic, sub-classical. Propositional DL is linear-time decidable (Maher, *TPLP* 1(6):691–711, 2001). Captures contrary-to-duty, reparation chains, hierarchical normative positions.

### Composability/Modularity
Rule sets compose monotonically syntactically; semantic interaction governed by `>` priority. Modular addition of modalities (OBL, INT, BEL) via Governatori-Rotolo conversion rules. Theory union may require recomputing superiority closure.

### Suitability for autoformalization to IR
High. The "if-then with exceptions and priorities" syntax aligns directly with guideline prose; LLM-targeted IR can emit rules with explicit `defeasible` / `strict` / `defeater` tags and `>` annotations. Stable lexicalization across runs is easier than full FOL because of the bounded rule grammar.

### Formal verification potential
Decidable, polynomial-time entailment in propositional case enables exhaustive contradiction detection across two guideline rule bases. Modal/temporal DDL adds complexity but remains decidable for fragments.

### Tooling/Ecosystem maturity
- `SPINdle` (Lam & Governatori, "The Making of SPINdle," RuleML 2009) — Java reasoner, scales to >1M rules, XML I/O, standard + modal extensions; latest release v2.2.4 on SourceForge.
- `Delores` (Maher et al.), `DR-DEVICE` (Bassiliades et al., RDF/Jena).
- `Regorous` (Governatori) — commercial DDL compliance checker; used in Amantea & Governatori, "Automated Compliance Checking for Medical Rosters," *Studies in Health Technology and Informatics* 327:723–727 (MIE 2025) — direct clinical compliance precedent.
- Modest GitHub footprint; SPINdle last active ~2015.

### Japan-specific considerations
PROLEG (Ken Satoh, NII) implements a *closely related* exception-based reasoning model for Japanese civil-code "Yoken-jijitsu-ron" (presupposed ultimate facts theory) over Prolog with structured rule/exception pairs; not formally identical to Nute-DL but interoperable in spirit. Satoh, Fungwacharakorn et al. (JURISIN, ICAIL 2021–2025) actively extend PROLEG, including NL interfaces and LLM-driven normative extraction (Vienna VCLA talk, Nov 2024). Minds (JCQHC) guideline methodology uses GRADE narrative, not DL; no Japanese clinical DL toolchain identified.

### Interoperability
- IR/ontology: maps cleanly to RDF/OWL rule layer; SHACL constraint violations can seed defeaters.
- DMN: defeasible rules ≈ DMN decision tables with override hit policies; lossy.
- FHIR/CQL: CQL expressions can populate rule antecedents; CQL has no native exception priority.
- ASP/Datalog: DL is translatable to stable-model ASP (Antoniou et al.); ideal back-end.
- Prolog / s(CASP) (Category 5 §11): goal-directed defeasible reasoning with justification trees is the canonical execution path; SPINdle exports Prolog-like rule lists, and the d-Prolog family (Nute) was the original implementation. s(CASP)'s constructive negation handles the `not-applicable` distinction defeasible rules require.
- Probabilistic Logic Programming (Category 5 §12): ProbLog / cplint / CP-logic add weighted facts on top of defeasible rules, addressing the "no probabilistic strength" pitfall below; conditional-probability tables on rule heads can encode GRADE certainty bands.
- Lean/Isabelle: DL meta-theory has been partially formalized (Maher); rule application not natively certified.

### Limitations/Known issues
Superiority elicitation is brittle; ambiguity-blocking vs ambiguity-propagating semantics yield divergent conclusions on the same theory. No native probabilistic strength — recommendation strength is encoded ordinally via `>` rather than as a measure; for quantitative weighting (e.g., GRADE certainty bands as numeric probabilities) overlay with Probabilistic Logic Programming (Category 5 §12). Quantification limited (mostly propositional / function-free).

### Training data proxy
Moderate. Governatori's corpus (governatori.net, ResearchGate) is in LLM training. Conferences: RuleML+RR, JURIX, ICAIL, DEON, COMMA. SPINdle paper has substantial citation footprint though no precise count was retrievable from indexed databases.

---

## 2. Deontic Logic for Obligations, Prohibitions, and Permissions

### Purpose
Formalize normative content of guidelines — what is *obligatory* (O), *permitted* (P), *forbidden* (F) — distinct from what is *true*. Handles contrary-to-duty (CTD) scenarios common in clinical exceptions ("if drug X contraindicated, then …").

### Maintainer/Standards body
None official. SDL: von Wright (1951). Dyadic deontic logic: Hansson, Lewis, Åqvist. Input/Output (I/O) logic: David Makinson & Leendert van der Torre, *J. Philosophical Logic* 29(4):383–408 (2000); 30(2):155–185 (2001); 32(4):391–416 (2003). Jones-Sergot agency. Governatori for computational DDL.

### Conceptual model
SDL: modal KD over `O`, with `Pp ≡ ¬O¬p`, `Fp ≡ O¬p`. Dyadic `O(B|A)` reads "in context A, B is obligatory". I/O logic treats norms as ordered pairs `(a, x)`; out₁..out₄ operators differ on reusability and reflexivity. Jones-Sergot adds agency operator `E`.

### Expressiveness/Semantics
Possible-worlds Kripke semantics for SDL. I/O has *no* truth-functional embedding of norms — norms are pre-logical. Suits norm bases that should not be subject to logical paradoxes (Ross, free-choice, gentle-murderer, Chisholm).

### Composability/Modularity
I/O is highly modular: norm sets are simple relations; combination = set union; consistency checked via constrained out operators. SDL composition tends to trigger CTD paradoxes (deontic explosion).

### Suitability for autoformalization to IR
High for tagged IR: each guideline recommendation lexicalizes as `(context, action, modality)` tuple. The IR can be a typed triple store with `O/P/F` markers, directly compilable to either SDL boxes or I/O pairs. Idempotency across LLM runs is improved by restricting to dyadic norms `(antecedent, deontic-consequent)`.

### Formal verification potential
SDL: decidable (PSPACE-complete for normal modal logics over propositional base). I/O: decidability depends on base logic; constrained variants are decidable. Cross-guideline conflict detection = inconsistency in `out(G ∪ G′, A)`. Empirical guideline study (Lomotan et al., *Qual Saf Health Care* 2010 / PMC2982946) showed wide variability in clinician interpretation of deontic terms ("must" median = 100, "may" = 37 on a 0–100 obligation scale), motivating formal disambiguation.

### Tooling/Ecosystem maturity
LogiKey framework (Benzmüller, Parent, van der Torre, 2018+) embeds deontic logics in Isabelle/HOL via shallow semantical embedding — yields verified deontic reasoning. SDLib, MleanCoP-DL prototypes. Limited stand-alone solvers; usually combined with DL or ASP.

### Japan-specific considerations
Ken Satoh (NII) chairs PROLALA and JURISIN; PROLEG's exception structure aligns with dyadic deontic readings. Chiaki Sakama (Wakayama) and Toshiko Wakaki (Shibaura, emerita) work on normative-flavoured logic programming. No Japan-specific clinical deontic toolchain identified.

### Interoperability
- HL7 FHIR + CQL: deontic tags can annotate CQL "ActivityDefinition" recommendations; round-tripping requires custom extension.
- DMN/BPMN: BPMN tasks can be tagged `mandatory`/`optional`; lossy.
- OWL: deontic role hierarchy expressible but quantified deontic logic is not in OWL 2 DL.
- Isabelle/HOL (LogiKey), Lean: shallow embedding feasible; this is the certified path.
- SAT/SMT: SDL via QBF or modal-SAT translation.

### Limitations/Known issues
Classical paradoxes (Ross, Chisholm, Forrester) bite SDL. Most paradoxes recur in pragmatic clinical text. Distinguishing prima facie vs all-things-considered obligations is unresolved without combining with defeasible logic.

### Training data proxy
Strong philosophical literature (Stanford Encyclopedia, Gabbay-Horty-Parent-van der Meyden-van der Torre *Handbook of Deontic Logic and Normative Systems*, 2013–2021). DEON conference biennial since 1991. Lower code/GitHub footprint than ASP or ML.

---

## 3. Dung-Style Abstract Argumentation Frameworks

### Purpose
Provide an attack-graph semantics for resolving conflicts among arguments (e.g., conflicting guideline recommendations) without commitment to internal argument structure.

### Maintainer/Standards body
Foundational: Phan Minh Dung, "On the acceptability of arguments and its fundamental role in nonmonotonic reasoning, logic programming, and n-person games," *Artificial Intelligence* 77(2):321–357 (1995). Community-curated via ICCMA (International Competition on Computational Models of Argumentation), organised biennially since ICCMA'15 (originally organised by Matthias Thimm, then Univ. Koblenz-Landau, now FernUniversität in Hagen, and Serena Villata, INRIA Sophia Antipolis; reported in Thimm & Villata, *Artificial Intelligence* 252:267–294, 2017). ICCMA'25 (sixth edition) concluded in November 2025, with results presented at the Arg&App workshop co-located with KR 2025 in Melbourne.

### Conceptual model
AF = `(A, R)` with A a set of arguments, R ⊆ A × A the attack relation. Acceptability semantics: conflict-free, admissible, complete, grounded (unique, skeptical), preferred (maximal admissible), stable (attacks every outsider), semi-stable, ideal, eager, stage.

### Expressiveness/Semantics
Pure graph theory; encoded as directed graph + extension predicate. Reasoning complexity ranges P (grounded) → coNP / Σ₂ᵖ (preferred, semi-stable skeptical acceptance).

### Composability/Modularity
Trivially compositional at graph level. Standard ICCMA APX/TGF/i23 formats. Modular extensions: bipolar AFs (support), value-based AFs (Bench-Capon), preference-based AFs, weighted AFs.

### Suitability for autoformalization to IR
Medium-low as primary IR (loses internal structure of guideline rules) — best used as *downstream* semantic layer over a structured argumentation IR (ASPIC+/ABA/Carneades). Highly idempotent: argument identifiers + attack edges are a canonical graph.

### Formal verification potential
Decision problems (credulous/skeptical acceptance) are standard; certified solvers exist. Contradiction detection = empty stable extension or asymmetric attacks across guideline sources.

### Tooling/Ecosystem maturity
- `ASPARTIX` / `ASPARTIX-V` (TU Wien, Dvořák/Wallner/Woltran) — ASP encodings; multiple ICCMA editions.
- `μ-toksia` (Niskanen & Järvisalo, University of Helsinki) — SAT-based; ranked first in all 21 main-track categories and several dynamic tracks at ICCMA 2019; formally documented in Niskanen & Järvisalo, "µ-toksia: An Efficient Abstract Argumentation Reasoner," KR 2020, pp. 800–804.
- `ConArg`, `pyglaf`, CoQuiAAS. ICCMA benchmark sets publicly available.

### Japan-specific considerations
Chiaki Sakama & Tjitze Rienstra, "Representing argumentation frameworks in answer set programming," *Fundamenta Informaticae* 155(3):261–292 (2017). Katsumi Inoue (NII) — abductive/inductive argumentation links. No clinical instantiation in Japan.

### Interoperability
- OWL/SHACL: arguments → OWL individuals; attacks → object property; coarse.
- ASP/Datalog: native (ASPARTIX). Ideal back-end with `clingo`.
- SAT/QBF: ICCMA solvers use SAT or QBF for Σ₂ᵖ tasks.
- Lean/Isabelle/Rocq: AF semantics have been formalized (e.g., Caminada labelling in Isabelle).

### Limitations/Known issues
"Abstract" — no semantics for argument content; on its own does not detect logical contradictions, only attack patterns. Self-defeating arguments problematic. Re-grounding the same guideline twice may yield syntactically different argument IDs unless naming policy is fixed.

### Training data proxy
Very strong: 30+ years of literature, COMMA biennial, ICCMA biennial, AAAI/IJCAI/KR/ECAI yearly tracks. Dung 1995 has over 1,400 citations per ACM Digital Library (and substantially more on Google Scholar), making it among the most-cited papers in nonmonotonic reasoning. Substantial open-source code on GitHub.

---

## 4. ASPIC+ / Carneades for Explainable Guideline Conflict Arguments

### Purpose
Structured argumentation: build arguments as inference trees from strict + defeasible rules over a base logic, then evaluate via Dung semantics. Carneades adds explicit *proof standards* and *burden of proof*, valuable for graded clinical evidence (GRADE strong vs weak).

### Maintainer/Standards body
ASPIC+: Henry Prakken (Utrecht/Groningen), Sanjay Modgil (KCL). Tutorial: Modgil & Prakken, *Argument & Computation* 5(1):31–62 (2014). Carneades: Thomas Gordon (Fraunhofer FOKUS), Henry Prakken, Douglas Walton, "The Carneades model of argument and burden of proof," *Artificial Intelligence* 171(10–15):875–896 (2007). LKIF Legal Knowledge Interchange Format associated with Carneades.

### Conceptual model
ASPIC+: language L with contrariness function; strict rules `Rs` (premises guarantee conclusion) and defeasible rules `Rd` (presumptive); arguments = inference trees; three attack types — *undermining* (premise), *rebutting* (defeasible conclusion), *undercutting* (defeasible inference). Preferences resolve attacks to defeats. Carneades: argument graphs with statement nodes, argument nodes (pro/con), proof standards (scintilla, preponderance, clear-and-convincing, beyond-reasonable-doubt), audiences with assumptions and weights.

### Expressiveness/Semantics
ASPIC+ inherits Dung semantics; rationality postulates (closure, consistency) guaranteed under well-formedness. Carneades supports stage-based evaluation; later versions (CAES) align with Dung. Carneades 4 implementation in Go.

### Composability/Modularity
Both modular: rules + premises + preferences are separate artefacts. ASPIC+ unifies ABA, classical-logic argumentation, defeasible logic as instantiations.

### Suitability for autoformalization to IR
Very high. The IR can be `Argument(id, premises, rule, conclusion, type∈{strict,defeasible})` + `Pref(rule_i > rule_j)`. Carneades' argumentation-scheme catalogue — 96 named schemes per Walton, Reed & Macagno, *Argumentation Schemes* (Cambridge University Press, 2008) — is directly recognizable in clinical guideline language (e.g., *argument from expert opinion*, *argument from established rule*) and increases idempotency: each recommendation matches a scheme template.

### Formal verification potential
Reasoning reduces to AF reasoning (NP / Σ₂ᵖ). Conflict between guidelines surfaces as mutual defeat without a single preferred extension containing both. Rationality postulates are theorem-prover-checkable (Heyninck & Straßer, *Argument & Computation* 12(1):3–47, 2021).

### Tooling/Ecosystem maturity
- `Carneades 4` (Go), https://github.com/carneades; v4.3 (released July 2017) with native Constraint Handling Rules (CHR) inference engine in Go (removes Prolog dependency); last commit activity October 2024.
- ASPIC+ engines: `EPR` (Edinburgh), `TOAST` (Snaith & Reed), Python `pyASPIC` prototypes.
- Less mature than ASPARTIX for raw AF, but uniquely supports explanation export.

### Japan-specific considerations
Direct precedent: Oliveira, Novais et al. (with JSPS KAKENHI Grant JP18K18115 funding for Oliveira), "Argumentation for Reasoning with Conflicting Clinical Guidelines and Preferences," AAAI 2018 workshop — uses ASPIC-G (ASPIC+ with Goals; Modgil-Prakken 2014, Oliveira et al. 2018) and ABA+ for patient-centric reasoning over conflicting guidelines. Carneades scheme catalog interoperates conceptually with PROLEG's rule/exception view (Satoh, NII).

### Interoperability
- AIF (Argument Interchange Format, Reed/Rahwan): standard JSON/RDF for argument exchange.
- LKIF: OWL-based, used by Carneades.
- CQL/FHIR: CQL "Library + ActivityDefinition" can populate premises.
- DMN: decision tables map to strict rules; loses defeasibility.
- ASP/Datalog: ASPIC+ encodings exist (Caminada-Sá-Alcântara).

### Limitations/Known issues
Preference elicitation; computational cost of large rule bases; multiple definitions in literature (Prakken 2010 vs Modgil & Prakken 2014). Carneades' newer versions diverge from earlier ones; cite version carefully.

### Training data proxy
Strong: COMMA, JURIX, ICAIL, AAAI. Modgil-Prakken 2014 tutorial is canonical and well-represented in LLM training. GitHub code modest (dozens of repos).

---

## 5. Assumption-Based Argumentation for Provenance-Labeled Disputes

### Purpose
Reduce argumentation to a deductive system over rules + named *assumptions* and *contraries*. Attacks are *exactly* the negation-by-contrary of an assumption. Natural fit for guideline provenance: each assumption tagged with source guideline / evidence quality.

### Maintainer/Standards body
Bondarenko, Dung, Kowalski, Toni, "An abstract, argumentation-theoretic approach to default reasoning," *Artificial Intelligence* 93(1–2):63–101 (1997). Maintained by Francesca Toni (Imperial College London) and her group. Tutorial: Toni, "A tutorial on assumption-based argumentation," *Argument & Computation* 5(1):89–117 (2014).

### Conceptual model
ABA framework `⟨L, R, A, ¯⟩`: deductive system `(L, R)`, set of assumptions `A ⊆ L`, contrary mapping `¯: A → L`. Arguments are deductions from assumptions; attacks are contrary-attacks on assumptions. ABA+ (Čyras & Toni, KR 2016) adds preferences over assumptions. Provenance labels are typically encoded as additional assumptions, e.g., `from(g_i)`.

### Expressiveness/Semantics
Flat ABA: equivalent in expressive power to Dung AF (semantics-preserving translation). Non-flat ABA captures autoepistemic logic, default logic, normal/extended LP. Six-valued labelling semantics (Schulz & Toni 2017) for finer-grained reasoning.

### Composability/Modularity
Highly modular: union of `R` and `A` sets; provenance preserved through assumption labels. Suitable for combining heterogeneous guideline sources where each source's assumptions remain identifiable.

### Suitability for autoformalization to IR
High. IR primitives: rule, assumption, contrary, provenance-tag — all stable, lexically narrow. ABA is closer to logic programming syntactically than ASPIC+, making LLM-emitted IR more idempotent.

### Formal verification potential
Decision problems same complexity as AF. Dispute derivation procedures (Dung, Kowalski & Toni, "Dialectic proof procedures for assumption-based, admissible argumentation," *Artificial Intelligence* 170(2):114–159, 2006; Dung-Mancarella-Toni 2007) yield proof-tree explanations. Čyras & Toni's preferences support patient-specific overrides with traceable provenance.

### Tooling/Ecosystem maturity
- `ABAplus` (Imperial) — Java/Python with ABA+ preferences.
- `ASPforABA` (Lehtonen, Wallner & Järvisalo, ASP-based, ICCMA 2023 ABA-track winner ahead of ACBAR, ASTRA, CRUSTABRI, FLEXABLE).
- `CaSAPI` (legacy). ICCMA 2023 added structured tracks featuring ABA; ABA tracks continued in ICCMA'25.

### Japan-specific considerations
- Toshiko Wakaki (Shibaura Institute of Technology, emerita) — "Assumption-based argumentation for extended disjunctive logic programming and its relation to nonmonotonic reasoning," *Argument & Computation* 15(3):309–353 (2024); and "Assumption-Based Argumentation Equipped with Preferences and its Application to Decision Making, Practical Reasoning, and Epistemic Reasoning," *Computational Intelligence* 33(4):706–736 (2017).
- Sakama-Inoue (Wakayama/NII) paraconsistent stable semantics connects to ABA via disjunctive LP.
- ABA's Kakas/Toni roots — strong link to Japan's logic-programming community.

### Interoperability
- ASP/Datalog: native (ASPforABA encodes ABA in clingo).
- OWL/SHACL: assumptions ≈ punned annotated triples; contraries as SHACL constraints.
- Prolog: original implementation language; integrates with PROLEG.
- AIF: ABA → AIF exporters exist.

### Limitations/Known issues
Non-flat ABA complexity higher; preferences semantics (ABA+) has several variants. Tooling less mature than ASPARTIX for raw AF.

### Training data proxy
Moderate-strong: Toni 2014 tutorial widely cited; ABA appears at COMMA, KR, IJCAI, AAAI, JELIA. GitHub presence smaller than Dung-AF tooling.

---

## 6. Paraconsistent Logic for Operating Over Inconsistent Guideline Sets

### Purpose
Continue to draw useful conclusions from a knowledge base containing contradictions (e.g., two guidelines disagreeing on a drug) without trivializing (avoid *ex falso quodlibet*).

### Maintainer/Standards body
No standards body. Foundational: Nuel Belnap, "A useful four-valued logic" (1977) — values `T, F, Both, Neither`. Newton da Costa C-systems (1960s). Graham Priest *LP* (Logic of Paradox) 1979. Carnielli-Coniglio Logics of Formal Inconsistency (LFI) with consistency operator `°`.

### Conceptual model
Belnap-Dunn `FDE` is a 4-valued lattice (`{∅, {T}, {F}, {T,F}}` under information and truth orderings). LP: 3-valued (`T, F, B`); designated values `{T, B}`. LFI: classical recapture under `°φ` (consistency assumption).

### Expressiveness/Semantics
Paraconsistent + paracomplete (FDE) or paraconsistent only (LP). FDE: no validities containing only `→`; classical equivalences fail (e.g., disjunctive syllogism). LFI restores classical reasoning where consistency is assumed.

### Composability/Modularity
KB union is safe: union of two consistent KBs may be inconsistent classically but still yields meaningful FDE/LP entailment. Natural fit for merging multi-source guidelines where contradiction is expected.

### Suitability for autoformalization to IR
Medium. The IR must carry a four-valued evidence label per atom — convenient for "evidence supports / refutes / both / unknown". LLM idempotency benefits from a small fixed value lattice. Less aligned with prose than rule logics.

### Formal verification potential
FDE has cut-free sequent calculi; entailment is coNP-complete. Paraconsistent description logics — Bienvenu, Bourgaux & Kozhemiachenko, "Queries With Exact Truth Values in Paraconsistent Description Logics" (KR 2024, pp. 145–155; arXiv:2408.07283) — enable inconsistency-tolerant OWL-style queries with tractable data complexity for Horn DLs. Sakama-Inoue paraconsistent stable semantics gives ASP-style operationalization.

### Tooling/Ecosystem maturity
Limited engineering tooling. `paraconsistent.dl` prototypes, Prolog-based BDI agents (Wagner), Coniglio's *LFI* implementations. Bienvenu et al. (2024) describe paraconsistent description logic query algorithms via reduction to classical DL — practically deployable on Pellet/HermiT after preprocessing.

### Japan-specific considerations
- Chiaki Sakama & Katsumi Inoue, "Paraconsistent stable semantics for extended disjunctive programs," *J. Logic and Computation* 5(3):265–285 (1995) — foundational, still cited.
- Sakama (Wakayama) "Extended well-founded semantics for paraconsistent logic programs," FGCS 1992.
- Japanese-French Laboratory for Informatics (CNRS–NII IRL 3527, Tokyo) — Bienvenu has a joint position at JFLI (her affiliation in the 2024 paper), indicating an active local research network at NII; follow-on work by Bienvenu, Bourgaux, Inoue & Jean, "A Rule-Based Approach to Specifying Preferences over Conflicting Facts and Querying Inconsistent Knowledge Bases" (KR 2025; arXiv:2508.07742) continues the collaboration.

### Interoperability
- OWL: paraconsistent DL semantics layer over standard OWL 2 KBs.
- SHACL: validation reports naturally produce `B` (both) when conflicting constraints fire.
- ASP/Datalog: encodable via 4-valued atoms (Sakama-Inoue).
- Lean/Isabelle/Rocq: FDE and LP have been formalized in proof assistants for meta-theory; not standard libraries.
- SMT: no native support; encode via reified truth values.

### Limitations/Known issues
Loses disjunctive syllogism and modus ponens-variants depending on connective choice — clinicians may find inferences counter-intuitive. Multiple competing systems (LP, RM3, LFI) without consensus. Communicating four-valued answers to clinicians is a UX problem.

### Training data proxy
Moderate. SEP entries strong; Priest's *Introduction to Non-Classical Logic* (Cambridge University Press, 2nd ed. 2008) widely available. Conferences: WoLLIC, JELIA, Logica. Limited GitHub code.

---

## 7. Event Calculus for Longitudinal Clinical Events

### Purpose
Reason about time-varying *fluents* (e.g., "patient on warfarin", "INR > 3") under a narrative of timestamped *events* (drug start, lab draw, dose change), supporting projection, postdiction, and persistence.

### Maintainer/Standards body
Robert Kowalski & Marek Sergot, "A logic-based calculus of events," *New Generation Computing* 4(1):67–95 (1986). Variants maintained by Murray Shanahan (Imperial), Erik Mueller (DEC, *Commonsense Reasoning* MIT/Morgan-Kaufmann 2006), Alexander Artikis (CEC/RTEC, NCSR Demokritos), Antonis Kakas (REC, Cyprus).

### Conceptual model
Predicates: `Happens(e, t)`, `Initiates(e, f, t)`, `Terminates(e, f, t)`, `HoldsAt(f, t)`, `Initially(f)`. Persistence axiom: a fluent holds until terminated. Encodable as Horn clauses with negation-as-failure → Prolog/ASP. Discrete (DEC) and continuous variants.

### Expressiveness/Semantics
First-order; semantics typically via circumscription (Shanahan) or stable model (Mueller, Kim et al.). Solves the frame problem locally. Domain-independent core ≈ 6 axioms.

### Composability/Modularity
Events and fluents are sortally typed and decoupled; multiple narratives can be composed by union; fluent termination governs interaction. Strong modularity for adding new events (lab, observation, prescription).

### Suitability for autoformalization to IR
High for longitudinal CDS: clinical events map naturally to `Happens(e, t)`; conditions map to fluents. LLM emission stable under fixed event-type vocabulary. Pairs naturally with FHIR Observation/MedicationAdministration resources (each carries a timestamp). Direct precedent: Bromuri, Brugues de la Torre, Dubosson & Schumacher, "Indexing the Event Calculus with Kd-trees to Monitor Diabetes" (arXiv:1710.01275) demonstrates EC on continuous glucose monitoring streams (288 events/day).

### Formal verification potential
EC programs are executable in Prolog (s(CASP), ASP via `clingo`). Reasoning about all patient histories satisfying a guideline = constraint-LP query. Conflict between guidelines on the same fluent timeline detectable as inconsistent `HoldsAt`.

### Tooling/Ecosystem maturity
- Mueller's DEC reasoner (SAT-based, 2006).
- `RTEC` (Artikis, NCSR Demokritos) — runtime EC over event streams.
- `jREC` (Java), `cached-EC`.
- s(CASP) (Arias, Carro, Gupta) for goal-directed EC under stable model semantics (Category 5 §11).
- Probabilistic Event Calculus (ProbEC / OSL_EC; Skarlatidis, Artikis, Filipou & Paliouras, "A probabilistic logic programming event calculus," *TPLP* 15(2):213–245, 2015) — EC axioms encoded as ProbLog facts/rules over noisy event streams; addresses sensor uncertainty and missed events in clinical telemetry. Implemented on the ProbLog stack (Category 5 §12).
- ProB (B-method) supports EC for verification.

### Japan-specific considerations
Katsumi Inoue (NII) — abductive event calculus and applications to systems biology (Fariñas del Cerro & Inoue eds., *Logical Modeling of Biological Systems*, Wiley/ISTE 2014). No direct JAMI/Minds tooling using EC.

### Interoperability
- FHIR: Observation/MedicationStatement/Procedure resources are timestamped — direct EC event sources.
- openEHR: ENTRY archetypes can be lowered to EC fluents.
- ASP/Datalog: native (clingo + temporal extensions).
- Prolog / s(CASP) (Category 5 §11): native execution substrate — EC axioms compile to Horn clauses one-to-one; SWI-Prolog's tabling handles narrative-long persistence queries.
- Probabilistic Logic Programming (Category 5 §12): ProbEC inherits ProbLog's SDD-based exact inference, giving certifiable probability bounds on `HoldsAt` queries under uncertain event observations.
- Lean/Isabelle: EC axiomatizations have been formalized in HOL and Isabelle/HOL (Mueller, Shanahan).
- TLA+: similar event-state philosophy; bridge possible but non-trivial.

### Limitations/Known issues
Naïve `HoldsAt` queries over long narratives are slow (quadratic in events × fluents); Artikis' indexing addresses this. Concurrency and triggered events require CEC variant. Classical EC has no native uncertainty — for noisy clinical event streams (missed observations, sensor errors, late EHR documentation) overlay with Probabilistic Event Calculus on a ProbLog backend (Category 5 §12), which gives marginal `HoldsAt` probabilities at the cost of #P-hard worst-case inference.

### Training data proxy
Strong: Kowalski-Sergot 1986 widely cited; Mueller's textbook present in LLM training; Shanahan's *Solving the Frame Problem* (MIT 1997). Conferences: CommonSense, IJCAI, LPNMR. GitHub presence modest but stable.

---

## 8. Allen Interval Algebra and Temporal Constraint Networks

### Purpose
Qualitative and quantitative reasoning about durations and orderings (e.g., "antibiotic must precede surgery by 1–4 hours", "monitor INR weekly for ≥ 4 weeks").

### Maintainer/Standards body
James F. Allen, "Maintaining knowledge about temporal intervals," *Communications of the ACM* 26(11):832–843 (1983) — 13 base relations. Point algebra: Vilain & Kautz 1986. TCSP/STN/STNU: Dechter, Meiri, Pearl, "Temporal constraint networks," *Artificial Intelligence* 49(1–3):61–95 (1991). STNU (with uncertainty): Morris, Muscettola, Vidal 2001.

### Conceptual model
13 jointly exhaustive, pairwise disjoint relations: `before, meets, overlaps, starts, during, finishes, equals` and inverses. QCN (qualitative constraint network) is graph over interval variables. STN: points + binary constraints `xⱼ − xᵢ ≤ c`; consistency by all-pairs shortest path (Floyd-Warshall). STNU: contingent edges modelling uncontrollable durations; dynamic controllability.

### Expressiveness/Semantics
IA satisfiability NP-complete; STN P-time; STNU dynamic controllability P-time (Morris 2014). Maximal tractable fragments (ORD-Horn, Nebel & Bürckert, *J. ACM* 42(1):43–66, 1995) capture most clinical needs.

### Composability/Modularity
QCNs and STNs compose by node + constraint union. Sub-network analysis supported.

### Suitability for autoformalization to IR
Very high for temporal IR: every guideline "X within Y hours of Z" or "between event A and event B" is a constraint edge. Bounded vocabulary (13 relations) gives strong idempotency.

### Formal verification potential
STN consistency = no negative cycle; certified algorithms in Coq/Isabelle exist. IA consistency via path consistency; Nebel-Bürckert ORD-Horn gives polynomial procedure. Cross-guideline temporal conflict reduces to STN/STNU inconsistency.

### Tooling/Ecosystem maturity
- `GQR` (qualitative reasoner, Westphal/Wölfl).
- `SparQ` (spatial-temporal calculi).
- `PyTemporal`, Java `Temporal` libs.
- STN: numerous (CMU CSPACE, NASA Europa, OpenSTNU).
- ASP encodings: Janhunen & Sioutis, "Allen's Interval Algebra Makes the Difference," in *Declarative Programming and Knowledge Management* (LNCS 12057, 2020; arXiv:1909.01128).

### Japan-specific considerations
Naoyuki Tamura (Kobe) & Mutsunori Banbara (Nagoya) — Sugar SAT-based CSP encoder (Tamura, Taga, Kitagawa, Banbara, "Compiling finite linear CSP into SAT," *Constraints* 14(2):254–272, 2009); Aspartame (Banbara, Gebser, Inoue, Ostrowski, Peano, Schaub, Soh, Tamura, Weise, LPNMR 2015 LNAI 9345 pp. 112–126); Becker, Cabalar, Diéguez, Hahn, Romero, Schaub, "Compiling Metric Temporal Answer Set Programming" (LPNMR 2024, LNCS 15245 pp. 15–29) extending ASP with metric temporal operators per Cabalar, Diéguez, Schaub & Schuhmann, *TPLP* 20(5):783–798 (2020) — directly applicable to STN encoding.

### Interoperability
- HL7 FHIR/openEHR: Timing element and Period datatype map to STN edges.
- CQL: temporal operators (`during`, `before`, `after`) align with Allen.
- OWL-Time ontology (W3C): qualitative reasoning bridges OWL ontologies.
- SMT: difference logic (DL) theory in Z3/CVC5 handles STN natively.
- Lean/Isabelle: STN algorithms have been formalized.

### Limitations/Known issues
Pure qualitative IA ignores durations; pure STN ignores qualitative disjunctions. STNU controllability has subtle semantics. Clinical "approximately" creates fuzzy intervals not natively supported.

### Training data proxy
Very strong: textbook material; conferences TIME (annual), IJCAI, AAAI, KR, ICAPS. Abundant code on GitHub.

---

## 9. LTL/MTL/STL over Patient Timelines

### Purpose
Specify and check temporal properties of patient trajectories: "if hyperkalemia, then dialysis within 6h"; "INR must remain in [2,3] over 14 days"; online monitoring of streaming labs/vitals.

### Maintainer/Standards body
LTL: Amir Pnueli, "The temporal logic of programs," FOCS 1977. MTL: Koymans, "Specifying real-time properties with metric temporal logic," *Real-Time Systems* 2(4):255–299 (1990). STL: Maler & Ničković, "Monitoring temporal properties of continuous signals," FORMATS 2004.

### Conceptual model
LTL: discrete linear time, operators `X, F, G, U, R`. MTL: time-bounded operators `F_[a,b]`, `G_[a,b]`. STL: real-valued predicates over signals with quantitative robustness `ρ(φ, w, t)`. Online vs offline monitoring; past-only fragments admit DFA monitors.

### Expressiveness/Semantics
LTL model checking PSPACE-complete; satisfiability PSPACE. MTL satisfiability undecidable in general, decidable for fragments (MITL, Alur-Feder-Henzinger). STL has quantitative robustness semantics — supports gradient-based falsification.

### Composability/Modularity
Conjunction of LTL/STL specs is direct. Module-thinking via formula assumptions/guarantees (Pnueli & Rosner). Monitors compose pointwise.

### Suitability for autoformalization to IR
High for *quantified* temporal recommendations. LTL/MTL benefits from controlled formula templates (Dwyer-Avrunin-Corbett specification patterns). STL adds real-valued thresholds matching numeric lab data.

### Formal verification potential
Mature: SPIN (LTL), NuSMV (LTL/CTL), PRISM (probabilistic), Uppaal (TCTL), Breach/S-TaLiRo (STL falsification), RTAMT, Reelay, MoonLight (online STL monitoring). Compositional verification well-studied. Direct clinical precedent: Bufo, Bartocci, Sanguinetti, Borelli, Lucangelo, Bortolussi, "Temporal Logic Based Monitoring of Assisted Ventilation in Intensive Care Patients" (ISoLA 2014, LNCS 8803); Lamp, Silvetti, Breton, Nenzi & Feng, "A Logic-Based Learning Approach to Explore Diabetes Patient Behaviors" (CMSB 2019, LNCS 11773). Recent extensions include Cumulative-Time STL (CT-STL; Chen, Zhang, Roy, Bartocci, Smolka, Stoller & Lin, EMSOFT 2025; arXiv:2504.10325) for cumulative-duration clinical specifications and Temporal Ensemble Logic (TEL; Li et al., TIME 2025, LIPIcs vol. 355) for clinical-trial representation.

### Tooling/Ecosystem maturity
Very high. Industrial-grade model checkers; runtime monitors deployed in automotive/avionics. STL increasingly used in medical CPS (artificial pancreas, ventilation, ICU monitoring).

### Japan-specific considerations
- Ichiro Hasuo (NII) led the JST ERATO MMSD (Metamathematics for Systems Design) project, JPMJER1603, Oct 2016–March 2025 (concluded with the founding of Imiron Co. Ltd. via JST START); positioning paper Hasuo, "Metamathematics for Systems Design," *New Generation Computing* 35(3):271–305 (2017); homepage https://www.jst.go.jp/erato/hasuo/en/.
- Akazaki & Hasuo, "Time Robustness in MTL and Expressivity in Hybrid System Falsification," CAV 2015, LNCS 9207 pp. 356–374, introducing `AvSTL`.
- Sato, An, Zhang, Hasuo, "Optimization-Based Model Checking and Trace Synthesis for Complex STL Specifications" (CAV 2024, LNCS 14683; arXiv:2408.06983).
- Primary application focus is automotive (ISO 34502), not healthcare, but methods transfer.
- Ishii, Yonezaki, Goldsztejn — interval-analysis STL monitoring (IEICE).

### Interoperability
- FHIR: Observation streams feed STL monitors directly.
- openEHR: longitudinal EHR queries lower to LTL formulae.
- TLA+: temporal logic of actions; alternative spec language with strong tooling (TLC, Apalache).
- Lean/Rocq/Isabelle: LTL/MTL semantics formalized; verified monitors (Schneider et al., RV 2020).
- SMT: bounded model checking via Z3/CVC5.
- ASP/Datalog: telingo (temporal ASP, Cabalar et al.).

### Limitations/Known issues
MTL undecidability for full logic; STL robustness sensitive to signal noise; specification authoring is hard for clinicians without templates. State-space explosion for large patient cohorts.

### Training data proxy
Very strong. Pnueli's work in every PL/verification textbook; STL prominent in CPS/CAV/HSCC literature. Active GitHub (S-TaLiRo, Breach, RTAMT, MoonLight).

---

## 10. Multi-Criteria Decision Analysis for Preference-Sensitive Recommendations

### Purpose
Quantify trade-offs across heterogeneous criteria (efficacy, safety, cost, QoL, patient preference) when guideline recommendations are preference-sensitive; operationalize GRADE Evidence-to-Decision (EtD) and Shared Decision-Making.

### Maintainer/Standards body
No single body. Foundational frameworks: AHP (Saaty 1980), MAUT/MAVT (Keeney & Raiffa 1976), TOPSIS (Hwang & Yoon 1981), ELECTRE (Roy 1968+), PROMETHEE (Brans & Vincke 1985). GRADE working group (Guyatt et al.) maintains EtD; MCDA for healthcare: Marsh et al., "Multiple Criteria Decision Analysis for Health Care Decision Making — Emerging Good Practices: Report 2 of the ISPOR MCDA Emerging Good Practices Task Force," *Value in Health* 19(2):125–137 (2016).

### Conceptual model
Alternatives × criteria performance matrix; weighting (subjective via AHP pairwise comparison, or swing-weighting); aggregation (weighted sum / utility / outranking / distance to ideal). Outranking (ELECTRE/PROMETHEE) uses concordance/discordance to build a partial preorder; rank-reversal phenomena documented.

### Expressiveness/Semantics
Numeric, not logical. MAUT axiomatized (Keeney-Raiffa); AHP has Saaty's eigenvector justification; ELECTRE/PROMETHEE preference-thresholds. Fuzzy MCDA accommodates uncertainty.

### Composability/Modularity
Criteria hierarchies decompose recursively (AHP). Patient-specific weights can override population weights. GRADE EtD frameworks structure inputs into a fixed schema (problem, desirable effects, undesirable effects, certainty of evidence, values, balance of effects, resources, equity, acceptability, feasibility).

### Suitability for autoformalization to IR
Medium. IR can carry recommendation metadata: criteria list, alternative list, performance matrix, weights. Idempotency requires fixing the criteria ontology (e.g., GRADE EtD schema). Not a logic; pairs with deontic/argumentation layers for *what* to do once a ranking is produced.

### Formal verification potential
Low intrinsic: MCDA gives a ranking, not a proof. Some axiomatic guarantees (independence, transitivity) checkable on weight elicitation. Sensitivity analysis is standard but not "formal verification" in the model-checking sense.

### Tooling/Ecosystem maturity
- R: `topsis`, `MCDA`, `ahpsurvey`, `PROMETHEE`, `RMCDA` (Najafi & Mirzaei, arXiv:2502.08677, 2025).
- Python: `pymcdm`, `scikit-criteria`.
- MAGIQ, Hiview, Web-HIPRE (legacy).
- GRADEpro GDT — supports EtD frameworks; the SHARE-IT project (Agoritsas et al., *BMJ* 2015;350:g7624 and follow-ups) integrates decision aids with digital guidelines and GRADE evidence summaries.

### Japan-specific considerations
Minds (Medical Information Network Distribution Service, operated by Japan Council for Quality Health Care, JCQHC) uses GRADE methodology in clinical practice guideline development; see "Minds Guide for Developing Clinical Practice Guidelines Ver. 2.0" (PubMed 30620850, 2018), superseded by the "Minds Manual for Guideline Development 2020 ver. 3.0" (Minds Manual Development Committee, JCQHC, 2021). MID-NET® (PMDA) and JADER (Japanese Adverse Drug Event Report) datasets — see Fujiwara, Kawasaki & Yamada, *PLoS ONE* 11(4):e0154425 (2016) — feed evidence into the "desirable/undesirable effects" rows of EtD. No specific Japanese MCDA-CDS integration tool identified; guideline formalization in Japan is dominated by narrative methodology rather than executable formalisms (identified gap).

### Interoperability
- FHIR: Patient `goal`, `Consent`, `RiskAssessment` resources can hold patient preferences/weights.
- openEHR: `EVALUATION` archetypes can record preference data.
- DMN: weighted-sum decision tables map MCDA aggregation.
- OWL/SHACL: criteria/alternatives ontology; SHACL validation of completeness.
- SMT/ASP: outranking formulations can be encoded but seldom done.

### Limitations/Known issues
Weight elicitation is fragile; rank-reversal across methods (especially TOPSIS, AHP); criteria not always preferentially independent. Patient-level customization scales poorly. MCDA outputs are not justifications — they need to be wrapped in an argumentation layer (ASPIC+/Carneades) for clinician trust.

### Training data proxy
Very strong in operations research / HTA literature; weaker in CS conferences. Textbooks: Belton & Stewart, *Multiple Criteria Decision Analysis: An Integrated Approach* (Kluwer 2002); Greco, Ehrgott & Figueira eds., *Multiple Criteria Decision Analysis: State of the Art Surveys*, 2nd ed. (Springer 2016). GitHub coverage modest but growing (pymcdm, RMCDA, scikit-criteria).