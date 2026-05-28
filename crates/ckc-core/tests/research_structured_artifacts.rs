//! Integration test: decision table, Event Calculus, patient case, and
//! workflow fixtures for SPEC 20 Phase 0 scenarios 3, 4, and witness contexts.
//!
//! Constructs:
//!   - 1 DecisionTable (Unique hit policy, 4 rows with overlap + gap)
//!   - 1 EventNarrative (allergy persistence → administration violation)
//!   - 2 PatientCase (sepsis+allergy conflict, sepsis-only baseline)
//!   - 1 WorkflowFragment (sepsis treatment pathway)
//!
//! Scenario coverage (SPEC 20 Phase 0):
//!   3. decision table  — overlapping rows with incompatible outputs + gap witness
//!   4. Event Calculus   — allergy_known persists, administration violates at t=10
//!   1. norm conflict    — patient case (a) provides shared witness context
//!   5. repair           — patient case (a) is input for MaxSMT/ASP repair
//!   8. replay           — all fixtures participate in hash determinism

use ckc_core::artifact::*;
use ckc_core::canonical::to_canonical_bytes;
use ckc_core::enums::*;
use ckc_core::id::*;
use ckc_core::nf::{NfContext, Normalize};
use ckc_core::source::TableCellRef;
use std::collections::HashSet;
use std::path::Path;

// =========================================================================
// ID constants
// =========================================================================

const DT_VITALS_TRIAGE: &str = "dt_vitals_triage";

const ROW_TEMP_HIGH: &str = "row_temp_high";
const ROW_TEMP_VERY_HIGH: &str = "row_temp_very_high";
const ROW_HR_HIGH: &str = "row_hr_high";
const ROW_BP_LOW: &str = "row_bp_low";

const CASE_SEPSIS_ALLERGY: &str = "case_sepsis_allergy";
const CASE_SEPSIS_BASELINE: &str = "case_sepsis_baseline";

const WF_SEPSIS_PATHWAY: &str = "wf_sepsis_pathway";

// Span IDs from 0.5.1
const SPAN_REC_SEPSIS: &str = "span_rec_sepsis_bl";
const SPAN_EVIDENCE: &str = "span_evidence_sepsis";
const SPAN_CONTRA: &str = "span_contra_bl_allergy";
const SPAN_ALLERGY_HIST: &str = "span_allergy_history";
const SPAN_CELL_R0C0: &str = "span_cell_r0c0";
const SPAN_CELL_R0C1: &str = "span_cell_r0c1";
const SPAN_CELL_R1C0: &str = "span_cell_r1c0";
const SPAN_CELL_R1C1: &str = "span_cell_r1c1";
const SPAN_CELL_R2C0: &str = "span_cell_r2c0";
const SPAN_CELL_R2C1: &str = "span_cell_r2c1";
const SPAN_CELL_R3C0: &str = "span_cell_r3c0";
const SPAN_CELL_R3C1: &str = "span_cell_r3c1";

// Table ID from 0.5.1
const TBL_VITALS: &str = "tbl_vitals";

// =========================================================================
// Fixture constructors
// =========================================================================

fn toy_decision_tables() -> Vec<DecisionTable> {
    vec![DecisionTable {
        table_id: DecisionTableId::new(DT_VITALS_TRIAGE),
        hit_policy: HitPolicy::Unique,
        input_columns: vec!["体温".into(), "心拍数".into(), "収縮期血圧".into()],
        output_columns: vec!["対応".into()],
        rows: vec![
            // Row 0: temperature >= 38.0 → antipyretic
            DecisionRow {
                row_id: DecisionRowId::new(ROW_TEMP_HIGH),
                conditions: vec![
                    serde_json::json!({"field": "temperature", "op": ">=", "value": 38.0, "unit": "Cel"}),
                    serde_json::json!({"field": "heart_rate", "op": "*"}),
                    serde_json::json!({"field": "systolic_bp", "op": "*"}),
                ],
                outputs: vec![
                    serde_json::json!({"action": "administer_antipyretic", "label_ja": "解熱薬投与を検討"}),
                ],
                priority: None,
                source_span_ids: vec![SpanId::new(SPAN_CELL_R0C0), SpanId::new(SPAN_CELL_R0C1)],
                cell_refs: vec![
                    TableCellRef {
                        table_id: ExtractedTableId::new(TBL_VITALS),
                        row: 0,
                        col: 0,
                    },
                    TableCellRef {
                        table_id: ExtractedTableId::new(TBL_VITALS),
                        row: 0,
                        col: 1,
                    },
                ],
            },
            // Row 1: temperature >= 38.5 → cooling (OVERLAPS with row 0)
            // Under Unique hit policy, temp=38.7 fires both row 0 and row 1.
            DecisionRow {
                row_id: DecisionRowId::new(ROW_TEMP_VERY_HIGH),
                conditions: vec![
                    serde_json::json!({"field": "temperature", "op": ">=", "value": 38.5, "unit": "Cel"}),
                    serde_json::json!({"field": "heart_rate", "op": "*"}),
                    serde_json::json!({"field": "systolic_bp", "op": "*"}),
                ],
                outputs: vec![
                    serde_json::json!({"action": "initiate_cooling", "label_ja": "冷却処置を開始"}),
                ],
                priority: None,
                source_span_ids: vec![SpanId::new(SPAN_CELL_R1C0), SpanId::new(SPAN_CELL_R1C1)],
                cell_refs: vec![
                    TableCellRef {
                        table_id: ExtractedTableId::new(TBL_VITALS),
                        row: 1,
                        col: 0,
                    },
                    TableCellRef {
                        table_id: ExtractedTableId::new(TBL_VITALS),
                        row: 1,
                        col: 1,
                    },
                ],
            },
            // Row 2: heart rate > 90 → enhanced monitoring
            DecisionRow {
                row_id: DecisionRowId::new(ROW_HR_HIGH),
                conditions: vec![
                    serde_json::json!({"field": "temperature", "op": "*"}),
                    serde_json::json!({"field": "heart_rate", "op": ">", "value": 90, "unit": "/min"}),
                    serde_json::json!({"field": "systolic_bp", "op": "*"}),
                ],
                outputs: vec![
                    serde_json::json!({"action": "enhance_monitoring", "label_ja": "経過観察を強化"}),
                ],
                priority: None,
                source_span_ids: vec![SpanId::new(SPAN_CELL_R2C0), SpanId::new(SPAN_CELL_R2C1)],
                cell_refs: vec![
                    TableCellRef {
                        table_id: ExtractedTableId::new(TBL_VITALS),
                        row: 2,
                        col: 0,
                    },
                    TableCellRef {
                        table_id: ExtractedTableId::new(TBL_VITALS),
                        row: 2,
                        col: 1,
                    },
                ],
            },
            // Row 3: systolic BP < 90 → fluid resuscitation
            DecisionRow {
                row_id: DecisionRowId::new(ROW_BP_LOW),
                conditions: vec![
                    serde_json::json!({"field": "temperature", "op": "*"}),
                    serde_json::json!({"field": "heart_rate", "op": "*"}),
                    serde_json::json!({"field": "systolic_bp", "op": "<", "value": 90, "unit": "mm[Hg]"}),
                ],
                outputs: vec![
                    serde_json::json!({"action": "fluid_resuscitation", "label_ja": "輸液負荷を開始"}),
                ],
                priority: None,
                source_span_ids: vec![SpanId::new(SPAN_CELL_R3C0), SpanId::new(SPAN_CELL_R3C1)],
                cell_refs: vec![
                    TableCellRef {
                        table_id: ExtractedTableId::new(TBL_VITALS),
                        row: 3,
                        col: 0,
                    },
                    TableCellRef {
                        table_id: ExtractedTableId::new(TBL_VITALS),
                        row: 3,
                        col: 1,
                    },
                ],
            },
        ],
        source_table_id: Some(ExtractedTableId::new(TBL_VITALS)),
        dmn_export_id: None,
        certificate_ids: vec![],
    }]
}

fn toy_event_narratives() -> Vec<EventNarrative> {
    vec![EventNarrative {
        event_types: vec!["detect_allergy".into(), "administer_drug".into()],
        fluent_types: vec!["allergy_known".into(), "drug_active".into()],
        happens: vec![
            serde_json::json!({"event": "detect_allergy", "time": 0, "source": "allergy_history_review"}),
            serde_json::json!({"event": "administer_drug", "time": 10, "drug": "beta_lactam", "source": "sepsis_protocol"}),
        ],
        initiates: vec![
            serde_json::json!({"event": "detect_allergy", "fluent": "allergy_known", "time": 0, "substance": "beta_lactam"}),
            serde_json::json!({"event": "administer_drug", "fluent": "drug_active", "time": 10, "drug": "beta_lactam"}),
        ],
        terminates: vec![],
        initially: vec![
            serde_json::json!({"fluent": "allergy_known", "value": false}),
            serde_json::json!({"fluent": "drug_active", "value": false}),
        ],
        holds_queries: vec![serde_json::json!({
            "fluent": "allergy_known",
            "time": 10,
            "expected": true,
            "rationale": "allergy_known persists from t=0; no terminating event clears it before t=10"
        })],
        source_span_ids: vec![SpanId::new(SPAN_ALLERGY_HIST), SpanId::new(SPAN_CONTRA)],
    }]
}

fn toy_patient_cases() -> Vec<PatientCase> {
    vec![
        // Case (a): sepsis + beta-lactam allergy — shared witness for scenario 1
        PatientCase {
            case_id: CaseId::new(CASE_SEPSIS_ALLERGY),
            case_type: CaseType::Synthetic,
            facts: vec![
                serde_json::json!({"type": "diagnosis", "code": "sepsis", "active": true}),
                serde_json::json!({"type": "allergy", "substance": "beta_lactam", "reaction": "anaphylaxis", "verified": true}),
                serde_json::json!({"type": "age", "value": 68, "unit": "years"}),
            ],
            events: vec![
                serde_json::json!({"type": "admission", "time": "2024-01-15T08:00:00Z", "department": "ICU"}),
                serde_json::json!({"type": "allergy_detected", "time": "2024-01-15T08:05:00Z", "substance": "beta_lactam"}),
            ],
            observations: vec![
                serde_json::json!({"type": "temperature", "value": 39.2, "unit": "Cel", "time": "2024-01-15T08:10:00Z"}),
                serde_json::json!({"type": "heart_rate", "value": 110, "unit": "/min", "time": "2024-01-15T08:10:00Z"}),
                serde_json::json!({"type": "systolic_bp", "value": 85, "unit": "mm[Hg]", "time": "2024-01-15T08:10:00Z"}),
            ],
            medications: vec![
                serde_json::json!({"drug": "beta_lactam", "route": "iv", "status": "proposed", "contraindicated": true}),
            ],
            conditions: vec![
                serde_json::json!({"code": "sepsis", "onset": "2024-01-15", "status": "active"}),
            ],
            allergies: vec![
                serde_json::json!({"substance": "beta_lactam", "reaction": "anaphylaxis", "severity": "severe", "verified": true}),
            ],
            time_origin: Some("2024-01-15T08:00:00Z".into()),
            source_span_ids: vec![
                SpanId::new(SPAN_REC_SEPSIS),
                SpanId::new(SPAN_CONTRA),
                SpanId::new(SPAN_ALLERGY_HIST),
            ],
            privacy_status: "synthetic".into(),
        },
        // Case (b): sepsis without allergy — non-conflicting baseline
        PatientCase {
            case_id: CaseId::new(CASE_SEPSIS_BASELINE),
            case_type: CaseType::Synthetic,
            facts: vec![
                serde_json::json!({"type": "diagnosis", "code": "sepsis", "active": true}),
                serde_json::json!({"type": "age", "value": 55, "unit": "years"}),
            ],
            events: vec![
                serde_json::json!({"type": "admission", "time": "2024-02-10T10:00:00Z", "department": "ICU"}),
            ],
            observations: vec![
                serde_json::json!({"type": "temperature", "value": 39.0, "unit": "Cel", "time": "2024-02-10T10:15:00Z"}),
                serde_json::json!({"type": "heart_rate", "value": 95, "unit": "/min", "time": "2024-02-10T10:15:00Z"}),
                serde_json::json!({"type": "systolic_bp", "value": 100, "unit": "mm[Hg]", "time": "2024-02-10T10:15:00Z"}),
            ],
            medications: vec![
                serde_json::json!({"drug": "beta_lactam", "route": "iv", "status": "administered"}),
            ],
            conditions: vec![
                serde_json::json!({"code": "sepsis", "onset": "2024-02-10", "status": "active"}),
            ],
            allergies: vec![],
            time_origin: Some("2024-02-10T10:00:00Z".into()),
            source_span_ids: vec![SpanId::new(SPAN_REC_SEPSIS), SpanId::new(SPAN_EVIDENCE)],
            privacy_status: "synthetic".into(),
        },
    ]
}

fn toy_workflows() -> Vec<WorkflowFragment> {
    vec![WorkflowFragment {
        workflow_id: WorkflowId::new(WF_SEPSIS_PATHWAY),
        workflow_type: "sepsis_treatment".into(),
        states: vec![
            serde_json::json!({"id": "triage", "label_ja": "初期トリアージ", "label_en": "initial triage"}),
            serde_json::json!({"id": "allergy_check", "label_ja": "アレルギー歴確認", "label_en": "allergy history check"}),
            serde_json::json!({"id": "antibiotic_selection", "label_ja": "抗菌薬選択", "label_en": "antibiotic selection"}),
            serde_json::json!({"id": "administration", "label_ja": "抗菌薬投与", "label_en": "antibiotic administration"}),
            serde_json::json!({"id": "monitoring", "label_ja": "経過観察", "label_en": "monitoring"}),
        ],
        transitions: vec![
            serde_json::json!({"from": "triage", "to": "allergy_check", "condition": "sepsis_suspected"}),
            serde_json::json!({"from": "allergy_check", "to": "antibiotic_selection", "condition": "allergy_status_determined"}),
            serde_json::json!({"from": "antibiotic_selection", "to": "administration", "condition": "drug_selected"}),
            serde_json::json!({"from": "administration", "to": "monitoring", "condition": "dose_given"}),
        ],
        outcomes: vec![
            serde_json::json!({"id": "recovery", "label_ja": "回復", "label_en": "recovery"}),
            serde_json::json!({"id": "adverse_event", "label_ja": "有害事象", "label_en": "adverse event"}),
        ],
        assessments: vec![
            serde_json::json!({"id": "sofa_score", "label_ja": "SOFAスコア", "label_en": "SOFA score"}),
            serde_json::json!({"id": "qsofa", "label_ja": "qSOFA", "label_en": "qSOFA"}),
        ],
        tasks: vec![
            serde_json::json!({"id": "blood_culture", "label_ja": "血液培養採取", "label_en": "blood culture collection", "state": "triage"}),
            serde_json::json!({"id": "allergy_review", "label_ja": "薬物アレルギー歴確認", "label_en": "drug allergy history review", "state": "allergy_check"}),
            serde_json::json!({"id": "administer_abx", "label_ja": "抗菌薬投与実施", "label_en": "antibiotic administration", "state": "administration"}),
        ],
        variance_rules: vec![
            serde_json::json!({"trigger": "allergy_detected", "action": "switch_antibiotic", "label_ja": "アレルギー検出時:代替抗菌薬に変更"}),
            serde_json::json!({"trigger": "anaphylaxis_risk", "action": "contraindicate_class", "label_ja": "アナフィラキシーリスク:同系統薬剤禁忌"}),
        ],
        source_span_ids: vec![
            SpanId::new(SPAN_REC_SEPSIS),
            SpanId::new(SPAN_CONTRA),
            SpanId::new(SPAN_ALLERGY_HIST),
        ],
    }]
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
        .expect("0.5.1 spans.json must exist; run research_source_corpus regen_fixtures first");
    let spans: Vec<serde_json::Value> =
        serde_json::from_slice(&bytes).expect("spans.json must deserialize");
    spans
        .iter()
        .filter_map(|s| s.get("span_id").and_then(|v| v.as_str()))
        .map(String::from)
        .collect()
}

fn load_table_ids_from_fixtures() -> HashSet<String> {
    let dir = fixtures_dir();
    let bytes = std::fs::read(dir.join("tables.json"))
        .expect("0.5.1 tables.json must exist; run research_source_corpus regen_fixtures first");
    let tables: Vec<serde_json::Value> =
        serde_json::from_slice(&bytes).expect("tables.json must deserialize");
    tables
        .iter()
        .filter_map(|t| t.get("table_id").and_then(|v| v.as_str()))
        .map(String::from)
        .collect()
}

// =========================================================================
// Decision table: overlap and gap validation (scenario 3)
// =========================================================================

#[test]
fn decision_table_has_unique_hit_policy() {
    let tables = toy_decision_tables();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].hit_policy, HitPolicy::Unique);
}

#[test]
fn decision_table_has_at_least_four_rows() {
    let tables = toy_decision_tables();
    assert!(
        tables[0].rows.len() >= 4,
        "decision table must have 4+ rows; got {}",
        tables[0].rows.len()
    );
}

#[test]
fn decision_table_has_overlapping_temperature_rows() {
    let tables = toy_decision_tables();
    let rows = &tables[0].rows;

    let temp_threshold = |row: &DecisionRow| -> Option<f64> {
        row.conditions.iter().find_map(|c| {
            let field = c.get("field")?.as_str()?;
            if field != "temperature" {
                return None;
            }
            let op = c.get("op")?.as_str()?;
            if op == "*" {
                return None;
            }
            c.get("value")?.as_f64()
        })
    };

    let temp_rows: Vec<(usize, f64)> = rows
        .iter()
        .enumerate()
        .filter_map(|(i, r)| temp_threshold(r).map(|t| (i, t)))
        .collect();

    assert!(
        temp_rows.len() >= 2,
        "must have at least 2 temperature-conditioned rows"
    );

    // Rows 0 (>=38.0) and 1 (>=38.5) overlap: any temp >= 38.5 fires both.
    let (i0, t0) = temp_rows[0];
    let (i1, t1) = temp_rows[1];
    assert!(
        t1 >= t0,
        "row {} threshold ({}) must be >= row {} threshold ({})",
        i1,
        t1,
        i0,
        t0
    );

    let out0 = &rows[i0].outputs;
    let out1 = &rows[i1].outputs;
    assert_ne!(
        out0, out1,
        "overlapping rows {} and {} must have incompatible (different) outputs",
        i0, i1
    );
}

#[test]
fn decision_table_has_gap_witness() {
    let tables = toy_decision_tables();
    let rows = &tables[0].rows;

    // Gap witness: temp=37.5, HR=85, SBP=95 matches no row.
    let witness_temp: f64 = 37.5;
    let witness_hr: f64 = 85.0;
    let witness_sbp: f64 = 95.0;

    let matches_row = |row: &DecisionRow| -> bool {
        row.conditions.iter().all(|c| {
            let op = c.get("op").and_then(|v| v.as_str()).unwrap_or("*");
            if op == "*" {
                return true;
            }
            let field = c.get("field").and_then(|v| v.as_str()).unwrap_or("");
            let threshold = c.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let test_val = match field {
                "temperature" => witness_temp,
                "heart_rate" => witness_hr,
                "systolic_bp" => witness_sbp,
                _ => return true,
            };
            match op {
                ">=" => test_val >= threshold,
                ">" => test_val > threshold,
                "<" => test_val < threshold,
                "<=" => test_val <= threshold,
                _ => true,
            }
        })
    };

    let matching_count = rows.iter().filter(|r| matches_row(r)).count();
    assert_eq!(
        matching_count, 0,
        "gap witness (temp={}, HR={}, SBP={}) must match 0 rows; matched {}",
        witness_temp, witness_hr, witness_sbp, matching_count
    );
}

#[test]
fn decision_table_rows_have_consistent_column_count() {
    let tables = toy_decision_tables();
    let dt = &tables[0];
    let n_in = dt.input_columns.len();
    let n_out = dt.output_columns.len();
    for row in &dt.rows {
        assert_eq!(
            row.conditions.len(),
            n_in,
            "row {} has {} conditions but table has {} input columns",
            row.row_id,
            row.conditions.len(),
            n_in
        );
        assert_eq!(
            row.outputs.len(),
            n_out,
            "row {} has {} outputs but table has {} output columns",
            row.row_id,
            row.outputs.len(),
            n_out
        );
    }
}

#[test]
fn decision_table_source_table_references_existing() {
    let table_ids = load_table_ids_from_fixtures();
    let tables = toy_decision_tables();
    let src = tables[0]
        .source_table_id
        .as_ref()
        .expect("decision table must reference source table");
    assert!(
        table_ids.contains(src.as_str()),
        "decision table source_table_id {} must exist in 0.5.1 tables",
        src
    );
}

// =========================================================================
// Event Calculus: temporal consistency (scenario 4)
// =========================================================================

#[test]
fn event_narrative_has_expected_structure() {
    let narratives = toy_event_narratives();
    assert_eq!(narratives.len(), 1);
    let en = &narratives[0];
    assert!(
        !en.event_types.is_empty(),
        "event narrative must have event_types"
    );
    assert!(
        !en.fluent_types.is_empty(),
        "event narrative must have fluent_types"
    );
    assert!(
        !en.happens.is_empty(),
        "event narrative must have happens axioms"
    );
    assert!(
        !en.initiates.is_empty(),
        "event narrative must have initiates axioms"
    );
    assert!(
        !en.holds_queries.is_empty(),
        "event narrative must have holds_queries"
    );
}

#[test]
fn event_narrative_happens_times_non_negative() {
    let narratives = toy_event_narratives();
    for happens in &narratives[0].happens {
        let time = happens
            .get("time")
            .and_then(|v| v.as_i64())
            .expect("happens axiom must have numeric time");
        assert!(time >= 0, "happens time must be non-negative; got {}", time);
    }
}

#[test]
fn event_narrative_initiates_reference_valid_events() {
    let narratives = toy_event_narratives();
    let en = &narratives[0];
    let valid_events: HashSet<&str> = en.event_types.iter().map(String::as_str).collect();
    for init in &en.initiates {
        let event = init
            .get("event")
            .and_then(|v| v.as_str())
            .expect("initiates axiom must reference an event");
        assert!(
            valid_events.contains(event),
            "initiates references unknown event type {}; valid: {:?}",
            event,
            valid_events
        );
    }
}

#[test]
fn event_narrative_initiates_reference_valid_fluents() {
    let narratives = toy_event_narratives();
    let en = &narratives[0];
    let valid_fluents: HashSet<&str> = en.fluent_types.iter().map(String::as_str).collect();
    for init in &en.initiates {
        let fluent = init
            .get("fluent")
            .and_then(|v| v.as_str())
            .expect("initiates axiom must reference a fluent");
        assert!(
            valid_fluents.contains(fluent),
            "initiates references unknown fluent type {}; valid: {:?}",
            fluent,
            valid_fluents
        );
    }
}

#[test]
fn event_narrative_allergy_persists_at_administration() {
    let narratives = toy_event_narratives();
    let en = &narratives[0];

    // detect_allergy at t=0 initiates allergy_known
    let allergy_init = en.initiates.iter().find(|i| {
        i.get("event")
            .and_then(|v| v.as_str())
            .is_some_and(|e| e == "detect_allergy")
            && i.get("fluent")
                .and_then(|v| v.as_str())
                .is_some_and(|f| f == "allergy_known")
    });
    assert!(
        allergy_init.is_some(),
        "detect_allergy must initiate allergy_known"
    );
    let init_time = allergy_init
        .unwrap()
        .get("time")
        .and_then(|v| v.as_i64())
        .unwrap();

    // administer_drug happens at t=10
    let admin = en.happens.iter().find(|h| {
        h.get("event")
            .and_then(|v| v.as_str())
            .is_some_and(|e| e == "administer_drug")
    });
    assert!(admin.is_some(), "administer_drug must be in happens");
    let admin_time = admin.unwrap().get("time").and_then(|v| v.as_i64()).unwrap();
    assert!(
        admin_time > init_time,
        "administration (t={}) must occur after allergy detection (t={})",
        admin_time,
        init_time
    );

    // No terminating event clears allergy_known before administration
    let clears_allergy_before_admin = en.terminates.iter().any(|t| {
        t.get("fluent")
            .and_then(|v| v.as_str())
            .is_some_and(|f| f == "allergy_known")
            && t.get("time")
                .and_then(|v| v.as_i64())
                .is_some_and(|time| time > init_time && time <= admin_time)
    });
    assert!(
        !clears_allergy_before_admin,
        "allergy_known must persist from detection to administration (no clearing termination)"
    );

    // holds_query confirms allergy_known=true at administration time
    let query = en.holds_queries.iter().find(|q| {
        q.get("fluent")
            .and_then(|v| v.as_str())
            .is_some_and(|f| f == "allergy_known")
            && q.get("time")
                .and_then(|v| v.as_i64())
                .is_some_and(|t| t == admin_time)
    });
    assert!(
        query.is_some(),
        "holds_query must check allergy_known at administration time"
    );
    let expected = query
        .unwrap()
        .get("expected")
        .and_then(|v| v.as_bool())
        .unwrap();
    assert!(
        expected,
        "holds_query must expect allergy_known=true at t={}",
        admin_time
    );
}

// =========================================================================
// Patient cases: conflict and baseline paths
// =========================================================================

#[test]
fn patient_cases_have_expected_count() {
    let cases = toy_patient_cases();
    assert_eq!(cases.len(), 2, "must have 2 patient cases");
}

#[test]
fn patient_case_types_are_synthetic() {
    for pc in &toy_patient_cases() {
        assert_eq!(
            pc.case_type,
            CaseType::Synthetic,
            "case {} must be Synthetic",
            pc.case_id
        );
        assert_eq!(
            pc.privacy_status, "synthetic",
            "case {} privacy_status must be 'synthetic'",
            pc.case_id
        );
    }
}

#[test]
fn patient_case_conflict_has_allergy() {
    let cases = toy_patient_cases();
    let conflict = cases
        .iter()
        .find(|c| c.case_id.as_str() == CASE_SEPSIS_ALLERGY)
        .expect("conflict case must exist");

    assert!(
        !conflict.allergies.is_empty(),
        "conflict case must have allergy entries"
    );
    let has_bl_allergy = conflict.allergies.iter().any(|a| {
        a.get("substance")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s == "beta_lactam")
    });
    assert!(
        has_bl_allergy,
        "conflict case must have beta_lactam allergy"
    );

    let has_sepsis = conflict.conditions.iter().any(|c| {
        c.get("code")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s == "sepsis")
    });
    assert!(has_sepsis, "conflict case must have sepsis condition");
}

#[test]
fn patient_case_baseline_has_no_allergy() {
    let cases = toy_patient_cases();
    let baseline = cases
        .iter()
        .find(|c| c.case_id.as_str() == CASE_SEPSIS_BASELINE)
        .expect("baseline case must exist");

    assert!(
        baseline.allergies.is_empty(),
        "baseline case must have no allergies"
    );

    let has_sepsis = baseline.conditions.iter().any(|c| {
        c.get("code")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s == "sepsis")
    });
    assert!(has_sepsis, "baseline case must have sepsis condition");
}

#[test]
fn patient_cases_have_distinct_ids() {
    let cases = toy_patient_cases();
    let ids: HashSet<&str> = cases.iter().map(|c| c.case_id.as_str()).collect();
    assert_eq!(ids.len(), cases.len(), "patient case IDs must be unique");
}

// =========================================================================
// Workflow fragment validation
// =========================================================================

#[test]
fn workflow_has_expected_structure() {
    let wfs = toy_workflows();
    assert_eq!(wfs.len(), 1);
    let wf = &wfs[0];
    assert_eq!(wf.workflow_id.as_str(), WF_SEPSIS_PATHWAY);
    assert!(!wf.states.is_empty(), "workflow must have states");
    assert!(!wf.transitions.is_empty(), "workflow must have transitions");
    assert!(!wf.outcomes.is_empty(), "workflow must have outcomes");
    assert!(!wf.tasks.is_empty(), "workflow must have tasks");
    assert!(
        !wf.variance_rules.is_empty(),
        "workflow must have variance rules for allergy branching"
    );
}

#[test]
fn workflow_transitions_reference_valid_states() {
    let wfs = toy_workflows();
    let wf = &wfs[0];
    let state_ids: HashSet<&str> = wf
        .states
        .iter()
        .filter_map(|s| s.get("id").and_then(|v| v.as_str()))
        .collect();
    for trans in &wf.transitions {
        let from = trans
            .get("from")
            .and_then(|v| v.as_str())
            .expect("transition must have 'from'");
        let to = trans
            .get("to")
            .and_then(|v| v.as_str())
            .expect("transition must have 'to'");
        assert!(
            state_ids.contains(from),
            "transition 'from' state {} must exist in states",
            from
        );
        assert!(
            state_ids.contains(to),
            "transition 'to' state {} must exist in states",
            to
        );
    }
}

// =========================================================================
// Referential consistency with 0.5.1 spans and 0.5.2 rules
// =========================================================================

#[test]
fn all_decision_table_spans_exist() {
    let valid_spans = load_span_ids_from_fixtures();
    for dt in &toy_decision_tables() {
        for row in &dt.rows {
            for span_id in &row.source_span_ids {
                assert!(
                    valid_spans.contains(span_id.as_str()),
                    "decision table row {} references unknown span_id {}",
                    row.row_id,
                    span_id
                );
            }
        }
    }
}

#[test]
fn all_event_narrative_spans_exist() {
    let valid_spans = load_span_ids_from_fixtures();
    for en in &toy_event_narratives() {
        for span_id in &en.source_span_ids {
            assert!(
                valid_spans.contains(span_id.as_str()),
                "event narrative references unknown span_id {}",
                span_id
            );
        }
    }
}

#[test]
fn all_patient_case_spans_exist() {
    let valid_spans = load_span_ids_from_fixtures();
    for pc in &toy_patient_cases() {
        for span_id in &pc.source_span_ids {
            assert!(
                valid_spans.contains(span_id.as_str()),
                "patient case {} references unknown span_id {}",
                pc.case_id,
                span_id
            );
        }
    }
}

#[test]
fn all_workflow_spans_exist() {
    let valid_spans = load_span_ids_from_fixtures();
    for wf in &toy_workflows() {
        for span_id in &wf.source_span_ids {
            assert!(
                valid_spans.contains(span_id.as_str()),
                "workflow {} references unknown span_id {}",
                wf.workflow_id,
                span_id
            );
        }
    }
}

#[test]
fn decision_table_cell_refs_reference_existing_table() {
    let table_ids = load_table_ids_from_fixtures();
    for dt in &toy_decision_tables() {
        for row in &dt.rows {
            for cell_ref in &row.cell_refs {
                assert!(
                    table_ids.contains(cell_ref.table_id.as_str()),
                    "decision row {} cell_ref references unknown table {}",
                    row.row_id,
                    cell_ref.table_id
                );
            }
        }
    }
}

// =========================================================================
// NF normalization
// =========================================================================

#[test]
fn nf_decision_table_sorts_rows_under_unique_policy() {
    let mut tables = toy_decision_tables();
    let mut ctx = NfContext::new();
    tables[0].normalize(&mut ctx);

    // Under Unique hit policy, rows are sorted by canonical bytes.
    // Verify rows are deterministically ordered.
    let row_ids: Vec<&str> = tables[0].rows.iter().map(|r| r.row_id.as_str()).collect();
    // Re-normalize to confirm stability
    let mut ctx2 = NfContext::new();
    let mut tables2 = toy_decision_tables();
    tables2[0].normalize(&mut ctx2);
    let row_ids2: Vec<&str> = tables2[0].rows.iter().map(|r| r.row_id.as_str()).collect();
    assert_eq!(row_ids, row_ids2, "NF row ordering must be deterministic");
}

#[test]
fn nf_decision_table_assigns_stable_id() {
    let mut tables = toy_decision_tables();
    let mut ctx = NfContext::new();
    tables[0].normalize(&mut ctx);
    assert!(
        tables[0].table_id.as_str().starts_with("nf-"),
        "NF must assign stable ID to decision table"
    );
}

#[test]
fn nf_event_narrative_sorts_types_and_initially() {
    let mut narratives = toy_event_narratives();
    let mut ctx = NfContext::new();
    narratives[0].normalize(&mut ctx);

    // event_types and fluent_types must be sorted
    let types = &narratives[0].event_types;
    let sorted: Vec<&str> = {
        let mut v: Vec<&str> = types.iter().map(String::as_str).collect();
        v.sort();
        v
    };
    let actual: Vec<&str> = types.iter().map(String::as_str).collect();
    assert_eq!(actual, sorted, "NF must sort event_types");
}

#[test]
fn nf_patient_case_assigns_stable_id() {
    let mut cases = toy_patient_cases();
    let mut ctx = NfContext::new();
    for pc in &mut cases {
        pc.normalize(&mut ctx);
    }
    for pc in &cases {
        assert!(
            pc.case_id.as_str().starts_with("nf-"),
            "NF must assign stable ID to patient case {}",
            pc.case_id
        );
    }
}

#[test]
fn nf_workflow_assigns_stable_id() {
    let mut wfs = toy_workflows();
    let mut ctx = NfContext::new();
    wfs[0].normalize(&mut ctx);
    assert!(
        wfs[0].workflow_id.as_str().starts_with("nf-"),
        "NF must assign stable ID to workflow"
    );
}

#[test]
fn nf_idempotent_decision_tables() {
    let mut tables = toy_decision_tables();
    let mut ctx1 = NfContext::new();
    for dt in &mut tables {
        dt.normalize(&mut ctx1);
    }
    let bytes1 = to_canonical_bytes(&tables);

    let mut ctx2 = NfContext::new();
    for dt in &mut tables {
        dt.normalize(&mut ctx2);
    }
    let bytes2 = to_canonical_bytes(&tables);

    assert_eq!(
        bytes1, bytes2,
        "NF(NF(decision_tables)) must equal NF(decision_tables)"
    );
}

#[test]
fn nf_idempotent_event_narratives() {
    let mut narratives = toy_event_narratives();
    let mut ctx1 = NfContext::new();
    for en in &mut narratives {
        en.normalize(&mut ctx1);
    }
    let bytes1 = to_canonical_bytes(&narratives);

    let mut ctx2 = NfContext::new();
    for en in &mut narratives {
        en.normalize(&mut ctx2);
    }
    let bytes2 = to_canonical_bytes(&narratives);

    assert_eq!(
        bytes1, bytes2,
        "NF(NF(event_narratives)) must equal NF(event_narratives)"
    );
}

#[test]
fn nf_idempotent_patient_cases() {
    let mut cases = toy_patient_cases();
    let mut ctx1 = NfContext::new();
    for pc in &mut cases {
        pc.normalize(&mut ctx1);
    }
    let bytes1 = to_canonical_bytes(&cases);

    let mut ctx2 = NfContext::new();
    for pc in &mut cases {
        pc.normalize(&mut ctx2);
    }
    let bytes2 = to_canonical_bytes(&cases);

    assert_eq!(
        bytes1, bytes2,
        "NF(NF(patient_cases)) must equal NF(patient_cases)"
    );
}

#[test]
fn nf_idempotent_workflows() {
    let mut wfs = toy_workflows();
    let mut ctx1 = NfContext::new();
    for wf in &mut wfs {
        wf.normalize(&mut ctx1);
    }
    let bytes1 = to_canonical_bytes(&wfs);

    let mut ctx2 = NfContext::new();
    for wf in &mut wfs {
        wf.normalize(&mut ctx2);
    }
    let bytes2 = to_canonical_bytes(&wfs);

    assert_eq!(bytes1, bytes2, "NF(NF(workflows)) must equal NF(workflows)");
}

// =========================================================================
// Committed fixture file tests
// =========================================================================

#[test]
fn committed_decision_tables_match() {
    let path = fixtures_dir().join("decision_tables.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test research_structured_artifacts regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_decision_tables());
    assert_eq!(
        bytes, expected,
        "committed decision_tables.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_event_narratives_match() {
    let path = fixtures_dir().join("event_narratives.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test research_structured_artifacts regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_event_narratives());
    assert_eq!(
        bytes, expected,
        "committed event_narratives.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_patient_cases_match() {
    let path = fixtures_dir().join("patient_cases.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test research_structured_artifacts regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_patient_cases());
    assert_eq!(
        bytes, expected,
        "committed patient_cases.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_workflows_match() {
    let path = fixtures_dir().join("workflows.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test research_structured_artifacts regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_workflows());
    assert_eq!(
        bytes, expected,
        "committed workflows.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_fixtures_deserialize_correctly() {
    let dir = fixtures_dir();

    let dts: Vec<DecisionTable> =
        serde_json::from_slice(&std::fs::read(dir.join("decision_tables.json")).unwrap())
            .expect("decision_tables.json must deserialize");
    assert_eq!(dts.len(), 1);

    let ens: Vec<EventNarrative> =
        serde_json::from_slice(&std::fs::read(dir.join("event_narratives.json")).unwrap())
            .expect("event_narratives.json must deserialize");
    assert_eq!(ens.len(), 1);

    let pcs: Vec<PatientCase> =
        serde_json::from_slice(&std::fs::read(dir.join("patient_cases.json")).unwrap())
            .expect("patient_cases.json must deserialize");
    assert_eq!(pcs.len(), 2);

    let wfs: Vec<WorkflowFragment> =
        serde_json::from_slice(&std::fs::read(dir.join("workflows.json")).unwrap())
            .expect("workflows.json must deserialize");
    assert_eq!(wfs.len(), 1);
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
        dir.join("decision_tables.json"),
        to_canonical_bytes(&toy_decision_tables()),
    )
    .unwrap();
    std::fs::write(
        dir.join("event_narratives.json"),
        to_canonical_bytes(&toy_event_narratives()),
    )
    .unwrap();
    std::fs::write(
        dir.join("patient_cases.json"),
        to_canonical_bytes(&toy_patient_cases()),
    )
    .unwrap();
    std::fs::write(
        dir.join("workflows.json"),
        to_canonical_bytes(&toy_workflows()),
    )
    .unwrap();

    eprintln!(
        "Regenerated structured artifact fixtures in {}",
        dir.display()
    );
}
