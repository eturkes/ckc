# Category 2: Japan-Specific Clinical Guideline, Terminology, and Health Data Standards

## 1. Minds Guideline Library and Minds/GRADE Methodology

### Purpose
National repository + methodological framework for Japanese CPGs: curates, evaluates (AGREE II), republishes society-developed CPGs; prescribes GRADE-based development methodology adapted for Japan.

### Maintainer/Standards body
Japan Council for Quality Health Care (公益財団法人日本医療機能評価機構), EBM Medical Information Department, Minds Project (Minds事業). Methodology: `Minds 診療ガイドライン作成マニュアル 2020 ver. 3.0` (prevailing as of 2026). Minds Tokyo GRADE Center (est. March 2019) affiliated with international GRADE Working Group.

### Conceptual model
Narrative CPGs around Clinical Questions (CQ) with structured metadata: PICO → systematic review (SR) → evidence-to-decision (EtD) → recommendation strength + certainty (GRADE 4-level: High/Moderate/Low/Very Low). Artifacts: scope (スコープ), CQ tables, SR reports, EtD tables, final recommendation + rationale. No machine-readable XML schema; PDF/HTML distribution; website provides bibliographic metadata records.

### Expressiveness/Semantics
Captures: clinical question, eligibility criteria, intervention/comparator, outcomes (importance 1–9), effect estimates, certainty rating, recommendation direction/strength, EtD criteria (benefits, harms, certainty, values, resources, equity, acceptability, feasibility). Cannot express: executable logic, drug-specific dosing rules, temporal pathway constraints. Free-text dominant; semantics via GRADE vocabulary.

### Composability/Modularity
Guideline-level granularity; CQ is conceptual atomic unit but not formally encoded for cross-guideline reuse. No FHIR/CQL/computable-IR linkage; references bibliographic only.

### Suitability for autoformalization to IR
Moderate-to-high upstream value: GRADE/EtD scaffolding gives LLM extractors a stable schema (CQ → population → intervention → recommendation strength + certainty). Convergence risks: free-text Japanese prose, society-specific terminology drift, hedged wording ("行うことを弱く推奨する"). Idempotency needs anchoring to CQ/recommendation IDs, which are not globally unique. Treat as canonical text source for IR generation, with Minds-AGREE evaluation as CPG quality gate.

### Formal verification potential
None natively; verify on downstream IR. GRADE certainty/strength map to modal/deontic operators (obligation, permission, prohibition), enabling cross-guideline contradiction detection after translation. Explicit COI + EtD transparency aids recommendation provenance.

### Tooling/Ecosystem maturity
`minds.jcqhc.or.jp` hosts accepted guidelines full text + manual. GRADEpro GDT is the referenced international authoring tool. No dedicated open-source Minds parser. Minds Tokyo GRADE Center disseminates methodology.

### Japan-specific considerations
MHLW commission; non-statutory de facto national reference. Manual 2020 ver. 3.0 templates partially updated February 2024. Mostly Japanese; some English summaries. Bibliographic records free; full-text reuse requires copyright-holding society/publisher permission. New 2024–2025 entries: `日本版敗血症診療ガイドライン2024` (J-SSCG2024, official release December 2024), `高血圧管理・治療ガイドライン2025` (Japanese Society of Hypertension, 2025-08-29), `骨粗鬆症の予防と治療ガイドライン2025年版`.

### Interoperability with the others on this list
Diseases referenced by Japanese clinical names; ICD-10/MEDIS disease-master linkage editorial, not formal. Drugs by brand/generic name (not YJ/HOT). No FHIR Clinical Reasoning, CQL, or DMN bindings; composes with FHIR JP Core only via human curation (recommendation entities → JP Core CodeableConcepts). For openEHR GDL2/ePath, Minds is the source from which executable rules are hand-/LLM-transcribed.

### Limitations/Known issues
Variable methodology adherence across societies; some guidelines narrative/expert-consensus, not GRADE. PDF-only distribution impedes extraction. No persistent per-recommendation identifiers across versions. Update cadence varies by society (3–10 years).

### Training data proxy
Strong Japanese: manual + ~hundreds of CPG PDFs on `minds.jcqhc.or.jp`; abundant J-STAGE/CiNii/Ichushi-Web method papers; Qiita/Zenn minimal. English exists (PubMed: "Minds Guide for Developing Clinical Practice Guidelines"). LLM data: large on guideline content, small on Minds technical metadata.

## 2. HL7 FHIR JP Core

### Purpose
National base FHIR Implementation Guide: minimum conformance profiles for Japanese patient data in HL7 FHIR R4; constrains FHIR Base to Japanese clinical/administrative/code-system realities.

### Maintainer/Standards body
日本医療情報学会 (JAMI) NeXEHRS課題研究会 FHIR日本実装検討WG (now FHIR国内実装基盤研究会), chaired by Prof. Kazuhiko Ohe (University of Tokyo); coordinates with HL7 Japan + MHLW. Latest released: v1.2.0-a (パブリックコメント募集版, generated 2025-01-24) on jpfhir.jp; ci-build v1.3.0-dev (generated 2026-01-28). Based on FHIR 4.0.1.

### Conceptual model
Resource-oriented RESTful: `StructureDefinition` profiles, `ValueSet`/`CodeSystem` bindings, `SearchParameter`s, `CapabilityStatement` constraining R4 resources (Patient, Observation, MedicationRequest/Dispense/Administration, AllergyIntolerance, Condition, DiagnosticReport, Encounter, etc.) to JP elements: kanji/kana names, JP postal address, HOT/YJ medication codes, JLAC10 lab codes, ICD-10 2013 JP diseases, MERIT-9 units.

### Expressiveness/Semantics
Strong for instance-level exchange (orders, results, prescriptions, allergies, problem list). JP code systems: `JP_AllergyIntolerance` allergen, dental body site, ECG duration. Document/Bundle profiles for diagnostic reports (radiology, endoscopy, microbiology, dental, lab). Does NOT define knowledge artifacts (PlanDefinition, ActivityDefinition, Library, Measure); SNOMED CT excluded (out of scope per IP statement).

### Composability/Modularity
Profile inheritance from FHIR Base; composition via `Reference`s/`Bundle`s; ValueSet expansion via embedded JP code systems. Extensible by domain IGs: FHIR 3文書6情報 / JP-CLINS for MHLW 電子カルテ情報共有サービス (JP-CLINS current published v1.12.0, 2026-02-16). Interoperates with international FHIR; not backward-compatible with HL7 v2.5/SS-MIX2 (conversion required).

### Suitability for autoformalization to IR
High for patient-data side of CDS: resource-profile metamodel canonical + machine-readable; stable profile/code-system URIs make LLM bindings deterministic. Natural target for the patient-context shape of a CDS IR; recommendation/guideline content NOT covered (use FHIR Clinical Reasoning at Base level).

### Formal verification potential
StructureDefinitions are formal constraints (FHIRPath invariants, cardinalities); validatable with HL7 IG Publisher, HAPI validator, Firely .NET SDK, Inferno. No temporal/process logic; contradiction detection schema-level only — clinical logic needs CQL/Clinical Reasoning on top.

### Tooling/Ecosystem maturity
JAMI WG GitHub (`jami-fhir-jp-wg`), HAPI FHIR JP servers, Simplifier/forge; FHIR Validator works with packages `jpfhir.jp.core#1.2.0-a`, `#1.3.0-dev`. Most major JP EHR vendors (Fujitsu, NEC, NTT Data, Finedex, SSS, etc.) publish adapters. MHLW 電子処方箋 + 電子カルテ情報共有サービス mandate JP Core-aligned profiles. Per working-draft notice on `jpfhir.jp/fhir/core/1.3.0-dev/output/guide-general.html`, not yet HL7 Japan-approved: "現時点において日本HL7協会が承認するものではない".

### Japan-specific considerations
Kanji/hiragana/katakana names (`Patient.name` `family`/`given` + kana extension); postal codes, district addresses, era-dated values via extensions. Licensing open (HL7-style permissive); SNOMED CT excluded for licensing; HOT = MEDIS-DC (free); YJ via 医薬情報研究所. MHLW alignment: 3文書6情報 (診療情報提供書, 退院時サマリ, 健診結果報告書 + 傷病名/アレルギー/感染症/薬剤禁忌/検査/処方) targets JP Core profiles; JP-CLINS v1.12.0 (2026-02-16) defines 2文書5情報 + 患者サマリー shapes for 電子カルテ情報共有サービス. Model operations began February 2025; MHLW grace period for institutions to qualify for the Medical DX Promotion System Establishment Allowance set to May 31, 2026; full-scale nationwide operation targeted 令和8年度 (FY2026).

### Interoperability with the others on this list
- SS-MIX2 → JP Core: documented conversion path (Tohoku Univ. — Song & Nakayama, J Med Syst 2023, DOI 10.1007/s10916-023-01993-6; IPS implementations).
- MEDIS masters: ValueSets bind MEDIS病名 (`urn:oid:1.2.392.200119.4.101.6` etc.), HOT, JLAC10.
- DPC/K/YJ/レセプト: MedicationRequest accepts YJ + RECETA codes; K-codes via `Procedure.code`.
- MedDRA/J: usable in `AllergyIntolerance.reaction.manifestation`, `AdverseEvent.event`.
- ICD-11: not yet in ValueSets (pending 2027 MHLW adoption).
- OMOP CDM: external mapping (Usagi).
- vs Category 1: patient-context substrate; FHIR Clinical Reasoning (PlanDefinition/ActivityDefinition/Library/Measure), CQL, CDS Hooks layered above, NOT redefined by JP Core; openEHR GDL2/DMN/BPMN/ePath are alternative/complementary process layers consuming JP Core via adapters.

### Limitations/Known issues
Pre-1.0 working draft; auto-machine-translated FHIR Base descriptions yield awkward Japanese. Imaging/genomics/care-plan coverage thinner than US Core. No SNOMED CT limits semantic richness for findings. No Knowledge Artifact (PlanDefinition, Library) profiles.

### Training data proxy
Strong Japanese: jpfhir.jp, hl7.jp seminar slides, JAMI proceedings, dozens of Qiita/Zenn hits ("FHIR JP Core"), GitHub `jami-fhir-jp-wg` + vendor sample repos. English FHIR docs abundant. LLM coding-agent support: good for FHIR Base, moderate for JP profiles.

## 3. SS-MIX2 Standardized Storage

### Purpose
National de facto standard for vendor-neutral export of hospital EHR data to a filesystem-based HL7 v2.5 store: secondary use, BCP, inter-hospital exchange. Origin: MHLW 電子的診療情報交換推進事業 (2006); SS-MIX2 since 2012.

### Maintainer/Standards body
JAMI standards body; SS-MIX普及推進コンソーシアム community support. Spec: Ver. 1.2i (2024-05-16; latest as of 2026). MHLW 厚生労働省標準規格 HS026. Also ISO/TS 24289:2021 ("Health informatics — Hierarchical file structure specification for secondary storage of health-related information").

### Conceptual model
Hierarchical folders keyed `patientID/診療日/データ種別` holding HL7 v2.5 ER7 messages. Standardized storage: 10 mandatory data types (patient demographics, ADT, prescription orders, injection orders, lab orders, lab results, allergy, etc.) with strict HL7 v2.5 message profiles per JAHIS. Extension storage (拡張ストレージ): arbitrary documents (PDF, CSV, XML, DICOM) for uncovered data, e.g., SEAMAT (Japanese Circulation Society) for ECG/UCG/CATH.

### Expressiveness/Semantics
Strong: orders, results, prescriptions, injections, diagnoses, allergy, admission/discharge, lab values with units (MERIT-9). Codes: ICD-10 JP, HOT, YJ, JLAC10, K-codes, MERIT-9. Weak: care plans, decision-support logic, structured progress notes, imaging. Atomic semantics per HL7 v2.5 segment.

### Composability/Modularity
File-per-event; no inter-event references beyond patientID + timestamp; limited reuse. FHIR bridges via demonstrated converters (Tohoku Univ. SS-MIX2 → FHIR R4 PHR, IPS) but many-to-many, lossy at edges.

### Suitability for autoformalization to IR
Moderate. Well-defined, vendor-stable schema enables deterministic parsing; provides patient-context input only — no knowledge/rule content. Idempotency: documented vendor "ゆらぎ" (notation drift), partially clarified in Ver.1.2; LLM parsing benefits from pre-normalization layer.

### Formal verification potential
None natively. HL7 v2.5 message structure schema-validatable; semantic verification downstream after FHIR/CDM conversion.

### Tooling/Ecosystem maturity
Mature commercial: most major JP EHR vendors ship adapters. Open: SS-MIX2 ビューア (community), Tohoku Univ. converters, J-MICC tools. PMDA MID-NET uses SS-MIX2 as source-of-truth ingestion format.

### Japan-specific considerations
Adoption per Takenouchi K. (SS-MIX普及推進コンソーシアム), "SS-MIX2初級編", 第44回医療情報学連合大会 (2024-11-21): 1,722 facilities with 標準化ストレージ as of 2024-03-31; 1,230 incl. prescription/injection data; 1,215 incl. lab results. HS026 referenced in MHLW 電子カルテ情報共有サービス materials; required for DPC data submission and many MHLW grant projects. Japanese docs; Japanese folder names; Shift-JIS/UTF-8 encodings (vendor-specific). No license fee.

### Interoperability with the others on this list
- JP Core: SS-MIX2 → FHIR R4 converter pattern operational at multiple academic centers.
- MEDIS masters: mandates MEDIS-coded values in HL7 fields (病名コード, HOT, JLAC10).
- MID-NET: SS-MIX2 is upstream feed.
- DPC/K/YJ: K-codes in procedure orders; YJ in prescriptions.
- vs Category 1: data plane below GDL2/CQL/DMN; not a knowledge representation.

### Limitations/Known issues
Filesystem lacks ACID transactionality; concurrent writes need application discipline. HL7 v2.5 fields sparsely populated in practice. Extension storage heterogeneous, unstandardized. No cross-encounter linking model.

### Training data proxy
Strong Japanese: JAMI spec PDFs, `ss-mix.org` slides, vendor whitepapers, Qiita SS-MIX2→FHIR posts, PubMed/PMC (Kimura M et al., Nakayama M et al.). LLM data substantial.

## 4. MEDIS Standard Masters

### Purpose
National master code tables for clinical concepts → semantic interop across hospital systems and claims. MEDIS-DC lists 11 standard masters (overview "標準マスターの概要と使い方 第24版", July 2025; current as of 2026).

### Maintainer/Standards body
一般財団法人 医療情報システム開発センター (MEDIS-DC), est. 1974, joint MHLW + METI jurisdiction. Working groups (e.g., 標準病名マスター作業班) with 社会保険診療報酬支払基金, 日本臨床検査医学会 (JLAC), 日本歯科医師会, etc.

### Conceptual model
Tabular CSV/Access/Excel masters from `medis.or.jp`: controlled vocabularies with versioned IDs, kanji display names, kana indexes, hierarchy/mappings to external standards.
- `ICD10対応標準病名マスター` (~20,000+ entries; lead-term + 索引 + 修飾語 + ICD-10 code; 1:1 with レセプト電算用傷病名マスター since 2002).
- `HOT基準番号` (13-digit medication code unifying YJ/薬価/レセプト電算/JAN).
- `JLAC10/JLAC11` (lab codes; JLAC11 17-digit list released March 2024; MHLW designated JLAC11 recommended standard 2025-07-01).
- `手術・処置マスター` (ICD-9-CM-aligned).
- `歯科マスター` (病名・処置).
- `看護実践用語マスター`, `症状所見マスター`, `画像検査マスター` (JJ1017), `医療機器DB`.

### Expressiveness/Semantics
Lexical-and-coding: terms, synonyms, codes, mappings, limited hierarchy; not OWL-ontological. Disease master includes ICD-10 code + 病態 axis. No pre-/post-coordination grammar comparable to SNOMED CT.

### Composability/Modularity
Masters cross-reference (HOT joins YJ + 薬価基準 + JAN + レセプト電算; 病名 joins 修飾語); each updated independently with versioning.

### Suitability for autoformalization to IR
Foundational terminology substrate. 病名マスター is the canonical disease vocabulary for pipeline normalization (avoiding hospital-local hausencodes). JLAC11 increasingly required for lab observations; HOT is the practical anchor for medication identity.

### Formal verification potential
Not formal logics; code-table equality + mapping consistency only — deterministic substrate for closed-world contradiction checks at the terminology layer.

### Tooling/Ecosystem maturity
Free download with attribution; embedded in EHR vendors, レセコン systems, JAHIS standards. JLACセンター under 医療データ活用基盤整備機構 (IDIAL, est. 2018-05; center opened 2023-11, full operations from 2024-04) manages JLAC11 付番.

### Japan-specific considerations
MHLW 厚生労働省標準規格: HS001 (HOTコードマスター), HS005 (ICD-10対応標準病名), HS024 (看護実践用語標準マスター), etc. Free license; kanji/kana sensitive (索引 supports kana). 2025-07: MHLW changed lab standard JLAC10 → JLAC11 (recommended). 病名 master updated 4 times/year.

### Interoperability with the others on this list
- JP Core: ValueSet bindings to MEDIS病名 (`urn:oid:1.2.392.200119.4.101.6` etc.), HOT, JLAC10.
- SS-MIX2: required code values in HL7 v2.5 fields.
- DPC/K/YJ: 病名 joined to レセプト電算用傷病名; YJ joins HOT.
- ICD-11 (2027): MEDIS-DC roadmap includes mapping via WHO-FIC Japan center.
- MedDRA/J: separate, not MEDIS-mapped; JADER/PMDA bridge.
- OMOP: Usagi mapping needed (JLAC10→LOINC, ICD-10 JP→SNOMED, HOT→RxNorm).
- vs Category 1: pure terminology layer consumed by CQL valuesets + openEHR archetype terminology bindings.

### Limitations/Known issues
JLAC10 under-adopted (MHLW 2024: "一般医療機関でJLAC10がほとんど使用されていない"); JLAC11 migration ongoing. ICD-10 JP locked at 2013 edition; ICD-11 transition 2027-01. Disease master under-specifies rare diseases. Vendor adoption uneven.

### Training data proxy
Moderate. PDFs/Excel on `medis.or.jp` + `byomei.org`; HELICS specifications. Limited English. Few GitHub repos beyond academic mappings; Qiita/Zenn modest. LLM abundance low-to-moderate.

## 5. PMDA Electronic Package Inserts, Review Reports, and Safety Information

### Purpose
Authoritative regulatory drug/device knowledge: legally mandated 添付文書 (electronic package inserts, 電子添文/注意事項等情報), 審査報告書 (PMDA review reports), 安全性情報 (safety bulletins, 使用上の注意改訂指示). Source-of-truth for indications, contraindications, dosing, warnings, interactions, pharmacovigilance.

### Maintainer/Standards body
独立行政法人 医薬品医療機器総合機構 (PMDA) under MHLW; legal basis 薬機法 (改正 2021-08; paper inserts abolished 2023-07). Industry guidance: JPMA 「医療用医薬品の電子化された添付文書 作成の手引き」 2024-04 revision. Latest device 電子添文 notice: 薬機安企発第4号 / 薬機安基発第69号, 令和7年3月3日 (2025-03-03) — adoption deadline 2028-03-31 (令和10年3月31日) for Class III/IV devices.

### Conceptual model
- 電子添文 (e-PI): XML per PMDA DTD/XSD, ~25 named sections (警告, 禁忌, 組成性状, 効能・効果, 用法・用量, 使用上の注意, 副作用, 薬物動態, 臨床成績, 薬効薬理, etc.); element tags 1:1 with new 記載要領 (effective 2019-04). Legacy SGML still accepted for older entries; new entries XML only; paired PDF for human readability.
- 審査報告書: PDF (typically 審査報告 (1) + (2)) covering quality, non-clinical, clinical pharmacology, efficacy/safety, PMDA conclusions; no machine schema.
- 安全性情報: PMDA PDFs + structured notification metadata; class-wide 使用上の注意改訂指示 oblige affected MAHs to revise e-PI promptly.

### Expressiveness/Semantics
e-PI XML: high section-level structural fidelity; embedded tables, lists, citations; free text within sections. Vocab: YJ codes, GS1 barcodes, MedDRA/J for adverse events, ATC drug class. Review reports: rich quantitative evidence, unstructured prose.

### Composability/Modularity
Per-product, versioned; supersedence via 改訂 notices. Cross-product composition (class-wide warnings) requires NLP/manual reconciliation across MAH-submitted files.

### Suitability for autoformalization to IR
Highly amenable to LLM extraction with high idempotency given the schema; section tags are stable anchors. Critical inputs 効能・効果, 用法・用量, 禁忌, 相互作用, 副作用 map directly to CDS rules (eligibility, dose, contraindication, drug-drug interaction, AE monitoring). Review reports useful for evidence provenance/dose justification but need summarization. Risk: free text within structured sections carries conditional logic ("腎機能障害患者では…") that LLMs must formalize uniformly across runs.

### Formal verification potential
XML schema-validatable. Cross-product contradiction detection (e.g., differing doses for same indication) feasible once lifted to a logical IR. Section-tag stability aids differential analysis.

### Tooling/Ecosystem maturity
PMDA `医療用医薬品情報検索` (`pmda.go.jp/PmdaSearch/`), 添文ナビ app (GS1 Japan + 日薬連 + 医機連). JPMA XML 作成ツール + validation guidance. Vendor DI systems via PMDA API/downloads. e-PI XML schemas published by PMDA/JPMA. PMDA mandates eCTD v4.0 electronic submissions from 2026-04-01 (first major regulator to mandate v4.0).

### Japan-specific considerations
Electronic distribution legally mandatory; paper abolished. Japanese-only (English translations non-binding). UTF-8 XML; some Shift-JIS legacy artifacts. Free public download. Device 電子添文 XML rollout staged (effective 2025-04-01; transition through 2028-03-31).

### Interoperability with the others on this list
- JP Core: MedicationKnowledge/Medication reference YJ + PMDA e-PI URL.
- MEDIS HOT/YJ: identifier substrate for product identity.
- MedDRA/J: 副作用 terms coded as MedDRA/J PT.
- JADER: e-PI 副作用 = prescriptive view; JADER = empirical reporting view.
- DPC/レセプト: 効能・効果 must match 保険適用 list.
- vs Category 1: primary knowledge source feeding CDS IR alongside Minds CPGs; FHIR Clinical Reasoning PlanDefinition natural target for dose/contraindication rules.

### Limitations/Known issues
Section text remains natural-language Japanese with conditional clauses (renal/hepatic adjustments, age strata) requiring nuanced extraction. Class-wide MHLW revision notices cause concurrent multi-MAH updates with transient product desync. PDF review reports lack machine schema. OTC/general-public inserts retain paper.

### Training data proxy
Strong Japanese: PMDA portals, JPMA手引き PDFs, GMP Platform articles, vendor blogs (renue, 日経DI). English limited. GitHub sparse but e-PI parsers exist. LLM regulatory-wording data substantial.

## 6. ICD-10 Japanese Modification and ICD-11 Mapping

### Purpose
National disease classification for mortality/morbidity statistics, hospital coding (DPC, レセプト), clinical record management; underpins disease vocabulary across Japan healthcare.

### Maintainer/Standards body
MHLW 政策統括官付参事官付国際分類情報管理室; WHO-FIC Collaborating Centre Japan; 社会保障審議会統計分科会疾病、傷害及び死因分類専門委員会. Current: ICD-10 (2013年版) 準拠統計分類, 告示 平成27年総務省告示第35号 (公示 2015-02-13; applied 平成28年1月1日 / 2016-01-01). ICD-11 (2023年版 MMS) 国内施行 2027-01 (令和9年1月); ICD-11 準拠基本分類表/疾病分類表/死因分類表 reported through the disease subcommittee 令和7年8月1日 (2025-08-01).

### Conceptual model
- ICD-10 JP: tree-structured alphanumeric codes (chapter → block → 3-char category → 4th-character subcategory); Japan-specific dagger/asterisk + modifier conventions; translation curated with Japanese medical society input.
- ICD-11: multi-axial foundation+linearization; ~35,000 codes (vs ~16,000 in ICD-10), 28 chapters (incl. Ch.26 伝統医学, Ch.V 生活機能, Ch.X extension codes); MMS subset is statutory.
- Mapping: WHO 10↔11 transition tables; MEDIS-DC + 国際分類情報管理室 produce Japan-localized crosswalks (ongoing).

### Expressiveness/Semantics
ICD-11 post-coordination via stem + extension codes (severity, anatomy, temporality, etiology) — far richer than ICD-10. Foundation is OWL-aligned; linearizations flat. ICD-10 JP purely hierarchical.

### Composability/Modularity
ICD-11 post-coordination compositional; ICD-10 lacks it. ICD-11 has SNOMED CT cross-walks at foundation layer.

### Suitability for autoformalization to IR
Critical disease ontology layer. ICD-11 foundation's IRI-addressable concepts are LLM-friendly anchors. ICD-10 JP suffices for current claims data but limits semantic richness. Convergence concern: society guideline disease wording rarely matches ICD codes verbatim — MEDIS病名 disambiguation needed.

### Formal verification potential
ICD-11 foundation OWL/RDF enables DL-reasoning (subsumption, disjointness); ICD-10 JP code-equality only. Cross-guideline contradiction detection more tractable with ICD-11 codes.

### Tooling/Ecosystem maturity
WHO ICD-11 Browser, Coding Tool, REST/JSON API; ICD-10 JP via MHLW告示 + MEDIS病名 master. Japanese ICD-11 portal under MHLW preparing 和訳 (2023年1月版 published 2024-07).

### Japan-specific considerations
Statutory via 統計法 §28. ICD-10 JP 2013 mandatory through 2026. 和訳 coordinated with 日本医学会/日本歯科医学会. Free distribution; ICD-11 open under WHO IP policy.

### Interoperability with the others on this list
- MEDIS病名 master: entries carry ICD-10 JP code; future ICD-11 migration.
- DPC: ICD-10 JP in class A/B condition coding.
- JP Core: ValueSet bindings to ICD-10 JP; ICD-11 ValueSets pending.
- OMOP: ICD-10 standard SNOMED mappings via Athena; ICD-11 mappings emerging.
- vs Category 1: disease classifier underlying any disease-keyed CQL valueset or DMN decision table.

### Limitations/Known issues
ICD-10 → ICD-11 not 1:1; many codes split/merge. Japan mapping work in progress. ICD-11 adoption requires system upgrades + training; many systems will run dual-coding 2027–2030+.

### Training data proxy
Strong international (WHO, English PubMed). Japanese: MHLW PDFs, 日本医学会 materials, J-STAGE. LLM abundance: high for ICD-10, moderate-rising for ICD-11.

## 7. DPC/PDPS, K Codes, YJ Codes, Receipt Codes, and Claims Crosswalks

### Purpose
Reimbursement code ecosystem: case-mix grouping (DPC), surgical/procedural coding (K), drug product coding (YJ + 薬価基準), claims encoding (レセプト電算 / 診療報酬コード). Determines reimbursement; backbone of national claims RWD.

### Maintainer/Standards body
MHLW 保険局 (診療報酬改定 every 2 years; latest 2024; next 2026-04). 厚生労働省医政局医薬産業振興・医療情報企画課 (旧経済課) manages 薬価基準コード. 医薬情報研究所 issues YJ. 社会保険診療報酬支払基金 issues レセプト電算用コード. 外科系学会社会保険委員会連合 (外保連) maintains STEM7 surgical taxonomy mapped to K-codes.

### Conceptual model
- DPC/PDPS: 14-digit case-mix code per admission (major diagnostic category + diagnosis + procedure + comorbidity + severity). As of 2024-06-01 (FY2024): 1,786 DPC hospitals, 483,721 DPC算定病床 (MHLW report to 中医協, 中医協 総－７－２, 令和6年4月10日; FY2025 data show decrease to 1,761 hospitals). Per-diem prospective payment.
- K-codes: alphanumeric (e.g., `K764`) hierarchical procedure codes from 医科点数表第10部, tied to point counts. 外保連 STEM7 (7-char: 部位3+操作2+到達法1+補助器械1) clinical taxonomy; 1:N to K-codes; mandatory in DPC data submission since 平成30年度 (2018).
- YJ codes: 12-character per-product codes (薬効4 + 投与経路成分3 + 剤形1 + 規格1 + 同一規格内2 + check1), developed/maintained by 株式会社医薬情報研究所; ≠ 薬価基準収載コード when 統一名収載.
- 薬価基準収載コード (厚労省コード): 12-char; tariff-line granularity.
- レセプト電算用コード: 9-digit; electronic claims via 支払基金.
- HOT基準番号: 13-char unifying anchor across the four above (MEDIS).
- 一般名コード: MHLW-issued for generic prescription.
- Class A/B condition codes: 7-digit local + ICD-10 in DPC claims; B1–B3 mandatory.

### Expressiveness/Semantics
Strictly classificatory/economic: encodes what was billed, not clinical detail. K-codes lack systematic surgical taxonomy (hence STEM7).

### Composability/Modularity
HOT = drug crosswalk hub. K↔STEM7 official table (`KコードSTEM7対応表 2024`). DPC composite built from constituent diagnosis/procedure codes per MHLW grouping logic.

### Suitability for autoformalization to IR
Useful for retrospective evaluation/quality-measure binding (CQL Measure-like). Not a primary IR concept axis: reimbursement-driven granularity bias (e.g., 病名 padding for billing). Evaluation substrate only.

### Formal verification potential
Coding rules deterministic → encodable as DMN-style decision tables for billing validation. Clinical-semantic verification limited.

### Tooling/Ecosystem maturity
MHLW 診療報酬情報提供サービス (`shinryohoshu.mhlw.go.jp`); 支払基金 master downloads; commercial DPC tools (MDV, JMDC); HOT mapping by MEDIS-DC.

### Japan-specific considerations
Revised every 2 years on April 1. Masters freely downloadable (MHLW/支払基金); Japanese CSV/Excel. MHLW designates YJ + レセプト電算 + 一般名コード as standard codes for 電子処方箋/電子カルテ情報共有サービス (2024 HELICS adoption process).

### Interoperability with the others on this list
- MEDIS HOT: hub linking all drug codes.
- ICD-10 JP: in DPC class B condition codes.
- SS-MIX2: claims-side data may be stored alongside.
- JP Core: MedicationRequest uses YJ; Procedure uses K-codes.
- MID-NET: uses YJ + HOT + ICD-10 + JLAC10.
- OMOP: Usagi mapping; JMDC drug code is a recognized OHDSI source vocabulary.
- vs Category 1: usable in CQL valuesets and DMN tables; relevant for ePath pathway resource-utilization checks.

### Limitations/Known issues
Reimbursement-biased coding harms diagnostic validity (PPV studies show variable accuracy). 統一名収載 vs 銘柄別収載 complicates drug-product identity. K-code granularity inconsistent. 2-year revision cycle creates temporal versioning challenges.

### Training data proxy
Strong Japanese documentation: vendor blogs, 外保連 PDFs, J-STAGE validation studies. LLM data moderate-high.

## 8. MedDRA/J and JADER for Adverse-Event Knowledge

### Purpose
MedDRA/J: bilingual (English-Japanese) hierarchical adverse-event terminology for pharmacovigilance, regulatory reporting, post-marketing surveillance. JADER: PMDA spontaneous adverse drug event report database, MedDRA-coded.

### Maintainer/Standards body
- MedDRA: ICH MedDRA Management Committee → MSSO (international) + JMO (日本メンテナンス機関, hosted at PMRJ) for Japanese version. Current: v29.0 (English release 2026-03-01, complex release with potential changes at all five hierarchy levels; multilingual 2026-03-15; MedDRA/J Japanese synonym version 2026-04-01). v28.1 (2025-09-01): simple change release, PT/LLT-level only; v28.0 (2025-03-01): complex. Updates twice yearly (1 March / 1 September).
- JADER: PMDA; public since April 2012 (data back to 2004-04); CSV downloads on `pmda.go.jp`.

### Conceptual model
- MedDRA/J: 5-level hierarchy — SOC (System Organ Class, 27) → HLGT → HLT → PT (Preferred Term) → LLT (Lowest Level Term); plus SMQ (Standardized MedDRA Queries) for cross-PT grouping. Each English term has unique Japanese counterpart at SOC–LLT; same Japanese term sometimes maps to different English LLTs (currency-flag mechanism preserves data consistency).
- JADER: 4 relational tables — DEMO (demographics), DRUG (drug name, route, dates, suspect/concomitant role), REAC (AE PT + outcome), HIST (primary illness); linked via case ID.

### Expressiveness/Semantics
MedDRA: standardized AE lexicon, not an ontology (no formal subsumption beyond hierarchy levels); SMQ = curated grouping (narrow/broad). JADER: spontaneous-report semantics — voluntary, biased toward serious/novel events, no denominator.

### Composability/Modularity
Hierarchy roll-ups (PT → HLT → SOC); SMQs reusable cross-cutting queries. JADER tables joinable; de-duplication required (multiple reports per case).

### Suitability for autoformalization to IR
MedDRA/J PTs = canonical AE vocabulary for the IR; SMQs ≈ predefined CQL valuesets. JADER feeds empirical AE-frequency priors; not knowledge per se.

### Formal verification potential
Code-table consistency only. Drug-AE causal claims require statistical signal detection (ROR, IC, BCPNN), not formal verification.

### Tooling/Ecosystem maturity
JMO portal (`jmo.pmrj.jp`): Web MedDRA/J Browser, ASCII files, Japanese synonym file (~1 month after each MedDRA version). Open-source R/Python signal-detection libraries work with JADER; PMDA tools.

### Japan-specific considerations
MedDRA/J subscription via JMO for Japan-HQ companies, via MSSO otherwise; free for regulators/academia/non-profit. JADER freely downloadable. Japanese docs on `pmrj.jp`; English on `meddra.org`.

### Interoperability with the others on this list
- e-PI 副作用 section: MedDRA/J PT-coded.
- JP Core: `AllergyIntolerance`, `AdverseEvent` bind MedDRA/J PT.
- OMOP CDM: MedDRA in OHDSI standard vocabulary.
- ICD-10/11: separate; ICD codes not used in JADER.
- vs Category 1: AE knowledge for CQL AE valuesets, DMN rule guards, CDS Hooks `patient-view`/`order-sign` warnings.

### Limitations/Known issues
Same-Japanese-term-multiple-English-LLT phenomenon complicates back-translation. JADER biases: under-reporting, Weber effect, no denominators, notoriety bias (regulator-action artifacts). Duplicate cases need de-duplication. Causality not adjudicated.

### Training data proxy
Strong international + Japanese coverage: JMO PDFs, abundant J-STAGE/PMC JADER disproportionality papers, GitHub JADER-cleaning repos. LLM data high.

## 9. OMOP CDM / OHDSI Japan Vocabulary Mapping

### Purpose
International common data model (Observational Medical Outcomes Partnership) for federated observational research; OHDSI-Japan maps Japanese vocabularies into OMOP standard concepts (SNOMED, RxNorm, LOINC) for cross-national network studies.

### Maintainer/Standards body
OHDSI consortium; Japan chapter (academic + industry; hospitals incl. 国立がん研究センター東病院, 愛媛大学; JMDC). CDM v5.4 = latest supported release (v6.0 published but unsupported by OHDSI tools/methods; v5.4 recommended production version as of 2026; CDM WG planning new 5-series release for 2026). Vocabulary refreshed via Athena.

### Conceptual model
Relational tables: `Person`, `Observation_Period`, `Visit_Occurrence`, `Condition_Occurrence`, `Drug_Exposure`, `Procedure_Occurrence`, `Measurement`, `Observation`, `Death`, `Note`, `Note_NLP`, `Specimen`, etc. Clinical concepts mapped to Standard Concept IDs via OHDSI vocabulary; source codes preserved in `*_source_concept_id`.

### Expressiveness/Semantics
Longitudinal patient events at concept granularity; temporal cohort definitions, drug exposure eras, condition eras. Limited unstructured/narrative support (Note + Note_NLP). Harmonization via SNOMED CT (conditions), RxNorm (drugs), LOINC (labs).

### Composability/Modularity
ATLAS UI + Hades R packages + Strategus pipelines: modular study packages (characterization, population-level estimation, patient-level prediction). Cohort definitions = reusable JSON artifacts.

### Suitability for autoformalization to IR
OMOP vocabulary = global concept-ID namespace, ideal cross-guideline reference axis. ATLAS Cohort JSON / OHDSI Phenotype Library are partially executable specs complementing the IR. Targets: cohort definitions, exposure/outcome valuesets, computable phenotypes. Convergence: stable concept_id semantics across releases.

### Formal verification potential
ATLAS cohort JSON → SQL deterministically. Cohort-overlap + concept-set consistency tooled (Cohort Diagnostics). Not theorem-prover-grade; supports reproducible audit.

### Tooling/Ecosystem maturity
ATLAS, Achilles, DataQualityDashboard, Hades, Usagi (namesake "rabbit" tool originally for Japanese-source mapping), Strategus, Eunomia, Ares. Active community.

### Japan-specific considerations
JP RWD vendors (JMDC, Medical Data Vision, RWD Co.) provide OMOP-mapped extracts. Academic conversions: 国立がん研究センター東病院 — Aoyagi Y, Baba M, Terao S, Ikeda Y, Nomura K, Sato A, "Feasibility of Converting EMR Data to OMOP CDM and Utilizing OHDSI Analysis Tools in Japan," Stud Health Technol Inform 2025;329:1946–1947, DOI 10.3233/SHTI251292, PMID 40776309 (breast cancer cohort 8,387 patients; 7,259 terms mapped to OHDSI standard vocabulary); 愛媛大学 — Kimura E, Kawakami Y, Inoue S, Okajima A, "A dataset for mapping the Japanese drugs to RxNorm standard concepts," Data in Brief 2025, DOI 10.1016/j.dib.2025.111418, PMC11926709 (LLM-assisted YJ→RxNorm mappings using Mixtral 8×7B with BioBERT-RAG). OHDSI vocabulary additions for Japan include JMDC drug code. JLAC10/11→LOINC mapping non-trivial.

### Interoperability with the others on this list
- MEDIS HOT/YJ → RxNorm: Ehime University 2025 dataset (Kimura et al., Data in Brief 10.1016/j.dib.2025.111418), LLM-assisted.
- JLAC10 → LOINC: incomplete; community effort.
- ICD-10 JP → SNOMED CT: Athena standard mappings (Japan-specific edge cases).
- SS-MIX2 → OMOP: ETL pipelines published (medRxiv 2025-06).
- JP Core ↔ OMOP: bidirectional adapters (FHIR-to-OMOP, OMOP-on-FHIR).
- vs Category 1: OMOP cohort JSON ≈ openEHR AQL ≈ FHIR CQL — three parallel computable-phenotype representations; the IR may target one canonical form and translate.

### Limitations/Known issues
Japanese-vocabulary mapping labor-intensive; coverage gaps in drugs (combination products, Japan-only generics) and labs (JLAC10 method specificity > LOINC). SS-MIX2 → OMOP ETL documented loss for nursing/narrative data. Race/ethnicity fields underspecified for Japan. Concept_id stability requires vocab version pinning.

### Training data proxy
Strong English (Book of OHDSI, ohdsi.org). Japanese: J-STAGE papers, OHDSI Japan symposium materials, growing Qiita/Zenn. GitHub: many ATLAS/Hades repos. LLM data: high for OHDSI core, moderate for Japan mappings.

## 10. MID-NET, NDB Open Data, and Real-World-Evidence Interfaces

### Purpose
National RWE/RWD platforms for pharmacoepidemiology, post-marketing safety, policy analytics. MID-NET = EHR-centric distributed federated network (PMDA-operated). NDB = nationwide claims + 特定健診 database (MHLW). NDB Open Data = aggregated public release.

### Maintainer/Standards body
- MID-NET: PMDA 医療情報科学部 (現 医療情報活用部); operational since 2018-04. Cooperative institutions reduced 10 拠点 → 9 拠点 (31 hospitals) effective 2024-12-01 after NTT Hospital Group withdrawal (PMDA "MID-NET® Update 2024" presentation, 医療情報科学部長 山口光峰, 2024-11-28, https://www.pmda.go.jp/files/000272301.pdf). Population "800万人超（2023年12月末時点）" (over 8 million, end of December 2023, same deck). Expansion of 10 Tokushukai Group hospitals (Tokushukai total: 20) began April 2024. December 2025 server expansion procurement.
- NDB: MHLW 保険局; launched 2009; NDB Open Data since 2016; 2020 legislation permits provision to private companies. "The NDB covers approximately 98% of data on healthcare services provided by medical institutions" (Yasunaga H., "Updated Information on NDB," Ann Clin Epidemiol 2024;6(3):73–76, DOI 10.37737/ace.24011, PMC11254583).

### Conceptual model
- MID-NET: distributed closed network; data centrally normalized to a MID-NET common data model populated from SS-MIX2 standardized storage at each site, standardized code mappings (ICD-10, YJ, HOT, JLAC10, MERIT-9, JAMI usage standard). On-site analysis center (PMDA, 新霞が関ビル) + MID-NET接続環境 remote access; user extraction programs require site approval; only aggregated results returned (no individual-level download). Per Yamaguchi M et al. 2019 (Pharmacoepidemiol Drug Saf 2019;28(10):1395–1404, DOI 10.1002/pds.4879, PMC6851601): "MID-NET® adopts a common data model that stores a wide variety of hospital information system (HIS) data… standardized based on the message specifications of SS-MIX2." Conceptually inspired by FDA Sentinel; distinct from OMOP CDM.
- NDB: centralized claims (医科 / 歯科 / 調剤 / DPC) + 特定健診; anonymized hash IDs. Four products: (1) General Data (special extraction + Onsite Research Center), (2) Sampling Data, (3) Accumulated Data, (4) Open Data (aggregated tables, MHLW website).

### Expressiveness/Semantics
- MID-NET: clinical lab results (with units), prescriptions, diagnoses, vitals — richer than claims; suits biomarker-defined cohorts.
- NDB: comprehensive claims but no lab values (except 特定健診 checkup data); diagnosis validity limited by reimbursement coding artifacts.

### Composability/Modularity
- MID-NET: distributed query, centrally aggregated results; programs reusable across sites.
- NDB: monolithic; standardized record layouts. Linkage to other national DBs (DPC, infectious disease surveillance, designated incurable disease, specific chronic pediatric disease) planned (2024-).

### Suitability for autoformalization to IR
Both = evaluation/validation substrates, not knowledge sources. MID-NET's normalized SS-MIX2-derived schema is closer to a CDS-ready longitudinal patient model. NDB Open Data aggregates support epidemiological priors. Idempotency benefits from MID-NET's enforced code standardization.

### Formal verification potential
None native. Empirical test bed: back-validate CDS IR recommendations against MID-NET/NDB cohorts to detect quantitative implausibility.

### Tooling/Ecosystem maturity
- MID-NET: PMDA SAS environment, on-site center, remote access (MID-NET接続環境). New MID-NET Guideline July 2024 changed operational procedures; on-site center relocated within 新霞が関ビル (20F→6F, completed December 2025). Year-round application since FY2024.
- NDB: MHLW Onsite Research Center; Open Data on MHLW website. Healthcare Intelligence Cloud (HIC) platform under development. JMDC, DPC databases = parallel commercial options.

### Japan-specific considerations
- MID-NET: pharmaceutical companies, researchers, PMDA all eligible; year-round application (since FY2024). Fees not differentiated by user type (industry vs academia); three categories by analysis type, calibrated to recover ~¥1.234 billion annual operating cost (MHLW WG document, https://www.mhlw.go.jp/file/05-Shingikai-11121000-Iyakushokuhinkyoku-Soumuka/0000171825.pdf). Compliant with GPSP省令 reliability standards.
- NDB: restricted access (originally national/local government, universities, quasi-public; broadened 2020 to private companies via formal application + advisory committee approval). Open Data freely downloadable.
- Both Japanese-language operationally.

### Interoperability with the others on this list
- SS-MIX2 → MID-NET: required ingestion format.
- MEDIS masters: MID-NET enforces JLAC10, HOT, YJ, ICD-10 across sites.
- DPC/レセプト: NDB primary source.
- JP Core: FHIR endpoints over MID-NET/NDB are research-stage.
- OMOP: NDB/JMDC OMOP conversions exist commercially; MID-NET native model is not OMOP.
- MedDRA/J: not natively in MID-NET (EHR-derived AE detection via lab/diagnosis signals).
- vs Category 1: verification/evaluation substrate; CQL Measure / OMOP cohort definitions / openEHR AQL = natural query forms; ePath outcomes benchmarkable against NDB.

### Limitations/Known issues
- MID-NET: small national share; selection bias (academic/specialty hospitals); on-site analysis only, no individual-level export; SAS/closed-env learning curve; historically variable cross-site mapping consistency.
- NDB: claims-only (no lab values), variable diagnosis PPV, no facility identifiers, complex application process, hashed IDs imperfect for longitudinal linkage, no mortality linkage.

### Training data proxy
Strong Japanese: PMDA `mid-net` portal, MHLW NDB pages, J-STAGE/PMC reviews (Sato & Yasunaga 2023, Yasunaga 2024, Suto et al. 2024). English: Yamaguchi et al. 2019 (Pharmacoepidemiol Drug Saf 28:1395–1404), Suto et al. 2024 (JMA Journal — NDB literature review). GitHub sparse (restricted environments). LLM data: moderate.
