//! Integration test: clinical formalization fixtures for SPEC 20 Phase 0 scenarios.
//!
//! Constructs 3 Rules (2 opposing + 1 provenance-incomplete), 2 ClinicalClaims,
//! PICOFrame, EtDFrame, and EvidenceAtom for toy scenarios 1, 5, 6, 7.
//!
//! Scenario coverage:
//!   1. norm conflict       — rule_sepsis_bl_recommend vs rule_bl_anaphylaxis_contra
//!   5. repair candidates   — same opposing rules as input for MaxSMT/ASP repair
//!   6. SHACL provenance    — rule_incomplete_provenance (empty spans + provenance)
//!   7. Lean proof          — same norm conflict as input for formal proof
//!
//! All objects reference source spans from 0.5.1 (toy_source_corpus).

use ckc_core::canonical::{ContentHash, content_hash, to_canonical_bytes};
use ckc_core::clinical::*;
use ckc_core::enums::*;
use ckc_core::id::*;
use ckc_core::nf::{NfContext, Normalize};
use ckc_core::profile::SemanticProfile;
use std::collections::HashSet;
use std::path::Path;

// =========================================================================
// ID constants (reusable by later subtask tests via JSON loading)
// =========================================================================

const RULE_SEPSIS_RECOMMEND: &str = "rule_sepsis_bl_recommend";
const RULE_BL_CONTRA: &str = "rule_bl_anaphylaxis_contra";
const RULE_INCOMPLETE: &str = "rule_incomplete_provenance";

const CLAIM_SEPSIS_RECOMMEND: &str = "claim_sepsis_bl_recommend";
const CLAIM_BL_CONTRA: &str = "claim_bl_contra";

const CQ_SEPSIS_ABX: &str = "cq_sepsis_abx";

// Span IDs from 0.5.1 (toy_source_corpus)
const SPAN_REC_SEPSIS: &str = "span_rec_sepsis_bl";
const SPAN_EVIDENCE: &str = "span_evidence_sepsis";
const SPAN_CONTRA: &str = "span_contra_bl_allergy";
const SPAN_ALLERGY_HIST: &str = "span_allergy_history";

// =========================================================================
// Fixture constructors
// =========================================================================

fn toy_rules() -> Vec<Rule> {
    vec![
        // Rule A: sepsis beta-lactam recommendation (scenario 1, 5, 7)
        Rule {
            rule_id: RuleId::new(RULE_SEPSIS_RECOMMEND),
            profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
            kind: RuleKind::Defeasible,
            context: "(dx sepsis) AND (adult patient)".into(),
            antecedent: "(dx sepsis) AND (adult patient)".into(),
            consequent: "(administer beta_lactam)".into(),
            norm: Some(Norm {
                context: "adult sepsis".into(),
                direction: RecommendationDirection::For,
                action: Action {
                    action_type: "administer".into(),
                    target_concept: ConceptId::new("concept_beta_lactam"),
                    parameters: serde_json::json!({
                        "dose_range": "standard",
                        "route": "iv"
                    }),
                    temporal_constraints: serde_json::json!({"onset": "immediate"}),
                    quantity_constraints: serde_json::json!({"min_dose_mg": 1000}),
                },
                recommendation_strength: RecommendationStrength::Strong,
                evidence_certainty: EvidenceCertainty::Moderate,
                original_modality_phrase_ja: "投与を推奨する".into(),
                deontic_projection: DeonticProjection::Recommended,
                exception_policy: "beta-lactam allergy contraindicates".into(),
                prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
            }),
            priority_over: vec![],
            exceptions: vec!["beta_lactam_anaphylaxis".into()],
            temporal_scope: Some("acute_phase".into()),
            population_scope: Some("adult".into()),
            source_span_ids: vec![SpanId::new(SPAN_REC_SEPSIS), SpanId::new(SPAN_EVIDENCE)],
            provenance: "JSICM sepsis guideline 2024, CQ1".into(),
            certificate_ids: vec![],
        },
        // Rule B: beta-lactam anaphylaxis contraindication (scenario 1, 5, 7)
        Rule {
            rule_id: RuleId::new(RULE_BL_CONTRA),
            profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
            kind: RuleKind::Defeasible,
            context: "(allergy_history beta_lactam anaphylaxis) AND (patient)".into(),
            antecedent: "(allergy_history beta_lactam anaphylaxis)".into(),
            consequent: "(contraindicate beta_lactam)".into(),
            norm: Some(Norm {
                context: "beta-lactam anaphylaxis history".into(),
                direction: RecommendationDirection::Against,
                action: Action {
                    action_type: "administer".into(),
                    target_concept: ConceptId::new("concept_beta_lactam"),
                    parameters: serde_json::json!({
                        "dose_range": "any",
                        "route": "any"
                    }),
                    temporal_constraints: serde_json::json!({}),
                    quantity_constraints: serde_json::json!({}),
                },
                recommendation_strength: RecommendationStrength::Strong,
                evidence_certainty: EvidenceCertainty::High,
                original_modality_phrase_ja: "禁忌である".into(),
                deontic_projection: DeonticProjection::Prohibited,
                exception_policy: "absolute contraindication".into(),
                prima_facie_or_all_things_considered: NormCommitment::AllThingsConsidered,
            }),
            priority_over: vec![RuleId::new(RULE_SEPSIS_RECOMMEND)],
            exceptions: vec![],
            temporal_scope: None,
            population_scope: None,
            source_span_ids: vec![SpanId::new(SPAN_CONTRA), SpanId::new(SPAN_ALLERGY_HIST)],
            provenance: "JSA drug allergy manual ed.3, contraindication section".into(),
            certificate_ids: vec![],
        },
        // Rule C: provenance-incomplete rule (scenario 6 — SHACL violation)
        Rule {
            rule_id: RuleId::new(RULE_INCOMPLETE),
            profiles: vec![SemanticProfile::Norm],
            kind: RuleKind::Strict,
            context: "(dx sepsis)".into(),
            antecedent: "(dx sepsis) AND (adult patient)".into(),
            consequent: "(monitor vital_signs)".into(),
            norm: None,
            priority_over: vec![],
            exceptions: vec![],
            temporal_scope: None,
            population_scope: None,
            source_span_ids: vec![],
            provenance: String::new(),
            certificate_ids: vec![],
        },
    ]
}

fn toy_pico() -> PICOFrame {
    PICOFrame {
        population: "成人敗血症患者".into(),
        intervention: "βラクタム系抗菌薬投与".into(),
        comparator: "非βラクタム系抗菌薬".into(),
        outcomes: vec!["28日死亡率".into(), "ICU在室日数".into()],
        cq_id: Some(CqId::new(CQ_SEPSIS_ABX)),
        scope: "inpatient ICU".into(),
        exclusions: vec!["βラクタムアレルギー既往".into()],
        source_span_ids: vec![SpanId::new(SPAN_REC_SEPSIS), SpanId::new(SPAN_EVIDENCE)],
    }
}

fn toy_etd() -> EtDFrame {
    EtDFrame {
        benefits: "28日死亡率の有意な低下 (RR 0.75, 95%CI 0.63-0.89)".into(),
        harms: "アナフィラキシーリスク (βラクタムアレルギー患者)".into(),
        certainty: EvidenceCertainty::Moderate,
        values: "mortality reduction valued highly by panel".into(),
        resources: "standard hospital formulary".into(),
        equity: "widely available in Japanese hospitals".into(),
        acceptability: "accepted standard of care per JSICM".into(),
        feasibility: "feasible in ICU setting".into(),
        recommendation_direction: RecommendationDirection::For,
        recommendation_strength: RecommendationStrength::Strong,
        source_span_ids: vec![SpanId::new(SPAN_REC_SEPSIS), SpanId::new(SPAN_EVIDENCE)],
    }
}

fn toy_evidence_atom() -> EvidenceAtom {
    EvidenceAtom {
        evidence_type: "meta_analysis".into(),
        source_span_ids: vec![SpanId::new(SPAN_EVIDENCE)],
        pico_ref: Some(CqId::new(CQ_SEPSIS_ABX)),
        effect_measure: Some("relative_risk".into()),
        value: Some(0.75),
        unit: None,
        confidence_interval: Some(ConfidenceInterval {
            lower: 0.63,
            upper: 0.89,
        }),
        certainty: EvidenceCertainty::Moderate,
        outcome_importance: Some("critical".into()),
        table_cell_refs: vec![],
    }
}

fn toy_claims() -> Vec<ClinicalClaim> {
    vec![
        // Claim A: sepsis beta-lactam recommendation (scenario 1, 5, 7)
        ClinicalClaim {
            claim_id: ClaimId::new(CLAIM_SEPSIS_RECOMMEND),
            claim_type: "recommendation".into(),
            profiles: vec![SemanticProfile::Evidence, SemanticProfile::Norm],
            source_span_ids: vec![SpanId::new(SPAN_REC_SEPSIS), SpanId::new(SPAN_EVIDENCE)],
            pico: Some(toy_pico()),
            etd: Some(toy_etd()),
            evidence_atoms: vec![toy_evidence_atom()],
            rule_ids: vec![RuleId::new(RULE_SEPSIS_RECOMMEND)],
            decision_table_ids: vec![],
            workflow_fragment_ids: vec![],
            gloss_ja: "敗血症患者にはβラクタム系抗菌薬の投与を強く推奨する".into(),
            gloss_en: "Beta-lactam antibiotics are strongly recommended \
                       for adult sepsis patients"
                .into(),
            status: "candidate".into(),
        },
        // Claim B: contraindication claim (scenario 1, 5, 7)
        ClinicalClaim {
            claim_id: ClaimId::new(CLAIM_BL_CONTRA),
            claim_type: "contraindication".into(),
            profiles: vec![SemanticProfile::Norm],
            source_span_ids: vec![SpanId::new(SPAN_CONTRA)],
            pico: None,
            etd: None,
            evidence_atoms: vec![],
            rule_ids: vec![RuleId::new(RULE_BL_CONTRA)],
            decision_table_ids: vec![],
            workflow_fragment_ids: vec![],
            gloss_ja: "βラクタム系抗菌薬にアナフィラキシーの既往がある患者には\
                       同系統薬の投与は禁忌である"
                .into(),
            gloss_en: "Beta-lactam antibiotics are contraindicated in patients \
                       with a history of beta-lactam anaphylaxis"
                .into(),
            status: "candidate".into(),
        },
    ]
}

// =========================================================================
// Fixture directory helpers
// =========================================================================

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/research_kernel/fixtures")
}

/// Load 0.5.1 span IDs from committed fixtures for referential validation.
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
// Referential consistency with 0.5.1 spans
// =========================================================================

#[test]
fn rules_reference_existing_spans() {
    let valid_spans = load_span_ids_from_fixtures();
    for rule in &toy_rules() {
        for span_id in &rule.source_span_ids {
            assert!(
                valid_spans.contains(span_id.as_str()),
                "rule {} references unknown span_id {}",
                rule.rule_id,
                span_id
            );
        }
    }
}

#[test]
fn claims_reference_existing_spans() {
    let valid_spans = load_span_ids_from_fixtures();
    for claim in &toy_claims() {
        for span_id in &claim.source_span_ids {
            assert!(
                valid_spans.contains(span_id.as_str()),
                "claim {} references unknown span_id {}",
                claim.claim_id,
                span_id
            );
        }
        if let Some(ref pico) = claim.pico {
            for span_id in &pico.source_span_ids {
                assert!(
                    valid_spans.contains(span_id.as_str()),
                    "claim {} PICO references unknown span_id {}",
                    claim.claim_id,
                    span_id
                );
            }
        }
        if let Some(ref etd) = claim.etd {
            for span_id in &etd.source_span_ids {
                assert!(
                    valid_spans.contains(span_id.as_str()),
                    "claim {} EtD references unknown span_id {}",
                    claim.claim_id,
                    span_id
                );
            }
        }
        for atom in &claim.evidence_atoms {
            for span_id in &atom.source_span_ids {
                assert!(
                    valid_spans.contains(span_id.as_str()),
                    "claim {} evidence atom references unknown span_id {}",
                    claim.claim_id,
                    span_id
                );
            }
        }
    }
}

#[test]
fn claims_reference_existing_rules() {
    let rule_ids: HashSet<String> = toy_rules().iter().map(|r| r.rule_id.0.clone()).collect();
    for claim in &toy_claims() {
        for rid in &claim.rule_ids {
            assert!(
                rule_ids.contains(rid.as_str()),
                "claim {} references unknown rule_id {}",
                claim.claim_id,
                rid
            );
        }
    }
}

#[test]
fn pico_cq_id_matches_source_span_cq() {
    let dir = fixtures_dir();
    let bytes = std::fs::read(dir.join("spans.json")).unwrap();
    let spans: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    let span_cq_ids: HashSet<String> = spans
        .iter()
        .filter_map(|s| s.get("cq_id").and_then(|v| v.as_str()))
        .map(String::from)
        .collect();

    let pico = toy_pico();
    if let Some(ref cq) = pico.cq_id {
        assert!(
            span_cq_ids.contains(cq.as_str()),
            "PICO cq_id {} has no corresponding span with that cq_id",
            cq
        );
    }
}

#[test]
fn evidence_atom_pico_ref_matches_pico_cq() {
    let atom = toy_evidence_atom();
    let pico = toy_pico();
    assert_eq!(
        atom.pico_ref, pico.cq_id,
        "evidence atom pico_ref must match PICO cq_id"
    );
}

#[test]
fn opposing_rules_target_same_concept() {
    let rules = toy_rules();
    let recommend = &rules[0];
    let contra = &rules[1];

    let rec_concept = &recommend.norm.as_ref().unwrap().action.target_concept;
    let con_concept = &contra.norm.as_ref().unwrap().action.target_concept;
    assert_eq!(
        rec_concept, con_concept,
        "opposing rules must target the same concept for conflict scenario"
    );
}

#[test]
fn opposing_rules_have_opposite_directions() {
    let rules = toy_rules();
    let rec_dir = &rules[0].norm.as_ref().unwrap().direction;
    let con_dir = &rules[1].norm.as_ref().unwrap().direction;
    assert_eq!(*rec_dir, RecommendationDirection::For);
    assert_eq!(*con_dir, RecommendationDirection::Against);
}

#[test]
fn contraindication_has_priority_over_recommendation() {
    let rules = toy_rules();
    let contra = &rules[1];
    assert!(
        contra
            .priority_over
            .iter()
            .any(|id| id.as_str() == RULE_SEPSIS_RECOMMEND),
        "contraindication rule must declare priority over recommendation"
    );
}

// =========================================================================
// Provenance-incomplete rule (scenario 6)
// =========================================================================

#[test]
fn provenance_incomplete_rule_has_empty_spans() {
    let rules = toy_rules();
    let incomplete = rules
        .iter()
        .find(|r| r.rule_id.as_str() == RULE_INCOMPLETE)
        .expect("provenance-incomplete rule must exist");
    assert!(
        incomplete.source_span_ids.is_empty(),
        "provenance-incomplete rule must have empty source_span_ids"
    );
    assert!(
        incomplete.provenance.is_empty(),
        "provenance-incomplete rule must have empty provenance"
    );
}

#[test]
fn provenance_incomplete_rule_detectable_programmatically() {
    let rules = toy_rules();
    let incomplete_ids: Vec<&str> = rules
        .iter()
        .filter(|r| r.source_span_ids.is_empty() && r.provenance.is_empty())
        .map(|r| r.rule_id.as_str())
        .collect();

    assert_eq!(
        incomplete_ids.len(),
        1,
        "exactly one rule must be provenance-incomplete"
    );
    assert_eq!(incomplete_ids[0], RULE_INCOMPLETE);
}

// =========================================================================
// Hash determinism
// =========================================================================

#[test]
fn canonical_hashes_deterministic_across_construction() {
    let h1_rules = content_hash(&toy_rules());
    let h2_rules = content_hash(&toy_rules());
    assert_eq!(h1_rules, h2_rules, "rule fixture hashes must be stable");

    let h1_claims = content_hash(&toy_claims());
    let h2_claims = content_hash(&toy_claims());
    assert_eq!(h1_claims, h2_claims, "claim fixture hashes must be stable");
}

#[test]
fn individual_rules_have_distinct_hashes() {
    let rules = toy_rules();
    let hashes: Vec<ContentHash> = rules.iter().map(content_hash).collect();
    let unique: HashSet<&str> = hashes.iter().map(|h| h.as_str()).collect();
    assert_eq!(
        unique.len(),
        hashes.len(),
        "each rule fixture must produce a unique content hash"
    );
}

#[test]
fn individual_claims_have_distinct_hashes() {
    let claims = toy_claims();
    let hashes: Vec<ContentHash> = claims.iter().map(content_hash).collect();
    let unique: HashSet<&str> = hashes.iter().map(|h| h.as_str()).collect();
    assert_eq!(
        unique.len(),
        hashes.len(),
        "each claim fixture must produce a unique content hash"
    );
}

// =========================================================================
// NF normalization
// =========================================================================

#[test]
fn nf_rule_recommendation_sorts_commutative_operands() {
    let mut rule = toy_rules()[0].clone();
    let original_antecedent = rule.antecedent.clone();
    let mut ctx = NfContext::new();
    rule.normalize(&mut ctx);

    // "(dx sepsis) AND (adult patient)" → "(adult patient) AND (dx sepsis)"
    assert_eq!(
        rule.antecedent, "(adult patient) AND (dx sepsis)",
        "NF must sort AND operands alphabetically"
    );
    assert_ne!(
        rule.antecedent, original_antecedent,
        "antecedent must change under NF"
    );
    // context also gets sorted
    assert_eq!(
        rule.context, "(adult patient) AND (dx sepsis)",
        "NF must sort context AND operands"
    );
}

#[test]
fn nf_rule_recommendation_sorts_span_ids() {
    let mut rule = toy_rules()[0].clone();
    let mut ctx = NfContext::new();
    rule.normalize(&mut ctx);

    // ["span_rec_sepsis_bl", "span_evidence_sepsis"] →
    // ["span_evidence_sepsis", "span_rec_sepsis_bl"]
    assert_eq!(
        rule.source_span_ids,
        vec![SpanId::new(SPAN_EVIDENCE), SpanId::new(SPAN_REC_SEPSIS),],
        "NF must sort source_span_ids alphabetically"
    );
}

#[test]
fn nf_rule_contraindication_sorts_span_ids() {
    let mut rule = toy_rules()[1].clone();
    let mut ctx = NfContext::new();
    rule.normalize(&mut ctx);

    // ["span_contra_bl_allergy", "span_allergy_history"] →
    // ["span_allergy_history", "span_contra_bl_allergy"]
    assert_eq!(
        rule.source_span_ids,
        vec![SpanId::new(SPAN_ALLERGY_HIST), SpanId::new(SPAN_CONTRA),],
        "NF must sort contraindication span_ids"
    );
}

#[test]
fn nf_deontic_projection_already_canonical() {
    let mut rule_a = toy_rules()[0].clone();
    let mut rule_b = toy_rules()[1].clone();
    let mut ctx = NfContext::new();
    rule_a.normalize(&mut ctx);
    rule_b.normalize(&mut ctx);

    // "投与を推奨する" → Recommended (already set)
    assert_eq!(
        rule_a.norm.as_ref().unwrap().deontic_projection,
        DeonticProjection::Recommended,
    );
    // "禁忌である" → Prohibited (already set)
    assert_eq!(
        rule_b.norm.as_ref().unwrap().deontic_projection,
        DeonticProjection::Prohibited,
    );

    // No Pass 9 rewrites expected (projections already match lexicon)
    let pass9_rewrites: Vec<_> = ctx.rewrites.iter().filter(|r| r.pass == 9).collect();
    assert!(
        pass9_rewrites.is_empty(),
        "deontic projections already match lexicon; \
         Pass 9 must produce no rewrites"
    );
}

#[test]
fn nf_assigns_stable_ids() {
    let mut rules = toy_rules();
    let mut ctx = NfContext::new();
    for rule in &mut rules {
        rule.normalize(&mut ctx);
    }

    for rule in &rules {
        assert!(
            rule.rule_id.as_str().starts_with("nf-"),
            "NF must assign stable ID (nf-...) to rule {}",
            rule.rule_id
        );
    }
}

#[test]
fn nf_stable_ids_are_deterministic() {
    let normalize_and_collect = || {
        let mut rules = toy_rules();
        let mut ctx = NfContext::new();
        for rule in &mut rules {
            rule.normalize(&mut ctx);
        }
        rules
            .iter()
            .map(|r| r.rule_id.0.clone())
            .collect::<Vec<_>>()
    };

    let ids1 = normalize_and_collect();
    let ids2 = normalize_and_collect();
    assert_eq!(ids1, ids2, "NF stable IDs must be identical across runs");
}

#[test]
fn nf_idempotent_rules() {
    let mut rules = toy_rules();
    let mut ctx1 = NfContext::new();
    for rule in &mut rules {
        rule.normalize(&mut ctx1);
    }
    let bytes_after_first = to_canonical_bytes(&rules);

    let mut ctx2 = NfContext::new();
    for rule in &mut rules {
        rule.normalize(&mut ctx2);
    }
    let bytes_after_second = to_canonical_bytes(&rules);

    assert_eq!(
        bytes_after_first, bytes_after_second,
        "NF(NF(rules)) must equal NF(rules)"
    );
}

#[test]
fn nf_idempotent_claims() {
    let mut claims = toy_claims();
    let mut ctx1 = NfContext::new();
    for claim in &mut claims {
        claim.normalize(&mut ctx1);
    }
    let bytes_after_first = to_canonical_bytes(&claims);

    let mut ctx2 = NfContext::new();
    for claim in &mut claims {
        claim.normalize(&mut ctx2);
    }
    let bytes_after_second = to_canonical_bytes(&claims);

    assert_eq!(
        bytes_after_first, bytes_after_second,
        "NF(NF(claims)) must equal NF(claims)"
    );
}

#[test]
fn nf_incomplete_rule_gets_stable_id_from_empty_spans() {
    let mut rule = toy_rules()[2].clone();
    assert_eq!(rule.rule_id.as_str(), RULE_INCOMPLETE);

    let mut ctx = NfContext::new();
    rule.normalize(&mut ctx);

    assert!(
        rule.rule_id.as_str().starts_with("nf-"),
        "even provenance-incomplete rules get stable IDs"
    );
    assert!(
        rule.source_span_ids.is_empty(),
        "empty source_span_ids must remain empty after NF"
    );
}

// =========================================================================
// Committed fixture file tests
// =========================================================================

#[test]
fn committed_rules_match() {
    let path = fixtures_dir().join("rules.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test toy_clinical_formalization regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_rules());
    assert_eq!(
        bytes, expected,
        "committed rules.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_claims_match() {
    let path = fixtures_dir().join("claims.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test toy_clinical_formalization regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_claims());
    assert_eq!(
        bytes, expected,
        "committed claims.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_fixtures_deserialize_correctly() {
    let dir = fixtures_dir();
    let rules: Vec<Rule> = serde_json::from_slice(&std::fs::read(dir.join("rules.json")).unwrap())
        .expect("rules.json must deserialize");
    let claims: Vec<ClinicalClaim> =
        serde_json::from_slice(&std::fs::read(dir.join("claims.json")).unwrap())
            .expect("claims.json must deserialize");

    assert_eq!(rules.len(), 3);
    assert_eq!(claims.len(), 2);
}

// =========================================================================
// Unique IDs
// =========================================================================

#[test]
fn all_rule_ids_are_unique() {
    let rules = toy_rules();
    let mut seen = HashSet::new();
    for rule in &rules {
        assert!(
            seen.insert(rule.rule_id.as_str()),
            "duplicate rule_id: {}",
            rule.rule_id
        );
    }
}

#[test]
fn all_claim_ids_are_unique() {
    let claims = toy_claims();
    let mut seen = HashSet::new();
    for claim in &claims {
        assert!(
            seen.insert(claim.claim_id.as_str()),
            "duplicate claim_id: {}",
            claim.claim_id
        );
    }
}

// =========================================================================
// Fixture regeneration (run with --ignored)
// =========================================================================

#[test]
#[ignore]
fn regen_fixtures() {
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join("rules.json"), to_canonical_bytes(&toy_rules())).unwrap();
    std::fs::write(dir.join("claims.json"), to_canonical_bytes(&toy_claims())).unwrap();

    eprintln!("Regenerated clinical fixtures in {}", dir.display());
}
