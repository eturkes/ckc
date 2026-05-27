use serde::{Deserialize, Serialize};

use crate::enums::{
    ActionType, ClaimStatus, ClaimType, DeonticProjection, EvidenceCertainty, EvidenceType,
    ExceptionPolicy, NormScope, OutcomeImportance, RecommendationDirection,
    RecommendationStrength, RuleKind,
};
use crate::id::{CertId, ClaimId, ConceptId, CqId, DecisionTableId, RuleId, SpanId, WorkflowId};
use crate::profile::SemanticProfile;
use crate::source::TableCellRef;

/// Confidence interval for an effect estimate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub low: f64,
    pub high: f64,
    pub level: f64,
}

/// Key-value parameter for an Action.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionParameter {
    pub key: String,
    pub value: String,
}

/// SPEC §10: PICO frame for structured evidence questions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PICOFrame {
    pub population: String,
    pub intervention: String,
    pub comparator: String,
    pub outcomes: Vec<String>,
    pub cq_id: Option<CqId>,
    pub scope: String,
    pub exclusions: Vec<String>,
    pub source_span_ids: Vec<SpanId>,
}

/// SPEC §10: Evidence-to-Decision framework fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// SPEC §10: single piece of evidence supporting a clinical claim.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceAtom {
    pub evidence_type: EvidenceType,
    pub source_span_ids: Vec<SpanId>,
    pub pico_ref: Option<CqId>,
    pub effect_measure: Option<String>,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub confidence_interval: Option<ConfidenceInterval>,
    pub certainty: EvidenceCertainty,
    pub outcome_importance: Option<OutcomeImportance>,
    pub table_cell_refs: Vec<TableCellRef>,
}

/// SPEC §10: clinical action with typed parameters and constraints.
/// Constraint fields are string expressions at v0; structured ASTs
/// arrive with surface syntax in task 0.3.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub target_concept: ConceptId,
    pub parameters: Vec<ActionParameter>,
    pub temporal_constraints: Vec<String>,
    pub quantity_constraints: Vec<String>,
}

/// SPEC §10: dyadic clinical norm (context, direction, action).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Norm {
    pub context: String,
    pub direction: RecommendationDirection,
    pub action: Action,
    pub recommendation_strength: RecommendationStrength,
    pub evidence_certainty: EvidenceCertainty,
    pub original_modality_phrase_ja: String,
    pub deontic_projection: DeonticProjection,
    pub exception_policy: ExceptionPolicy,
    pub prima_facie_or_all_things_considered: NormScope,
}

/// SPEC §10: formalized rule with optional norm.
/// Logical expression fields (context, antecedent, consequent) are
/// strings at v0; structured ASTs arrive with surface syntax in task 0.3.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub rule_id: RuleId,
    pub profiles: Vec<SemanticProfile>,
    pub kind: RuleKind,
    pub context: String,
    pub antecedent: String,
    pub consequent: String,
    pub norm: Option<Norm>,
    pub priority_over: Vec<RuleId>,
    pub exceptions: Vec<RuleId>,
    pub temporal_scope: Option<String>,
    pub population_scope: Option<String>,
    pub source_span_ids: Vec<SpanId>,
    pub provenance: String,
    pub certificate_ids: Vec<CertId>,
}

/// SPEC §10: clinical claim with evidence, rules, and review status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClinicalClaim {
    pub claim_id: ClaimId,
    pub claim_type: ClaimType,
    pub profiles: Vec<SemanticProfile>,
    pub source_span_ids: Vec<SpanId>,
    pub pico: Option<PICOFrame>,
    pub etd: Option<EtDFrame>,
    pub evidence_atoms: Vec<EvidenceAtom>,
    pub rule_ids: Vec<RuleId>,
    pub decision_table_ids: Vec<DecisionTableId>,
    pub workflow_fragment_ids: Vec<WorkflowId>,
    pub gloss_ja: String,
    pub gloss_en: String,
    pub status: ClaimStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::content_hash_of;

    fn sample_action() -> Action {
        Action {
            action_type: ActionType::Administer,
            target_concept: ConceptId::new("c-beta-lactam"),
            parameters: vec![ActionParameter {
                key: "route".into(),
                value: "iv".into(),
            }],
            temporal_constraints: vec!["within 1 hour of diagnosis".into()],
            quantity_constraints: vec![],
        }
    }

    fn sample_norm() -> Norm {
        Norm {
            context: "(dx sepsis)".into(),
            direction: RecommendationDirection::For,
            action: sample_action(),
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: "推奨する".into(),
            deontic_projection: DeonticProjection::Recommendation,
            exception_policy: ExceptionPolicy::Defeasible,
            prima_facie_or_all_things_considered: NormScope::PrimaFacie,
        }
    }

    #[test]
    fn pico_frame_serde_roundtrip() {
        let pico = PICOFrame {
            population: "成人敗血症患者".into(),
            intervention: "βラクタム系抗菌薬投与".into(),
            comparator: "非βラクタム系抗菌薬".into(),
            outcomes: vec!["28日死亡率".into(), "ICU在室日数".into()],
            cq_id: Some(CqId::new("cq-001")),
            scope: "ICU入室成人".into(),
            exclusions: vec!["βラクタムアレルギー歴".into()],
            source_span_ids: vec![SpanId::new("span-001")],
        };
        let json = serde_json::to_string(&pico).unwrap();
        let back: PICOFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pico);
    }

    #[test]
    fn etd_frame_serde_roundtrip() {
        let etd = EtDFrame {
            benefits: "死亡率低下".into(),
            harms: "アレルギー反応リスク".into(),
            certainty: EvidenceCertainty::Moderate,
            values: "患者は生存率を重視".into(),
            resources: "標準的コスト".into(),
            equity: "広く利用可能".into(),
            acceptability: "臨床的に受容".into(),
            feasibility: "実施可能".into(),
            recommendation_direction: RecommendationDirection::For,
            recommendation_strength: RecommendationStrength::Strong,
            source_span_ids: vec![SpanId::new("span-010"), SpanId::new("span-011")],
        };
        let json = serde_json::to_string(&etd).unwrap();
        let back: EtDFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(back, etd);
    }

    #[test]
    fn evidence_atom_serde_roundtrip() {
        let atom = EvidenceAtom {
            evidence_type: EvidenceType::SystematicReview,
            source_span_ids: vec![SpanId::new("span-020")],
            pico_ref: Some(CqId::new("cq-001")),
            effect_measure: Some("RR".into()),
            value: Some(0.85),
            unit: None,
            confidence_interval: Some(ConfidenceInterval {
                low: 0.72,
                high: 0.95,
                level: 0.95,
            }),
            certainty: EvidenceCertainty::Moderate,
            outcome_importance: Some(OutcomeImportance::Critical),
            table_cell_refs: vec![],
        };
        let json = serde_json::to_string(&atom).unwrap();
        let back: EvidenceAtom = serde_json::from_str(&json).unwrap();
        assert_eq!(back, atom);
    }

    #[test]
    fn evidence_atom_minimal() {
        let atom = EvidenceAtom {
            evidence_type: EvidenceType::ExpertOpinion,
            source_span_ids: vec![SpanId::new("span-030")],
            pico_ref: None,
            effect_measure: None,
            value: None,
            unit: None,
            confidence_interval: None,
            certainty: EvidenceCertainty::VeryLow,
            outcome_importance: None,
            table_cell_refs: vec![],
        };
        let json = serde_json::to_string(&atom).unwrap();
        let back: EvidenceAtom = serde_json::from_str(&json).unwrap();
        assert_eq!(back, atom);
    }

    #[test]
    fn action_serde_roundtrip() {
        let action = sample_action();
        let json = serde_json::to_string(&action).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(back, action);
    }

    #[test]
    fn norm_serde_roundtrip() {
        let norm = sample_norm();
        let json = serde_json::to_string(&norm).unwrap();
        let back: Norm = serde_json::from_str(&json).unwrap();
        assert_eq!(back, norm);
    }

    #[test]
    fn rule_serde_roundtrip() {
        let rule = Rule {
            rule_id: RuleId::new("rule-sepsis-beta-lactam"),
            profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
            kind: RuleKind::Defeasible,
            context: "(dx sepsis)".into(),
            antecedent: "(and (dx sepsis) (adult patient))".into(),
            consequent: "(administer beta_lactam)".into(),
            norm: Some(sample_norm()),
            priority_over: vec![],
            exceptions: vec![RuleId::new("rule-beta-lactam-allergy")],
            temporal_scope: Some("acute phase".into()),
            population_scope: Some("adult ICU patients".into()),
            source_span_ids: vec![SpanId::new("span-001")],
            provenance: "autoformalized".into(),
            certificate_ids: vec![],
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        assert_eq!(back, rule);
    }

    #[test]
    fn rule_strict_minimal() {
        let rule = Rule {
            rule_id: RuleId::new("rule-def-sepsis"),
            profiles: vec![SemanticProfile::Classical],
            kind: RuleKind::Strict,
            context: "definition".into(),
            antecedent: "(infection) (organ-dysfunction)".into(),
            consequent: "(dx sepsis)".into(),
            norm: None,
            priority_over: vec![],
            exceptions: vec![],
            temporal_scope: None,
            population_scope: None,
            source_span_ids: vec![SpanId::new("span-050")],
            provenance: "manual".into(),
            certificate_ids: vec![],
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        assert_eq!(back, rule);
    }

    #[test]
    fn clinical_claim_serde_roundtrip() {
        let claim = ClinicalClaim {
            claim_id: ClaimId::new("claim-sepsis-rec-001"),
            claim_type: ClaimType::Normative,
            profiles: vec![SemanticProfile::Norm, SemanticProfile::Evidence],
            source_span_ids: vec![SpanId::new("span-001"), SpanId::new("span-010")],
            pico: Some(PICOFrame {
                population: "成人敗血症患者".into(),
                intervention: "βラクタム系抗菌薬".into(),
                comparator: "非βラクタム系".into(),
                outcomes: vec!["28日死亡率".into()],
                cq_id: Some(CqId::new("cq-001")),
                scope: "ICU成人".into(),
                exclusions: vec![],
                source_span_ids: vec![SpanId::new("span-001")],
            }),
            etd: None,
            evidence_atoms: vec![],
            rule_ids: vec![RuleId::new("rule-sepsis-beta-lactam")],
            decision_table_ids: vec![],
            workflow_fragment_ids: vec![],
            gloss_ja: "敗血症にはβラクタム系抗菌薬を推奨する".into(),
            gloss_en: "Beta-lactam antibiotics are recommended for sepsis".into(),
            status: ClaimStatus::Candidate,
        };
        let json = serde_json::to_string(&claim).unwrap();
        let back: ClinicalClaim = serde_json::from_str(&json).unwrap();
        assert_eq!(back, claim);
    }

    #[test]
    fn rule_canonical_hash_deterministic() {
        let rule = Rule {
            rule_id: RuleId::new("rule-test"),
            profiles: vec![SemanticProfile::Norm],
            kind: RuleKind::Defeasible,
            context: "test".into(),
            antecedent: "A".into(),
            consequent: "B".into(),
            norm: None,
            priority_over: vec![],
            exceptions: vec![],
            temporal_scope: None,
            population_scope: None,
            source_span_ids: vec![SpanId::new("span-001")],
            provenance: "test".into(),
            certificate_ids: vec![],
        };
        let h1 = content_hash_of(&rule).unwrap();
        let h2 = content_hash_of(&rule).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn clinical_claim_canonical_hash_deterministic() {
        let claim = ClinicalClaim {
            claim_id: ClaimId::new("claim-test"),
            claim_type: ClaimType::Factual,
            profiles: vec![SemanticProfile::Classical],
            source_span_ids: vec![SpanId::new("span-001")],
            pico: None,
            etd: None,
            evidence_atoms: vec![],
            rule_ids: vec![],
            decision_table_ids: vec![],
            workflow_fragment_ids: vec![],
            gloss_ja: "テスト".into(),
            gloss_en: "test".into(),
            status: ClaimStatus::Candidate,
        };
        let h1 = content_hash_of(&claim).unwrap();
        let h2 = content_hash_of(&claim).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn confidence_interval_serde() {
        let ci = ConfidenceInterval {
            low: 0.72,
            high: 0.95,
            level: 0.95,
        };
        let json = serde_json::to_string(&ci).unwrap();
        let back: ConfidenceInterval = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ci);
    }

    #[test]
    fn action_parameter_serde() {
        let param = ActionParameter {
            key: "dose".into(),
            value: "2g".into(),
        };
        let json = serde_json::to_string(&param).unwrap();
        let back: ActionParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back, param);
    }
}
