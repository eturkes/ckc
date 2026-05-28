//! Integration test: terminology concepts and variant fixtures for SPEC 20 Phase 0 scenario 2.
//!
//! Constructs 10 Concepts with TerminologyBindings for toy scenarios:
//!   2. e-graph variant convergence — 5 beta-lactam surface-form variants share egraph_class_id
//!   1, 5, 7. norm conflict, repair, proof — sepsis and anaphylaxis concepts
//!   3, 4. decision table, Event Calculus — vital-sign concepts (体温, 心拍数, 収縮期血圧)
//!
//! All concepts reference source spans from 0.5.1 (toy_source_corpus).

use ckc_core::canonical::{ContentHash, content_hash, to_canonical_bytes};
use ckc_core::enums::*;
use ckc_core::id::*;
use ckc_core::nf::{NfContext, Normalize};
use ckc_core::source::*;
use std::collections::HashSet;
use std::path::Path;

// =========================================================================
// ID constants
// =========================================================================

const CONCEPT_BETA_LACTAM: &str = "concept_beta_lactam";
const CONCEPT_BL_KATAKANA: &str = "concept_bl_variant_katakana";
const CONCEPT_BL_HYPHENATED: &str = "concept_bl_variant_hyphenated";
const CONCEPT_BL_ENGLISH: &str = "concept_bl_variant_english";
const CONCEPT_BL_BRAND: &str = "concept_bl_variant_brand";
const CONCEPT_SEPSIS: &str = "concept_sepsis";
const CONCEPT_ANAPHYLAXIS: &str = "concept_anaphylaxis";
const CONCEPT_BODY_TEMP: &str = "concept_body_temperature";
const CONCEPT_HEART_RATE: &str = "concept_heart_rate";
const CONCEPT_BLOOD_PRESSURE: &str = "concept_blood_pressure";

const ECLASS_BETA_LACTAM: &str = "eclass_beta_lactam";
const ECLASS_SEPSIS: &str = "eclass_sepsis";
const ECLASS_ANAPHYLAXIS: &str = "eclass_anaphylaxis";
const ECLASS_BODY_TEMP: &str = "eclass_body_temperature";
const ECLASS_HEART_RATE: &str = "eclass_heart_rate";
const ECLASS_BLOOD_PRESSURE: &str = "eclass_blood_pressure";

// Span IDs from 0.5.1
const SPAN_REC_SEPSIS: &str = "span_rec_sepsis_bl";
const SPAN_EVIDENCE: &str = "span_evidence_sepsis";
const SPAN_TERM_BL_GREEK: &str = "span_term_bl_greek";
const SPAN_TERM_BL_KATAKANA: &str = "span_term_bl_katakana";
const SPAN_CONTRA: &str = "span_contra_bl_allergy";
const SPAN_ALLERGY_HIST: &str = "span_allergy_history";
const SPAN_CELL_R0C0: &str = "span_cell_r0c0";
const SPAN_CELL_R1C0: &str = "span_cell_r1c0";
const SPAN_CELL_R2C0: &str = "span_cell_r2c0";
const SPAN_CELL_R3C0: &str = "span_cell_r3c0";

// =========================================================================
// Fixture constructors
// =========================================================================

fn toy_concepts() -> Vec<Concept> {
    vec![
        // Primary beta-lactam concept (referenced by 0.5.2 rules as target_concept)
        Concept {
            concept_id: ConceptId::new(CONCEPT_BETA_LACTAM),
            label_ja: "βラクタム系抗菌薬".into(),
            label_en: Some("beta-lactam antibiotics".into()),
            semantic_type: "drug_class".into(),
            terminology_bindings: vec![
                TerminologyBinding {
                    system: "MEDIS".into(),
                    code: Some("Y0100".into()),
                    version: Some("2024".into()),
                    label: "βラクタム系抗菌薬".into(),
                    status: BindingStatus::Exact,
                    mapping_relation: "equivalent".into(),
                    provenance: "medis_drug_master_2024".into(),
                    confidence: 0.98,
                    license_status: "permitted".into(),
                    valid_from: Some("2024-01-01".into()),
                    valid_to: None,
                },
                TerminologyBinding {
                    system: "HOT".into(),
                    code: Some("1190100".into()),
                    version: Some("202403".into()),
                    label: "βラクタム系抗生物質製剤".into(),
                    status: BindingStatus::Broad,
                    mapping_relation: "broader".into(),
                    provenance: "hot_code_master_202403".into(),
                    confidence: 0.90,
                    license_status: "permitted".into(),
                    valid_from: Some("2024-03-01".into()),
                    valid_to: None,
                },
                TerminologyBinding {
                    system: "ICD-10".into(),
                    code: Some("Y40.1".into()),
                    version: Some("2019".into()),
                    label: "Beta-lactam antibiotics, penicillins".into(),
                    status: BindingStatus::Broad,
                    mapping_relation: "broader".into(),
                    provenance: "icd10_2019_ch20".into(),
                    confidence: 0.85,
                    license_status: "permitted".into(),
                    valid_from: Some("2019-01-01".into()),
                    valid_to: None,
                },
            ],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_BETA_LACTAM)),
            source_span_ids: vec![
                SpanId::new(SPAN_REC_SEPSIS),
                SpanId::new(SPAN_TERM_BL_GREEK),
            ],
        },
        // Variant: katakana form (ベータラクタム)
        Concept {
            concept_id: ConceptId::new(CONCEPT_BL_KATAKANA),
            label_ja: "ベータラクタム系薬剤".into(),
            label_en: Some("beta-lactam agents".into()),
            semantic_type: "drug_class".into(),
            terminology_bindings: vec![TerminologyBinding {
                system: "MEDIS".into(),
                code: Some("Y0100".into()),
                version: Some("2024".into()),
                label: "ベータラクタム系薬剤".into(),
                status: BindingStatus::Exact,
                mapping_relation: "equivalent".into(),
                provenance: "medis_drug_master_2024".into(),
                confidence: 0.95,
                license_status: "permitted".into(),
                valid_from: Some("2024-01-01".into()),
                valid_to: None,
            }],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_BETA_LACTAM)),
            source_span_ids: vec![SpanId::new(SPAN_TERM_BL_KATAKANA)],
        },
        // Variant: hyphenated Greek form (β-ラクタム)
        Concept {
            concept_id: ConceptId::new(CONCEPT_BL_HYPHENATED),
            label_ja: "β-ラクタム系抗菌薬".into(),
            label_en: Some("β-lactam antibiotics".into()),
            semantic_type: "drug_class".into(),
            terminology_bindings: vec![TerminologyBinding {
                system: "MEDIS".into(),
                code: Some("Y0100".into()),
                version: Some("2024".into()),
                label: "β-ラクタム系抗菌薬".into(),
                status: BindingStatus::Exact,
                mapping_relation: "equivalent".into(),
                provenance: "medis_drug_master_2024".into(),
                confidence: 0.95,
                license_status: "permitted".into(),
                valid_from: Some("2024-01-01".into()),
                valid_to: None,
            }],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_BETA_LACTAM)),
            source_span_ids: vec![SpanId::new(SPAN_ALLERGY_HIST)],
        },
        // Variant: English form
        Concept {
            concept_id: ConceptId::new(CONCEPT_BL_ENGLISH),
            label_ja: "ベータラクタム".into(),
            label_en: Some("beta-lactam".into()),
            semantic_type: "drug_class".into(),
            terminology_bindings: vec![TerminologyBinding {
                system: "ICD-10".into(),
                code: Some("Y40.1".into()),
                version: Some("2019".into()),
                label: "Beta-lactam antibiotics, penicillins".into(),
                status: BindingStatus::Broad,
                mapping_relation: "broader".into(),
                provenance: "icd10_2019_ch20".into(),
                confidence: 0.85,
                license_status: "permitted".into(),
                valid_from: Some("2019-01-01".into()),
                valid_to: None,
            }],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_BETA_LACTAM)),
            source_span_ids: vec![SpanId::new(SPAN_TERM_BL_GREEK)],
        },
        // Variant: brand name (ceftriaxone, a 3rd-gen cephalosporin beta-lactam)
        Concept {
            concept_id: ConceptId::new(CONCEPT_BL_BRAND),
            label_ja: "セフトリアキソン".into(),
            label_en: Some("ceftriaxone".into()),
            semantic_type: "drug_class".into(),
            terminology_bindings: vec![
                TerminologyBinding {
                    system: "HOT".into(),
                    code: Some("1149028".into()),
                    version: Some("202403".into()),
                    label: "セフトリアキソンナトリウム".into(),
                    status: BindingStatus::Exact,
                    mapping_relation: "equivalent".into(),
                    provenance: "hot_code_master_202403".into(),
                    confidence: 0.98,
                    license_status: "permitted".into(),
                    valid_from: Some("2024-03-01".into()),
                    valid_to: None,
                },
                TerminologyBinding {
                    system: "MEDIS".into(),
                    code: Some("Y0100".into()),
                    version: Some("2024".into()),
                    label: "βラクタム系抗菌薬".into(),
                    status: BindingStatus::Narrow,
                    mapping_relation: "narrower".into(),
                    provenance: "medis_drug_master_2024".into(),
                    confidence: 0.92,
                    license_status: "permitted".into(),
                    valid_from: Some("2024-01-01".into()),
                    valid_to: None,
                },
            ],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_BETA_LACTAM)),
            source_span_ids: vec![SpanId::new(SPAN_REC_SEPSIS)],
        },
        // Sepsis
        Concept {
            concept_id: ConceptId::new(CONCEPT_SEPSIS),
            label_ja: "敗血症".into(),
            label_en: Some("sepsis".into()),
            semantic_type: "diagnosis".into(),
            terminology_bindings: vec![
                TerminologyBinding {
                    system: "MEDIS".into(),
                    code: Some("BQEF".into()),
                    version: Some("2024".into()),
                    label: "敗血症".into(),
                    status: BindingStatus::Exact,
                    mapping_relation: "equivalent".into(),
                    provenance: "medis_disease_master_2024".into(),
                    confidence: 0.99,
                    license_status: "permitted".into(),
                    valid_from: Some("2024-01-01".into()),
                    valid_to: None,
                },
                TerminologyBinding {
                    system: "ICD-10".into(),
                    code: Some("A41.9".into()),
                    version: Some("2019".into()),
                    label: "Sepsis, unspecified organism".into(),
                    status: BindingStatus::Exact,
                    mapping_relation: "equivalent".into(),
                    provenance: "icd10_2019_ch1".into(),
                    confidence: 0.95,
                    license_status: "permitted".into(),
                    valid_from: Some("2019-01-01".into()),
                    valid_to: None,
                },
            ],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_SEPSIS)),
            source_span_ids: vec![SpanId::new(SPAN_REC_SEPSIS), SpanId::new(SPAN_EVIDENCE)],
        },
        // Anaphylaxis
        Concept {
            concept_id: ConceptId::new(CONCEPT_ANAPHYLAXIS),
            label_ja: "アナフィラキシー".into(),
            label_en: Some("anaphylaxis".into()),
            semantic_type: "adverse_event".into(),
            terminology_bindings: vec![
                TerminologyBinding {
                    system: "MedDRA/J".into(),
                    code: Some("10002198".into()),
                    version: Some("26.1".into()),
                    label: "アナフィラキシー反応".into(),
                    status: BindingStatus::Exact,
                    mapping_relation: "equivalent".into(),
                    provenance: "meddra_j_v26.1".into(),
                    confidence: 0.97,
                    license_status: "restricted".into(),
                    valid_from: Some("2023-09-01".into()),
                    valid_to: None,
                },
                TerminologyBinding {
                    system: "ICD-10".into(),
                    code: Some("T78.2".into()),
                    version: Some("2019".into()),
                    label: "Anaphylactic shock, unspecified".into(),
                    status: BindingStatus::Related,
                    mapping_relation: "related".into(),
                    provenance: "icd10_2019_ch19".into(),
                    confidence: 0.80,
                    license_status: "permitted".into(),
                    valid_from: Some("2019-01-01".into()),
                    valid_to: None,
                },
            ],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_ANAPHYLAXIS)),
            source_span_ids: vec![SpanId::new(SPAN_CONTRA), SpanId::new(SPAN_ALLERGY_HIST)],
        },
        // Body temperature
        Concept {
            concept_id: ConceptId::new(CONCEPT_BODY_TEMP),
            label_ja: "体温".into(),
            label_en: Some("body temperature".into()),
            semantic_type: "vital_sign".into(),
            terminology_bindings: vec![TerminologyBinding {
                system: "JLAC11".into(),
                code: Some("9N611000000000001".into()),
                version: Some("2023".into()),
                label: "体温".into(),
                status: BindingStatus::Exact,
                mapping_relation: "equivalent".into(),
                provenance: "jlac11_vitals_2023".into(),
                confidence: 0.99,
                license_status: "permitted".into(),
                valid_from: Some("2023-01-01".into()),
                valid_to: None,
            }],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_BODY_TEMP)),
            source_span_ids: vec![SpanId::new(SPAN_CELL_R0C0), SpanId::new(SPAN_CELL_R1C0)],
        },
        // Heart rate
        Concept {
            concept_id: ConceptId::new(CONCEPT_HEART_RATE),
            label_ja: "心拍数".into(),
            label_en: Some("heart rate".into()),
            semantic_type: "vital_sign".into(),
            terminology_bindings: vec![TerminologyBinding {
                system: "JLAC11".into(),
                code: Some("9A751000000000001".into()),
                version: Some("2023".into()),
                label: "心拍数".into(),
                status: BindingStatus::Exact,
                mapping_relation: "equivalent".into(),
                provenance: "jlac11_vitals_2023".into(),
                confidence: 0.99,
                license_status: "permitted".into(),
                valid_from: Some("2023-01-01".into()),
                valid_to: None,
            }],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_HEART_RATE)),
            source_span_ids: vec![SpanId::new(SPAN_CELL_R2C0)],
        },
        // Blood pressure (systolic)
        Concept {
            concept_id: ConceptId::new(CONCEPT_BLOOD_PRESSURE),
            label_ja: "収縮期血圧".into(),
            label_en: Some("systolic blood pressure".into()),
            semantic_type: "vital_sign".into(),
            terminology_bindings: vec![TerminologyBinding {
                system: "JLAC11".into(),
                code: Some("9A051000000000001".into()),
                version: Some("2023".into()),
                label: "収縮期血圧".into(),
                status: BindingStatus::Exact,
                mapping_relation: "equivalent".into(),
                provenance: "jlac11_vitals_2023".into(),
                confidence: 0.99,
                license_status: "permitted".into(),
                valid_from: Some("2023-01-01".into()),
                valid_to: None,
            }],
            egraph_class_id: Some(EGraphClassId::new(ECLASS_BLOOD_PRESSURE)),
            source_span_ids: vec![SpanId::new(SPAN_CELL_R3C0)],
        },
    ]
}

/// Extracts the 5 beta-lactam variant concepts.
fn beta_lactam_variants() -> Vec<Concept> {
    toy_concepts().into_iter().take(5).collect()
}

// =========================================================================
// Fixture directory helpers
// =========================================================================

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/research_kernel/fixtures")
}

fn load_span_ids_from_fixtures() -> HashSet<String> {
    let dir = fixtures_dir();
    let bytes = std::fs::read(dir.join("spans.json"))
        .expect("0.5.1 spans.json must exist; run toy_source_corpus regen_fixtures first");
    let spans: Vec<serde_json::Value> =
        serde_json::from_slice(&bytes).expect("spans.json must deserialize");
    spans
        .iter()
        .filter_map(|s| s.get("span_id").and_then(|v| v.as_str()))
        .map(String::from)
        .collect()
}

// =========================================================================
// E-graph variant convergence (scenario 2)
// =========================================================================

#[test]
fn all_beta_lactam_variants_share_egraph_class() {
    let variants = beta_lactam_variants();
    assert_eq!(
        variants.len(),
        5,
        "fixture must have 5 beta-lactam variants"
    );

    let class_ids: HashSet<&str> = variants
        .iter()
        .map(|c| {
            c.egraph_class_id
                .as_ref()
                .expect("every variant must have egraph_class_id")
                .as_str()
        })
        .collect();
    assert_eq!(
        class_ids.len(),
        1,
        "all beta-lactam variants must share one egraph_class_id; got {:?}",
        class_ids
    );
    assert!(class_ids.contains(ECLASS_BETA_LACTAM));
}

#[test]
fn beta_lactam_variants_share_semantic_type() {
    let variants = beta_lactam_variants();
    let types: HashSet<&str> = variants.iter().map(|c| c.semantic_type.as_str()).collect();
    assert_eq!(
        types.len(),
        1,
        "all beta-lactam variants must share semantic_type; got {:?}",
        types
    );
}

#[test]
fn beta_lactam_variants_have_distinct_labels() {
    let variants = beta_lactam_variants();
    let labels: HashSet<&str> = variants.iter().map(|c| c.label_ja.as_str()).collect();
    assert_eq!(
        labels.len(),
        variants.len(),
        "each variant must have a distinct label_ja"
    );
}

#[test]
fn variant_count_sufficient_for_egraph_convergence() {
    let variants = beta_lactam_variants();
    assert!(
        variants.len() >= 5,
        "roadmap requires 4 named variants + 1 brand = 5 minimum"
    );
    let has_brand = variants
        .iter()
        .any(|c| c.concept_id.as_str() == CONCEPT_BL_BRAND);
    assert!(has_brand, "variant set must include a brand-name variant");
}

#[test]
fn brand_variant_has_narrow_binding() {
    let concepts = toy_concepts();
    let brand = concepts
        .iter()
        .find(|c| c.concept_id.as_str() == CONCEPT_BL_BRAND)
        .expect("brand variant must exist");
    let has_narrow = brand
        .terminology_bindings
        .iter()
        .any(|b| b.status == BindingStatus::Narrow);
    assert!(
        has_narrow,
        "brand variant (specific drug) must have a Narrow binding to class"
    );
}

// =========================================================================
// Non-beta-lactam concepts
// =========================================================================

#[test]
fn non_variant_concepts_have_distinct_egraph_classes() {
    let concepts = toy_concepts();
    let non_bl: Vec<&Concept> = concepts
        .iter()
        .filter(|c| {
            c.egraph_class_id
                .as_ref()
                .is_none_or(|e| e.as_str() != ECLASS_BETA_LACTAM)
        })
        .collect();
    let class_ids: HashSet<&str> = non_bl
        .iter()
        .filter_map(|c| c.egraph_class_id.as_ref().map(|e| e.as_str()))
        .collect();
    assert_eq!(
        class_ids.len(),
        non_bl.len(),
        "each non-variant concept must have a unique egraph_class_id"
    );
}

#[test]
fn vital_sign_concepts_reference_table_cells() {
    let concepts = toy_concepts();
    let vitals: Vec<&Concept> = concepts
        .iter()
        .filter(|c| c.semantic_type == "vital_sign")
        .collect();
    assert_eq!(vitals.len(), 3, "fixture must have 3 vital-sign concepts");
    for concept in &vitals {
        assert!(
            concept
                .source_span_ids
                .iter()
                .any(|s| s.as_str().starts_with("span_cell_")),
            "vital-sign concept {} must reference table cell spans",
            concept.concept_id
        );
    }
}

// =========================================================================
// Referential consistency with 0.5.1 spans
// =========================================================================

#[test]
fn all_concepts_reference_existing_spans() {
    let valid_spans = load_span_ids_from_fixtures();
    for concept in &toy_concepts() {
        assert!(
            !concept.source_span_ids.is_empty(),
            "concept {} must reference at least one source span",
            concept.concept_id
        );
        for span_id in &concept.source_span_ids {
            assert!(
                valid_spans.contains(span_id.as_str()),
                "concept {} references unknown span_id {}",
                concept.concept_id,
                span_id
            );
        }
    }
}

// =========================================================================
// Binding validity
// =========================================================================

#[test]
fn all_bindings_have_valid_status() {
    for concept in &toy_concepts() {
        assert!(
            !concept.terminology_bindings.is_empty(),
            "concept {} must have at least one terminology binding",
            concept.concept_id
        );
        for binding in &concept.terminology_bindings {
            let valid = matches!(
                binding.status,
                BindingStatus::Exact
                    | BindingStatus::Broad
                    | BindingStatus::Narrow
                    | BindingStatus::Related
            );
            assert!(
                valid,
                "concept {} binding to {} has non-positive status {:?}; \
                 toy fixtures must use Exact, Broad, Narrow, or Related",
                concept.concept_id, binding.system, binding.status
            );
        }
    }
}

#[test]
fn all_bindings_have_nonempty_provenance() {
    for concept in &toy_concepts() {
        for binding in &concept.terminology_bindings {
            assert!(
                !binding.provenance.is_empty(),
                "concept {} binding to {} must have nonempty provenance",
                concept.concept_id,
                binding.system
            );
        }
    }
}

#[test]
fn binding_confidence_in_valid_range() {
    for concept in &toy_concepts() {
        for binding in &concept.terminology_bindings {
            assert!(
                (0.0..=1.0).contains(&binding.confidence),
                "concept {} binding to {} has confidence {} outside [0, 1]",
                concept.concept_id,
                binding.system,
                binding.confidence
            );
        }
    }
}

// =========================================================================
// Hash determinism
// =========================================================================

#[test]
fn canonical_hashes_deterministic_across_construction() {
    let h1 = content_hash(&toy_concepts());
    let h2 = content_hash(&toy_concepts());
    assert_eq!(h1, h2, "concept fixture hashes must be stable");
}

#[test]
fn individual_concepts_have_distinct_hashes() {
    let concepts = toy_concepts();
    let hashes: Vec<ContentHash> = concepts.iter().map(content_hash).collect();
    let unique: HashSet<&str> = hashes.iter().map(|h| h.as_str()).collect();
    assert_eq!(
        unique.len(),
        hashes.len(),
        "each concept fixture must produce a unique content hash"
    );
}

// =========================================================================
// Unique IDs
// =========================================================================

#[test]
fn all_concept_ids_are_unique() {
    let concepts = toy_concepts();
    let mut seen = HashSet::new();
    for concept in &concepts {
        assert!(
            seen.insert(concept.concept_id.as_str()),
            "duplicate concept_id: {}",
            concept.concept_id
        );
    }
}

#[test]
fn all_egraph_class_ids_are_well_formed() {
    let concepts = toy_concepts();
    for concept in &concepts {
        let eid = concept
            .egraph_class_id
            .as_ref()
            .expect("all toy concepts must have egraph_class_id");
        assert!(
            eid.as_str().starts_with("eclass_"),
            "egraph_class_id {} must start with 'eclass_'",
            eid
        );
    }
}

// =========================================================================
// NF normalization
// =========================================================================

#[test]
fn nf_sorts_source_span_ids() {
    let mut concept = toy_concepts()[0].clone();
    assert_eq!(
        concept.source_span_ids,
        vec![
            SpanId::new(SPAN_REC_SEPSIS),
            SpanId::new(SPAN_TERM_BL_GREEK),
        ],
        "pre-NF spans must be in construction order"
    );
    let mut ctx = NfContext::new();
    concept.normalize(&mut ctx);
    assert_eq!(
        concept.source_span_ids,
        vec![
            SpanId::new(SPAN_REC_SEPSIS),
            SpanId::new(SPAN_TERM_BL_GREEK),
        ],
        "span_rec_sepsis_bl < span_term_bl_greek alphabetically; order preserved"
    );
}

#[test]
fn nf_sorts_anaphylaxis_span_ids() {
    let concepts = toy_concepts();
    let mut concept = concepts
        .iter()
        .find(|c| c.concept_id.as_str() == CONCEPT_ANAPHYLAXIS)
        .unwrap()
        .clone();
    let mut ctx = NfContext::new();
    concept.normalize(&mut ctx);
    // ["span_contra_bl_allergy", "span_allergy_history"] →
    // ["span_allergy_history", "span_contra_bl_allergy"]
    assert_eq!(
        concept.source_span_ids,
        vec![SpanId::new(SPAN_ALLERGY_HIST), SpanId::new(SPAN_CONTRA),],
        "NF must sort anaphylaxis concept source_span_ids"
    );
}

#[test]
fn nf_sorts_terminology_bindings() {
    let mut concept = toy_concepts()[0].clone();
    assert_eq!(
        concept.terminology_bindings.len(),
        3,
        "primary beta-lactam must have 3 bindings"
    );
    let pre_nf_systems: Vec<&str> = concept
        .terminology_bindings
        .iter()
        .map(|b| b.system.as_str())
        .collect();
    assert_eq!(
        pre_nf_systems,
        vec!["MEDIS", "HOT", "ICD-10"],
        "pre-NF ordering must match construction order"
    );

    let mut ctx = NfContext::new();
    concept.normalize(&mut ctx);

    let post_nf_first = concept.terminology_bindings[0].system.as_str();
    let post_nf_last = concept.terminology_bindings.last().unwrap().system.as_str();
    // Bindings sorted by canonical JSON bytes — deterministic
    assert_ne!(
        post_nf_first, post_nf_last,
        "bindings must be sorted deterministically"
    );
}

#[test]
fn nf_assigns_stable_ids() {
    let mut concepts = toy_concepts();
    let mut ctx = NfContext::new();
    for concept in &mut concepts {
        concept.normalize(&mut ctx);
    }
    for concept in &concepts {
        assert!(
            concept.concept_id.as_str().starts_with("nf-"),
            "NF must assign stable ID (nf-...) to concept {}",
            concept.concept_id
        );
    }
}

#[test]
fn nf_stable_ids_are_deterministic() {
    let normalize_and_collect = || {
        let mut concepts = toy_concepts();
        let mut ctx = NfContext::new();
        for concept in &mut concepts {
            concept.normalize(&mut ctx);
        }
        concepts
            .iter()
            .map(|c| c.concept_id.0.clone())
            .collect::<Vec<_>>()
    };
    let ids1 = normalize_and_collect();
    let ids2 = normalize_and_collect();
    assert_eq!(ids1, ids2, "NF stable IDs must be identical across runs");
}

#[test]
fn nf_idempotent_concepts() {
    let mut concepts = toy_concepts();
    let mut ctx1 = NfContext::new();
    for concept in &mut concepts {
        concept.normalize(&mut ctx1);
    }
    let bytes_after_first = to_canonical_bytes(&concepts);

    let mut ctx2 = NfContext::new();
    for concept in &mut concepts {
        concept.normalize(&mut ctx2);
    }
    let bytes_after_second = to_canonical_bytes(&concepts);

    assert_eq!(
        bytes_after_first, bytes_after_second,
        "NF(NF(concepts)) must equal NF(concepts)"
    );
}

#[test]
fn nf_variants_retain_shared_egraph_class() {
    let mut variants = beta_lactam_variants();
    let mut ctx = NfContext::new();
    for v in &mut variants {
        v.normalize(&mut ctx);
    }
    let class_ids: HashSet<&str> = variants
        .iter()
        .map(|c| c.egraph_class_id.as_ref().unwrap().as_str())
        .collect();
    assert_eq!(
        class_ids.len(),
        1,
        "NF must preserve shared egraph_class_id across variants"
    );
}

// =========================================================================
// Cross-reference with 0.5.2 rules
// =========================================================================

#[test]
fn rule_target_concept_exists() {
    let dir = fixtures_dir();
    let rule_bytes = std::fs::read(dir.join("rules.json")).expect("0.5.2 rules.json must exist");
    let rules: Vec<serde_json::Value> =
        serde_json::from_slice(&rule_bytes).expect("rules.json must parse");
    let concept_id_set: HashSet<String> = toy_concepts()
        .iter()
        .map(|c| c.concept_id.0.clone())
        .collect();

    for rule in &rules {
        if let Some(norm) = rule.get("norm")
            && let Some(action) = norm.get("action")
            && let Some(target) = action.get("target_concept")
        {
            let target_str = target.as_str().unwrap();
            assert!(
                concept_id_set.contains(target_str),
                "rule norm action target_concept {} must exist \
                         in terminology fixtures",
                target_str
            );
        }
    }
}

// =========================================================================
// Committed fixture file tests
// =========================================================================

#[test]
fn committed_concepts_match() {
    let path = fixtures_dir().join("concepts.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test toy_terminology regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_concepts());
    assert_eq!(
        bytes, expected,
        "committed concepts.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_concepts_deserialize_correctly() {
    let dir = fixtures_dir();
    let concepts: Vec<Concept> =
        serde_json::from_slice(&std::fs::read(dir.join("concepts.json")).unwrap())
            .expect("concepts.json must deserialize");
    assert_eq!(concepts.len(), 10);
}

// =========================================================================
// Fixture regeneration (run with --ignored)
// =========================================================================

#[test]
#[ignore]
fn regen_fixtures() {
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("concepts.json"),
        to_canonical_bytes(&toy_concepts()),
    )
    .unwrap();
    eprintln!("Regenerated terminology fixtures in {}", dir.display());
}
