use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::enums::{CaseType, HitPolicy};
use crate::id::{
    BundleId, CaseId, CertificateId, DecisionRowId, DecisionTableId, DmnExportId, ExtractedTableId,
    RuleId, SpanId, WitnessId, WorkflowId,
};
use crate::source::TableCellRef;

// ---------------------------------------------------------------------------
// SPEC 10 types: structured artifacts
// ---------------------------------------------------------------------------

/// Decision table row with conditions, outputs, and source grounding (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DecisionRow {
    pub row_id: DecisionRowId,
    pub conditions: Vec<Value>,
    pub outputs: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    pub source_span_ids: Vec<SpanId>,
    pub cell_refs: Vec<TableCellRef>,
}

/// DMN-style decision table with hit policy and certificate linkage (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DecisionTable {
    pub table_id: DecisionTableId,
    pub hit_policy: HitPolicy,
    pub input_columns: Vec<String>,
    pub output_columns: Vec<String>,
    pub rows: Vec<DecisionRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_table_id: Option<ExtractedTableId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmn_export_id: Option<DmnExportId>,
    pub certificate_ids: Vec<CertificateId>,
}

/// Clinical pathway or workflow fragment with states, transitions, and
/// outcome/assessment tracking (SPEC 10).
/// Fields `states`, `transitions`, `outcomes`, `assessments`, `tasks`, and
/// `variance_rules` use open JSON values at schema v0; later phases refine
/// these into typed sub-element ASTs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowFragment {
    pub workflow_id: WorkflowId,
    pub workflow_type: String,
    pub states: Vec<Value>,
    pub transitions: Vec<Value>,
    pub outcomes: Vec<Value>,
    pub assessments: Vec<Value>,
    pub tasks: Vec<Value>,
    pub variance_rules: Vec<Value>,
    pub source_span_ids: Vec<SpanId>,
}

/// Event Calculus narrative with typed events, fluents, and axioms (SPEC 10).
/// Axiom arrays (`happens`, `initiates`, `terminates`, `initially`,
/// `holds_queries`) use open JSON values at schema v0.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EventNarrative {
    pub event_types: Vec<String>,
    pub fluent_types: Vec<String>,
    pub happens: Vec<Value>,
    pub initiates: Vec<Value>,
    pub terminates: Vec<Value>,
    pub initially: Vec<Value>,
    pub holds_queries: Vec<Value>,
    pub source_span_ids: Vec<SpanId>,
}

/// Synthetic or fixture patient case for execution and conflict testing
/// (SPEC 10). Clinical data arrays use open JSON values at schema v0.
/// PHI cases belong to later governed deployments; initial cases are
/// `synthetic` or `fixture` only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PatientCase {
    pub case_id: CaseId,
    pub case_type: CaseType,
    pub facts: Vec<Value>,
    pub events: Vec<Value>,
    pub observations: Vec<Value>,
    pub medications: Vec<Value>,
    pub conditions: Vec<Value>,
    pub allergies: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_origin: Option<String>,
    pub source_span_ids: Vec<SpanId>,
    pub privacy_status: String,
}

/// Replayable execution witness from a target backend (SPEC 10).
/// A CKC artifact is executable when at least one target can produce a
/// replayable witness, model, proof, or explicit unsat/core result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionWitness {
    pub witness_id: WitnessId,
    pub bundle_id: BundleId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<CaseId>,
    pub context_facts: Vec<Value>,
    pub trace: Vec<Value>,
    pub applicable_rules: Vec<RuleId>,
    pub defeated_rules: Vec<RuleId>,
    pub violated_constraints: Vec<String>,
    pub models: Vec<Value>,
    pub unsat_cores: Vec<Value>,
    pub source_span_ids: Vec<SpanId>,
    pub certificate_ids: Vec<CertificateId>,
}
