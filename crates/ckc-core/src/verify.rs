use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::canonical::ContentHash;
use crate::enums::{CertificateClass, ConflictClassification, ReplayStatus, Severity};
use crate::id::{
    ArgumentGraphId, AssuranceNodeId, AuditTraceId, CertificateId, ConflictId, SpanId, WitnessId,
};

// ---------------------------------------------------------------------------
// SPEC 10 types: verification and assurance
// ---------------------------------------------------------------------------

/// Detected logical incompatibility or factual inconsistency with
/// source-grounded evidence and review prompts (SPEC 10, 15).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Conflict {
    pub conflict_id: ConflictId,
    pub conflict_type: String,
    pub severity: Severity,
    pub confidence: f64,
    pub minimal_artifact_set: Vec<ContentHash>,
    pub source_spans: Vec<SpanId>,
    pub normalized_view: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<WitnessId>,
    pub repair_candidates: Vec<Value>,
    pub solver_evidence: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument_graph_id: Option<ArgumentGraphId>,
    pub human_review_question_ja: String,
    pub human_review_question_en: String,
    pub classification: ConflictClassification,
}

/// Dung-style argumentation graph with typed edges and extension
/// summaries (SPEC 10, 13.3).
/// Arrays `arguments`, `attack_edges`, `support_edges`, `undercut_edges`,
/// `defeat_edges`, and `extension_summaries` use open JSON values at
/// schema v0.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ArgumentGraph {
    pub argument_graph_id: ArgumentGraphId,
    pub arguments: Vec<Value>,
    pub attack_edges: Vec<Value>,
    pub support_edges: Vec<Value>,
    pub undercut_edges: Vec<Value>,
    pub defeat_edges: Vec<Value>,
    pub extension_summaries: Vec<Value>,
    pub source_span_ids: Vec<SpanId>,
}

/// Proof or verification certificate linking input artifacts to
/// solver/checker results and replay status (SPEC 10, 12.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Certificate {
    pub certificate_id: CertificateId,
    pub certificate_class: CertificateClass,
    pub input_artifact_hashes: Vec<ContentHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler_hash: Option<ContentHash>,
    pub solver_or_checker: String,
    pub command_manifest: Value,
    pub result: String,
    pub proof_artifact_hashes: Vec<ContentHash>,
    pub replay_status: ReplayStatus,
    pub diagnostics: Vec<Value>,
}

/// GSN/SACM-style assurance-case node linking claims to evidence
/// artifacts (SPEC 10, 17).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AssuranceNode {
    pub node_id: AssuranceNodeId,
    pub node_type: String,
    pub claim: String,
    pub evidence_artifact_ids: Vec<ContentHash>,
    pub status: String,
    pub children: Vec<AssuranceNodeId>,
}

/// Research observability trace covering pipeline stages, model
/// invocations, retrieval events, and verifier events (SPEC 10, 17).
/// Trace arrays use open JSON values at schema v0 for
/// OpenTelemetry-style span data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AuditTrace {
    pub trace_id: AuditTraceId,
    pub stage_spans: Vec<Value>,
    pub model_invocations: Vec<Value>,
    pub retrieval_events: Vec<Value>,
    pub verifier_events: Vec<Value>,
    pub artifact_hashes: Vec<ContentHash>,
    pub redaction_status: String,
    pub audit_export_refs: Vec<String>,
}
