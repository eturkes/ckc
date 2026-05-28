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
