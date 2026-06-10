# Category 9: Evaluation, Clinical Validation, and Human-Centered CDS

## 1. Gold-Standard Guideline-to-IR Corpus with Clinician/Formalist Adjudication

### Purpose
Construct dual-annotated corpora of clinical practice guideline (CPG) text paired with formal intermediate representation (IR), to train, evaluate, and regression-test autoformalization pipelines and serve as a frozen "gold standard" for semantic equivalence judgments.

### Maintainer/Standards body
No single steward. Reference precedents: GLIF3 (InterMed Collaboratory: Columbia, Harvard/Brigham & Women's, Stanford); Asbru library (Vienna University of Technology / Ben-Gurion University, Asgaard project); PROforma exemplars (Cancer Research UK ACL → OpenClinical.net / InferMed Arezzo / Oxford Tallis); EBM-NLP (Nye et al., ACL 2018; Northeastern / UT Austin / KCL); CPG-on-FHIR (HL7 CDS WG, package `hl7.fhir.uv.cpg#2.0.0` STU2, FHIR R4); GGPONC 2.0 (Charité / German Guideline Program in Oncology) for SNOMED-CT-grounded German oncology corpus.

### Conceptual model
Each corpus item = `(source_passage, IR_artifact, provenance_metadata, adjudication_trace)`. Adjudication is two-stage: (a) independent annotation by ≥2 annotators (one clinical, one formal-methods); (b) adjudication by a third senior reviewer when disagreement exceeds threshold. IAA computed with Cohen's κ (two annotators, categorical), Fleiss' κ (≥3 annotators), Krippendorff's α (interval/missing data), or γ-agreement for span-level (GGPONC 2.0 reports γ = .94 on SNOMED-CT top-level annotation).

### Expressiveness/Semantics
Schema determined by target IR. JSON Schema for the IR annotation envelope + nested formal payload (e.g., CQL ELM, FHIR PlanDefinition/ActivityDefinition, GLIF3 flowchart node, Asbru plan, PROforma task network, OWL axiom, SMT-LIB, Lean term). Annotation captures: recommendation strength, GRADE certainty, deontic operator, temporal qualifier, eligibility constraint, action class, evidence citation, exception clauses.

### Composability/Modularity
Items sliced at the recommendation / CQ (Clinical Question) granularity, aligning with Minds CQ structure. Corpus is composable across guideline pairs (enables Method 3 contradiction benchmarks) and across IR targets (multi-target reference allows comparing IR languages).

### Suitability for autoformalization to IR
Direct: this is the canonical evaluation substrate. Train/dev/test splits stratified by guideline, society, and recommendation type. Supports few-shot retrieval, supervised fine-tuning, RLHF reward modeling, and held-out integrity testing. EBM-NLP (4,993 abstracts; PICO spans; MIT license, GitHub `bepnye/EBM-NLP`) provides PICO-level supervision; CPG-on-FHIR pilots (WHO SMART Guidelines for ANC, immunizations; CDC opioid prescribing) provide whole-recommendation supervision.

### Formal verification potential
Indirect. Corpus is the test oracle for downstream verification: each gold IR must itself type-check, parse, and pass round-trip identity under the IR's own conformance tools (e.g., CQL-to-ELM, FHIR `$validate`, Asbru parser). Verification of IR consistency is delegated to Methods 3 and 5.

### Tooling/Ecosystem maturity
Mature general NLP annotation tooling (brat, INCEpTION, Prodigy, Label Studio, MAE). Clinical-domain corpora published: EBM-NLP, GGPONC 2.0 (ACL Anthology 2022.lrec-1.389). Adjudication protocol patterns are well-established (TRUST process; iterative guideline refinement with intra-/inter-annotator F-measure ≥ 0.93 reported in clinical syntactic parsing work).

### Japan-specific considerations
Minds (JCQHC) publishes `Minds 診療ガイドライン作成マニュアル 2020 Ver. 3.0`; recommendations are structured at the CQ level following GRADE. No publicly released JSON/XML schema for Minds diagnostic guidelines was located. Society guidelines (e.g., JCS/JHFS 2025 Heart Failure Guideline, published in full-text XML on J-STAGE) are candidate raw inputs. JP Core v1.2.0 (released December 2024 by the JAMI NeXEHRS Academic Research Group) provides Japan-localized FHIR profiles for the IR target side. AMED funding mechanisms (e.g., the program that has supported JRS J-MID since 2017) are the realistic pathway for corpus construction; no AMED-funded autoformalization-of-CPG project (2022–2025) was specifically identified. JCQHC adjudication infrastructure (Minds peer review committees) is a natural fit for clinician annotation.

### Interoperability
Output IR fields map to: Category 1 FHIR R4 (PlanDefinition, ActivityDefinition, Library, Measure); CQL ELM; openEHR archetypes; Category 2 OWL/SHACL ontology bindings (SNOMED CT, LOINC, ICD-10/11, JLAC10, YJ codes); Category 3 deontic and defeasible logic literals; Category 4 Lean/Rocq/Isabelle term constructors; Category 5 SMT-LIB; Category 6 retrieval signals. Provenance fields (BPMN/DMN traces) link to Category 8.

### Limitations/Known issues
Annotation cost (clinical + formalist labor); IAA degrades on long, ambiguous, deontically-rich passages; GLIF3 and Asbru never produced a large public annotated corpus; PROforma exemplars (CREDO, LISA, ORAMA) are small. Selection bias toward English-language CPGs. Risk of "schema lock-in" — the choice of IR shapes future annotations.

### Training data proxy
Moderate. GitHub (`bepnye/EBM-NLP`, `cqframework/`, `HL7/cqf-recommendations`), AMIA / MedInfo / JAMIA / npj Digital Medicine / JBI literature; weaker for Japanese (J-STAGE 医療情報学雑誌, JAMI proceedings).

---

## 2. Semantic Equivalence, Idempotency, and Convergence Benchmarks

### Purpose
Quantify multi-run consistency of LLM autoformalization: same input → same (or semantically equivalent) IR across runs, prompt paraphrases, and model versions.

### Maintainer/Standards body
No formal body. Methodological precedents: ProofNet (Azerbayev et al., arXiv:2302.12433, Lean 3 + mathlib, 371 parallel statement–proof items); FormalAlign (arXiv:2410.10135, dual-loss alignment scoring; reported Alignment-Selection Score 99.21% vs GPT-4 88.91% on FormL4-Basic); BEq / BEq+ (Poiroux, Weiss, Kunčak, Bosselut, arXiv:2406.07222, "Reliable Evaluation and Benchmarks for Statement Autoformalization"; reference-based metric correlated with human eval; current techniques achieve up to 45.1% on undergraduate-math autoformalization); ProofNetVerif (labeled correct/incorrect autoformalization pairs).

### Conceptual model
Run autoformalizer N times on identical input. Metrics: (i) syntactic identity rate, (ii) AST-level graph-isomorphism rate over canonicalized IR, (iii) logical-equivalence rate via ATP / SMT under a shared theory, (iv) embedding-cosine + neural alignment (FormalAlign-style), (v) idempotency = `f(f(x)) ≡ f(x)`, (vi) round-trip = `informalize(formalize(x)) ≈ x`. Convergence rate = fraction of runs landing in the dominant equivalence class. Test-retest reliability via Krippendorff's α on categorical IR features.

### Expressiveness/Semantics
Equivalence is theory-relative. Surface-string equivalence < α-equivalence < β-η-equivalence < first-order logical equivalence < theory-modulo equivalence (SMT, ASP). BLEU/BERTScore are explicitly inadequate for formal-language equivalence.

### Composability/Modularity
Composable with Method 1 (gold corpus) for reference-based equivalence (BEq+), and with Method 5 (metamorphic relations) where idempotency is itself an MR. Pluggable equivalence checkers: SMT solver (Z3, cvc5), tactic-based proof (Lean `decide`, `omega`), ELM AST normalizer, OWL reasoner (HermiT, Pellet).

### Suitability for autoformalization to IR
Central. Idempotency is the operational success criterion specified by the user. Convergence rate is a direct optimization target. FormalAlign-style alignment training and BEq+ reference-based scoring are directly portable substrates.

### Formal verification potential
High — equivalence checking is itself a verification task; passes through SAT/SMT/ATP. False-equivalence risk (incomplete proof procedure → conservative inequality).

### Tooling/Ecosystem maturity
Mature for math (Lean mathlib, ProofNet); immature for clinical IR. CQL-to-ELM canonicalization exists but no consensus semantic-equivalence benchmark for clinical guidelines. FHIR `$validate` is structural, not semantic.

### Japan-specific considerations
No Japanese-language autoformalization benchmark identified. JAMI / JSAI / 言語処理学会 venues are appropriate publication outlets; PMDA's 2024 "Report on AI-based SaMD" (PMDA file 000266100) emphasises transparency, data quality and reproducibility — idempotency metrics directly support PMDA reproducibility expectations.

### Interoperability
Equivalence checkers consume Category 4 (Lean, Isabelle, Rocq) and Category 5 (SMT-LIB, ASP) backends. AST canonicalization layers attach to Category 1 IRs. Test harness can emit Category 8 traces (BPMN/DMN execution paths).

### Limitations/Known issues
Logical equivalence is undecidable in general; theory-restricted decision procedures must be chosen. ATP timeouts conflate "different" with "unproved equivalent". Embedding metrics (BERTScore, BLEU) systematically over-reward surface similarity.

### Training data proxy
Strong in AI/math (ProofNet, FormalAlign, MiniF2F on arXiv, NeurIPS, ICLR). Weak in clinical NLP — gap to be filled.

---

## 3. Contradiction/Collision Benchmark with Synthetic and Real Cross-Guideline Cases

### Purpose
Measure precision/recall of contradiction detection between recommendations from two or more guidelines covering overlapping clinical territory (multimorbidity, drug-drug interaction, screening thresholds, anticoagulation/antiplatelet regimens, oncology dosing).

### Maintainer/Standards body
Research-community-driven. Key precedents: GLINDA (Stanford BMIR + VA, Musen/Tu; combined BioSTORM with ATHENA-EON; demonstrated on JNC-7 + VA CKD); the BPMN-derived constraint-solver framework for multimorbidity polypharmacy (PMC6993806); the disagreement-detection system between guidelines and new evidence (PMC8861732, oncology trial-vs-guideline detection); contradiction modeling (arXiv:1708.00850, mammography ACOG vs USPSTF case study). Note: GLINDA (Stanford) and GLIDE (McMaster/Yale, Shiffman et al.) are distinct projects often conflated in the literature.

### Conceptual model
Benchmark = `{(guideline_pair, patient_context, expected_conflict_label, expected_resolution)}`. Sources: (a) synthetic pairs generated by perturbing single guidelines (negation, threshold shift, eligibility narrowing); (b) curated real-world pairs (AHA vs ESC anticoagulation, ACOG vs USPSTF mammography, JCS vs ESC heart failure, ASCO vs ESMO oncology). Metrics: precision, recall, F1 on contradiction class; resolution-suggestion accuracy; clinician utility rating.

### Expressiveness/Semantics
Conflicts decomposed into: direct (logical negation), action (drug A vs drug B simultaneously), temporal (incompatible ordering), threshold (different cutoffs on same variable), epistemic (different evidence grading). Captured in defeasible/deontic logic (Category 3) or as constraint-system violations.

### Composability/Modularity
Pairs constructed from Method 1 corpus items. Conflict detectors are plug-in modules over the IR — SMT for threshold/arithmetic, ASP for normative conflicts, OWL reasoning for ontological inconsistency, BPMN/DMN simulators for workflow clashes.

### Suitability for autoformalization to IR
A primary downstream validation task: if the IR is correct, contradictions must be detectable mechanically. Negative results indicate either autoformalization failure or genuine guideline incompatibility.

### Formal verification potential
Highest of any Category-9 method. Native to SMT/SAT/ASP and theorem provers. The PMC6993806 framework demonstrates constraint-solver-based conflict detection with a metric for selecting "best" resolutions.

### Tooling/Ecosystem maturity
Low: no widely-adopted public benchmark. TREAT (Treat Acutely) and similar projects are localized. Most published work uses ad-hoc two-guideline pairs.

### Japan-specific considerations
Critical for Japan: Japanese society guidelines (JCS, JSH, JDS, JSN, JGES, JSCO) often diverge from ESC/AHA/ASCO due to population-specific evidence (e.g., DOAC dosing thresholds for low-body-weight Japanese patients, JCS atrial fibrillation guidance vs ESC). The CDS must adjudicate or surface these. Minds CQ structure facilitates pair construction. JADER (PMDA, publicly downloadable CSV since 2012) provides empirical DDI grounding.

### Interoperability
Consumes Categories 1–5 IRs; emits explanations consumable by Category 7 (retrieval/citation) and Category 10 (explanation evaluation). DDI cross-checking integrates with HL7 CDS Hooks `medication-prescribe` hook.

### Limitations/Known issues
Ground-truth "true contradiction" labels require senior clinician adjudication and are context-dependent. Many apparent conflicts dissolve under finer patient stratification. Synthetic-only benchmarks risk distribution shift from real conflicts.

### Training data proxy
Moderate. JAMIA, JBI, AIME (CIG workshop series), AMIA proceedings. Drug-interaction conflicts well-covered in pharmacovigilance literature; oncology and cardiology guideline-harmonization literature exists.

---

## 4. CQL/FHIR/DMN Conformance and Unit-Test Suites

### Purpose
Mechanically verify that an IR (or its execution engine) conforms to standard specifications and produces correct outputs on standardized test inputs.

### Maintainer/Standards body
- CQL: HL7 Clinical Decision Support WG; current Normative Release 1 = CQL v1.5.3; Trial-Use Ballot CQL 2.0.0 in progress (`build.fhir.org/ig/HL7/cql/`).
- CQL test suite: tests within HL7/cql repository (XML format identical to FHIRPath tests); `cqframework/clinical_quality_language` (CQL-to-ELM reference translator).
- FHIR validation: HL7 FHIR `$validate` operation; Inferno Framework (ONC HealthIT, `inferno-framework`, ONC (g)(10) Standardized API Test Kit v8.0.0, last updated 9 March 2026; HTI-1 Final Rule support, monthly release cadence); HAPI FHIR Validator.
- DMN TCK: community-led, vendor-neutral (`dmn-tck/tck` on GitHub); compliance-level-3 test cases (FEEL expressions, decision tables).
- CQF (Clinical Quality Framework): originated 2013 CMS+ONC, transitioned to HL7 CQI + CDS WGs in 2016.
- OHDSI Achilles: Apache 2.0; pre-computes >170 characterization analyses on OMOP CDM v5.3/5.4; Data Quality Dashboard (DQD) is the modern DQ-check companion (Kahn-framework checks).

### Conceptual model
Black-box conformance: `(spec_section_id, test_input, expected_output, tolerance)`. BDD layer (Cucumber / Gherkin) wraps "Given–When–Then" patient scenarios. Inferno's Ruby DSL adds `assert_conformance_to_logical_model`, `assert_must_support_elements_present`, `resource_is_valid?`.

### Expressiveness/Semantics
CQL: full HL7-specified semantics (set/list/interval, temporal, terminology operators). FHIR: structural + Must-Support + terminology binding. DMN: FEEL expressions + decision-table hit policies. All are deterministic and finitely-decidable.

### Composability/Modularity
Highly modular: each test case is a self-contained file. Inferno Test Kits compose into Test Suites; DMN TCK tests target specific FEEL constructs.

### Suitability for autoformalization to IR
High for the CQL/FHIR/DMN target side: any autoformalized artifact must pass conformance before any clinical claim. Low for the natural-language source side — these suites test the IR engine, not the NL→IR translator.

### Formal verification potential
Conformance is bounded testing, not verification. However, DMN's S-FEEL fragment and CQL subsets are amenable to SMT translation.

### Tooling/Ecosystem maturity
Mature: Inferno deployed on `inferno.healthit.gov`; DMN TCK has multi-vendor (Camunda, Drools/Fujitsu DXP, OpenRules, Oracle, ACTICO) implementations. CQL evaluation services emerging (CQF Ruler, dqm-engine).

### Japan-specific considerations
JP Core v1.2.0 (JAMI NeXEHRS, December 2024) defines Japan-realm FHIR profiles requiring custom Inferno Test Kits. PMDA's "Dash for SaMD2" (2024–2028 five-year plan, 6-month SaMD review target announced 22 January 2024) and IDATEN framework (pre-approved post-marketing modifications) expect software-engineering rigor including conformance testing. JAMI standards JAMISDP02 (Prescription HL7 FHIR), JAMISDP03 (Health Examination Result Report HL7 FHIR), JAMISDP04 (ePath data elements), HL7J-FHIR-001 (Medical Information Provision Form), HL7J-FHIR-002 (Discharge Summary) define the local FHIR resources to be validated.

### Interoperability
Native to Category 1 (FHIR/CQL/openEHR) and Category 8 (DMN/BPMN). Achilles/DQD integrate with Category 6 OMOP-based retrieval/cohorting.

### Limitations/Known issues
Conformance ≠ clinical correctness. Inferno tests check IG conformance, not patient outcomes. DMN TCK omits Level-4 vendor-specific extensions. CQL ELM normalization is canonical but not unique up to semantic equivalence.

### Training data proxy
Strong (CQL/FHIR/DMN heavily represented on GitHub; HL7 Confluence; AMIA Clinical Informatics Conference 2025 posters reference these).

---

## 5. Metamorphic and Property-Based Tests for Guideline-to-IR Transformations

### Purpose
Generate test cases via metamorphic relations (MRs) and randomized property-based generators; detect autoformalizer faults without an oracle.

### Maintainer/Standards body
Methodological foundations: Chen, Cheung, Yiu (1998) for metamorphic testing; QuickCheck (Claessen & Hughes 2000, Haskell) → Hypothesis (David MacIver, Python), fast-check (TypeScript), ScalaCheck. METAL framework (arXiv:2312.06056) and follow-ups (arXiv:2511.02108 — extending MT to LLMs across NLP tasks; arXiv:2603.24774 — MT-LLM survey "From Untestable to Testable"; arXiv:2406.06864 — "Validating LLM-Generated Programs with Metamorphic Prompt Testing") extend MT to LLMs.

### Conceptual model
Define metamorphic relations: paraphrase invariance (`f(paraphrase(x)) ≡ f(x)`); idempotency (`f(f(x)) ≡ f(x)`); commutativity of merging unrelated CQs; monotonicity of recommendation strength under stricter eligibility; associativity of conjunctive eligibility. Property generators sample synthetic CQs and check invariants over thousands of runs. The MT-LLM survey reports a manually-validated true-positive rate of ~62% across 937 oracle violations.

### Expressiveness/Semantics
MRs express necessary conditions on transformation, not full specification. Property tests cover Hyland-style preconditions/postconditions. Strong for robustness/fairness; weak for semantic correctness without an oracle.

### Composability/Modularity
Highly composable: MRs are first-class objects, parametrized over input distributions, transformation operators, and equivalence predicates.

### Suitability for autoformalization to IR
Direct fit. Idempotency (a stated success criterion) is implementable as an MR. Paraphrase invariance tests robustness to Japanese ↔ English rendering or to surface variation in Minds CQ phrasing. Property-based testing fits clinical-rule evaluators: recommendation-strength monotonicity under eligibility tightening, etc.

### Formal verification potential
MRs are sound necessary-condition checks. Property tests do not constitute formal verification but increase coverage and fault-discovery rate. Per arXiv:2406.06864 ("Validating LLM-Generated Programs with Metamorphic Prompt Testing"): "metamorphic prompt testing is able to detect 75% of the erroneous programs generated by GPT-4, with a false positive rate of 8.6%" on HumanEval.

### Tooling/Ecosystem maturity
Mature general-purpose: Hypothesis, QuickCheck, fast-check, ScalaCheck. LLM-targeted: METAL, LLMORPH, MT-LLM frameworks. Clinical-MT example: Jaganathan, Kahanda, Kanewala 2025 (ICD-coding robustness/fairness MT on BioMed LLMs, Smart Health vol. 36, article 100564, doi:10.1016/j.smhl.2025.100564, ScienceDirect S235264832500025X).

### Japan-specific considerations
Japanese paraphrase MRs require Japanese clinical-text generators and back-translation tooling. JIS Z 8520:2022-aligned dialogue principles place additional constraints. PMDA SaMD review expects systematic testing evidence; MT logs are auditable.

### Interoperability
MR-driven test harnesses consume the IR and equivalence checker from Method 2. Outputs feed Category 7 retrieval drift detection and Category 8 trace assertions.

### Limitations/Known issues
False-positive rate ~38% in published LLM-MT studies; MR design is itself a research problem. Defining "equivalence" for clinical IR is contested. Distribution of generated cases may not match real CPG distribution.

### Training data proxy
Strong in software-engineering literature (ICSE, ISSTA, ICSME, ASE); growing in NLP (ACL, EMNLP); thin in clinical-MT.

---

## 6. Shadow-Mode / Silent-Trial CDS Evaluation

### Purpose
Run a candidate CDS in production against live data without surfacing outputs to clinicians, to measure real-world performance, alert load, and timing prior to active deployment.

### Maintainer/Standards body
No formal standard. Regulatory acceptance: FDA SaMD pre-market evidence framework; FDA Clinical Decision Support Software Final Guidance (issued 6 January 2026, re-issued 29 January 2026; town hall 11 March 2026; FDA document 191560) supersedes the September 2022 CDS guidance and expands enforcement discretion to single-recommendation outputs satisfying Non-Device CDS criteria including AI features when clinicians can understand and verify underlying logic and data inputs; PMDA permits silent validation under the IDATEN framework and Dash for SaMD2. DECIDE-AI reporting guideline (Vasey et al., Nat Med 2022) covers early-stage live evaluation.

### Conceptual model
`Deployment with shadow_mode=true`: model runs on production data stream, generates outputs to a logging endpoint only, no clinician notification. Metrics: AUROC, calibration, lead-time vs. confirmed outcome, alert volume per 1000 patient-days, would-be override rate (proxy), workflow-impact estimation. Pre-registers primary/secondary endpoints, sample-size calculations, and stopping rules.

### Expressiveness/Semantics
Empirical, statistical. Architecturally orthogonal to the CDS logic — same artifact runs silent or live by configuration flag.

### Composability/Modularity
Multiple algorithms can be shadowed in parallel. Trial NCT05943938 (Emory) compared Epic Sepsis Model V1, V2, and an internal Emory model in shadow mode.

### Suitability for autoformalization to IR
Indirect but essential. After autoformalization + verification produce a candidate rule, silent-trial deployment quantifies real-world conformity to clinician behavior and exposes IR errors that only surface against patient-data distribution shift.

### Formal verification potential
None directly; complementary to verification.

### Tooling/Ecosystem maturity
Mature: Epic supports "silent mode" Best Practice Advisories; TREWS at Johns Hopkins demonstrated silent → live transition across five hospitals (n = 590,736 monitored patients; Adams et al., Nat Med 2022, in-hospital mortality reduction 3.3% absolute when alert acted on within 3 h); Epic Sepsis Model V2 validation (Farooq et al., AMIA CIC 2025 poster P42: non-sepsis positive alert rate 3.3%→1.2%; sepsis encounters 26.1%→6.8%); silent gradient-boosted sepsis model evaluated against Sepsis-3, SEP-1, CDC ASE across 9 hospitals (Dutta et al., data collected 2024; JAMA Network Open April 2026).

### Japan-specific considerations
PMDA's two-step approval model (2024 onward) and Dash for SaMD2 (MHLW September 2023 five-year strategy) permit conditional clinical-utility evidence — silent-trial logs can constitute the early-evidence dossier. Japan's AI Promotion Act enacted June 2025 establishes a national AI Strategy Headquarters and mandates a Basic AI Plan; unlike the EU AI Act, the Japanese law imposes no penalties and only light obligations on developers, though PMDA continues to expect scientifically robust autonomous-update demonstrations (including explainable AI) for adaptive SaMDs under IDATEN. AMED grants commonly require real-world evidence components. JADER and J-MID (>534M images, 1.65M cases as of April 2024) are candidate data sources for retrospective silent evaluation. MHLW Next-Generation Medical Infrastructure Act (次世代医療基盤法) governs the legal basis for using de-identified clinical data.

### Interoperability
Requires Category 1 EHR integration (FHIR Subscriptions for streaming; CDS Hooks 2.0.1 STU2 (March 2025) for hook-based invocation without UI surfacing). Outputs flow to Category 8 audit logs.

### Limitations/Known issues
Selection bias if shadow population differs from intended deployment; "label leakage" when proxy outcomes are downstream of standard care; cannot capture clinician–CDS interaction effects (counterfactual unobservable); cannot detect automation bias. Northwell systematic review (2025/2026, J Gen Intern Med, doi:10.1007/s11606-026-10381-y) found no model exceeded mean AUROC 0.79 across 22 Epic CDS validation studies (Epic Sepsis Model AUROC 0.65; Epic Deterioration Index 0.79), under-performing vendor-reported intervals.

### Training data proxy
Moderate (JAMIA, NEJM Catalyst, Nat Med, npj Digital Medicine).

---

## 7. Alert-Fatigue Mitigation and Tiered Alert Governance

### Purpose
Reduce nuisance alerting (overall pooled DDI override prevalence of 90%, CI95% 85–95%, across 11 studies and 570,776 prescriptions per Felisberto et al., Health Informatics Journal 2024;30(2), doi:10.1177/14604582241263242) by tiering severity, suppressing low-value alerts, and instituting alert governance.

### Maintainer/Standards body
Methodological precedents: Ancker JS, Edwards A, Nosal S, Hauser D, Mauer E, Kaushal R. "Effects of workload, work complexity, and repeated alerts on alert fatigue in a clinical decision support system." BMC Med Inform Decis Mak 2017;17:36 (doi:10.1186/s12911-017-0430-8); Phansalkar, van der Sijs, Tucker et al., JAMIA 2013;20(3):489–93 (consensus list of 33 class-based low-priority DDIs that should be non-interruptive); van der Sijs et al. JAMIA 2008;15:439 ("Turning off frequently overridden drug alerts: limited opportunities for doing it safely"); Bates/Kuperman ten commandments (JAMIA 2003;10(6):523–30); I-MeDeSA instrument (Zachariah et al., JAMIA 2011 Suppl 1:i62). CDS Hooks (HL7 STU2 v2.0.1, March 2025; CC-BY-4.0) provides "information card", "suggestion card", "app link card" types as a tiering mechanism.

### Conceptual model
Three-tier (or four-tier) design: hard-stop (must acknowledge with override reason) / soft-stop (interruptive but dismissible) / informational (non-interruptive, in-line). Governance committee reviews override rates, false-positive rates, time-to-action, and alert-to-action ratio. Contextual relevance scoring multiplies a base severity by patient-context factors.

### Expressiveness/Semantics
Tier and channel are properties of the alert artifact; logic generates content while the delivery layer applies tiering. Override-reason taxonomy is part of the schema.

### Composability/Modularity
Tier policy is a decoupled layer over rule evaluation. CDS Hooks cards externalize delivery format.

### Suitability for autoformalization to IR
The IR must carry recommendation strength + actionability metadata; tiering is a post-IR projection. The Phansalkar 33-DDI non-interruptive list is itself a formalizable knowledge artifact suitable for IR encoding.

### Formal verification potential
Low; tiering is a policy decision. Can be checked for consistency (no rule both hard-stop and informational; no two rules issuing contradictory tiers on the same trigger).

### Tooling/Ecosystem maturity
Mature in commercial EHRs (Epic, Cerner). Open: CDS Hooks library `hl7.fhir.uv.cds-hooks-library` v1.0.1 (R4, generated 2025-03-12). Governance practices documented but not standardized.

### Japan-specific considerations
Japanese hospital adoption of DDI knowledge bases (e.g., JAPIC, IMSpro) varies; PMDA-regulated SaMD must follow IEC 62366-1 usability requirements when interruptive alerts are involved. JADER can inform localized priority of DDI alerts. AAMI HE75:2025 alignment with FDA HF expectations affects design of Japan-marketed SaMD as well.

### Interoperability
Receives Category 3 deontic/argumentation severity; emits via CDS Hooks (Category 1). Override-rate metrics feed Category 12 RE-AIM Implementation dimension.

### Limitations/Known issues
"Turning off" alerts has liability implications (Kesselheim, Cresswell, Phansalkar, Bates, Sheikh — Health Affairs 2011;30(12):2310-7). Over-suppression risks missed-harm. Override-rate is a noisy proxy for alert quality.

### Training data proxy
Strong (JAMIA, Health Affairs, AMIA Annual Symposium proceedings).

---

## 8. CDS Five Rights

### Purpose
Apply Osheroff et al.'s five-dimensional design heuristic to ensure CDS interventions deliver the right information, to the right person, in the right format, through the right channel, at the right time.

### Maintainer/Standards body
Originated by Jerome Osheroff et al. (HIMSS guidebook "Improving Outcomes with Clinical Decision Support: An Implementer's Guide," 2nd ed., 2012); endorsed by AHRQ (CDS Connect); CMS recommends the framework as best practice for health-IT-enabled QI.

### Conceptual model
A configuration space spanned by five orthogonal axes: information (evidence-based, actionable), person (clinician/patient/caregiver/care-team), format (alert/order set/info button/dashboard/storyboard BPA), channel (EHR/portal/mobile/SMS), time (pre-encounter/at-order/post-result/longitudinal).

### Expressiveness/Semantics
Heuristic, not formal. Operationalized via CDS/QI Worksheets and Health Service Blueprints.

### Composability/Modularity
Each Right is independently designable; intervention specifications enumerate the choice on each axis.

### Suitability for autoformalization to IR
The IR should annotate each recommendation with default Five-Rights choices to drive deployment; LLM-augmented CDS can leverage these as templating slots. AHRQ CDS Connect demonstrated guideline-to-CDS translation (cholesterol management proof-of-concept) producing shareable, interoperable artifacts.

### Formal verification potential
None. Audit-evaluable via checklist.

### Tooling/Ecosystem maturity
AHRQ CDS Connect public artifact repository; vocabulary for CDS-Hooks contexts aligns with "right channel/time".

### Japan-specific considerations
MHLW Society 5.0 / 医療DX (medical DX) initiatives align with Five-Rights framing. Patient-channel adoption in Japan favors patient portals tied to マイナンバー (My Number) integration and PHR initiatives.

### Interoperability
Maps to Category 1 (channel = FHIR/CDS Hooks endpoint), Category 6 (right time = retrieval-trigger event), Category 9 explanation rendering (right format).

### Limitations/Known issues
Imprecise definitions; not a substitute for usability engineering. Conflated with the medication "Five Rights" (different framework).

### Training data proxy
Strong (AHRQ documents, AMIA, HIMSS).

---

## 9. Human Factors Engineering and ISO 9241-210 User-Centered Design

### Purpose
Apply the human-centred design process to reduce use-error risk and improve usability, accessibility, and clinician cognitive load.

### Maintainer/Standards body
ISO 9241-210:2019 (Ergonomics of human-system interaction — Part 210: Human-centred design for interactive systems; ISO TC 159/SC 4); IEC 62366-1:2015 +A1:2020 / ANSI/AAMI/IEC 62366-1:2015 (R2021)+AMD1:2020 (Medical devices — Application of usability engineering); ANSI/AAMI HE75:2009 (R2018), updated as HE75:2025 (maps HFE activities to design controls per AAMI TIR59); ISO 14971:2019 (risk-management linkage). JIS Z 8520:2022 is the Japanese national adoption (IDT translation) of ISO 9241-110:2020 "Interaction principles" (replacing the earlier JIS Z 8520:2008 "Dialogue principles"); JIS Z 8521 corresponds to ISO 9241-11.

### Conceptual model
Iterative loop: (1) understand context of use → (2) specify user requirements → (3) produce design solutions → (4) evaluate against requirements. Evaluation modalities: formative (during design) and summative (validation). Specific techniques: HFMEA, cognitive walkthrough, think-aloud, NASA-TLX (workload), SUS, usability testing with ≥15 participants per user group.

### Expressiveness/Semantics
Process standard, not product standard. Outputs include Use Specification, User Interface Specification, Known Use Errors, Usability Engineering File (UEF), URRA (use-related risk analysis).

### Composability/Modularity
Modular activities; HE75:2025 explicitly maps HFE activities to medical-device design controls.

### Suitability for autoformalization to IR
Indirect; HFE governs the human-facing surface of the CDS, not the IR itself. However, IR fields should support generation of formative-evaluation artifacts (use-case scripts, error scenarios).

### Formal verification potential
None.

### Tooling/Ecosystem maturity
Highly mature regulatory ecosystem; FDA 2016 HF guidance is the de facto global benchmark.

### Japan-specific considerations
PMDA cites IEC 62366-1 in SaMD review; JIS Z 8520:2022 is binding in Japan as the IDT national standard for human-system interaction principles. JIS T 0601-1-6 (collateral standard for usability of medical electrical equipment, IEC 60601-1-6 IDT) applies for SaMD-as-feature-of-device. AMED requires usability engineering files for medical-device grants.

### Interoperability
Outputs (Use Specification, URRA) feed Category 12 implementation evaluation; HFMEA results inform Category 7 alert tiering.

### Limitations/Known issues
Resource-intensive; small-N summative tests under-power statistical claims; LLM-CDS specifically lacks established HFE patterns for handling probabilistic/hallucinated outputs.

### Training data proxy
Strong (AAMI, IEC, ISO documents; JEITA Healthcare IT Committee publications).

---

## 10. Explanation Quality Evaluation: Traceability, Proof Readability, Clinical Actionability

### Purpose
Rate the quality of CDS explanations along axes of citation precision, proof/derivation readability, clinical actionability, and decision-time utility.

### Maintainer/Standards body
No standard. Methodological inputs: G-Eval (Liu et al., "G-Eval: NLG Evaluation using GPT-4 with Better Human Alignment", EMNLP 2023, arXiv:2303.16634; LLM-as-judge with chain-of-thought, log-probability-weighted scoring; Spearman 0.514 on summarization SummEval); MedThink-Bench with LLM-w-Rationale (Pearson r up to 0.87 vs. expert evaluation at 1.4% of evaluation time, npj Digital Medicine 2025/2026, PMC12796170, doi:10.1038/s41746-025-02208-7); Naproche/Naproche-SAD (controlled-NL proof checker, Cramer et al.); ForTheL (formal-theory-language for natural-style mathematics).

### Conceptual model
Multi-dimensional rubric per explanation: (a) citation precision (% statements with verifiable, correct source), (b) proof readability (controlled-NL score; cyclomatic complexity of derivation; expert Likert), (c) clinical actionability (Likert: would the explanation change my action?), (d) faithfulness (does the explanation reflect the IR's actual reasoning, not post-hoc rationalization?), (e) decision-time efficiency (seconds-to-comprehend).

### Expressiveness/Semantics
Mix of automatable (citation match, lexical overlap with IR predicates) and human-judgment metrics. G-Eval and LLM-w-Rationale provide scalable proxies but suffer from position bias and verbosity bias (per "The Silent Judge", arXiv:2509.26072).

### Composability/Modularity
Stackable rubrics; each axis evaluated independently and aggregated.

### Suitability for autoformalization to IR
Direct: explanations are generated from the IR; quality of explanation is a downstream indicator of IR fidelity. Citation precision ties to Category 7 retrieval-citation linking.

### Formal verification potential
Controlled-NL (Naproche-SAD, ForTheL) provides machine-checkable proof prose — high potential when the target IR is a Lean/Isabelle term that can be paraphrased through CNL.

### Tooling/Ecosystem maturity
G-Eval, DeepEval, Opik, EvalAssist (AAAI 2025), RAGAS available as OSS. Naproche-SAD is research-grade. Clinical-specific rubrics not standardized.

### Japan-specific considerations
Japanese-language explanation quality requires Japanese-tuned LLM judges; bias profiles of judge models in Japanese are under-studied. PMDA's 2024 transparency expectations (per file 000266100 "Report on AI-based SaMD") make explanation traceability a regulatory asset.

### Interoperability
Bridges Category 7 (retrieval/citations) and Category 4 (formal proofs). Feeds Category 11 reporting checklists (TRIPOD-LLM emphasizes human oversight + task-specific performance reporting).

### Limitations/Known issues
LLM-judge calibration drifts across model versions; explanations risk plausible-but-unfaithful confabulation; clinician annotators are expensive and inter-rater variability is high.

### Training data proxy
Moderate (NeurIPS, EMNLP, ACL; npj Digital Medicine — MedThink-Bench).

---

## 11. Equity, Subgroup, External-Validation, and Calibration Analyses

### Purpose
Detect performance disparities across patient subgroups; validate on geographically/temporally distinct cohorts; assess calibration; ensure standardized reporting.

### Maintainer/Standards body
TRIPOD+AI (Collins, Moons, Dhiman et al., BMJ 2024;385:e078378, doi:10.1136/bmj-2023-078378, published 16 April 2024; an updated TRIPOD+AI adherence assessment tool was published in J Clin Epidemiol 2025, doi:10.1016/j.jclinepi.2025.111814); TRIPOD-LLM (Gallifant et al., Nature Medicine 2025, doi:10.1038/s41591-024-03425-5; 19 main items + 50 sub-items, living document); MI-CLAIM (Norgeot et al., Nat Med 2020;26:1320–4); CONSORT-AI (Liu et al., Nat Med 2020;26:1364–74; 14 extensions for AI RCT reporting); SPIRIT-AI (protocol counterpart, Cruz Rivera et al., Nat Med 2020;26:1351–63); DECIDE-AI (Vasey et al., Nat Med 2022;28:924–33, doi:10.1038/s41591-022-01772-9); STARD-AI; CLAIM 2024 update (Tejani et al., Radiol Artif Intell 2024;6(4):e240300, doi:10.1148/ryai.240300; formal Delphi consensus of a 72-member panel).

### Conceptual model
Required reporting + analytic methods: subgroup stratification (sex, age, race/ethnicity, SES, insurance status, geography); external validation in temporally and geographically distinct cohorts; calibration assessment (calibration-in-the-large, calibration slope, Brier score, calibration plots, ICI); fairness metrics (demographic parity, equal opportunity, equalized odds); EHR data-drift monitoring (population, label, concept drift).

### Expressiveness/Semantics
Statistical reporting framework; checklists are normative-procedural.

### Composability/Modularity
Each checklist item is independently auditable. TRIPOD-LLM uses a modular format applicable across LLM research designs.

### Suitability for autoformalization to IR
Indirect for IR construction itself; direct for any downstream LLM-augmented inference step. TRIPOD-LLM requires disclosure of training/testing datasets and performance benchmarking — covers RAG + autoformalization combinations.

### Formal verification potential
None directly; some fairness criteria (demographic parity, equalized odds) are linear constraints amenable to verification on tabular models, less so for LLMs.

### Tooling/Ecosystem maturity
Mature checklists; tooling for calibration (`scikit-learn calibration`, `pycaleva`, `mlr3`); fairness (Fairlearn, AIF360); EQUATOR Network reporting portal.

### Japan-specific considerations
External validation in Japanese populations is regulatorily expected by PMDA: the 2018 Japan-first AI medical device approval required clinical validation in Japanese patient populations. JADER, MID-NET (PMDA's pharmacoepidemiology network) and NDB (National Database of Health Insurance Claims) are key external-validation substrates. Subgroup considerations differ from Western contexts (limited race-stratified data; age stratification critical given Japan's super-aged demographics).

### Interoperability
Calibration outputs flow into Category 12 RE-AIM Effectiveness dimension; subgroup analyses inform Category 7 retrieval gating; data-drift monitors integrate with Category 8 audit trails.

### Limitations/Known issues
TRIPOD+AI compliance in published literature remains incomplete: Singh (Cureus 2025; doi:10.7759/cureus.97176) "Effect of TRIPOD+AI Guidelines on the Reporting Quality of Artificial Intelligence Prediction Models in Orthopaedic Surgery: An 18-Month Bibliometric Study" — across 522 eligible studies (pre-TRIPOD+AI n=214, post n=308), confidence-interval reporting remained low (18.7% vs 16.6%, p=0.61) and full Item-9 compliance showed only a non-significant increase (32.7% → 39.9%, p=0.11). External validation rarely matches deployment distribution. Race/ethnicity stratification is poorly recorded in Japanese EHRs.

### Training data proxy
Strong (BMJ, Nat Med, npj Digital Medicine, JAMIA).

---

## 12. Implementation Science Frameworks: CFIR, NASSS, RE-AIM

### Purpose
Plan, evaluate, and explain real-world adoption of CDS interventions across organizational, social, and technical dimensions.

### Maintainer/Standards body
CFIR — Damschroder et al., Implement Sci 2009;4:50 (doi:10.1186/1748-5908-4-50); CFIR 2.0 (Damschroder, Reardon, Widerquist, Lowery, Implementation Science 2022;17:75, doi:10.1186/s13012-022-01245-0; CFIR 2.0 User Guide published 2025 as a five-step guide for conducting implementation research; CFIR Outcomes Addendum, Implementation Science 2022, PMC8783408, doi:10.1186/s13012-021-01181-5). NASSS — Greenhalgh et al., J Med Internet Res 2017;19(11):e367 (doi:10.2196/jmir.8775, PMC5688245). RE-AIM — Glasgow, Vogt, Boles, Am J Public Health 1999;89(9):1322–7; re-aim.org maintains framework guidance.

### Conceptual model
- CFIR: 5 domains × 39 constructs (Innovation, Outer Setting, Inner Setting, Individuals, Implementation Process).
- NASSS: 7 domains (Condition, Technology, Value Proposition, Adopters, Organisation, Wider Context, Embedding & Adaptation Over Time) × complexity levels (simple/complicated/complex).
- RE-AIM: 5 dimensions (Reach, Effectiveness, Adoption, Implementation, Maintenance).

### Expressiveness/Semantics
Qualitative determinant frameworks (CFIR, NASSS) + mixed-method outcome framework (RE-AIM). Often used jointly: CFIR/NASSS to diagnose facilitators/barriers; RE-AIM to quantify impact.

### Composability/Modularity
Frameworks are commonly combined (RE-AIM + CFIR for CDS implementation per the Breathewell study, PMC7063029; NASSS-informed CDS scoping review, PMC10373265). Each construct/dimension is independently assessable.

### Suitability for autoformalization to IR
Not applicable to IR construction; applicable to system deployment. NASSS specifically flags "embedding & adaptation over time" — relevant for autoformalization pipelines requiring periodic re-formalization as guidelines update.

### Formal verification potential
None.

### Tooling/Ecosystem maturity
Mature: re-aim.org, CFIR Guide (cfirguide.org), NASSS-CAT instrument for complexity assessment.

### Japan-specific considerations
Japanese healthcare delivery has distinct constraints: universal insurance with biennial fee-schedule revision (診療報酬改定) materially affects CDS adoption economics; high physician autonomy and society-guideline authority shape inner-setting culture; consolidating role of Designated Functional Hospitals (特定機能病院). MHLW Society 5.0 / 医療DX推進本部 sets national digital-transformation goals. AMED implementation-science funding tracks exist. Sustainability (RE-AIM Maintenance, NASSS Embedding) is sharply constrained by reimbursement coverage of SaMD (Dash for SaMD2 includes reimbursement-pathway updates).

### Interoperability
Outputs link to Category 8 (BPMN/workflow embedding), Category 11 (RE-AIM Effectiveness aligns with TRIPOD/CONSORT outcome reporting), Category 10 (NASSS Adopters dimension shapes explanation requirements).

### Limitations/Known issues
Frameworks are descriptive, not prescriptive; subjective coding. Abell B, Naicker S, Rodwell D, et al., "Identifying barriers and facilitators to successful implementation of computerized clinical decision support systems in hospitals: a NASSS framework-informed scoping review" (Implementation Science 2023;18:32, doi:10.1186/s13012-023-01287-y, PMC10373265): "No determinants were assigned to the 'Embedding and Adaptation Over Time' domain." CFIR 2.0 not yet widely adopted in CDS literature.

### Training data proxy
Strong (Implementation Science journal, JMIR, JAMIA, BMJ; Japanese venues: 医療情報学雑誌, JJMI).
