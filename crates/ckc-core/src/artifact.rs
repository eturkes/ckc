use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::enums::{CaseType, HitPolicy};
use crate::id::{
    BundleId, CaseId, CertificateId, DecisionRowId, DecisionTableId, DmnExportId,
    ExtractedTableId, RuleId, SpanId, WitnessId, WorkflowId,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::{content_hash, to_canonical_bytes};

    fn fixture_decision_row() -> DecisionRow {
        DecisionRow {
            row_id: DecisionRowId::new("row_vitals_001"),
            conditions: vec![
                serde_json::json!({"field": "temperature", "op": ">=", "value": 38.0}),
                serde_json::json!({"field": "heart_rate", "op": ">=", "value": 90}),
            ],
            outputs: vec![
                serde_json::json!({"action": "sepsis_alert", "level": "high"}),
            ],
            priority: Some(1),
            source_span_ids: vec![SpanId::new("span_tbl_r1")],
            cell_refs: vec![TableCellRef {
                table_id: ExtractedTableId::new("tbl_vitals_001"),
                row: 1,
                col: 0,
            }],
        }
    }

    fn fixture_decision_table() -> DecisionTable {
        DecisionTable {
            table_id: DecisionTableId::new("dt_vitals_triage"),
            hit_policy: HitPolicy::Priority,
            input_columns: vec!["temperature".into(), "heart_rate".into()],
            output_columns: vec!["alert_action".into()],
            rows: vec![fixture_decision_row()],
            source_table_id: Some(ExtractedTableId::new("tbl_vitals_001")),
            dmn_export_id: Some(DmnExportId::new("dmn_vitals_triage_001")),
            certificate_ids: vec![],
        }
    }

    fn fixture_workflow_fragment() -> WorkflowFragment {
        WorkflowFragment {
            workflow_id: WorkflowId::new("wf_sepsis_pathway"),
            workflow_type: "epath_oat".into(),
            states: vec![
                serde_json::json!({"id": "triage", "label": "初期トリアージ"}),
                serde_json::json!({"id": "abx_admin", "label": "抗菌薬投与"}),
                serde_json::json!({"id": "monitoring", "label": "経過観察"}),
            ],
            transitions: vec![
                serde_json::json!({"from": "triage", "to": "abx_admin", "condition": "sepsis_confirmed"}),
                serde_json::json!({"from": "abx_admin", "to": "monitoring", "condition": "dose_given"}),
            ],
            outcomes: vec![
                serde_json::json!({"id": "recovery", "label": "回復"}),
            ],
            assessments: vec![
                serde_json::json!({"id": "sofa_score", "label": "SOFAスコア"}),
            ],
            tasks: vec![
                serde_json::json!({"id": "blood_culture", "label": "血液培養採取"}),
            ],
            variance_rules: vec![
                serde_json::json!({"trigger": "allergy_detected", "action": "switch_abx"}),
            ],
            source_span_ids: vec![SpanId::new("span_pathway_001")],
        }
    }

    fn fixture_event_narrative() -> EventNarrative {
        EventNarrative {
            event_types: vec!["administer_drug".into(), "detect_allergy".into()],
            fluent_types: vec!["allergy_known".into(), "drug_active".into()],
            happens: vec![
                serde_json::json!({"event": "detect_allergy", "time": 0}),
                serde_json::json!({"event": "administer_drug", "time": 10}),
            ],
            initiates: vec![
                serde_json::json!({"event": "detect_allergy", "fluent": "allergy_known", "time": 0}),
                serde_json::json!({"event": "administer_drug", "fluent": "drug_active", "time": 10}),
            ],
            terminates: vec![
                serde_json::json!({"event": "clear_allergy", "fluent": "allergy_known"}),
            ],
            initially: vec![
                serde_json::json!({"fluent": "allergy_known", "value": false}),
            ],
            holds_queries: vec![
                serde_json::json!({"fluent": "allergy_known", "time": 10, "expected": true}),
            ],
            source_span_ids: vec![SpanId::new("span_ec_001"), SpanId::new("span_ec_002")],
        }
    }

    fn fixture_patient_case() -> PatientCase {
        PatientCase {
            case_id: CaseId::new("case_sepsis_allergy_001"),
            case_type: CaseType::Synthetic,
            facts: vec![
                serde_json::json!({"type": "diagnosis", "code": "sepsis", "active": true}),
                serde_json::json!({"type": "age", "value": 65, "unit": "years"}),
            ],
            events: vec![
                serde_json::json!({"type": "admission", "time": "2024-01-15T08:00:00Z"}),
            ],
            observations: vec![
                serde_json::json!({"type": "temperature", "value": 39.2, "unit": "celsius"}),
                serde_json::json!({"type": "heart_rate", "value": 110, "unit": "bpm"}),
            ],
            medications: vec![
                serde_json::json!({"drug": "beta_lactam", "route": "iv", "status": "proposed"}),
            ],
            conditions: vec![
                serde_json::json!({"code": "sepsis", "onset": "2024-01-15"}),
            ],
            allergies: vec![
                serde_json::json!({"substance": "beta_lactam", "reaction": "anaphylaxis", "severity": "severe"}),
            ],
            time_origin: Some("2024-01-15T08:00:00Z".into()),
            source_span_ids: vec![SpanId::new("span_case_001")],
            privacy_status: "synthetic".into(),
        }
    }

    fn fixture_execution_witness() -> ExecutionWitness {
        ExecutionWitness {
            witness_id: WitnessId::new("witness_conflict_001"),
            bundle_id: BundleId::new("bundle_sepsis_toy"),
            case_id: Some(CaseId::new("case_sepsis_allergy_001")),
            context_facts: vec![
                serde_json::json!({"fact": "dx_sepsis", "value": true}),
                serde_json::json!({"fact": "allergy_beta_lactam", "value": true}),
            ],
            trace: vec![
                serde_json::json!({"step": 1, "rule": "rule_recommend_beta_lactam", "result": "applicable"}),
                serde_json::json!({"step": 2, "rule": "rule_contraindicate_beta_lactam", "result": "applicable"}),
                serde_json::json!({"step": 3, "conflict": "norm_conflict_detected"}),
            ],
            applicable_rules: vec![
                RuleId::new("rule_recommend_beta_lactam"),
                RuleId::new("rule_contraindicate_beta_lactam"),
            ],
            defeated_rules: vec![],
            violated_constraints: vec!["no_concurrent_recommend_and_contraindicate".into()],
            models: vec![
                serde_json::json!({"solver": "z3", "status": "sat", "model": {"sepsis": true, "allergy": true}}),
            ],
            unsat_cores: vec![],
            source_span_ids: vec![SpanId::new("span_s1"), SpanId::new("span_s2")],
            certificate_ids: vec![],
        }
    }

    // -- Serde round-trip tests --

    #[test]
    fn decision_row_roundtrip() {
        let row = fixture_decision_row();
        let json = serde_json::to_string(&row).unwrap();
        let rt: DecisionRow = serde_json::from_str(&json).unwrap();
        assert_eq!(row, rt);
    }

    #[test]
    fn decision_table_roundtrip() {
        let table = fixture_decision_table();
        let json = serde_json::to_string(&table).unwrap();
        let rt: DecisionTable = serde_json::from_str(&json).unwrap();
        assert_eq!(table, rt);
    }

    #[test]
    fn workflow_fragment_roundtrip() {
        let wf = fixture_workflow_fragment();
        let json = serde_json::to_string(&wf).unwrap();
        let rt: WorkflowFragment = serde_json::from_str(&json).unwrap();
        assert_eq!(wf, rt);
    }

    #[test]
    fn event_narrative_roundtrip() {
        let en = fixture_event_narrative();
        let json = serde_json::to_string(&en).unwrap();
        let rt: EventNarrative = serde_json::from_str(&json).unwrap();
        assert_eq!(en, rt);
    }

    #[test]
    fn patient_case_roundtrip() {
        let pc = fixture_patient_case();
        let json = serde_json::to_string(&pc).unwrap();
        let rt: PatientCase = serde_json::from_str(&json).unwrap();
        assert_eq!(pc, rt);
    }

    #[test]
    fn execution_witness_roundtrip() {
        let ew = fixture_execution_witness();
        let json = serde_json::to_string(&ew).unwrap();
        let rt: ExecutionWitness = serde_json::from_str(&json).unwrap();
        assert_eq!(ew, rt);
    }

    // -- Optional field omission --

    #[test]
    fn decision_row_optional_priority_omitted() {
        let mut row = fixture_decision_row();
        row.priority = None;
        let json = serde_json::to_string(&row).unwrap();
        assert!(!json.contains("priority"));
        let rt: DecisionRow = serde_json::from_str(&json).unwrap();
        assert_eq!(row, rt);
    }

    #[test]
    fn decision_table_optional_fields_omitted() {
        let mut table = fixture_decision_table();
        table.source_table_id = None;
        table.dmn_export_id = None;
        let json = serde_json::to_string(&table).unwrap();
        assert!(!json.contains("source_table_id"));
        assert!(!json.contains("dmn_export_id"));
        let rt: DecisionTable = serde_json::from_str(&json).unwrap();
        assert_eq!(table, rt);
    }

    #[test]
    fn patient_case_optional_time_origin_omitted() {
        let mut pc = fixture_patient_case();
        pc.time_origin = None;
        let json = serde_json::to_string(&pc).unwrap();
        assert!(!json.contains("time_origin"));
        let rt: PatientCase = serde_json::from_str(&json).unwrap();
        assert_eq!(pc, rt);
    }

    #[test]
    fn execution_witness_optional_case_id_omitted() {
        let mut ew = fixture_execution_witness();
        ew.case_id = None;
        let json = serde_json::to_string(&ew).unwrap();
        assert!(!json.contains("case_id"));
        let rt: ExecutionWitness = serde_json::from_str(&json).unwrap();
        assert_eq!(ew, rt);
    }

    // -- Canonical JSON byte stability --

    #[test]
    fn decision_row_canonical_stability() {
        let row = fixture_decision_row();
        assert_eq!(to_canonical_bytes(&row), to_canonical_bytes(&row));
        assert_eq!(content_hash(&row), content_hash(&row));
    }

    #[test]
    fn decision_table_canonical_stability() {
        let table = fixture_decision_table();
        assert_eq!(to_canonical_bytes(&table), to_canonical_bytes(&table));
        assert_eq!(content_hash(&table), content_hash(&table));
    }

    #[test]
    fn workflow_fragment_canonical_stability() {
        let wf = fixture_workflow_fragment();
        assert_eq!(to_canonical_bytes(&wf), to_canonical_bytes(&wf));
        assert_eq!(content_hash(&wf), content_hash(&wf));
    }

    #[test]
    fn event_narrative_canonical_stability() {
        let en = fixture_event_narrative();
        assert_eq!(to_canonical_bytes(&en), to_canonical_bytes(&en));
        assert_eq!(content_hash(&en), content_hash(&en));
    }

    #[test]
    fn patient_case_canonical_stability() {
        let pc = fixture_patient_case();
        assert_eq!(to_canonical_bytes(&pc), to_canonical_bytes(&pc));
        assert_eq!(content_hash(&pc), content_hash(&pc));
    }

    #[test]
    fn execution_witness_canonical_stability() {
        let ew = fixture_execution_witness();
        assert_eq!(to_canonical_bytes(&ew), to_canonical_bytes(&ew));
        assert_eq!(content_hash(&ew), content_hash(&ew));
    }

    // -- Cross-type referential consistency --

    #[test]
    fn decision_table_contains_its_rows() {
        let table = fixture_decision_table();
        let row = fixture_decision_row();
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].row_id, row.row_id);
    }

    #[test]
    fn execution_witness_references_patient_case() {
        let ew = fixture_execution_witness();
        let pc = fixture_patient_case();
        assert_eq!(ew.case_id, Some(pc.case_id));
    }

    #[test]
    fn decision_table_source_table_ref() {
        let table = fixture_decision_table();
        assert_eq!(
            table.source_table_id,
            Some(ExtractedTableId::new("tbl_vitals_001"))
        );
    }

    // -- Empty arrays are valid --

    #[test]
    fn workflow_fragment_empty_arrays() {
        let wf = WorkflowFragment {
            workflow_id: WorkflowId::new("wf_empty"),
            workflow_type: "stub".into(),
            states: vec![],
            transitions: vec![],
            outcomes: vec![],
            assessments: vec![],
            tasks: vec![],
            variance_rules: vec![],
            source_span_ids: vec![],
        };
        let json = serde_json::to_string(&wf).unwrap();
        let rt: WorkflowFragment = serde_json::from_str(&json).unwrap();
        assert_eq!(wf, rt);
    }

    #[test]
    fn event_narrative_empty_arrays() {
        let en = EventNarrative {
            event_types: vec![],
            fluent_types: vec![],
            happens: vec![],
            initiates: vec![],
            terminates: vec![],
            initially: vec![],
            holds_queries: vec![],
            source_span_ids: vec![],
        };
        let json = serde_json::to_string(&en).unwrap();
        let rt: EventNarrative = serde_json::from_str(&json).unwrap();
        assert_eq!(en, rt);
    }

    #[test]
    fn patient_case_fixture_type() {
        let mut pc = fixture_patient_case();
        pc.case_type = CaseType::Fixture;
        let json = serde_json::to_string(&pc).unwrap();
        assert!(json.contains("\"fixture\""));
        let rt: PatientCase = serde_json::from_str(&json).unwrap();
        assert_eq!(pc.case_type, rt.case_type);
    }

    // -- Distinct fixtures produce distinct hashes --

    #[test]
    fn distinct_types_distinct_hashes() {
        let h_row = content_hash(&fixture_decision_row());
        let h_table = content_hash(&fixture_decision_table());
        let h_wf = content_hash(&fixture_workflow_fragment());
        let h_en = content_hash(&fixture_event_narrative());
        let h_pc = content_hash(&fixture_patient_case());
        let h_ew = content_hash(&fixture_execution_witness());

        let hashes = [&h_row, &h_table, &h_wf, &h_en, &h_pc, &h_ew];
        for (i, a) in hashes.iter().enumerate() {
            for b in hashes.iter().skip(i + 1) {
                assert_ne!(a, b, "hash collision between fixture types");
            }
        }
    }
}
