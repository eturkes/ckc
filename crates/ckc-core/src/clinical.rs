use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::enums::{
    DeonticProjection, EvidenceCertainty, NormCommitment, RecommendationDirection,
    RecommendationStrength, RuleKind,
};
use crate::id::{
    CertificateId, ClaimId, ConceptId, CqId, DecisionTableId, RuleId, SpanId, WorkflowId,
};
use crate::profile::SemanticProfile;
use crate::source::TableCellRef;

// ---------------------------------------------------------------------------
// Helper types
// ---------------------------------------------------------------------------

/// Numeric confidence interval bounds for effect estimates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
}

// ---------------------------------------------------------------------------
// SPEC 10 types: evidence and clinical formalization
// ---------------------------------------------------------------------------

/// PICO frame for clinical question segmentation (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PICOFrame {
    pub population: String,
    pub intervention: String,
    pub comparator: String,
    pub outcomes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cq_id: Option<CqId>,
    pub scope: String,
    pub exclusions: Vec<String>,
    pub source_span_ids: Vec<SpanId>,
}

/// Evidence-to-Decision frame with GRADE dimensions (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EtDFrame {
    pub benefits: String,
    pub harms: String,
    pub certainty: EvidenceCertainty,
    pub values: String,
    pub resources: String,
    pub equity: String,
    pub acceptability: String,
    pub feasibility: String,
    pub recommendation_direction: RecommendationDirection,
    pub recommendation_strength: RecommendationStrength,
    pub source_span_ids: Vec<SpanId>,
}

/// Single evidence atom with effect estimate and source grounding (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceAtom {
    pub evidence_type: String,
    pub source_span_ids: Vec<SpanId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pico_ref: Option<CqId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_measure: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_interval: Option<ConfidenceInterval>,
    pub certainty: EvidenceCertainty,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_importance: Option<String>,
    pub table_cell_refs: Vec<TableCellRef>,
}

/// Clinical action with concept target and constraints (SPEC 10).
/// Fields `parameters`, `temporal_constraints`, `quantity_constraints` use
/// open JSON values at schema v0; later phases refine these into typed ASTs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Action {
    pub action_type: String,
    pub target_concept: ConceptId,
    pub parameters: Value,
    pub temporal_constraints: Value,
    pub quantity_constraints: Value,
}

/// Dyadic clinical norm: (context, direction, action) with strength,
/// certainty, deontic projection, and defeasibility metadata (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Norm {
    pub context: String,
    pub direction: RecommendationDirection,
    pub action: Action,
    pub recommendation_strength: RecommendationStrength,
    pub evidence_certainty: EvidenceCertainty,
    pub original_modality_phrase_ja: String,
    pub deontic_projection: DeonticProjection,
    pub exception_policy: String,
    pub prima_facie_or_all_things_considered: NormCommitment,
}

/// Formalized rule with profile admission, defeasible kind, optional norm,
/// and certificate linkage (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Rule {
    pub rule_id: RuleId,
    pub profiles: Vec<SemanticProfile>,
    pub kind: RuleKind,
    pub context: String,
    pub antecedent: String,
    pub consequent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub norm: Option<Norm>,
    pub priority_over: Vec<RuleId>,
    pub exceptions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporal_scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population_scope: Option<String>,
    pub source_span_ids: Vec<SpanId>,
    pub provenance: String,
    pub certificate_ids: Vec<CertificateId>,
}

/// Top-level clinical claim aggregating evidence, rules, and glosses (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClinicalClaim {
    pub claim_id: ClaimId,
    pub claim_type: String,
    pub profiles: Vec<SemanticProfile>,
    pub source_span_ids: Vec<SpanId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pico: Option<PICOFrame>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etd: Option<EtDFrame>,
    pub evidence_atoms: Vec<EvidenceAtom>,
    pub rule_ids: Vec<RuleId>,
    pub decision_table_ids: Vec<DecisionTableId>,
    pub workflow_fragment_ids: Vec<WorkflowId>,
    pub gloss_ja: String,
    pub gloss_en: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_action() -> Action {
        Action {
            action_type: "administer".into(),
            target_concept: ConceptId::new("concept_beta_lactam"),
            parameters: serde_json::json!({"dose_range": "standard", "route": "iv"}),
            temporal_constraints: serde_json::json!({"onset": "immediate"}),
            quantity_constraints: serde_json::json!({"min_dose_mg": 1000}),
        }
    }

    fn fixture_norm() -> Norm {
        Norm {
            context: "sepsis in adult patients".into(),
            direction: RecommendationDirection::For,
            action: fixture_action(),
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: "投与を推奨する".into(),
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "beta-lactam allergy contraindicates".into(),
            prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
        }
    }

    fn fixture_pico_frame() -> PICOFrame {
        PICOFrame {
            population: "成人敗血症患者".into(),
            intervention: "βラクタム系抗菌薬投与".into(),
            comparator: "非βラクタム系抗菌薬".into(),
            outcomes: vec!["28日死亡率".into(), "ICU在室日数".into()],
            cq_id: Some(CqId::new("cq_sepsis_abx_001")),
            scope: "inpatient ICU".into(),
            exclusions: vec!["βラクタムアレルギー".into()],
            source_span_ids: vec![SpanId::new("span_s1")],
        }
    }

    fn fixture_etd_frame() -> EtDFrame {
        EtDFrame {
            benefits: "reduced 28-day mortality".into(),
            harms: "anaphylaxis risk in allergic patients".into(),
            certainty: EvidenceCertainty::Moderate,
            values: "mortality reduction valued highly".into(),
            resources: "standard hospital formulary".into(),
            equity: "widely available".into(),
            acceptability: "accepted standard of care".into(),
            feasibility: "feasible in ICU setting".into(),
            recommendation_direction: RecommendationDirection::For,
            recommendation_strength: RecommendationStrength::Strong,
            source_span_ids: vec![SpanId::new("span_s1"), SpanId::new("span_s2")],
        }
    }

    fn fixture_confidence_interval() -> ConfidenceInterval {
        ConfidenceInterval {
            lower: 0.45,
            upper: 0.82,
        }
    }

    fn fixture_evidence_atom() -> EvidenceAtom {
        EvidenceAtom {
            evidence_type: "outcome_effect".into(),
            source_span_ids: vec![SpanId::new("span_evidence_001")],
            pico_ref: Some(CqId::new("cq_sepsis_abx_001")),
            effect_measure: Some("odds_ratio".into()),
            value: Some(0.62),
            unit: None,
            confidence_interval: Some(fixture_confidence_interval()),
            certainty: EvidenceCertainty::Moderate,
            outcome_importance: Some("critical".into()),
            table_cell_refs: vec![],
        }
    }

    fn fixture_rule() -> Rule {
        Rule {
            rule_id: RuleId::new("rule_sepsis_beta_lactam_001"),
            profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
            kind: RuleKind::Defeasible,
            context: "sepsis AND adult_patient".into(),
            antecedent: "(dx sepsis) AND (adult patient)".into(),
            consequent: "(administer beta_lactam)".into(),
            norm: Some(fixture_norm()),
            priority_over: vec![],
            exceptions: vec!["beta_lactam_allergy".into()],
            temporal_scope: Some("acute_phase".into()),
            population_scope: Some("adult".into()),
            source_span_ids: vec![SpanId::new("span_s1")],
            provenance: "guideline_sepsis_2024_cq1".into(),
            certificate_ids: vec![],
        }
    }

    fn fixture_clinical_claim() -> ClinicalClaim {
        ClinicalClaim {
            claim_id: ClaimId::new("claim_sepsis_beta_lactam"),
            claim_type: "recommendation".into(),
            profiles: vec![SemanticProfile::Evidence, SemanticProfile::Norm],
            source_span_ids: vec![SpanId::new("span_s1")],
            pico: Some(fixture_pico_frame()),
            etd: Some(fixture_etd_frame()),
            evidence_atoms: vec![fixture_evidence_atom()],
            rule_ids: vec![RuleId::new("rule_sepsis_beta_lactam_001")],
            decision_table_ids: vec![],
            workflow_fragment_ids: vec![],
            gloss_ja: "敗血症にはβラクタム系抗菌薬の投与を強く推奨する".into(),
            gloss_en: "Beta-lactam antibiotics are strongly recommended for sepsis".into(),
            status: "candidate".into(),
        }
    }

    // -- Serde round-trip tests --

    #[test]
    fn pico_frame_roundtrip() {
        let pico = fixture_pico_frame();
        let json = serde_json::to_string(&pico).unwrap();
        let rt: PICOFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(pico, rt);
    }

    #[test]
    fn etd_frame_roundtrip() {
        let etd = fixture_etd_frame();
        let json = serde_json::to_string(&etd).unwrap();
        let rt: EtDFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(etd, rt);
    }

    #[test]
    fn evidence_atom_roundtrip() {
        let atom = fixture_evidence_atom();
        let json = serde_json::to_string(&atom).unwrap();
        let rt: EvidenceAtom = serde_json::from_str(&json).unwrap();
        assert_eq!(atom, rt);
    }

    #[test]
    fn action_roundtrip() {
        let action = fixture_action();
        let json = serde_json::to_string(&action).unwrap();
        let rt: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, rt);
    }

    #[test]
    fn norm_roundtrip() {
        let norm = fixture_norm();
        let json = serde_json::to_string(&norm).unwrap();
        let rt: Norm = serde_json::from_str(&json).unwrap();
        assert_eq!(norm, rt);
    }

    #[test]
    fn rule_roundtrip() {
        let rule = fixture_rule();
        let json = serde_json::to_string(&rule).unwrap();
        let rt: Rule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule, rt);
    }

    #[test]
    fn clinical_claim_roundtrip() {
        let claim = fixture_clinical_claim();
        let json = serde_json::to_string(&claim).unwrap();
        let rt: ClinicalClaim = serde_json::from_str(&json).unwrap();
        assert_eq!(claim, rt);
    }

    #[test]
    fn confidence_interval_roundtrip() {
        let ci = fixture_confidence_interval();
        let json = serde_json::to_string(&ci).unwrap();
        let rt: ConfidenceInterval = serde_json::from_str(&json).unwrap();
        assert_eq!(ci, rt);
    }

    // -- Optional field omission --

    #[test]
    fn pico_frame_optional_cq_id_omitted() {
        let mut pico = fixture_pico_frame();
        pico.cq_id = None;
        let json = serde_json::to_string(&pico).unwrap();
        assert!(!json.contains("cq_id"));
        let rt: PICOFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(pico, rt);
    }

    #[test]
    fn evidence_atom_minimal_options() {
        let mut atom = fixture_evidence_atom();
        atom.pico_ref = None;
        atom.effect_measure = None;
        atom.value = None;
        atom.unit = None;
        atom.confidence_interval = None;
        atom.outcome_importance = None;
        let json = serde_json::to_string(&atom).unwrap();
        assert!(!json.contains("pico_ref"));
        assert!(!json.contains("effect_measure"));
        assert!(!json.contains("\"value\""));
        assert!(!json.contains("\"unit\""));
        assert!(!json.contains("confidence_interval"));
        assert!(!json.contains("outcome_importance"));
        let rt: EvidenceAtom = serde_json::from_str(&json).unwrap();
        assert_eq!(atom, rt);
    }

    #[test]
    fn rule_minimal_options() {
        let mut rule = fixture_rule();
        rule.norm = None;
        rule.temporal_scope = None;
        rule.population_scope = None;
        let json = serde_json::to_string(&rule).unwrap();
        assert!(!json.contains("\"norm\""));
        assert!(!json.contains("temporal_scope"));
        assert!(!json.contains("population_scope"));
        let rt: Rule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule, rt);
    }

    #[test]
    fn clinical_claim_minimal_options() {
        let mut claim = fixture_clinical_claim();
        claim.pico = None;
        claim.etd = None;
        let json = serde_json::to_string(&claim).unwrap();
        assert!(!json.contains("\"pico\""));
        assert!(!json.contains("\"etd\""));
        let rt: ClinicalClaim = serde_json::from_str(&json).unwrap();
        assert_eq!(claim, rt);
    }
}
