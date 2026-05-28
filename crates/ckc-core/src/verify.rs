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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::{content_hash, to_canonical_bytes};

    fn fixture_conflict() -> Conflict {
        Conflict {
            conflict_id: ConflictId::new("conflict_sepsis_beta_lactam_001"),
            conflict_type: "norm_contradiction".into(),
            severity: Severity::High,
            confidence: 0.95,
            minimal_artifact_set: vec![
                ContentHash("sha256:aaaa000000000000000000000000000000000000000000000000000000000001".into()),
                ContentHash("sha256:aaaa000000000000000000000000000000000000000000000000000000000002".into()),
            ],
            source_spans: vec![SpanId::new("span_s1"), SpanId::new("span_s2")],
            normalized_view: serde_json::json!({
                "rule_a": {"direction": "for", "action": "administer_beta_lactam"},
                "rule_b": {"direction": "against", "action": "administer_beta_lactam"},
                "shared_context": "sepsis AND beta_lactam_allergy"
            }),
            witness: Some(WitnessId::new("witness_conflict_001")),
            repair_candidates: vec![
                serde_json::json!({"type": "add_priority", "rule": "rule_contraindicate", "over": "rule_recommend"}),
                serde_json::json!({"type": "add_exception", "rule": "rule_recommend", "exception": "beta_lactam_allergy"}),
            ],
            solver_evidence: vec![
                serde_json::json!({"solver": "z3", "status": "sat", "core_size": 2}),
            ],
            argument_graph_id: Some(ArgumentGraphId::new("ag_sepsis_conflict_001")),
            human_review_question_ja: "βラクタムアレルギー患者への敗血症治療で、推奨と禁忌が矛盾しています。優先順位を確認してください。".into(),
            human_review_question_en: "For sepsis treatment in beta-lactam allergic patients, recommendation and contraindication conflict. Please verify priority.".into(),
            classification: ConflictClassification::TrueConflict,
        }
    }

    fn fixture_argument_graph() -> ArgumentGraph {
        ArgumentGraph {
            argument_graph_id: ArgumentGraphId::new("ag_sepsis_conflict_001"),
            arguments: vec![
                serde_json::json!({"id": "arg_recommend", "conclusion": "administer_beta_lactam", "premises": ["dx_sepsis", "adult_patient"]}),
                serde_json::json!({"id": "arg_contraindicate", "conclusion": "withhold_beta_lactam", "premises": ["allergy_beta_lactam"]}),
            ],
            attack_edges: vec![
                serde_json::json!({"from": "arg_contraindicate", "to": "arg_recommend", "type": "rebut"}),
            ],
            support_edges: vec![
                serde_json::json!({"from": "evidence_rct_001", "to": "arg_recommend"}),
            ],
            undercut_edges: vec![],
            defeat_edges: vec![
                serde_json::json!({"from": "arg_contraindicate", "to": "arg_recommend", "status": "unresolved"}),
            ],
            extension_summaries: vec![
                serde_json::json!({"semantics": "grounded", "accepted": ["arg_contraindicate"], "rejected": ["arg_recommend"]}),
            ],
            source_span_ids: vec![SpanId::new("span_s1"), SpanId::new("span_s2")],
        }
    }

    fn fixture_certificate() -> Certificate {
        Certificate {
            certificate_id: CertificateId::new("cert_z3_norm_conflict_001"),
            certificate_class: CertificateClass::C4Executable,
            input_artifact_hashes: vec![ContentHash(
                "sha256:bbbb000000000000000000000000000000000000000000000000000000000001".into(),
            )],
            compiler_hash: Some(ContentHash(
                "sha256:cccc000000000000000000000000000000000000000000000000000000000001".into(),
            )),
            solver_or_checker: "z3".into(),
            command_manifest: serde_json::json!({
                "command": "z3",
                "args": ["-smt2", "norm_conflict.smt2"],
                "timeout_ms": 30000
            }),
            result: "sat".into(),
            proof_artifact_hashes: vec![ContentHash(
                "sha256:dddd000000000000000000000000000000000000000000000000000000000001".into(),
            )],
            replay_status: ReplayStatus::Passed,
            diagnostics: vec![
                serde_json::json!({"level": "info", "message": "satisfying assignment found in 42ms"}),
            ],
        }
    }

    fn fixture_assurance_node() -> AssuranceNode {
        AssuranceNode {
            node_id: AssuranceNodeId::new("goal_top_001"),
            node_type: "goal".into(),
            claim:
                "Accepted CKC artifacts are source-grounded, deterministic, and formally checkable"
                    .into(),
            evidence_artifact_ids: vec![ContentHash(
                "sha256:eeee000000000000000000000000000000000000000000000000000000000001".into(),
            )],
            status: "supported".into(),
            children: vec![
                AssuranceNodeId::new("strategy_verification_001"),
                AssuranceNodeId::new("strategy_grounding_001"),
            ],
        }
    }

    fn fixture_audit_trace() -> AuditTrace {
        AuditTrace {
            trace_id: AuditTraceId::new("trace_run_toy_001"),
            stage_spans: vec![
                serde_json::json!({"stage": "normalize", "start_ms": 0, "end_ms": 12, "status": "ok"}),
                serde_json::json!({"stage": "compile_smt", "start_ms": 12, "end_ms": 45, "status": "ok"}),
                serde_json::json!({"stage": "verify_z3", "start_ms": 45, "end_ms": 87, "status": "ok"}),
            ],
            model_invocations: vec![
                serde_json::json!({"model": "gpt-4o", "purpose": "formalize", "input_hash": "sha256:ff01", "output_hash": "sha256:ff02", "tokens_in": 1200, "tokens_out": 450}),
            ],
            retrieval_events: vec![
                serde_json::json!({"query": "敗血症 βラクタム", "analyzer": "kuromoji", "hits": 5, "top_span": "span_s1"}),
            ],
            verifier_events: vec![
                serde_json::json!({"verifier": "z3", "input_hash": "sha256:bb01", "result": "sat", "elapsed_ms": 42}),
                serde_json::json!({"verifier": "clingo", "input_hash": "sha256:bb02", "result": "SAT", "models": 1, "elapsed_ms": 15}),
            ],
            artifact_hashes: vec![
                ContentHash(
                    "sha256:ffff000000000000000000000000000000000000000000000000000000000001"
                        .into(),
                ),
                ContentHash(
                    "sha256:ffff000000000000000000000000000000000000000000000000000000000002"
                        .into(),
                ),
            ],
            redaction_status: "none".into(),
            audit_export_refs: vec!["runs/toy/audit/trace_run_toy_001.jsonl".into()],
        }
    }

    // -- Serde round-trip tests --

    #[test]
    fn conflict_roundtrip() {
        let c = fixture_conflict();
        let json = serde_json::to_string(&c).unwrap();
        let rt: Conflict = serde_json::from_str(&json).unwrap();
        assert_eq!(c, rt);
    }

    #[test]
    fn argument_graph_roundtrip() {
        let ag = fixture_argument_graph();
        let json = serde_json::to_string(&ag).unwrap();
        let rt: ArgumentGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(ag, rt);
    }

    #[test]
    fn certificate_roundtrip() {
        let cert = fixture_certificate();
        let json = serde_json::to_string(&cert).unwrap();
        let rt: Certificate = serde_json::from_str(&json).unwrap();
        assert_eq!(cert, rt);
    }

    #[test]
    fn assurance_node_roundtrip() {
        let node = fixture_assurance_node();
        let json = serde_json::to_string(&node).unwrap();
        let rt: AssuranceNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, rt);
    }

    #[test]
    fn audit_trace_roundtrip() {
        let trace = fixture_audit_trace();
        let json = serde_json::to_string(&trace).unwrap();
        let rt: AuditTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(trace, rt);
    }

    // -- Optional field omission --

    #[test]
    fn conflict_optional_witness_omitted() {
        let mut c = fixture_conflict();
        c.witness = None;
        let json = serde_json::to_string(&c).unwrap();
        assert!(!json.contains("\"witness\""));
        let rt: Conflict = serde_json::from_str(&json).unwrap();
        assert_eq!(c, rt);
    }

    #[test]
    fn conflict_optional_argument_graph_id_omitted() {
        let mut c = fixture_conflict();
        c.argument_graph_id = None;
        let json = serde_json::to_string(&c).unwrap();
        assert!(!json.contains("argument_graph_id"));
        let rt: Conflict = serde_json::from_str(&json).unwrap();
        assert_eq!(c, rt);
    }

    #[test]
    fn certificate_optional_compiler_hash_omitted() {
        let mut cert = fixture_certificate();
        cert.compiler_hash = None;
        let json = serde_json::to_string(&cert).unwrap();
        assert!(!json.contains("compiler_hash"));
        let rt: Certificate = serde_json::from_str(&json).unwrap();
        assert_eq!(cert, rt);
    }

    // -- Canonical JSON byte stability --

    #[test]
    fn conflict_canonical_stability() {
        let c = fixture_conflict();
        assert_eq!(to_canonical_bytes(&c), to_canonical_bytes(&c));
        assert_eq!(content_hash(&c), content_hash(&c));
    }

    #[test]
    fn argument_graph_canonical_stability() {
        let ag = fixture_argument_graph();
        assert_eq!(to_canonical_bytes(&ag), to_canonical_bytes(&ag));
        assert_eq!(content_hash(&ag), content_hash(&ag));
    }

    #[test]
    fn certificate_canonical_stability() {
        let cert = fixture_certificate();
        assert_eq!(to_canonical_bytes(&cert), to_canonical_bytes(&cert));
        assert_eq!(content_hash(&cert), content_hash(&cert));
    }

    #[test]
    fn assurance_node_canonical_stability() {
        let node = fixture_assurance_node();
        assert_eq!(to_canonical_bytes(&node), to_canonical_bytes(&node));
        assert_eq!(content_hash(&node), content_hash(&node));
    }

    #[test]
    fn audit_trace_canonical_stability() {
        let trace = fixture_audit_trace();
        assert_eq!(to_canonical_bytes(&trace), to_canonical_bytes(&trace));
        assert_eq!(content_hash(&trace), content_hash(&trace));
    }

    // -- Cross-type referential consistency --

    #[test]
    fn conflict_references_argument_graph() {
        let c = fixture_conflict();
        let ag = fixture_argument_graph();
        assert_eq!(c.argument_graph_id, Some(ag.argument_graph_id));
    }

    #[test]
    fn conflict_and_argument_graph_share_source_spans() {
        let c = fixture_conflict();
        let ag = fixture_argument_graph();
        assert_eq!(c.source_spans, ag.source_span_ids);
    }

    #[test]
    fn assurance_node_children_are_valid_ids() {
        let node = fixture_assurance_node();
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].as_str(), "strategy_verification_001");
        assert_eq!(node.children[1].as_str(), "strategy_grounding_001");
    }

    #[test]
    fn certificate_input_hashes_nonempty() {
        let cert = fixture_certificate();
        assert!(!cert.input_artifact_hashes.is_empty());
        assert!(
            cert.input_artifact_hashes[0]
                .as_str()
                .starts_with("sha256:")
        );
    }

    // -- Empty arrays are valid --

    #[test]
    fn argument_graph_empty_edges() {
        let ag = ArgumentGraph {
            argument_graph_id: ArgumentGraphId::new("ag_empty"),
            arguments: vec![],
            attack_edges: vec![],
            support_edges: vec![],
            undercut_edges: vec![],
            defeat_edges: vec![],
            extension_summaries: vec![],
            source_span_ids: vec![],
        };
        let json = serde_json::to_string(&ag).unwrap();
        let rt: ArgumentGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(ag, rt);
    }

    #[test]
    fn audit_trace_empty_arrays() {
        let trace = AuditTrace {
            trace_id: AuditTraceId::new("trace_empty"),
            stage_spans: vec![],
            model_invocations: vec![],
            retrieval_events: vec![],
            verifier_events: vec![],
            artifact_hashes: vec![],
            redaction_status: "none".into(),
            audit_export_refs: vec![],
        };
        let json = serde_json::to_string(&trace).unwrap();
        let rt: AuditTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(trace, rt);
    }

    #[test]
    fn assurance_node_leaf_no_children() {
        let leaf = AssuranceNode {
            node_id: AssuranceNodeId::new("solution_leaf_001"),
            node_type: "solution".into(),
            claim: "Z3 proves norm conflict satisfiable".into(),
            evidence_artifact_ids: vec![ContentHash(
                "sha256:eeee000000000000000000000000000000000000000000000000000000000001".into(),
            )],
            status: "supported".into(),
            children: vec![],
        };
        let json = serde_json::to_string(&leaf).unwrap();
        let rt: AssuranceNode = serde_json::from_str(&json).unwrap();
        assert_eq!(leaf, rt);
    }

    // -- Enum serialization in context --

    #[test]
    fn conflict_classification_serializes_correctly() {
        let c = fixture_conflict();
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"true_conflict\""));
    }

    #[test]
    fn certificate_class_serializes_correctly() {
        let cert = fixture_certificate();
        let json = serde_json::to_string(&cert).unwrap();
        assert!(json.contains("\"C4-Executable\""));
    }

    #[test]
    fn replay_status_serializes_correctly() {
        let cert = fixture_certificate();
        let json = serde_json::to_string(&cert).unwrap();
        assert!(json.contains("\"passed\""));
    }

    #[test]
    fn severity_serializes_correctly() {
        let c = fixture_conflict();
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"high\""));
    }

    // -- Distinct fixtures produce distinct hashes --

    #[test]
    fn distinct_types_distinct_hashes() {
        let h_conflict = content_hash(&fixture_conflict());
        let h_ag = content_hash(&fixture_argument_graph());
        let h_cert = content_hash(&fixture_certificate());
        let h_node = content_hash(&fixture_assurance_node());
        let h_trace = content_hash(&fixture_audit_trace());

        let hashes = [&h_conflict, &h_ag, &h_cert, &h_node, &h_trace];
        for (i, a) in hashes.iter().enumerate() {
            for b in hashes.iter().skip(i + 1) {
                assert_ne!(a, b, "hash collision between fixture types");
            }
        }
    }
}
