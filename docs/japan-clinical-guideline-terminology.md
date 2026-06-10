# Category 2: Japan-Specific Clinical Guideline, Terminology, and Health Data Standards

## 1. Minds Guideline Library and Minds/GRADE Methodology

### Purpose
National repository and methodological framework for Japanese clinical practice guidelines (CPGs). Minds curates, evaluates (AGREE II), and republishes society-developed CPGs and prescribes a GRADE-based development methodology adapted for Japan.

### Maintainer/Standards body
Japan Council for Quality Health Care (公益財団法人日本医療機能評価機構), EBM Medical Information Department, Minds Project (Minds事業). Current methodology version: `Minds 診療ガイドライン作成マニュアル 2020 ver. 3.0` (remains the prevailing edition as of 2026; Minds Tokyo GRADE Center, established March 2019, is affiliated with the international GRADE Working Group).

### Conceptual model
Narrative-text CPGs organized around Clinical Questions (CQ) with structured metadata: PICO formulation → systematic review (SR) → evidence-to-decision (EtD) framework → recommendation strength + certainty of evidence (GRADE 4-level: High/Moderate/Low/Very Low). Document artifacts include scope (スコープ), CQ tables, SR reports, EtD tables, and a final recommendation with rationale. No machine-readable XML schema; PDFs/HTML are primary distribution. Minds website provides bibliographic metadata records for indexed guidelines.

### Expressiveness/Semantics
Captures: clinical question, eligibility criteria, intervention/comparator, outcomes (with importance rating 1–9), effect estimates, certainty rating, recommendation direction/strength, EtD criteria (benefits, harms, certainty, values, resources, equity, acceptability, feasibility). Cannot natively express: executable logic, drug-specific dosing rules, temporal pathway constraints. Free-text dominant; semantics carried by GRADE vocabulary.

### Composability/Modularity
Guideline-level granularity. CQ is the atomic reusable unit conceptually but is not formally encoded for cross-guideline reuse. No standard linkage to FHIR/CQL or computable IRs. References are bibliographic only.

### Suitability for autoformalization to IR
Moderate-to-high upstream value: the GRADE/EtD scaffolding gives LLM extractors a stable schema (CQ → population → intervention → recommendation strength + certainty). Convergence risk: free-text Japanese prose with society-specific terminology drift; recommendation wording is hedged ("行うことを弱く推奨する"). Idempotency requires anchoring to CQ IDs and recommendation IDs that are not globally unique. Best treated as the canonical text source from which the IR is generated, with Minds-AGREE evaluation acting as a CPG quality gate before autoformalization.

### Formal verification potential
None natively. Verification must be performed on a downstream IR. GRADE certainty levels and recommendation strengths can be mapped to modal/deontic operators (obligation, permission, prohibition), enabling contradiction detection across guidelines after translation. Aiding factor: explicit COI and EtD transparency increases traceability of recommendation provenance.

### Tooling/Ecosystem maturity
Minds website (`minds.jcqhc.or.jp`) hosts full text of accepted guidelines and the manual. GRADEpro GDT is the international authoring tool referenced. No dedicated open-source parser for Minds CPGs. Minds Tokyo GRADE Center disseminates methodology.

### Japan-specific considerations
Operated under MHLW commission; non-statutory but the de facto national reference. Minds Manual 2020 ver. 3.0 is the prevailing standard in Japan (templates were partially updated as recently as February 2024). Most content is Japanese-language; some guidelines provide English summaries. Public access to bibliographic records is free; full-text reuse requires permission from the copyright-holding society/publisher. Site listed several new 2024–2025 entries including `日本版敗血症診療ガイドライン2024` (J-SSCG2024, official release December 2024), `高血圧管理・治療ガイドライン2025` (Japanese Society of Hypertension, released 2025-08-29), and `骨粗鬆症の予防と治療ガイドライン2025年版`.

### Interoperability with the others on this list
Minds content references diseases by Japanese clinical names, often informally — linkage to ICD-10/MEDIS standard disease master is editorial, not formal. Drug references typically use brand or generic names (not YJ/HOT). No FHIR Clinical Reasoning, CQL, or DMN bindings. Composes with FHIR JP Core only at the human-curation step (mapping recommendation entities to JP Core CodeableConcepts). For openEHR GDL2/ePath, Minds is the source from which executable rules must be hand- or LLM-transcribed.

### Limitations/Known issues
Variable adherence to Minds methodology across societies; some "guidelines" still use narrative/expert-consensus rather than GRADE. PDF-only distribution impedes structured extraction. No persistent identifiers for individual recommendations across versions. Update cadence varies by society (3–10 years).

### Training data proxy
Strong Japanese-language coverage: full manual and ~hundreds of CPGs in PDF on `minds.jcqhc.or.jp`; abundant J-STAGE / CiNii / Ichushi-Web papers describing Minds method; Qiita/Zenn coverage minimal (not a developer-facing artifact). English papers exist (PubMed: "Minds Guide for Developing Clinical Practice Guidelines"). LLM Japanese training data on guideline content: large; on Minds technical metadata: small.

---

## 2. HL7 FHIR JP Core

### Purpose
National base FHIR Implementation Guide defining the minimum conformance profiles for accessing Japanese patient data in HL7 FHIR R4. Constrains FHIR Base resources to Japanese clinical, administrative, and code-system realities.

### Maintainer/Standards body
日本医療情報学会 (JAMI) NeXEHRS課題研究会 FHIR日本実装検討WG (now reorganized as FHIR国内実装基盤研究会), chaired by Prof. Kazuhiko Ohe (University of Tokyo). Coordinates with HL7 Japan and MHLW. Latest released: v1.2.0-a (パブリックコメント募集版, generated 2025-01-24) on jpfhir.jp; ci-build v1.3.0-dev (latest generated 2026-01-28). Based on FHIR 4.0.1.

### Conceptual model
Resource-oriented RESTful data model. Defines `StructureDefinition` profiles, `ValueSet`/`CodeSystem` bindings, `SearchParameter`s, and `CapabilityStatement` constraining FHIR R4 resources (Patient, Observation, MedicationRequest/Dispense/Administration, AllergyIntolerance, Condition, DiagnosticReport, Encounter, etc.) to JP-specific elements (kanji/kana name representations, JP postal address, HOT/YJ medication codes, JLAC10 lab codes, ICD-10 2013 JP diseases, MERIT-9 units).

### Expressiveness/Semantics
Strong for instance-level clinical data exchange (orders, results, prescriptions, allergies, problem list). Includes JP-specific code systems (e.g., `JP_AllergyIntolerance` allergen, dental body site, ECG duration code systems). Document/Bundle profiles support diagnostic reports (radiology, endoscopy, microbiology, dental, lab). Does NOT define knowledge artifacts (PlanDefinition, ActivityDefinition, Library, Measure) in JP Core; SNOMED CT not used in JP Core (explicitly out of scope per IP statement).

### Composability/Modularity
Profile inheritance from FHIR Base; resource composition via `Reference`s and `Bundle`s. ValueSet expansion via embedded JP code systems. Can be extended by domain IGs (e.g., FHIR 3文書6情報 / JP-CLINS for the MHLW 電子カルテ情報共有サービス, JP-CLINS current published edition v1.12.0, 2026-02-16). Interoperates by design with international FHIR ecosystems but not backward-compatible with HL7 v2.5 / SS-MIX2 (conversion required).

### Suitability for autoformalization to IR
High for patient-data side of CDS; the resource-profile metamodel is canonical and machine-readable. Convergence/idempotency: stable URIs for profiles and code systems make LLM-generated bindings deterministic. JP Core is the natural target for the patient-context shape of a CDS IR; recommendation/guideline content itself is NOT covered by JP Core (use FHIR Clinical Reasoning resources at the Base level).

### Formal verification potential
StructureDefinitions are formal constraints (FHIRPath invariants, cardinalities). Validatable with HL7 IG Publisher, HAPI validator, Firely .NET SDK, and Inferno. No native temporal/process logic. Contradiction detection limited to schema-level conflicts; clinical-logic verification requires CQL/Clinical Reasoning on top.

### Tooling/Ecosystem maturity
JAMI WG GitHub (`jami-fhir-jp-wg`), HAPI FHIR JP servers, Simplifier/forge projects; FHIR Validator works with JP Core packages (`jpfhir.jp.core#1.2.0-a`, `#1.3.0-dev`). Vendor: most major Japanese EHR vendors (Fujitsu, NEC, NTT Data, Finedex, SSS, etc.) publish JP Core adapters. MHLW 電子処方箋 and 電子カルテ情報共有サービス mandate FHIR profiles aligned with JP Core. Per the working-draft notice on `jpfhir.jp/fhir/core/1.3.0-dev/output/guide-general.html`, JP Core is not yet formally approved by HL7 Japan: "現時点において日本HL7協会が承認するものではない".

### Japan-specific considerations
Handles kanji/hiragana/katakana name fields explicitly (`Patient.name` with `family`/`given` + extension for kana). Postal codes, district address structures, era-dated values supported via extensions. Licensing: open (HL7-style permissive); SNOMED-CT excluded for licensing reasons; HOT codes are MEDIS-DC (free); YJ codes via 医薬情報研究所. Active MHLW alignment: 3文書6情報 (診療情報提供書, 退院時サマリ, 健診結果報告書 + 傷病名/アレルギー/感染症/薬剤禁忌/検査/処方) targets JP Core profiles; JP-CLINS implementation guide (current v1.12.0, published 2026-02-16) defines the 2文書5情報 + 患者サマリー FHIR data shapes consumed by 電子カルテ情報共有サービス. Model operations began February 2025; the MHLW grace period for medical institutions to support the service to qualify for the Medical DX Promotion System Establishment Allowance was set to May 31, 2026, with full-scale nationwide operation targeted for 令和8年度 (FY2026).

### Interoperability with the others on this list
- SS-MIX2 → JP Core: well-documented conversion path (Tohoku Univ. studies — e.g., Song & Nakayama, J Med Syst 2023, DOI 10.1007/s10916-023-01993-6; IPS implementations).
- MEDIS masters: JP Core ValueSets bind to MEDIS病名 (`urn:oid:1.2.392.200119.4.101.6` etc.), HOT, JLAC10.
- DPC/K/YJ/レセプト: JP Core MedicationRequest accepts YJ + RECETA codes; procedure codes use K-codes via `Procedure.code`.
- MedDRA/J: usable in `AllergyIntolerance.reaction.manifestation`, `AdverseEvent.event` bindings.
- ICD-11: not yet in JP Core ValueSets (pending 2027 MHLW adoption).
- OMOP CDM: external mapping (Usagi) needed.
- vs Category 1: JP Core is the patient-context substrate; FHIR Clinical Reasoning (PlanDefinition/ActivityDefinition/Library/Measure), CQL, and CDS Hooks are layered above and are NOT redefined by JP Core. openEHR GDL2/DMN/BPMN/ePath are alternative or complementary process layers that can consume JP Core resources via adapters.

### Limitations/Known issues
Still pre-1.0 (working draft); auto-machine-translated FHIR Base descriptions create awkward Japanese wording. Coverage of imaging, genomics, and care-plan resources is thinner than US Core. Lack of SNOMED CT limits semantic richness for clinical findings. No JP Core profiles for Knowledge Artifact (PlanDefinition, Library) resources.

### Training data proxy
Strong Japanese-language footprint: jpfhir.jp, hl7.jp seminar slides, JAMI proceedings, multiple Qiita/Zenn articles (search "FHIR JP Core" yields dozens), GitHub `jami-fhir-jp-wg` org and several vendor sample repos. International FHIR documentation in English is abundant. LLM coding-agent support: good for FHIR Base, moderate for JP-specific profiles.

---

## 3. SS-MIX2 Standardized Storage

### Purpose
National de facto standard for vendor-neutral export of Japanese hospital EHR data to a filesystem-based, HL7 v2.5-formatted store, enabling secondary use, BCP, and inter-hospital information exchange. Originated from MHLW 電子的診療情報交換推進事業 (2006); SS-MIX2 since 2012.

### Maintainer/Standards body
日本医療情報学会 (JAMI) standards body; SS-MIX普及推進コンソーシアム operates community support. Current spec: Ver. 1.2i (2024-05-16 release; remains the latest as of 2026). Recognized as MHLW 厚生労働省標準規格 HS026. Also approved as ISO/TS 24289:2021 ("Health informatics — Hierarchical file structure specification for secondary storage of health-related information").

### Conceptual model
Filesystem-based store: hierarchical folder structure keyed by `patientID/診療日/データ種別` containing HL7 v2.5 ER7-encoded messages. Standardized storage (10 mandatory data types — patient demographics, ADT, prescription orders, injection orders, lab orders, lab results, allergy, etc.) uses strictly defined HL7 v2.5 message profiles per JAHIS specifications. Extension storage (拡張ストレージ) accepts arbitrary structured/unstructured documents (PDF, CSV, XML, DICOM) for data not covered by the standard, e.g., SEAMAT (Japanese Circulation Society) for ECG/UCG/CATH.

### Expressiveness/Semantics
Strong: orders, results, prescriptions, injections, diagnoses, allergy, admission/discharge, observation/lab values with units (MERIT-9). Code systems referenced: ICD-10 JP, HOT, YJ, JLAC10, K-codes, MERIT-9. Weak: care plans, decision-support logic, structured progress notes, imaging. Atomic semantics defined per HL7 v2.5 segment.

### Composability/Modularity
File-per-event granularity; no inter-event references beyond patientID + timestamp. Limited reuse mechanisms. Bridges to FHIR via demonstrated converters (Tohoku Univ. SS-MIX2 → FHIR R4 PHR, IPS implementations) but conversion is many-to-many and lossy at edges.

### Suitability for autoformalization to IR
Moderate. Schema is well-defined and stable across vendors, enabling deterministic parsing. Provides patient-context input to a CDS IR but lacks knowledge/rule content. Idempotency: HL7 v2 has documented vendor-specific "ゆらぎ" (notation drift) that Ver.1.2 attempted to clarify; LLM parsing benefits from a pre-normalization layer.

### Formal verification potential
None natively. HL7 v2.5 message structure can be schema-validated; semantic verification is downstream after FHIR or CDM conversion.

### Tooling/Ecosystem maturity
Mature commercial ecosystem: most major Japanese EHR vendors ship SS-MIX2 adapters. Open viewers: SS-MIX2 ビューア (community), Tohoku Univ. converters, J-MICC tools. PMDA's MID-NET uses SS-MIX2 as its source-of-truth ingestion format.

### Japan-specific considerations
Adoption per Takenouchi K. (SS-MIX普及推進コンソーシアム), "SS-MIX2初級編" at 第44回医療情報学連合大会 (2024-11-21): 1,722 facilities had deployed SS-MIX2 標準化ストレージ as of 2024-03-31, of which 1,230 included prescription/injection data and 1,215 included laboratory results. MHLW standard HS026 (referenced in MHLW 電子カルテ情報共有サービス materials and required for DPC data submission and many MHLW grant projects). Japanese-language documentation; spec uses Japanese folder names and Shift-JIS/UTF-8 message encodings (vendor-specific). No license fee.

### Interoperability with the others on this list
- JP Core: direct converter pattern (SS-MIX2 → FHIR R4) exists and is operational at multiple academic centers.
- MEDIS masters: SS-MIX2 mandates MEDIS-coded master values in HL7 fields (病名コード, HOT, JLAC10).
- MID-NET: SS-MIX2 is the upstream feed.
- DPC/K/YJ: K-codes appear in procedure orders; YJ codes in prescriptions.
- vs Category 1: not a knowledge representation; sits below GDL2/CQL/DMN as data plane.

### Limitations/Known issues
Filesystem semantics lack ACID transactionality; concurrent write contention requires application discipline. HL7 v2.5 segment fields are sparsely populated in practice. Extension storage is heterogeneous and not standardized. No native cross-encounter linking model.

### Training data proxy
Strong Japanese-language coverage: JAMI spec PDFs, SS-MIX普及推進コンソーシアム slides on `ss-mix.org`, vendor whitepapers, Qiita posts on SS-MIX2→FHIR conversion, multiple PubMed/PMC papers (Kimura M et al., Nakayama M et al.). LLM Japanese training data: substantial.

---

## 4. MEDIS Standard Masters

### Purpose
National master code tables for clinical concepts to enable semantic interoperability across Japanese hospital systems and claims. The MEDIS-DC website lists 11 standard masters maintained by the foundation (overview document "標準マスターの概要と使い方 第24版", July 2025; current edition as of 2026).

### Maintainer/Standards body
一般財団法人 医療情報システム開発センター (MEDIS-DC), established 1974 under joint jurisdiction of MHLW and METI. Working groups (e.g., 標準病名マスター作業班) maintain each master in cooperation with 社会保険診療報酬支払基金, 日本臨床検査医学会 (JLAC), 日本歯科医師会, etc.

### Conceptual model
Tabular code-list masters in CSV/Access/Excel, downloadable from `medis.or.jp`. Each master defines a controlled vocabulary with versioned IDs, display names (kanji), kana indexes, hierarchy/mapping to external standards.
- `ICD10対応標準病名マスター` (~20,000+ disease entries; lead-term + 索引 + 修飾語 + ICD-10 code; linked 1:1 with レセプト電算用傷病名マスター since 2002).
- `HOT基準番号` (13-digit medication code unifying YJ/薬価/レセプト電算/JAN).
- `JLAC10/JLAC11` (lab item codes; JLAC11 17-digit code list released March 2024; MHLW designated JLAC11 as recommended standard 2025-07-01).
- `手術・処置マスター` (ICD-9-CM-aligned).
- `歯科マスター` (病名・処置).
- `看護実践用語マスター`, `症状所見マスター`, `画像検査マスター` (JJ1017), `医療機器DB`.

### Expressiveness/Semantics
Lexical-and-coding: provides terms, synonyms, codes, mappings, and limited hierarchy. Not ontological in the OWL sense. Disease master includes ICD-10 code and 病態 axis. No native pre-/post-coordination grammar comparable to SNOMED CT.

### Composability/Modularity
Masters cross-reference each other (e.g., HOT joins YJ + 薬価基準 + JAN + レセプト電算). 病名 master joins to 修飾語 master. Each master is updated independently with versioning.

### Suitability for autoformalization to IR
Foundational for IR terminology substrate. The 病名マスター provides the canonical disease vocabulary that an autoformalization pipeline should normalize to (avoiding hospital-local hausencodes). JLAC11 increasingly required for lab observations. HOT codes are the practical anchor for medication identity.

### Formal verification potential
Not formal logics; code-table equality and mapping consistency only. Useful as the deterministic substrate against which IR concepts can be checked (closed-world contradiction at the terminology layer).

### Tooling/Ecosystem maturity
Free download with attribution; widely embedded in EHR vendors, レセコン systems, JAHIS standards. JLACセンター (operated under 医療データ活用基盤整備機構, IDIAL — IDIAL established 2018-05; JLACセンター opened 2023-11 with full operations from 2024-04) manages JLAC11 付番.

### Japan-specific considerations
MHLW 厚生労働省標準規格 designations include HS001 (HOTコードマスター), HS005 (ICD-10対応標準病名), HS024 (看護実践用語標準マスター), etc. Free license; Japanese kanji/kana sensitive (索引 supports kana). 2025-07: MHLW changed lab standard from JLAC10 to JLAC11 (recommended). 病名 master updated 4 times/year.

### Interoperability with the others on this list
- JP Core: ValueSet bindings to MEDIS病名 (`urn:oid:1.2.392.200119.4.101.6` etc.), HOT, JLAC10.
- SS-MIX2: required code values inside HL7 v2.5 fields.
- DPC/K/YJ: 病名 master joined to レセプト電算用傷病名; YJ joins to HOT.
- ICD-11 (2027): MEDIS-DC roadmap includes mapping work via WHO-FIC Japan center.
- MedDRA/J: separate; not directly mapped by MEDIS, but JADER/PMDA bridge.
- OMOP: requires Usagi mapping (lab JLAC10→LOINC, disease ICD-10 JP→SNOMED, HOT→RxNorm).
- vs Category 1: pure terminology layer; consumed by CQL valueset operations and openEHR archetype terminology bindings.

### Limitations/Known issues
JLAC10 under-adopted (per MHLW 2024 documents, "一般医療機関でJLAC10がほとんど使用されていない"); migration to JLAC11 ongoing. ICD-10 JP is locked at 2013 edition; ICD-11 transition scheduled for 2027-01. Disease master has known under-specification for rare diseases. Vendor adoption uneven.

### Training data proxy
Moderate. PDFs and Excel masters on `medis.or.jp` and `byomei.org`; HELICS specifications. Limited English. Few GitHub repos beyond academic mappings. Qiita/Zenn coverage modest. LLM training-data abundance: low-to-moderate.

---

## 5. PMDA Electronic Package Inserts, Review Reports, and Safety Information

### Purpose
Authoritative regulatory drug/device knowledge: legally mandated 添付文書 (electronic package inserts, 電子添文/注意事項等情報), 審査報告書 (PMDA review reports), and 安全性情報 (safety bulletins, 使用上の注意改訂指示). Source-of-truth for indications, contraindications, dosing, warnings, interactions, and pharmacovigilance.

### Maintainer/Standards body
独立行政法人 医薬品医療機器総合機構 (PMDA) under MHLW; legal basis 薬機法 (改正 2021-08, paper inserts abolished 2023-07). Industry guidance: JPMA 「医療用医薬品の電子化された添付文書 作成の手引き」 2024-04 revision. Latest device 電子添文 notice: 薬機安企発第4号 / 薬機安基発第69号, 令和7年3月3日 (2025-03-03) — adoption deadline 2028-03-31 (令和10年3月31日) for Class III/IV devices.

### Conceptual model
- 電子添文 (e-PI): XML files conforming to a PMDA DTD/XSD schema with ~25 named sections (警告, 禁忌, 組成性状, 効能・効果, 用法・用量, 使用上の注意, 副作用, 薬物動態, 臨床成績, 薬効薬理, etc.). Each section has element tags 1:1 with new 記載要領 (2019-04 effective). Legacy SGML still accepted for older entries; new entries are XML only. Paired PDF retained for human readability.
- 審査報告書: PDF (typically split into 審査報告 (1) and 審査報告 (2)) summarizing quality, non-clinical, clinical pharmacology, efficacy/safety, PMDA conclusions; no machine schema beyond PDF.
- 安全性情報: PMDA-issued PDFs and structured notification metadata; includes class-wide 使用上の注意改訂指示 obliging affected MAHs to revise e-PI promptly.

### Expressiveness/Semantics
e-PI XML: high structural fidelity at section level; embedded tables, lists, citations. Free-text within sections. Vocabulary: YJ codes, GS1 barcodes, MedDRA/J for adverse events, ATC for drug class. Review reports: rich quantitative evidence but unstructured prose.

### Composability/Modularity
e-PI is per-product, versioned; supersedence via 改訂 notices. Cross-product composition (e.g., class-wide warnings) currently requires NLP/manual reconciliation across MAH-submitted files.

### Suitability for autoformalization to IR
e-PI XML is highly amenable to LLM extraction with high idempotency given the schema; section-level tags provide stable anchors. Critical inputs: 効能・効果, 用法・用量, 禁忌, 相互作用, 副作用 — directly map to CDS rules (eligibility, dose, contraindication, drug-drug interaction, adverse-event monitoring). Review reports useful for evidence provenance and dose justification but require summarization. Risk: free-text within structured sections still contains conditional logic ("腎機能障害患者では…") that LLMs must formalize uniformly across runs.

### Formal verification potential
XML is schema-validatable. Cross-product contradiction detection (e.g., two guidelines recommending different doses for the same indication) is feasible once both are lifted to a logical IR. Section-tag stability helps differential analysis.

### Tooling/Ecosystem maturity
PMDA `医療用医薬品情報検索` (`pmda.go.jp/PmdaSearch/`), 添文ナビ app (GS1 Japan + 日薬連 + 医機連). JPMA XML 作成ツール and validation guidance. Vendor DI systems integrate via PMDA API/downloads. e-PI XML schemas published by PMDA/JPMA. PMDA also announced mandatory eCTD v4.0 electronic submissions effective 2026-04-01 (the first major regulatory authority to mandate v4.0).

### Japan-specific considerations
Legally mandatory electronic distribution; paper abolished. Japanese-language only (English insert translations are non-binding). XML files use UTF-8; can contain Shift-JIS legacy artifacts in some sources. Access: free public download from PMDA. Device 電子添文 XML schema rollout staged (effective 2025-04-01; transition window through 2028-03-31).

### Interoperability with the others on this list
- JP Core: MedicationKnowledge / Medication can reference YJ + link to PMDA e-PI URL.
- MEDIS HOT/YJ: identifier substrate for product identity.
- MedDRA/J: 副作用 section terms coded in MedDRA/J PT.
- JADER: e-PI 副作用 is the prescriptive view; JADER is the empirical reporting view.
- DPC/レセプト: 効能・効果 must match 保険適用 list.
- vs Category 1: e-PI is a primary knowledge source feeding the CDS IR alongside Minds CPGs. FHIR Clinical Reasoning PlanDefinition is a natural target for dose/contraindication rules extracted from e-PI.

### Limitations/Known issues
Section text is still natural-language Japanese with conditional clauses (renal/hepatic adjustments, age strata) requiring nuanced extraction. Class-wide MHLW revision notices cause concurrent multi-MAH updates that may transiently desync products. PDF review reports lack machine schema. OTC/general-public inserts retain paper.

### Training data proxy
Strong Japanese-language documentation: PMDA portals, JPMA手引き PDFs, GMP Platform articles, vendor (renue, 日経DI) blogs. English limited. GitHub repos sparse but several exist for e-PI parsing. LLM Japanese training data on regulatory wording: substantial.

---

## 6. ICD-10 Japanese Modification and ICD-11 Mapping

### Purpose
National disease classification for mortality/morbidity statistics, hospital coding (DPC, レセプト), and clinical record management. Underpins disease vocabulary across Japan healthcare.

### Maintainer/Standards body
MHLW 政策統括官付参事官付国際分類情報管理室; WHO-FIC Collaborating Centre Japan; 社会保障審議会統計分科会疾病、傷害及び死因分類専門委員会. Current: ICD-10 (2013年版) 準拠統計分類, 告示 平成27年総務省告示第35号 (公示 2015-02-13; applied from 平成28年1月1日 / 2016-01-01). Adoption: ICD-11 (2023年版 MMS) 国内施行 2027-01 (令和9年1月); ICD-11 準拠基本分類表/疾病分類表/死因分類表 reported through 社会保障審議会統計分科会 disease subcommittee in 令和7年8月1日 (2025-08-01).

### Conceptual model
- ICD-10 JP: tree-structured alphanumeric codes (chapter → block → 3-char category → 4th-character subcategory), with Japan-specific dagger/asterisk and modifier conventions; translation curated with Japanese medical society input.
- ICD-11: multi-axial, foundation+linearization model. ~35,000 codes (vs ~16,000 ICD-10), 28 chapters (incl. Ch.26 伝統医学, Ch.V 生活機能, Ch.X extension codes). MMS subset is the statutory portion.
- Mapping: WHO-published 10↔11 transition tables; MEDIS-DC and 国際分類情報管理室 produce Japan-localized crosswalks (work ongoing).

### Expressiveness/Semantics
ICD-11 introduces post-coordination via stem + extension codes (severity, anatomy, temporality, etiology), enabling far richer semantics than ICD-10. Foundation component is OWL-aligned; linearizations are flat. ICD-10 JP is purely hierarchical.

### Composability/Modularity
ICD-11 post-coordination supports compositional expression. ICD-10 lacks this. Both link to clinical terms; ICD-11 has SNOMED CT cross-walks at the foundation layer.

### Suitability for autoformalization to IR
Critical disease ontology layer for the IR. ICD-11 foundation's IRI-addressable concepts are LLM-friendly anchors. ICD-10 JP suffices for current Japanese claims data but limits semantic richness. Convergence concern: society-specific guideline disease wording rarely matches ICD codes verbatim — MEDIS病名 disambiguation needed.

### Formal verification potential
ICD-11 foundation has OWL/RDF representation enabling DL-reasoning (subsumption, disjointness). ICD-10 JP supports only code-equality reasoning. Contradiction detection across guidelines using ICD-11 codes is more tractable than with ICD-10 codes.

### Tooling/Ecosystem maturity
WHO ICD-11 Browser, Coding Tool, API (REST/JSON); ICD-10 JP distributed as MHLW告示 + MEDIS病名 master. Japanese-language ICD-11 portal under MHLW preparing 和訳 (2023年1月版 published 2024-07).

### Japan-specific considerations
Statutory enforcement via 統計法 §28. ICD-10 JP 2013 still mandatory through 2026. 和訳 of ICD-11 terminology coordinated with 日本医学会/日本歯科医学会. Free distribution. WHO licensing for ICD-11 is open under WHO IP policy.

### Interoperability with the others on this list
- MEDIS病名 master: each entry carries ICD-10 JP code; future migration to ICD-11.
- DPC: uses ICD-10 JP codes in class A/B condition coding.
- JP Core: ValueSet bindings to ICD-10 JP; ICD-11 ValueSets pending.
- OMOP: ICD-10 has standard SNOMED mappings via Athena; ICD-11 mappings emerging.
- vs Category 1: disease classifier underlying any disease-keyed CQL valueset or DMN decision table.

### Limitations/Known issues
ICD-10 → ICD-11 transition is not 1:1; many codes split/merge. Mapping work for Japan still in progress. ICD-11 adoption requires hospital system upgrades, training; many systems will run dual-coding 2027–2030+.

### Training data proxy
Strong international (WHO, English-language PubMed). Japanese: MHLW PDFs, 日本医学会 materials, J-STAGE articles. LLM training-data abundance: high for ICD-10, moderate-rising for ICD-11.

---

## 7. DPC/PDPS, K Codes, YJ Codes, Receipt Codes, and Claims Crosswalks

### Purpose
Reimbursement code ecosystem for Japanese health insurance: case-mix grouping (DPC), surgical/procedural coding (K), drug product coding (YJ + 薬価基準), claims encoding (レセプト電算 / 診療報酬コード). Determines reimbursement and forms backbone of national claims RWD.

### Maintainer/Standards body
MHLW 保険局 (診療報酬改定 every 2 years; latest 2024 revision; next revision scheduled 2026-04). 厚生労働省医政局医薬産業振興・医療情報企画課 (旧経済課) manages 薬価基準コード. 医薬情報研究所 issues YJ. 社会保険診療報酬支払基金 issues レセプト電算用コード. 外科系学会社会保険委員会連合 (外保連) maintains STEM7 surgical taxonomy mapped to K-codes.

### Conceptual model
- DPC/PDPS: 14-digit case-mix code per admission combining major diagnostic category + diagnosis + procedure + comorbidity + severity. As of 2024-06-01 (FY2024) there were 1,786 DPC hospitals operating 483,721 DPC算定病床, per MHLW report to 中医協 (中医協 総－７－２, 令和6年4月10日; subsequent FY2025 data show a decrease to 1,761 hospitals). Per-diem prospective payment.
- K-codes: Alphanumeric (e.g., `K764`) hierarchical procedure codes from 医科点数表第10部. Tied to point counts (reimbursement). 外保連 STEM7 (7-char composite: 部位3+操作2+到達法1+補助器械1) provides clinical taxonomy; 1:N relationship to K-codes; mandatory in DPC data submission since 平成30年度 (2018).
- YJ codes: 12-character individual product codes (薬効4 + 投与経路成分3 + 剤形1 + 規格1 + 同一規格内2 + check1) for each medicinal product, developed and maintained by 株式会社医薬情報研究所. ≠ 薬価基準収載コード when 統一名収載.
- 薬価基準収載コード (厚労省コード): 12-char; tariff-line granularity.
- レセプト電算用コード: 9-digit; used in electronic claims by 支払基金.
- HOT基準番号: 13-char unifying anchor across the four above (MEDIS).
- 一般名コード: MHLW-issued for generic prescription.
- Class A/B condition codes: 7-digit local + ICD-10 in DPC claims; B1–B3 mandatory.

### Expressiveness/Semantics
Strictly classificatory/economic. Encodes what was billed, not necessarily what was clinically done in detail. K-codes lack systematic surgical taxonomy (hence STEM7).

### Composability/Modularity
HOT serves as crosswalk hub for drugs. K↔STEM7 is published as official table (`KコードSTEM7対応表 2024`). DPC composite code is built from constituent diagnosis/procedure codes per MHLW DPC grouping logic.

### Suitability for autoformalization to IR
Useful for retrospective evaluation/quality-measure binding (CQL Measure-like). Not appropriate as primary IR concept axis because of reimbursement-driven granularity bias (e.g., 病名 padding for billing). Use as evaluation substrate only.

### Formal verification potential
Coding rules are deterministic; can be encoded as decision tables (DMN-style) for billing validation. Clinical-semantic verification limited.

### Tooling/Ecosystem maturity
MHLW 診療報酬情報提供サービス (`shinryohoshu.mhlw.go.jp`); 支払基金 master downloads; commercial DPC analysis tools (MDV, JMDC). HOT mapping by MEDIS-DC.

### Japan-specific considerations
Revised every 2 years on April 1. Master files freely downloadable from MHLW/支払基金. Japanese-language masters; CSV/Excel. MHLW designates YJ + レセプト電算 + 一般名コード as standard codes for 電子処方箋/電子カルテ情報共有サービス (2024 HELICS adoption process).

### Interoperability with the others on this list
- MEDIS HOT: hub linking all drug codes.
- ICD-10 JP: appears in DPC class B condition codes.
- SS-MIX2: claims-side data may be stored alongside.
- JP Core: MedicationRequest uses YJ; Procedure uses K-codes.
- MID-NET: uses YJ + HOT + ICD-10 + JLAC10.
- OMOP: requires Usagi mapping; JMDC drug code is a recognized OHDSI source vocabulary.
- vs Category 1: usable in CQL valuesets and DMN tables; relevant for ePath pathway resource utilization checks.

### Limitations/Known issues
Reimbursement-biased coding harms diagnostic validity (PPV studies show variable accuracy). 統一名収載 vs 銘柄別収載 distinction complicates drug-product identity. K-code granularity inconsistent. Revision cycle creates temporal versioning challenges.

### Training data proxy
Strong Japanese-language documentation; many vendor blog posts, 外保連 PDFs, J-STAGE validation studies. LLM training-data: moderate-high.

---

## 8. MedDRA/J and JADER for Adverse-Event Knowledge

### Purpose
MedDRA/J: bilingual (English-Japanese) hierarchical adverse-event terminology for pharmacovigilance, regulatory reporting, and post-marketing surveillance. JADER: PMDA's spontaneous adverse drug event report database, MedDRA-coded.

### Maintainer/Standards body
- MedDRA: ICH MedDRA Management Committee → MSSO (international) + JMO (日本メンテナンス機関, hosted at PMRJ) for Japanese version. Current: MedDRA v29.0 (English release 2026-03-01, a complex release with potential changes at all five hierarchy levels; multilingual release 2026-03-15; MedDRA/J Japanese synonym version released 2026-04-01). The prior version v28.1 (2025-09-01) was a simple change release with PT/LLT-level modifications only; v28.0 was released 2025-03-01 as a complex release. Updates twice yearly (1 March / 1 September).
- JADER: PMDA; publicly available since April 2012 (data back to 2004-04). CSV downloads on `pmda.go.jp`.

### Conceptual model
- MedDRA/J: 5-level hierarchy — SOC (System Organ Class, 27) → HLGT → HLT → PT (Preferred Term) → LLT (Lowest Level Term). Plus SMQ (Standardized MedDRA Queries) for cross-PT grouping. Each English term has a unique Japanese counterpart at SOC–LLT; cultural/linguistic mappings sometimes assign the same Japanese term to different English LLTs (currency-flag mechanism preserves data consistency).
- JADER: 4 relational tables — DEMO (patient demographics), DRUG (drug name, route, dates, suspect/concomitant role), REAC (AE PT + outcome), HIST (primary illness). Linked via case ID.

### Expressiveness/Semantics
MedDRA: standardized adverse-event lexicon; not an ontology (no formal subsumption beyond hierarchy levels). SMQ is curated grouping (narrow/broad).
JADER: spontaneous-report semantics — voluntary, biased toward serious/novel events; lacks denominator.

### Composability/Modularity
MedDRA hierarchy enables roll-up analyses (PT → HLT → SOC). SMQs are reusable cross-cutting queries. JADER tables are joinable but de-duplication is required (multiple reports per case).

### Suitability for autoformalization to IR
MedDRA/J PTs are the canonical adverse-event vocabulary for the IR. SMQs as predefined queries are natural CQL valueset analogues. JADER feeds empirical AE-frequency priors for the IR but is not knowledge per se.

### Formal verification potential
Code-table consistency only. Bridging to drug-AE causal claims requires statistical signal detection (ROR, IC, BCPNN), not formal verification.

### Tooling/Ecosystem maturity
JMO portal (`jmo.pmrj.jp`) with Web-based MedDRA/J Browser, ASCII files, Japanese synonym file (released ~1 month after each MedDRA version). Multiple open-source signal-detection libraries in R/Python work with JADER. PMDA tools for JADER.

### Japan-specific considerations
MedDRA/J subscription via JMO for Japan-HQ companies; via MSSO for others. Free for regulators/academia/non-profit. JADER freely downloadable. Japanese-language documentation extensive on `pmrj.jp`; English on `meddra.org`.

### Interoperability with the others on this list
- e-PI 副作用 section: PT-coded in MedDRA/J.
- JP Core: `AllergyIntolerance`, `AdverseEvent` can bind to MedDRA/J PT.
- OMOP CDM: MedDRA included in OHDSI standard vocabulary.
- ICD-10/11: separate; ICD codes not used in JADER.
- vs Category 1: AE knowledge consumed by CQL adverse-event valuesets, DMN rule guards, and CDS Hooks `patient-view`/`order-sign` warnings.

### Limitations/Known issues
MedDRA/J's same-Japanese-term-multiple-English-LLT phenomenon complicates back-translation. JADER suffers from spontaneous-reporting bias (under-reporting, Weber effect, no denominators); regulator-action artifacts (notoriety bias). Duplicate cases require de-duplication. Causality not adjudicated.

### Training data proxy
Strong international and Japanese coverage. JMO PDFs, abundant J-STAGE/PMC papers using JADER for disproportionality analyses. GitHub repos for JADER cleaning exist. LLM training-data: high.

---

## 9. OMOP CDM / OHDSI Japan Vocabulary Mapping

### Purpose
International common-data-model (Observational Medical Outcomes Partnership) for federated observational research; OHDSI-Japan community maps Japanese vocabularies into OMOP standard concepts (SNOMED, RxNorm, LOINC) to participate in cross-national network studies.

### Maintainer/Standards body
OHDSI international consortium; Japan chapter (academic + industry; several Japanese hospitals (e.g., 国立がん研究センター東病院, 愛媛大学) and JMDC). CDM version: OMOP CDM v5.4 (latest supported release by OHDSI; v6.0 published but not supported by OHDSI tools/methods — v5.4 remains the recommended production version as of 2026, with a CDM Working Group planning a new 5-series release for 2026). Vocabulary refreshed via Athena.

### Conceptual model
Relational tables: `Person`, `Observation_Period`, `Visit_Occurrence`, `Condition_Occurrence`, `Drug_Exposure`, `Procedure_Occurrence`, `Measurement`, `Observation`, `Death`, `Note`, `Note_NLP`, `Specimen`, etc. All clinical concepts mapped to a Standard Concept ID via the OHDSI vocabulary. Source codes preserved in `*_source_concept_id` fields.

### Expressiveness/Semantics
Captures longitudinal patient events at concept-level granularity. Supports temporal cohort definitions, drug exposure eras, condition eras. Limited for unstructured/narrative content (Note table + NLP). Strong semantic harmonization via SNOMED CT (conditions), RxNorm (drugs), LOINC (labs).

### Composability/Modularity
ATLAS UI + Hades R package suite + Strategus pipelines provide modular study packages (characterization, population-level estimation, patient-level prediction). Cohort definitions are reusable JSON artifacts.

### Suitability for autoformalization to IR
The OMOP vocabulary provides a global concept-ID namespace ideal as the cross-guideline reference axis. ATLAS Cohort JSON / OHDSI Phenotype Library are partially executable specifications that can complement the IR. Autoformalization targets: cohort definitions, exposure/outcome valuesets, computable phenotypes. Convergence: stable concept_id semantics across OHDSI releases.

### Formal verification potential
ATLAS cohort JSON can be translated to SQL deterministically. Cohort-overlap and concept-set consistency checks are tooled (Cohort Diagnostics). Not theorem-prover-grade, but supports reproducible audit.

### Tooling/Ecosystem maturity
ATLAS, Achilles, DataQualityDashboard, Hades R packages, Usagi (the namesake "rabbit" tool was originally for Japanese-source mapping), Strategus, Eunomia, Ares. Active community.

### Japan-specific considerations
Several Japanese RWD vendors (JMDC, Medical Data Vision, RWD Co.) provide OMOP-mapped extracts. Academic OMOP conversions at 国立がん研究センター東病院 (Aoyagi Y, Baba M, Terao S, Ikeda Y, Nomura K, Sato A, "Feasibility of Converting EMR Data to OMOP CDM and Utilizing OHDSI Analysis Tools in Japan," Stud Health Technol Inform 2025;329:1946–1947, DOI 10.3233/SHTI251292, PMID 40776309 — breast cancer cohort 8,387 patients, 7,259 terms mapped to OHDSI standard vocabulary), 愛媛大学 (Kimura E, Kawakami Y, Inoue S, Okajima A, "A dataset for mapping the Japanese drugs to RxNorm standard concepts," Data in Brief 2025, DOI 10.1016/j.dib.2025.111418, PMC11926709 — LLM-assisted YJ→RxNorm mappings using Mixtral 8×7B with BioBERT-RAG). OHDSI vocabulary additions for Japan include JMDC drug code. JLAC10/11→LOINC mapping non-trivial.

### Interoperability with the others on this list
- MEDIS HOT/YJ → RxNorm: Ehime University 2025 dataset (Kimura et al., Data in Brief 10.1016/j.dib.2025.111418) provides LLM-assisted mappings.
- JLAC10 → LOINC: incomplete; community effort.
- ICD-10 JP → SNOMED CT: via Athena standard mappings (with Japan-specific edge cases).
- SS-MIX2 → OMOP: ETL pipelines published (medRxiv 2025-06).
- JP Core ↔ OMOP: bidirectional adapters (FHIR-to-OMOP and OMOP-on-FHIR).
- vs Category 1: OMOP cohort JSON ≈ openEHR query AQL ≈ FHIR CQL — three parallel representations of computable phenotypes; the IR may target one canonical form and translate.

### Limitations/Known issues
Mapping Japanese vocabularies to OMOP standard concepts is labor-intensive; coverage gaps in drug (combination products, Japan-only generics) and lab (JLAC10 method specificity > LOINC). ETL of SS-MIX2 → OMOP has documented loss for nursing/narrative data. Race/ethnicity fields underspecified for Japan. Concept_id stability requires vocab version pinning.

### Training data proxy
Strong English documentation (Book of OHDSI, ohdsi.org). Japanese: J-STAGE papers, OHDSI Japan symposium materials, growing Qiita/Zenn posts. GitHub: many ATLAS/Hades repos. LLM training-data: high for OHDSI core, moderate for Japan mappings.

---

## 10. MID-NET, NDB Open Data, and Real-World-Evidence Interfaces

### Purpose
National RWE/RWD platforms enabling pharmacoepidemiology, post-marketing safety, and policy analytics. MID-NET = EHR-centric distributed federated network (PMDA-operated). NDB = nationwide claims + 特定健診 database (MHLW). NDB Open Data = aggregated public release.

### Maintainer/Standards body
- MID-NET: PMDA 医療情報科学部 (現 医療情報活用部). Operational since 2018-04. Cooperative medical institutions reduced from 10 拠点 to 9 拠点 (31 hospitals total) effective 2024-12-01 after NTT Hospital Group withdrawal (per PMDA "MID-NET® Update 2024" presentation by 医療情報科学部長 山口光峰, 2024-11-28, https://www.pmda.go.jp/files/000272301.pdf). Database population "800万人超（2023年12月末時点）" (over 8 million as of end of December 2023, per the same PMDA slide deck); a further expansion of 10 Tokushukai Group hospitals (bringing Tokushukai's total to 20) began April 2024. Subsequent functional enhancements include a December 2025 server expansion procurement.
- NDB: MHLW 保険局; launched 2009. NDB Open Data since 2016. 2020 legislation permits provision to private companies. "The NDB covers approximately 98% of data on healthcare services provided by medical institutions" (Yasunaga H., "Updated Information on NDB," Ann Clin Epidemiol 2024;6(3):73–76, DOI 10.37737/ace.24011, PMC11254583).

### Conceptual model
- MID-NET: Distributed closed network; data centrally normalized to a MID-NET common data model populated from SS-MIX2 standardized storage at each site, with standardized code mappings (ICD-10, YJ, HOT, JLAC10, MERIT-9, JAMI usage standard). On-site analysis center (PMDA, 新霞が関ビル) + MID-NET接続環境 remote access; user-developed extraction programs require site approval before execution; aggregated results returned (individual-level data cannot be downloaded). Per Yamaguchi M et al. 2019 (Pharmacoepidemiol Drug Saf 2019;28(10):1395–1404, DOI 10.1002/pds.4879, PMC6851601): "MID-NET® adopts a common data model that stores a wide variety of hospital information system (HIS) data… standardized based on the message specifications of SS-MIX2." Conceptually inspired by FDA Sentinel; distinct from OMOP CDM.
- NDB: Centralized claims (医科 / 歯科 / 調剤 / DPC) + 特定健診. Anonymized hash IDs. Four data products: (1) General Data (special extraction + Onsite Research Center), (2) Sampling Data, (3) Accumulated Data, (4) Open Data (aggregated tables, MHLW website).

### Expressiveness/Semantics
- MID-NET: clinical lab results (with units), prescriptions, diagnoses, vitals — richer than claims. Suitable for biomarker-defined cohorts.
- NDB: comprehensive claims but no lab values (except 特定健診 health checkup data); diagnosis validity limited by reimbursement coding artifacts.

### Composability/Modularity
- MID-NET: distributed query model; results aggregated centrally. Programs reusable across sites.
- NDB: monolithic database; standardized record layouts. Linkage to other national DBs (DPC, infectious disease surveillance, designated incurable disease, specific chronic pediatric disease) planned (2024-).

### Suitability for autoformalization to IR
Both serve as evaluation/validation substrates for CDS IR, not as knowledge sources. MID-NET's normalized SS-MIX2-derived schema is closer to a CDS-ready longitudinal patient model. NDB Open Data aggregates support epidemiological priors. Idempotency benefits from MID-NET's enforced code standardization.

### Formal verification potential
None native. Can serve as empirical test bed: a CDS IR's recommendations can be back-validated against MID-NET/NDB cohorts to detect quantitative implausibility.

### Tooling/Ecosystem maturity
- MID-NET: PMDA-provided SAS environment, on-site center, remote access (MID-NET接続環境). New MID-NET Guideline July 2024 changed operational procedures; on-site center relocated within 新霞が関ビル (20F→6F, completed December 2025). Year-round application accepted since FY2024.
- NDB: MHLW Onsite Research Center, NDB Open Data published on MHLW website (aggregated tables). Healthcare Intelligence Cloud (HIC) platform under development. JMDC, DPC databases parallel commercial options.

### Japan-specific considerations
- MID-NET access: pharmaceutical companies, researchers, PMDA all eligible; application is now year-round (since FY2024). Fees not differentiated by user type (industry vs academia); three categories by analysis type, calibrated to recover ~¥1.234 billion annual operating cost (per MHLW WG document, https://www.mhlw.go.jp/file/05-Shingikai-11121000-Iyakushokuhinkyoku-Soumuka/0000171825.pdf). Compliant with GPSP省令 reliability standards.
- NDB: access restricted (originally only national/local government, universities, quasi-public; broadened 2020 to include private companies via formal application + advisory committee approval). NDB Open Data freely downloadable.
- Both Japanese-language operationally.

### Interoperability with the others on this list
- SS-MIX2 → MID-NET: required ingestion format.
- MEDIS masters: MID-NET enforces JLAC10, HOT, YJ, ICD-10 across sites.
- DPC/レセプト: NDB primary source.
- JP Core: emerging FHIR endpoints over MID-NET/NDB are research-stage.
- OMOP: NDB/JMDC OMOP conversions exist commercially; MID-NET native model is not OMOP.
- MedDRA/J: not natively in MID-NET (focus is EHR-derived AE detection via lab/diagnosis signals).
- vs Category 1: serves as the verification/evaluation substrate. CQL Measure / OMOP cohort definitions / openEHR AQL queries are the natural query forms; ePath outcomes can be benchmarked against NDB.

### Limitations/Known issues
- MID-NET: small relative national share; selection bias (academic/specialty hospitals); on-site analysis only, no individual-level export; learning curve for SAS/closed env; mapping consistency across sites historically variable.
- NDB: claims-only (no lab values), diagnosis PPV variable, no facility identifiers, complex application process, hashed IDs imperfect for longitudinal linkage, no linkage to mortality.

### Training data proxy
Strong Japanese-language coverage. PMDA `mid-net` portal, MHLW NDB pages, J-STAGE/PMC reviews (Sato & Yasunaga 2023, Yasunaga 2024, Suto et al. 2024). English: Yamaguchi et al. 2019 (Pharmacoepidemiol Drug Saf 28:1395–1404), Suto et al. 2024 (JMA Journal — NDB literature review). GitHub repos sparse (restricted environments). LLM training-data: moderate.
