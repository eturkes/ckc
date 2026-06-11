# Category 10: Regulatory Compliance, Assurance, Privacy, and Operational Lifecycle

## 1. GSN Assurance Cases and Safety Cases

### Purpose
Graphical structured argument that a system satisfies stated safety/assurance claims, evidence-supported, assumptions/context explicit; argues residual-risk acceptability for SaMD/AI-CDS to regulators/internal review boards.

### Maintainer/Standards body
SCSC Assurance Case Working Group (ACWG), GSN_SWG sub-group, publishes GSN Community Standard. OMG maintains Structured Assurance Case Metamodel (SACM). ISO/IEC 15026 (assurance case content). Common tools: AdvoCATE (NASA), ASCE (Adelard).

### Conceptual model
DAG of Goals, Strategies, Solutions (evidence), Contexts, Assumptions, Justifications. Modular extensions: away-goals, contracts, modules; pattern catalogs for argument reuse.

### Expressiveness/Semantics
Informal/semi-formal; pragmatic semantics from SCSC text. SACM: MOF-conformant metamodel, rigorous abstract syntax. OntoGSN (2025): 1:1 OWL formalization of GSN v3 with SWRL rules + SPARQL queries.

### Composability/Modularity
Modular GSN: contract modules, away-goals; SACM packaging. Patterns (e.g., ALARP, hazard-directed) enable reuse across guideline domains.

### Suitability for autoformalization to IR
Top claim "Autoformalization produces idempotent, contradiction-free IR" decomposes into measurable sub-goals (semantic-convergence metric, contradiction-detection coverage, ontology stability). Solutions = formal-verification artefacts, retrieval evaluations, LLM-eval reports. IR itself can be an evidence artefact; assurance arguments versioned alongside it.

### Formal verification potential
Argument well-formedness checkable via SACM/OntoGSN. Assurance 2.0 (Bloomfield/Rushby) and Isabelle/SACM (Brucker/Foster): machine-checked confidence calculation, proof-carrying assurance cases. Defeaters and "eliminative argumentation" admit logical analysis.

### Tooling/Ecosystem maturity
Mature: ASCE, AdvoCATE, Astah GSN, CertWare, D-Case Editor (Japanese), Isabelle/SACM. GSN v3 (SCSC-141C, 2021) current. OntoGSN (arXiv 2506.11023, 2025) emerging.

### Japan-specific considerations
Long-standing assurance-case research via AIST/DEOS (D-Case, GSN derivative). MHLW does not mandate GSN; PMDA reviews increasingly expect structured safety justifications for AI-SaMD, aligned with GSN. JIPDEC and IPA used GSN-style argumentation in critical-infrastructure consulting.

### Interoperability
Solutions can reference: FHIR profiles/CQL artefacts (Cat 1); Minds/JP-Core/PMDA claims (Cat 2); SHACL/OWL ontology constraints (Cat 3); Lean/Isabelle/TLA+ proofs (Cat 4); SMT/SAT/ASP results (Cat 5); deontic/temporal logic verifications (Cat 6); retrieval evaluation (Cat 7); LLM eval logs (Cat 8); clinical validation (Cat 9). OntoGSN OWL alignment links directly to Cat 3 ontologies.

### Limitations/Known issues
Haddon-Cave (Nimrod review): "confirmation-biased" top goals, over-confident structures. Maintenance burden for dynamic AI. Argument quality depends on reviewer expertise; weak quantitative confidence model.

### Training data proxy
Moderate. SCSC publications open-access; many IEEE SafeComp/DSN papers. OntoGSN, Assurance 2.0 arXiv preprints. Less abundant in LLM training corpora than ISO standards.

## 2. ISO 14971, IEC 62304, and IEC 62366-1 Mapping

### Purpose
Integrated risk management (ISO 14971:2019), software lifecycle (IEC 62304:2006+A1:2015), usability engineering (IEC 62366-1:2015+A1:2020) for medical devices incl. SaMD; map shared inputs/outputs (hazards, use errors, software hazardous situations) into one coherent QMS.

### Maintainer/Standards body
ISO TC 210, IEC SC 62A. AAMI publishes US harmonized versions + TIRs (TIR57 cybersecurity, TIR45 agile, TIR34971 ML, TR 80002-1 software risk, TR 80002-3 lifecycle process).

### Conceptual model
14971: iterative risk-management process (identify hazards → estimate/evaluate risk → control → residual-risk → PMS). 62304: software safety class (A/B/C) drives lifecycle rigor. 62366-1: use-specification → use-related risk → UI design → formative/summative evaluation.

### Expressiveness/Semantics
Process-oriented; no formal semantics. Risk = probability × severity (qualitative or quantitative tables). Traceability matrix = primary semantic link.

### Composability/Modularity
Modular by design: 62304 software items + SOUP; 62366 user-task decomposition; 14971 hazard chains.

### Suitability for autoformalization to IR
Each ingested guideline → IR fragment = 62304 "software item" with safety class; autoformalization errors map to 14971 "software hazardous situations". 62366 use-error analysis covers clinician misinterpretation of CDS recommendations. AAMI TIR34971:2023 (BS/AAMI 34971:2023, "Application of ISO 14971 to machine learning in artificial intelligence") extends 14971 to ML hazards (dataset shift, training/data leakage); ISO/DTS 24971-2 ("Medical devices – Guidance on the application of ISO 14971: Part 2: Machine learning in artificial intelligence"), strongly informed by AAMI TIR34971, at near-final approval stage, publication expected 2026.

### Formal verification potential
Low for the standards themselves. Traceability machine-checkable; risk-control verification often executable tests. AAMI TIR34971 ML-specific hazard taxonomy amenable to ontology encoding.

### Tooling/Ecosystem maturity
Very mature: Jama, Polarion, Cognition Cockpit, Greenlight Guru. AAMI marketplace for TIRs. ISO 24971:2020 guidance; ISO/DTS 24971-2 (AI extension) nearing publication 2026. IEC 62304 Edition 2 in approval stage: comment resolution March 2026, approval starting May 2026, publication targeted August 2026; replaces three-class A/B/C with two-level rigor classification (Level I/II), removes general QMS requirements (left to ISO 13485), expands scope to all health software, adds AI/ML lifecycle requirements + integrated cybersecurity provisions.

### Japan-specific considerations
PMDA recognizes JIS T 14971/JIS T 62304/JIS T 62366-1 (JIS translations). PMD Act QMS Ministerial Ordinance (薬機法 QMS省令) aligns with ISO 13485. PMDA Science Board Subcommittee "Report on AI-based Software as a Medical Device (SaMD)" (28 August 2023, files/000266099.pdf English) applies TPLC concepts compatible with TIR34971. MHLW Notification PSB/MDED No. 1116-2 historically governs SaMD review interpretation.

### Interoperability
Risk-management file artefacts expressible in SACM/GSN (Cat 1 of this category). Hazards in OWL/SHACL ontologies (Cat 3 of overall taxonomy). 62366 use-specification interfaces with BPMN clinical workflow. IEC 62304 §5.3 software architecture maps to Cat 4 formal models.

### Limitations/Known issues
Standards predate generative AI; 62304 ed.1 ignores learning models (Edition 2 adds defined AI/ML development lifecycle). Manual traceability heavy. A/B/C criticized as too coarse (Edition 2 → Level I/II).

### Training data proxy
High. Widely written about (books, regulator guidance, vendor blogs). PMDA notifications + JIS translations available; AAMI TIR text paywalled, reducing direct training presence.

## 3. FDA / PMDA / IMDRF CDS and SaMD Regulatory Analysis

### Purpose
Define which software functions are regulated as medical devices, classification, premarket evidence, and life-cycle controls incl. AI/ML modifications.

### Maintainer/Standards body
IMDRF SaMD WG and AIMD WG; FDA CDRH; Japan MHLW/PMDA; EU MDR/IVDR + AI Act; UK MHRA. Cross-references ISO/IEC standards.

### Conceptual model
- IMDRF SaMD Risk Categorization (I–IV): significance of information × healthcare situation.
- IMDRF AIMD WG "Machine Learning-enabled Medical Devices (MLMD): Key terms and definitions" (IMDRF/AIMD WG/N67 FINAL:2022), issued 6 May 2022.
- IMDRF/AIML WG/N88 FINAL:2025 "Good Machine Learning Practice for Medical Device Development: Guiding Principles" (published 29 January 2025) finalizes 10 GMLP guiding principles; IMDRF/SaMD WG/N81 FINAL:2025 "Characterization Considerations for Medical Device Software and Software-Specific Risk" (published 29 January 2025) codifies software/AI characterization for global harmonization.
- FDA CDS exemption criteria (21st Century Cures §3060, enacted 13 December 2016) — four-prong test; non-exempt CDS = device. Revised CDS guidance published 6 January 2026 supersedes the 2022 version; permits a single, clinically appropriate recommendation without automatic device classification when only one clinically appropriate option exists (enforcement discretion); clarifies acceptable data inputs (demographics, symptoms, lab results, medications, problem lists, discharge summaries, trusted external sources such as clinical guidelines and peer-reviewed literature).
- FDA finalized "Marketing Submission Recommendations for a Predetermined Change Control Plan for Artificial Intelligence-Enabled Device Software Functions" 3 December 2024 (Federal Register 4 December 2024; webinar 14 January 2025), expanded from ML-only to AI-enabled.
- FDA draft guidance "Artificial Intelligence-Enabled Device Software Functions: Lifecycle Management and Marketing Submission Recommendations" 6 January 2025 (Federal Register 7 January 2025; comment period closed 7 April 2025): TPLC-based; model description, data lineage/splits, performance, bias analysis/mitigation, human-AI workflow, post-market monitoring, PCCP integration.
- FDA QMSR (amending 21 CFR Part 820 to incorporate ISO 13485:2016 by reference) effective 2 February 2026; inspections under Compliance Program 7382.850.
- PMDA IDATEN ("Improvement Design within Approval for Timely Evaluation and Notice") = PACMP for medical devices; statutory provisions in revised PMD Act published 2019, in force 1 September 2020 (PMDA presentation files/000269712.pdf, 2024).
- MHLW "DASH for SaMD" (24 November 2020) and "DASH for SaMD 2" (jointly MHLW + METI, 6 September 2023, mhlw.go.jp/content/11121000/001142990.pdf): multiple commercialization pathways incl. two-step approval (MHLW Notification PSB/MDED No. 1116-2 style staged Tier I analytical / Tier II clinical) and SaMD-for-general-public.

### Expressiveness/Semantics
Regulatory text; semi-structured via decision trees (MHLW "Guidelines on the Determination of Whether Programs Are Regarded as Medical Devices," 31 March 2021, partially revised 31 March 2023; pmda.go.jp/files/000240233.pdf). Vocabulary: IMDRF N67 MLMD Key Terms + N88 GMLP principles (above).

### Composability/Modularity
Modular: classification + premarket + QMS + PMS. PCCP and IDATEN modularize the change-management envelope.

### Suitability for autoformalization to IR
Claim "this CDS is non-device CDS" rests on the four-prong test (per 6 January 2026 guidance incl. single-recommendation carve-out), expressible in IR as deontic constraints (clinician must be able to independently review recommendation rationale ⇒ IR must expose provenance). PMDA TPLC approach (28 August 2023 AI-SaMD report) aligns with IR-versioning + GSN.

### Formal verification potential
Low directly; via mapping into deontic logic / argumentation frameworks for compliance-checking (cf. LegalRuleML).

### Tooling/Ecosystem maturity
Mature regulatory ecosystem. FDA AI/ML Action Plan (Jan 2021); PCCP draft guidance April 2023 (final + AI-DSF + CDS revision + QMSR dated above). August 2025: FDA + Health Canada + UK MHRA jointly published five guiding principles for PCCPs in ML-enabled devices. IMDRF documents public (incl. N81 and N88 FINAL:2025). PMDA Japanese-language notifications on pmda.go.jp; English summaries growing.

### Japan-specific considerations
Critical. Chambers Digital Healthcare 2025 – Japan, verbatim: "nearly 100 AI-powered SaMDs have received PMDA approval and insurance coverage." medRxiv scoping review of official PMDA SaMD Excel spreadsheet (downloaded 31 May 2025; approvals on or before 31 December 2024): 151 total PMDA-approved SaMD, 40 officially designated AI-utilizing, 20 specifically radiology. Instruments: PMD Act; MHLW program-determination guidelines (31 Mar 2021, partially revised 31 Mar 2023); PMDA SaMD One-Stop Consultation Desk (iryou-kiki program sougou soudan madoguchi); "SaMD Development Guidance" (29 May 2023; 2nd edition 5 June 2024). AI Promotion Act (推進法, enacted 28 May 2025, most provisions effective 4 June 2025): light obligations, no penalties. IDATEN remains principal post-approval AI-SaMD update mechanism; being extended to cover continuous-learning performance changes at manufacturer/distributor level and at actual medical facilities.

### Interoperability
Claims feed GSN assurance cases (Cat 1). Classification rules formalizable as DMN decision tables. PCCP/IDATEN protocols link to versioned-knowledge CI/CD (Cat 9). PMS data feed Cat 12.

### Limitations/Known issues
Heterogeneous global frameworks; PCCP and IDATEN differ in scope. EU AI Act adds layered obligations for high-risk healthcare AI not aligned with US/Japan timelines; EU AI Omnibus political agreement of 7 May 2026 extended deadline for AI systems embedded in MDR/IVDR-regulated products to 2 August 2028. CDS exemption boundary remains nuanced for autoformalization-based guideline retrieval; 6 January 2026 revision relaxes the "more than one recommendation" reading but does not exempt CDS failing the transparency/independent-review prong.

### Training data proxy
High. FDA/IMDRF documents heavily indexed. PMDA English summaries growing; Japanese-language primary notifications less represented in LLM training corpora.

## 4. NIST AI RMF and ISO/IEC 42001 AI Management

### Purpose
NIST AI RMF 1.0 (26 January 2023) + Generative AI Profile (NIST AI 600-1, 26 July 2024): organizational risk-management framework. ISO/IEC 42001:2023: certifiable AI management system. Both govern AI lifecycle incl. healthcare CDS.

### Maintainer/Standards body
NIST (AI RMF + GenAI Profile, ARIA program, AI Agent Standards Initiative launched 17 February 2026 by NIST CAISI). ISO/IEC JTC 1/SC 42 (42001; 23894 risk management; 5338 AI lifecycle; 22989 concepts).

### Conceptual model
AI RMF: four functions GOVERN, MAP, MEASURE, MANAGE × 72 sub-categories. GenAI Profile: 12 risk areas (CBRN information/capabilities, confabulation, dangerous/violent/hateful content, data privacy, harmful bias/homogenization, human-AI configuration, information integrity, information security, intellectual property, obscene/degrading/abusive content, value chain and component integration, environmental impacts), 200+ suggested actions. 42001: Annex SL high-level structure (Plan-Do-Check-Act), Annex A controls, Annex B AI-specific implementation guidance.

### Expressiveness/Semantics
Process/control catalog; informal text. Mapping tables (AI RMF ↔ 42001 ↔ EU AI Act) increasingly available.

### Composability/Modularity
RMF profiles (sector/use-case). 42001 integrates with ISO 27001/9001 via Annex SL.

### Suitability for autoformalization to IR
MAP (context establishment) informs IR scope; MEASURE drives evaluation metrics (semantic convergence, contradiction count); MANAGE governs retraining decisions. 42001 Annex A.6 (AI system impact assessment) + A.7 (data for AI) = control points for the autoformalization pipeline.

### Formal verification potential
Limited; controls are policies; audit evidence procedural.

### Tooling/Ecosystem maturity
NIST Playbook + AIRC resource center. 42001 certifications by accredited bodies (Schellman, BSI, NSF) since 2024. CSA "Agentic AI Profile" of NIST AI RMF (2025) extends to LLM agents. NIST IR 8596 ipd ("Cyber AI Profile", preliminary draft Cybersecurity Framework Profile for AI) 16 December 2025, 45-day comment period through 30 January 2026; NIST AI Agent Standards Initiative 17 February 2026; concept note for AI RMF Profile on Trustworthy AI in Critical Infrastructure 7 April 2026.

### Japan-specific considerations
METI/MIC "AI Guidelines for Business" v1.0 (19 April 2024) and AI Promotion Act (enacted 28 May 2025, most provisions effective 4 June 2025): non-binding/principles-based, reference international norms; organizations seeking global market access typically adopt 42001 + NIST AI RMF as a single control set. PMDA AI-SaMD review framework aligns with TPLC concepts in NIST RMF.

### Interoperability
Maps to MITRE ATLAS (adversarial threats); ISO 14971/AAMI TIR34971 (medical-device risk); ISO 27001 (security); LINDDUN (privacy).

### Limitations/Known issues
RMF voluntary; "actions" not requirements. GenAI Profile predates agentic-system prevalence (CSA agentic profile + NIST AI Agent Standards Initiative address delegation gap). 42001 audit market still maturing; bias/transparency controls (A.6.2.6, A.7) lack precise metrics.

### Training data proxy
High. NIST documents public-domain, widely cited; 42001 text behind ISO paywall but summaries pervasive.

## 5. APPI and Next-Generation Medical Infrastructure Act Privacy Governance

### Purpose
APPI: Japanese personal-information protection law. NGMIA (次世代医療基盤法): special pathway for medical-data research/development.

### Maintainer/Standards body
Personal Information Protection Commission (PPC, 個人情報保護委員会). Cabinet Office secretariat for NGMIA; MHLW sectoral guidance.

### Conceptual model
APPI 2022 amendments (in force 1 April 2022): mandatory breach notification to PPC + data subjects (esp. sensitive info, cyberattacks, financial harm, or >1,000 individuals); extraterritorial reach (5,000-person threshold removed); pseudonymously-processed information (仮名加工情報) for internal analysis with relaxed obligations (third-party transfer generally prohibited); personally-referable information (個人関連情報) for cookie-like data; penalties up to ¥100 million for entities, imprisonment up to 1 year for individuals. Special-care-required personal information (要配慮個人情報, Art. 2(3)) includes medical history; requires opt-in consent; cannot be transferred via opt-out third-party mechanism.

NGMIA (Act No. 28 of 2017; amended Act No. 35 of 2023, promulgated 26 May 2023, effective 1 April 2024): certified anonymization business operators ("Authorised De-identified Medical Information Preparers") receive identifiable medical data opt-out; 2023 amendment adds pseudonymized medical information (仮名加工医療情報) usable for PMDA submissions incl. rare-disease/outlier data, plus linkage to NDB and Kaigo-DB. Art. 2(4) defines pseudonymized medical data as information "which can be prepared in a way that makes it not possible to identify a specific individual unless collated with other information."

### Expressiveness/Semantics
Statutory text + PPC guidelines + Q&As; not formal.

### Composability/Modularity
Layered: APPI (general) → NGMIA (medical) → MHLW Ethics Guidelines on Life Science and Medical Research Involving Human Subjects (revised March 2023).

### Suitability for autoformalization to IR
IR must annotate guideline-derived rules with permissible data classes (要配慮 vs not), legal basis (opt-in/opt-out), processing-purpose limitations. Cross-border transfer (Art. 28): IR/system must record recipient jurisdiction + transfer ground (adequacy / equivalent system / consent).

### Formal verification potential
Compliance rules formalizable in deontic logic + policy languages (XACML, ODRL, LegalRuleML); cross-border-transfer decision encodable in DMN.

### Tooling/Ecosystem maturity
Mature legal-tech ecosystem; PPC publishes Q&A in Japanese. Limited Japan-specific automated compliance tooling.

### Japan-specific considerations
Adequacy: only EEA + UK whitelisted (mutual EU-Japan adequacy decision in force since 23 January 2019). Cross-border consent route requires disclosure of (1) recipient country, (2) that country's data-protection legislation, (3) recipient's protective measures. Accredited NGMIA operators (per Chambers Digital Healthcare 2025 – Japan): Life Data Initiative (LDI, certified pseudonymized-info creator December 2024, operates "千年カルテ" with ~100+ institutions); J-MIMO (Japan Medical Association's Medical Information Management Organization, ISO 27001-certified); FAST-HDJ (Fair and Safe Use of Anonymised Standardised Health Data of Japan). As of end-February 2025: LDI 45 provision cases, J-MIMO 14, FAST-HDJ 4; total 153 cooperating medical institutions. MID-NET (PMDA), NDB (MHLW), J-MID (Japan Medical Imaging Database) operate under additional purpose-specific governance. MHLW "Guidelines for the Appropriate Handling of Personal Information in the Field of Health, Labor, and Welfare" (December 2024) reiterate APPI obligations in clinical contexts.

### Interoperability
Constrains data into Cat 7 (de-identification), Cat 8 (AIBOM/dataset profile), Cat 11 (drift monitoring needs data access). Cat 10 audit logs must capture access to 要配慮 data. EHDS Regulation (Regulation (EU) 2025/327): published OJ 5 March 2025, in force 26 March 2025, staged application milestones through 2031 for secondary-use access — affects any cross-border research collaboration touching EU data.

### Limitations/Known issues
NGMIA adoption slow; consent-opt-out + accreditation barriers fragment data. Pseudonymized info cannot be transferred to third parties (except certified users and PMDA). vs GDPR: APPI lacks DPIA mandate; legitimate-interests basis absent. EDPS-EC joint guidelines on GDPR–EU AI Act interplay expected early 2026.

### Training data proxy
Moderate. APPI English summaries abundant (law-firm publications); NGMIA detail thinner in English. Japanese-language primary sources less indexed.

## 6. STRIDE / LINDDUN Threat Modeling and Zero Trust Architecture

### Purpose
Systematic identification of security threats (STRIDE), privacy threats (LINDDUN), and architectural defense via identity-centric Zero Trust (NIST SP 800-207, August 2020; SP 800-207A cloud-native, 2023; SP 800-204D supply-chain integrity in cloud-native CI/CD).

### Maintainer/Standards body
Microsoft (STRIDE), KU Leuven (LINDDUN — Wuyts, Joosen, Hovsepyan), NIST (Zero Trust SPs), MITRE (ATLAS), OWASP (LLM Top 10 v2025, ML Top 10, Agentic Applications Top 10 2026).

### Conceptual model
STRIDE: Spoofing / Tampering / Repudiation / Information disclosure / DoS / Elevation, per DFD element. LINDDUN: Linkability / Identifiability / Non-repudiation / Detectability / Disclosure / Unawareness / Non-compliance; three variants — LINDDUN Go (lean game), LINDDUN Pro (DFD + threat trees), LINDDUN Maestro. Zero Trust (SP 800-207): Policy Engine + Policy Administrator + Policy Enforcement Point + Continuous Diagnostics; no implicit trust by network location; seven tenets.

### Expressiveness/Semantics
Catalog + DFD-based; threat trees provide pattern language. Zero Trust tenets (7 in SP 800-207) are policy statements.

### Composability/Modularity
DFD decomposition naturally modular. Zero Trust components composable services (e.g., SPIFFE/SPIRE identity, OPA/Cedar policy).

### Suitability for autoformalization to IR
Threat-modeling mitigations become requirements traceable into IR-pipeline service architecture (signed retrieval, integrity-protected ontology updates). LINDDUN GenAI extension (Liao, Bellemans, Sion et al., "A LINDDUN-based Privacy Threat Modeling Framework for GenAI", arXiv 2603.06051, March 2026): 100 GenAI-specific examples across 3 LINDDUN threat types relevant to LLM-based autoformalization (prompt-injection-driven linkability, identifiability via memorization). OWASP LLM Top 10 2025 maps to MITRE ATLAS (AML.T0051.000 direct prompt injection, AML.T0051.001 indirect, AML.T0054 jailbreak, AML.T0048 supply-chain initial access, AML.T0020 training-data poisoning).

### Formal verification potential
DFD + threats encodable in Alloy/TAMARIN for protocol analysis. Zero Trust policy expressible in Rego/Cedar with formal evaluation.

### Tooling/Ecosystem maturity
Microsoft Threat Modeling Tool, IriusRisk, ThreatModeler, OWASP pytm; LINDDUN open materials; PILLAR (LLM-driven LINDDUN automation, arXiv 2410.08755, October 2024); MITRE ATLAS Navigator + Arsenal CALDERA plugin; MITRE AI Incident Sharing initiative launched October 2024; OWASP LLM Top 10 v2025 by 500+ experts from 110+ companies.

### Japan-specific considerations
IPA publishes "Ten Major Security Threats" (情報セキュリティ10大脅威) annually + STRIDE-aligned guidance. JPCERT/CC coordinates incident response. 3省2ガイドライン framework: MHLW "Guidelines on Safety Management of Healthcare Information Systems v6.0" (May 2023, restructured into four chapters Overview/Governance/Management/Control) + jointly issued METI + MIC "Safety Management Guidelines for Providers of Information Systems and Services Handling Medical Information v1.1" (July 2023) mandate access control, audit, encryption, risk-based controls equivalent to Zero Trust tenets; mutually referencing ("considered the diversification and sophistication of cyberattacks in recent years, and the Guidelines for the Safety Management of Medical Information Systems Ver. 6.0").

### Interoperability
Threat list feeds GSN assurance case (Cat 1) + ISO 14971/TIR57 risk register (Cat 2). LINDDUN privacy threats feed Cat 5/Cat 7 controls. Zero Trust integrates with Cat 9 deployment (admission control via OPA/Kyverno) + Cat 10 audit/observability.

### Limitations/Known issues
STRIDE/LINDDUN largely manual; coverage depends on DFD completeness. Classical LINDDUN taxonomy lacks model-centric threats (data poisoning, model extraction). Zero Trust deployment heavyweight; SP 800-207 is guidance not certification.

### Training data proxy
High. Abundant academic and industry literature (USENIX Security, S&P, CCS, NDSS); OWASP/MITRE openly published.

## 7. De-Identification and Privacy-Preserving Record Linkage

### Purpose
Reduce re-identification risk in clinical datasets for training/evaluating CDS components; link records across institutions without disclosing identifiers.

### Maintainer/Standards body
HIPAA Safe Harbor / Expert Determination (HHS); ISO/IEC 20889 (de-identification techniques); NIST SP 800-188. Academic: Sweeney (k-anonymity), Machanavajjhala (l-diversity), Li (t-closeness), Dwork (differential privacy). PPRL: Schnell/Bachteler/Reiher (Bloom-filter PPRL, BMC Medical Informatics and Decision Making 2009 9:41); Karapiperis/Verykios (LSH-based blocking with homomorphic matching, IEEE TKDE 2015 27(4):909–921); Karapiperis/Gkoulalas-Divanis/Verykios (FEDERAL framework, IEEE TKDE 2018 30(2):292–304); Karapiperis/Verykios (Hamming LSH, Knowledge and Information Systems 2016 49(3):861–884).

### Conceptual model
Syntactic anonymization (k-anonymity, l-diversity, t-closeness, suppression/generalization); differential privacy (ε,δ); cryptographic (HE, SMPC, garbled circuits); PPRL (Bloom-filter encoding of q-grams + LSH blocking + Dice-coefficient similarity).

### Expressiveness/Semantics
Mathematical guarantees (DP), set-theoretic (k-anonymity), probabilistic (PPRL match quality).

### Composability/Modularity
DP composition theorems; PPRL pipelines (encode → block → match → reconcile).

### Suitability for autoformalization to IR
CDS pipeline must record, in IR or metadata, the de-identification regime per training/evaluation dataset (HIPAA SH, DP ε, NGMIA anonymized-vs-pseudonymized). SPDX 3.0 Dataset profile fields (anonymization method, sensitivity indicator, known biases) make this machine-readable. HHS OCR HIPAA Security Rule NPRM (published 27 December 2024; Federal Register 6 January 2025; expected final ~May 2026, 240-day compliance window) would, as proposed: eliminate the "addressable" implementation specification distinction, require ePHI encryption at rest and in transit, mandate MFA, require network segmentation, mandate annually-tested formal incident response plans.

### Formal verification potential
DP mechanism-correctness proofs in Lean/Coq (Sato, Barthe et al.). k-anonymity checkable via SQL constraints. PPRL Bloom-filter cryptanalysis attacks (Kuzu/Kantarcioglu/Durham/Malin, PETS 2011, pp. 226–246; Christen/Schnell/Vatsalan/Ranbaduge, PAKDD 2017, pp. 628–640) require hardened variants (salting, padding, attribute-level Bloom filters).

### Tooling/Ecosystem maturity
Mature: ARX, μ-ARGUS, Amnesia, OpenDP, Google DP libraries, TensorFlow Privacy, PySyft, Crypten. PPRL: anonlink, secretflow.

### Japan-specific considerations
NGMIA distinguishes 匿名加工医療情報 (anonymized) vs 仮名加工医療情報 (pseudonymized, post-2023 amendment); production only by certified preparers (LDI, J-MIMO, FAST-HDJ). PPC anonymization-processing standards under APPI for "anonymously processed information" (匿名加工情報) require outlier removal — different threshold than HIPAA Safe Harbor; the reason the 2023 NGMIA amendment introduced pseudonymized medical info retaining rare-disease/outlier data for PMDA submissions. MID-NET uses on-site secure analysis to avoid raw-data export.

### Interoperability
Privacy guarantees feed Cat 5 governance argument; PPRL encoded datasets feed Cat 8 dataset BOM; ε values feed Cat 4 formal proofs.

### Limitations/Known issues
Bloom-filter PPRL vulnerable to frequency analysis without padding/salting (Schnell hardened variant + diffusion). DP ε budget management complex across multi-step LLM pipelines. k-anonymity unsound against background-knowledge attacks.

### Training data proxy
High. Rich academic literature (TPDP, PETS, KDD, IEEE TKDE). Japanese-specific guidance moderate.

## 8. SBOM/AIBOM and Reproducible Artifact Supply Chain

### Purpose
Machine-readable inventory of all software/model/dataset components + provenance: vulnerability response, license/IP compliance, reproducibility — critical for AI-CDS retrieving guidelines and depending on LLMs + ontologies + retrievers.

### Maintainer/Standards body
SPDX (Linux Foundation; ISO/IEC 5962:2021 for SPDX 2; SPDX 3.0 profiles Core + Software + AI + Dataset + Build + Security + Usage + Lite; SPDX 3.1 RC1 adds Functional Safety, Hardware, Operations, Service, Supply Chain). CycloneDX (OWASP; ML-BOM extension). SLSA (Google/OpenSSF; build-integrity levels). Sigstore (Linux Foundation; cosign + Fulcio + Rekor). in-toto (CNCF; ITE-6 attestation envelope). Hugging Face Model Cards; MLflow.

### Conceptual model
SBOM = directed graph of packages/files/relationships. SPDX 3.0 AI Profile: AI/AIPackage with autonomyType, safetyRiskAssessment (High/Serious/Medium/Low), model architecture, energy use, model limitations, explainability. Dataset profile: collection process, biases, sensitivity indicator, anonymization method, intended use. in-toto attestation = (statement type, subject, predicate); SLSA Provenance is predicate type "https://slsa.dev/provenance/v1". Reference: Rajbahadur, Gallaba, Rashno, Suriyawongkul, Bennet, Stewart, Hassan, "Building an Open AIBOM Standard in the Wild" (arXiv 2510.07070, accepted ICSE 2026 SEIP): 36 new AI/Dataset fields integrated into SPDX 3.0.

### Expressiveness/Semantics
SPDX formal data model (RDF/SHACL). CycloneDX JSON Schema. Sigstore Rekor transparency log = immutable signing record.

### Composability/Modularity
Profile-based extensibility; one document may carry Software + AI + Dataset + Build + Security profiles.

### Suitability for autoformalization to IR
Per ingested guideline, IR artefact's SBOM/AIBOM records: source URL, retrieval timestamp, LLM model + version + temperature, ontology version, autoformalization prompt hash, evaluation report. Reproducibility of "semantic convergence" experiments requires bit-stable pinning of all inputs.

### Formal verification potential
Provenance verification via cosign + Rekor; policy enforcement via Kyverno/Sigstore Policy Controller or Conftest+OPA on SPDX/SLSA attestations.

### Tooling/Ecosystem maturity
Mature: cosign, slsa-github-generator, GitHub Actions actions/attest-build-provenance, Tekton Chains, syft, grype, trivy, dependency-track, Interlynk, Anchore. SPDX 3.0 published 2024; CISA-facilitated "SBOM for AI" Tiger Team use-case document (aibom-squad repository) first public draft 2025, building on the G7 June 2025 shared vision for AI SBOMs.

### Japan-specific considerations
METI promotes SBOM adoption (METI SBOM Introduction Guide 2023, expanded editions in medical-device sector). PMDA medical-device cybersecurity guidance recommends SBOM. IPA aligns with SPDX. Japan-resident dataset provenance under NGMIA (LDI, J-MIMO) should appear in Dataset profile.

### Interoperability
Gating artefact for Cat 9 versioned-knowledge CI/CD. Pairs with Cat 6 (Zero Trust admission control verifies provenance before deployment). Cat 4 formal proofs can reference SBOM-pinned artefacts. License fields cross-reference Cat 8/JP-Core terminology source licenses.

### Limitations/Known issues
AIBOM adoption nascent; many optional fields → incomplete documents. SPDX vs CycloneDX dual-track conversion burden. Provenance for dataset content (vs. dataset identifier) still under research. SLSA L3+ needs build-environment isolation.

### Training data proxy
High. SPDX/CycloneDX/SLSA documentation open; many GitHub repos, Linux Foundation papers, ICSE SEIP 2026.

## 9. Versioned Knowledge Deployment / Knowledge CI/CD

### Purpose
Continuously and safely promote new versions of clinical knowledge artefacts (FHIR profiles, IGs, CQL libraries, IR ontologies, autoformalized guidelines, LLM prompts) from authoring to production with rollback and canary semantics.

### Maintainer/Standards body
HL7 (FHIR IG Publisher, FSH/SUSHI, Simplifier). Argo CD, Flux (CNCF). Kubeflow. MLflow Model Registry. DVC (Iterative). Backstage. Semantic Versioning (semver.org).

### Conceptual model
GitOps: declarative state in Git → reconciliation operator → cluster. FHIR IG pipeline: FSH source → SUSHI compile → IG Publisher → published IG with NPM-style packages. Knowledge artefacts versioned semver with deprecation policy. Blue/green + canary deployments gated by automated tests + clinical-validation thresholds.

### Expressiveness/Semantics
Declarative manifests + reconciliation; FSH formal grammar (ANTLR v4) defined in FHIR Shorthand specification (Mixed Normative/Trial Use R2, February 2022).

### Composability/Modularity
Per-artefact pipelines; IG packages depend on other IGs via NPM-style resolution; DVC tracks dataset versions decoupled from code.

### Suitability for autoformalization to IR
Each guideline ingestion = pipeline run producing: (a) IR artefact, (b) ontology delta, (c) provenance/attestation, (d) automated evaluation incl. idempotency (re-run hash equals prior), contradiction-detection report, clinical validation. Canary routes a percentage of CDS queries to new IR version; rollback automatic on metric breach.

### Formal verification potential
Pipeline correctness (well-typed Kubernetes manifests, OPA-validated). Idempotency tests automatable as property-based checks.

### Tooling/Ecosystem maturity
Mature: Argo CD, Flux, Argo Workflows, Tekton, GitHub Actions, GitLab CI, MLflow, DVC, BentoML, KServe, Seldon, Ray Serve. FHIR-specific: SUSHI 3.19.0 (April 2026 release), IG Publisher autobuild via build.fhir.org, Firely Terminal for CI validation. FSH v3.0.0 specification states "over 600 IG-development projects using FSH"; FSH Finder index refreshed 2026-05-19. FHIR R6 normative ballot cycle opened January 2026 (hl7.org/fhir/6.0.0-ballot1), entire specification balloted as full Normative ANSI Standard; final publication expected 2027 at earliest after additional ballot rounds.

### Japan-specific considerations
JP-Core IG (HL7 Japan) published via same SUSHI/IG-Publisher toolchain. PMDA IDATEN/PACMP (PMD Act statutory provisions 2019, in force 1 September 2020) = regulatory wrapper for canary-style AI-SaMD updates; pre-approved changes notified within defined window without resubmitting full approval. MHLW "DASH for SaMD 2" (6 September 2023) accommodates iterative SaMD deployment via two-step approval and clarified pathways.

### Interoperability
Pipeline gates check Cat 8 SBOM/AIBOM signatures; deploy artefacts feed Cat 10 observability; promote Cat 4 formal-proof status; capture Cat 11 drift baselines at promotion. ONC HTI-1 Final Rule AI/predictive-DSI transparency requirements + USCDI v3 baseline (effective 1 January 2026) condition CEHRT-bound CDS deployments; HTI-5 proposed rule (published 29 December 2025; comment period closed 27 February 2026) proposes FHIR-API prioritization, certification-criteria changes, revised information-blocking provisions.

### Limitations/Known issues
Clinical-knowledge testing harnesses immature; version equivalence hard to define for LLM-driven IR. Rollback of a knowledge artefact that has clinically affected patients is procedurally complex (PMS implications under Cat 12).

### Training data proxy
High for GitOps; moderate for FHIR IG tooling; low for IR-specific knowledge CI/CD (novel).

## 10. Observability, Audit Logging, and Continuous Verification

### Purpose
Capture traces, prompts, responses, retrieval contexts, tool calls, clinical-data accesses per CDS invocation; continuous integration testing of clinical knowledge; forensic reconstruction of any recommendation.

### Maintainer/Standards body
OpenTelemetry (CNCF; GenAI semantic conventions WG defines `gen_ai.*` attributes). LLMOps: LangSmith (LangChain), Langfuse (MIT-licensed; OTel-native SDK v4), Helicone, Arize Phoenix (Apache 2.0), W&B Weave, Traceloop OpenLLMetry, Laminar (Apache 2.0). IHE ATNA (Audit Trail and Node Authentication; RESTful Query supplement ITI-81/ITI-82 over FHIR AuditEvent). HL7 FHIR AuditEvent (R5; jointly managed with DICOM/IHE), based on RFC 3881 / DICOM PS3.15 Annex A5. ISO 27789 EHR audit events. NIST SP 800-92.

### Conceptual model
OTel trace = span tree with attributes; baggage propagation for trace-level attributes (user, model, session). Langfuse SDK v4 OTel-native; spans → observations (spans, generations, events). FHIR AuditEvent records who/what/when/where/why of clinical data access. IHE Basic Audit Log Patterns (BALP) IG defines reusable AuditEvent profiles (Create/Read/Update/Delete/Query, with/without Patient subject, IUA/SAML/OAuth authorization).

### Expressiveness/Semantics
OTel formal semantic conventions; FHIR AuditEvent constrained binding to DICOM/IHE/ISO code systems.

### Composability/Modularity
Pluggable exporters; multiple parallel backends (OTel collector fan-out).

### Suitability for autoformalization to IR
Instrument every autoformalization run: retrieval span (source guideline ID + version), LLM span (model, prompt template hash, tokens, temperature), validation span (contradiction-detection result, idempotency hash). Continuous-verification harness re-runs canonical guidelines daily, asserts IR equivalence; deviations open tickets.

### Formal verification potential
Audit-log immutability via cryptographic hash chains (e.g., Trillian); CQL/SHACL invariants continuously evaluated against produced IR.

### Tooling/Ecosystem maturity
Very mature. Langfuse About page (May 2026): "Langfuse is the most widely adopted LLM Engineering platform with 27,157 GitHub stars, 50M+ SDK installs per month" (repo at 27,435 stars by 18 May 2026); MIT-licensed self-host (ClickHouse acquired Langfuse January 2026 as part of its $400M Series D; product remains open-source MIT). LangSmith commercial, seat+trace pricing ($39/seat/mo + $0.50/1k base traces). Phoenix open-source Apache 2.0. FHIR AuditEvent in major FHIR servers (HAPI, Microsoft, Firely).

### Japan-specific considerations
3省2ガイドライン (MHLW v6.0 + METI/MIC v1.1; full titles/structure in Cat 6 entry) requires detailed access logs, retention, tamper-evidence; v6.0 added requirements for the online qualification check (online insurance eligibility verification) introduced April 2023. PMDA expects audit trails as QMS evidence; MHLW personal-information handling guidelines (December 2024) reiterate audit-log obligations under APPI breach-notification rules.

### Interoperability
Audit-log payloads carry IDs of Cat 1 artefacts, Cat 4 proof certificates, Cat 8 SBOM digests; feed Cat 11 drift monitors + Cat 12 incident-response workflows. AuditEvent.entity can reference IG package version + IR hash.

### Limitations/Known issues
LLM prompts/responses may contain PHI / 要配慮 data — redaction needed before storage. Trace volume cost; sampling distorts evaluation. GenAI semantic conventions still evolving (stability pre-1.0).

### Training data proxy
High. OTel/Langfuse/LangSmith docs abundant; FHIR AuditEvent specs open.

## 11. Model, Rule, Ontology, and Terminology Drift Monitoring

### Purpose
Detect divergence from baseline of input data distributions, model performance, clinical-rule efficacy, ontology semantics, terminology versions; trigger revalidation or retraining.

### Maintainer/Standards body
Academic: Gama et al. (DDM, 2004), Baena-García et al. (EDDM, 2006), Bifet/Gavaldà (ADWIN, 2007), Page (PHT, 1954), Bach/Maloof (Paired Learners, 2008), Ross/Adams/Tasoulis/Hand (ECDD, 2012), Nishida/Yamauchi (STEPD, 2007), Sobhani/Beigy (DoF, 2011), KSWIN (Kolmogorov-Smirnov windowing). Industry/OSS: Evidently AI, NannyML, Arize, WhyLabs, Fiddler, Aporia. Terminology bodies: SNOMED International (monthly releases), WHO (ICD-11), MEDIS-DC (Japan medical-record standards), HOT9 drug master.

### Conceptual model
Data drift: PSI, KS test, Wasserstein distance, JS divergence, kdq-tree, PCA-reconstruction residual. Concept drift (supervised): error-rate monitors DDM/EDDM/ADWIN/KSWIN (DDM rule: drift when p_i + s_i ≥ p_min + 3·s_min); performance-without-labels via NannyML CBPE/DLE. Ontology/terminology drift: structural diff (added/deprecated codes), semantic diff (changed parent-child), version-effective dating.

### Expressiveness/Semantics
Statistical tests with thresholds; ADWIN provides theoretical window-change-detection guarantees (two windows, fixed and variable, sliding over incoming stream). Ontology change ops formalizable in OWL change languages (e.g., OWLDiff).

### Composability/Modularity
Per-feature, per-cohort, per-rule monitors composable. Terminology drift monitors plug into the ETL/IR generation step.

### Suitability for autoformalization to IR
IR depends on stable ontology + terminology + LLM. Triggers: (a) SNOMED CT version changes ≥ x% of mapped concepts, (b) ICD-10-JP/MEDIS update changes a concept used in a rule, (c) HOT9 drug master adds/deprecates a code referenced in IR, (d) underlying LLM checkpoint changes embedding distribution beyond threshold. Each event → reformalization with diff-based regression test.

### Formal verification potential
Ontology-change impact analysis encodable in SPARQL/SHACL. Bisimulation between successive IR versions verifiable for subset of rules.

### Tooling/Ecosystem maturity
Mature for tabular/image: river, scikit-multiflow, Evidently, NannyML, Alibi Detect, Frouros. LLM-drift tooling immature; embedding drift via Arize Phoenix, WhyLabs LangKit, Fiddler.

### Japan-specific considerations
SNOMED CT not universally adopted in Japan (historically no national license). JP-specific terminologies: MEDIS-DC standard masters (病名マスター, 手術処置マスター), ICD-10 2013 Japanese edition (ICD-11 transition underway), HOT9 (薬価基準収載医薬品コード), JLAC10/JLAC11 (lab codes). Versioning cadence + Japanese-language label changes require monitoring distinct from international updates. Minds guidelines versioned with explicit publication date; guideline drift = new edition supersedes old.

### Interoperability
Drift events emitted to Cat 10 observability bus; feed Cat 12 incident workflows; trigger Cat 9 re-deployment; may invalidate Cat 4 proofs (re-proof under new terminology).

### Limitations/Known issues
Energy/accuracy trade-offs across detectors (Poth, Kirchner, Brand, "How to Sustainably Monitor ML-Enabled Systems? Accuracy and Energy Efficiency Tradeoffs in Concept Drift Detection", arXiv 2404.19452, ICT4S 2024); three classes: (a) accurate-but-energy-heavy KSWIN, (b) balanced HDDM_W/ADWIN, (c) very low energy but unusable accuracy HDDM_A/PageHinkley/DDM/EDDM. Unsupervised drift detection often noisy.

### Training data proxy
High for general drift; low for ontology-/guideline-version drift (novel domain).

## 12. Incident Response and Post-Market Surveillance Workflows

### Purpose
Detect, triage, mitigate, report, learn from adverse events, near-misses, security incidents, AI-specific failures (hallucination, bias, prompt injection) of the deployed CDS.

### Maintainer/Standards body
FDA MedWatch + MAUDE database. PMDA Pharmaceuticals and Medical Devices Safety Information (PMDSI) + JADER (Japanese Adverse Drug Event Report) database; medical-device adverse events (不具合, fuguai) reported under PMD Act. IMDRF Adverse Event Terminology (AET; Annex A/B/C/D/E/F). AI Incident Database (AIID, Responsible AI Collaborative). MITRE ATLAS + AI Incident Sharing initiative (launched October 2024, 15+ collaborating organizations incl. Microsoft, JPMorgan Chase, Citigroup, CrowdStrike, Intel, Verizon Business). NIST AI RMF MANAGE function. Google SRE blameless postmortem methodology.

### Conceptual model
Detect (monitoring, complaints) → triage (severity, scope) → contain → eradicate → recover → blameless post-incident review → regulatory reporting → CAPA. AI red-team exercises = proactive complement.

### Expressiveness/Semantics
Structured taxonomies: IMDRF AET codes; PMDA fuguai code lists; AIID schema; MITRE ATLAS tactic/technique IDs.

### Composability/Modularity
Modular workflow: detection sources (Cat 10/11), triage (clinical safety officer + cybersecurity), reporting tracks (PMDA Class I-III, FDA MDR, EU MDR Vigilance).

### Suitability for autoformalization to IR
IR provenance enables root-cause analysis: recommendation traceable to guideline source + IR version + ontology + LLM checkpoint + prompt. Postmortem outputs feed Cat 1 GSN assurance case (new defeaters/evidence) + Cat 9 deployment (new test cases).

### Formal verification potential
Limited; postmortems narrative. Some root-cause analysis via formal counterexample replay if IR is verifiable.

### Tooling/Ecosystem maturity
Mature for traditional safety (PMDA reporting portal; FDA eMDR). AI-specific tooling growing: AIID incident IDs reached 1,361 by January 2026 (AIID Blog "AI Incident Roundup – November and December 2025 and January 2026"); MIT AI Incident Tracker classifies "more than 1,400 real-world reported incidents." Stanford HAI 2026 AI Index Report (April 2026): "Documented AI incidents continued to rise, with the AI Incident Database recording 362 in 2025, up from 233 in 2024." MITRE ATLAS Arsenal CALDERA red-team plugin (Microsoft co-developed), OpenAI/Anthropic eval harnesses, Microsoft PyRIT. Blameless postmortem templates widely available (Google SRE Workbook).

### Japan-specific considerations
PMD Act: Marketing Authorization Holders (MAH) report adverse events to PMDA; serious injuries/deaths within 10 days. JADER = Spontaneous Reporting System with reports from companies, medical institutions, post-marketing clinical trials, drug-use-result surveys, specified drug-use surveys (Tsuchiya, Hosomi, Mochinaga et al., "Quality evaluation of the Japanese Adverse Drug Event Report database (JADER)", Pharmacoepidemiol Drug Saf 2020 29(2):173–181, doi 10.1002/pds.4944). Post-marketing surveillance (製造販売後調査, PMS) + Re-examination (再審査) regulated under PMD Act. SaMD: IDATEN (PMD Act statutory provisions published 2019, in force 1 September 2020) links post-market learning to pre-approved change envelopes. MHLW + PMDA expect MAH to integrate cybersecurity-incident reporting; METI/MIC guideline v1.1 (July 2023) defines escalation between providers and medical institutions. JPCERT/CC coordinates cyber incidents; IPA provides national vulnerability handling.

### Interoperability
Inputs: Cat 10 (audit/observability), Cat 11 (drift). Outputs update Cat 1 (GSN), Cat 2 (risk file), Cat 3 (regulatory submissions), Cat 9 (rollback). AIID/ATLAS reports cross-reference Cat 6 threat-model entries.

### Limitations/Known issues
JADER has known under-reporting and reporting bias (Tsuchiya et al., 2020). Cross-jurisdictional reporting (PMDA + FDA + EU) duplicative; IMDRF AET harmonization incomplete. AI-specific incidents often lack IMDRF-AET-compatible severity coding. Blameless culture hard to sustain under regulatory pressure. EU AI Act obligations for AI systems embedded in MDR/IVDR-regulated medical devices deferred to 2 August 2028 by the 7 May 2026 EU AI Omnibus political agreement (originally 2 August 2027 under Article 6(1) for products requiring third-party conformity assessment), giving manufacturers additional preparation time for incident-reporting alignment.

### Training data proxy
High for FDA/PMDA/IMDRF processes; moderate for AI-specific incident response (newer literature, NIST workshops, IEEE SafeComp, USENIX Security AI tracks).
