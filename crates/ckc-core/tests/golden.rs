use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

use ckc_core::artifact::{
    DecisionRow, DecisionTable, EventNarrative, ExecutionWitness, PatientCase, WorkflowFragment,
};
use ckc_core::canonical::{ContentHash, content_hash, to_canonical_bytes};
use ckc_core::clinical::{
    Action, ClinicalClaim, ConfidenceInterval, EtDFrame, EvidenceAtom, Norm, PICOFrame, Rule,
};
use ckc_core::compile::{CompilationMap, CompileDiagnostic, CompiledTarget, SymbolMapping};
use ckc_core::enums::*;
use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind, ArtifactMeta};
use ckc_core::id::*;
use ckc_core::profile::SemanticProfile;
use ckc_core::source::{
    BBox, Concept, CorpusDocument, ExtractedTable, ExtractorVote, SourceSpan, TableCellRef,
    TerminologyBinding,
};
use ckc_core::verify::{ArgumentGraph, AssuranceNode, AuditTrace, Certificate, Conflict};

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn schema_dir() -> PathBuf {
    workspace_root().join("schemas")
}

fn golden_dir() -> PathBuf {
    workspace_root().join("schemas").join("golden")
}

// ---------------------------------------------------------------------------
// Assertion helpers
// ---------------------------------------------------------------------------

fn check_golden<T: Serialize>(fixture: &T, stem: &str) {
    let bytes = to_canonical_bytes(fixture);
    let path = golden_dir().join(format!("{stem}.json"));
    let golden =
        std::fs::read(&path).unwrap_or_else(|e| panic!("read golden {}: {e}", path.display()));
    assert!(
        bytes == golden,
        "canonical bytes mismatch for {stem} (got {} bytes, golden {} bytes)",
        bytes.len(),
        golden.len()
    );
}

fn check_roundtrip<T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug>(
    fixture: &T,
    stem: &str,
) {
    let bytes1 = to_canonical_bytes(fixture);
    let hash1 = content_hash(fixture);
    let rt: T =
        serde_json::from_slice(&bytes1).unwrap_or_else(|e| panic!("deserialize {stem}: {e}"));
    let bytes2 = to_canonical_bytes(&rt);
    let hash2 = content_hash(&rt);
    assert_eq!(bytes1, bytes2, "bytes differ after roundtrip for {stem}");
    assert_eq!(hash1, hash2, "hash differs after roundtrip for {stem}");
    assert_eq!(*fixture, rt, "value differs after roundtrip for {stem}");
}

fn check_schema<T: schemars::JsonSchema>(stem: &str) {
    let schema = schemars::schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap() + "\n";
    let path = schema_dir().join(format!("{stem}.schema.json"));
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read schema {}: {e}", path.display()));
    assert!(json == golden, "schema mismatch for {stem}");
}

// ---------------------------------------------------------------------------
// Golden fixture constructors
// ---------------------------------------------------------------------------

fn golden_content_hash() -> ContentHash {
    ContentHash("sha256:a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1".into())
}

fn golden_bbox() -> BBox {
    BBox {
        x: 72.0,
        y: 200.0,
        width: 468.0,
        height: 14.0,
    }
}

fn golden_table_cell_ref() -> TableCellRef {
    TableCellRef {
        table_id: ExtractedTableId::new("tbl_vitals_001"),
        row: 1,
        col: 0,
    }
}

fn golden_extractor_vote() -> ExtractorVote {
    ExtractorVote {
        extractor: "pymupdf".into(),
        raw_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
        confidence: 0.99,
    }
}

fn golden_confidence_interval() -> ConfidenceInterval {
    ConfidenceInterval {
        lower: 0.45,
        upper: 0.82,
    }
}

fn golden_corpus_document() -> CorpusDocument {
    CorpusDocument {
        doc_id: DocId::new("doc_gl_sepsis_2024"),
        title_ja: "敗血症診療ガイドライン2024".into(),
        title_en: Some("Sepsis Management Guideline 2024".into()),
        source_type: "guideline".into(),
        publisher: "日本集中治療医学会".into(),
        society: "JSICM".into(),
        edition: "2024".into(),
        publication_date: Some("2024-03-01".into()),
        access_date: None,
        license_status: "permitted_research".into(),
        content_hash: ContentHash(
            "sha256:a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1".into(),
        ),
        extraction_manifest_id: ManifestId::new("manifest_gl_sepsis_2024"),
        supersedes: None,
        superseded_by: None,
    }
}

fn golden_source_span() -> SourceSpan {
    SourceSpan {
        span_id: SpanId::new("span_gl_s1"),
        doc_id: DocId::new("doc_gl_sepsis_2024"),
        section_path: vec!["CQ1".into(), "推奨".into()],
        cq_id: Some(CqId::new("cq_sepsis_abx_001")),
        page: Some(42),
        bbox: Some(golden_bbox()),
        table_cell: None,
        raw_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
        nfkc_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
        search_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
        display_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
        language: Language::Ja,
        previous_span_id: None,
        next_span_id: Some(SpanId::new("span_gl_s2")),
        extractor_votes: vec![golden_extractor_vote()],
        confidence: 0.99,
    }
}

fn golden_extracted_table() -> ExtractedTable {
    ExtractedTable {
        table_id: ExtractedTableId::new("tbl_vitals_001"),
        doc_id: DocId::new("doc_gl_sepsis_2024"),
        caption_span_id: Some(SpanId::new("span_tbl_caption")),
        cell_span_ids: vec![SpanId::new("span_cell_r0c0"), SpanId::new("span_cell_r0c1")],
        row_headers: vec!["体温".into()],
        column_headers: vec!["項目".into(), "基準値".into()],
        reading_order: vec![SpanId::new("span_cell_r0c0"), SpanId::new("span_cell_r0c1")],
        extraction_votes: vec![ExtractorVote {
            extractor: "yomitoku".into(),
            raw_text: "体温|基準値".into(),
            confidence: 0.92,
        }],
        normalized_table_hash: ContentHash(
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        ),
    }
}

fn golden_terminology_binding() -> TerminologyBinding {
    TerminologyBinding {
        system: "MEDIS".into(),
        code: Some("MEDIS001".into()),
        version: Some("2024".into()),
        label: "敗血症".into(),
        status: BindingStatus::Exact,
        mapping_relation: "equivalent".into(),
        provenance: "medis_master_2024".into(),
        confidence: 0.95,
        license_status: "permitted".into(),
        valid_from: Some("2024-01-01".into()),
        valid_to: None,
    }
}

fn golden_concept() -> Concept {
    Concept {
        concept_id: ConceptId::new("concept_sepsis"),
        label_ja: "敗血症".into(),
        label_en: Some("sepsis".into()),
        semantic_type: "diagnosis".into(),
        terminology_bindings: vec![golden_terminology_binding()],
        egraph_class_id: Some(EGraphClassId::new("eclass_sepsis")),
        source_span_ids: vec![SpanId::new("span_gl_s1")],
    }
}

fn golden_pico_frame() -> PICOFrame {
    PICOFrame {
        population: "成人敗血症患者".into(),
        intervention: "βラクタム系抗菌薬投与".into(),
        comparator: "非βラクタム系抗菌薬".into(),
        outcomes: vec!["28日死亡率".into()],
        cq_id: Some(CqId::new("cq_sepsis_abx_001")),
        scope: "inpatient_icu".into(),
        exclusions: vec!["βラクタムアレルギー".into()],
        source_span_ids: vec![SpanId::new("span_gl_s1")],
    }
}

fn golden_etd_frame() -> EtDFrame {
    EtDFrame {
        benefits: "reduced mortality".into(),
        harms: "anaphylaxis risk".into(),
        certainty: EvidenceCertainty::Moderate,
        values: "mortality reduction valued highly".into(),
        resources: "standard formulary".into(),
        equity: "widely available".into(),
        acceptability: "accepted standard of care".into(),
        feasibility: "feasible in ICU".into(),
        recommendation_direction: RecommendationDirection::For,
        recommendation_strength: RecommendationStrength::Strong,
        source_span_ids: vec![SpanId::new("span_gl_s1")],
    }
}

fn golden_evidence_atom() -> EvidenceAtom {
    EvidenceAtom {
        evidence_type: "outcome_effect".into(),
        source_span_ids: vec![SpanId::new("span_ev_001")],
        pico_ref: Some(CqId::new("cq_sepsis_abx_001")),
        effect_measure: Some("odds_ratio".into()),
        value: Some(0.62),
        unit: None,
        confidence_interval: Some(golden_confidence_interval()),
        certainty: EvidenceCertainty::Moderate,
        outcome_importance: Some("critical".into()),
        table_cell_refs: vec![],
    }
}

fn golden_action() -> Action {
    Action {
        action_type: "administer".into(),
        target_concept: ConceptId::new("concept_beta_lactam"),
        parameters: serde_json::json!({"route": "iv"}),
        temporal_constraints: serde_json::json!({"onset": "immediate"}),
        quantity_constraints: serde_json::json!({"min_dose_mg": 1000}),
    }
}

fn golden_norm() -> Norm {
    Norm {
        context: "sepsis in adult patients".into(),
        direction: RecommendationDirection::For,
        action: golden_action(),
        recommendation_strength: RecommendationStrength::Strong,
        evidence_certainty: EvidenceCertainty::Moderate,
        original_modality_phrase_ja: "投与を推奨する".into(),
        deontic_projection: DeonticProjection::Recommended,
        exception_policy: "beta-lactam allergy contraindicates".into(),
        prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
    }
}

fn golden_rule() -> Rule {
    Rule {
        rule_id: RuleId::new("rule_sepsis_bl_001"),
        profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
        kind: RuleKind::Defeasible,
        context: "sepsis AND adult".into(),
        antecedent: "(dx sepsis) AND (adult patient)".into(),
        consequent: "(administer beta_lactam)".into(),
        norm: Some(golden_norm()),
        priority_over: vec![],
        exceptions: vec!["beta_lactam_allergy".into()],
        temporal_scope: Some("acute_phase".into()),
        population_scope: None,
        source_span_ids: vec![SpanId::new("span_gl_s1")],
        provenance: "guideline_sepsis_2024_cq1".into(),
        certificate_ids: vec![],
    }
}

fn golden_clinical_claim() -> ClinicalClaim {
    ClinicalClaim {
        claim_id: ClaimId::new("claim_sepsis_bl"),
        claim_type: "recommendation".into(),
        profiles: vec![SemanticProfile::Evidence, SemanticProfile::Norm],
        source_span_ids: vec![SpanId::new("span_gl_s1")],
        pico: Some(golden_pico_frame()),
        etd: None,
        evidence_atoms: vec![golden_evidence_atom()],
        rule_ids: vec![RuleId::new("rule_sepsis_bl_001")],
        decision_table_ids: vec![],
        workflow_fragment_ids: vec![],
        gloss_ja: "敗血症にβラクタム系抗菌薬の投与を強く推奨する".into(),
        gloss_en: "Strongly recommend beta-lactam for sepsis".into(),
        status: "candidate".into(),
    }
}

fn golden_decision_row() -> DecisionRow {
    DecisionRow {
        row_id: DecisionRowId::new("row_vitals_r1"),
        conditions: vec![serde_json::json!({"field": "temperature", "op": ">=", "value": 38.0})],
        outputs: vec![serde_json::json!({"action": "sepsis_alert"})],
        priority: Some(1),
        source_span_ids: vec![SpanId::new("span_tbl_r1")],
        cell_refs: vec![golden_table_cell_ref()],
    }
}

fn golden_decision_table() -> DecisionTable {
    DecisionTable {
        table_id: DecisionTableId::new("dt_vitals_triage"),
        hit_policy: HitPolicy::Priority,
        input_columns: vec!["temperature".into()],
        output_columns: vec!["alert_action".into()],
        rows: vec![golden_decision_row()],
        source_table_id: Some(ExtractedTableId::new("tbl_vitals_001")),
        dmn_export_id: None,
        certificate_ids: vec![],
    }
}

fn golden_workflow_fragment() -> WorkflowFragment {
    WorkflowFragment {
        workflow_id: WorkflowId::new("wf_sepsis_pathway"),
        workflow_type: "epath_oat".into(),
        states: vec![serde_json::json!({"id": "triage", "label": "初期トリアージ"})],
        transitions: vec![serde_json::json!({"from": "triage", "to": "treatment"})],
        outcomes: vec![serde_json::json!({"id": "recovery"})],
        assessments: vec![],
        tasks: vec![serde_json::json!({"id": "blood_culture"})],
        variance_rules: vec![],
        source_span_ids: vec![SpanId::new("span_wf_001")],
    }
}

fn golden_event_narrative() -> EventNarrative {
    EventNarrative {
        event_types: vec!["administer_drug".into(), "detect_allergy".into()],
        fluent_types: vec!["allergy_known".into()],
        happens: vec![serde_json::json!({"event": "detect_allergy", "time": 0})],
        initiates: vec![serde_json::json!({"event": "detect_allergy", "fluent": "allergy_known"})],
        terminates: vec![],
        initially: vec![serde_json::json!({"fluent": "allergy_known", "value": false})],
        holds_queries: vec![
            serde_json::json!({"fluent": "allergy_known", "time": 10, "expected": true}),
        ],
        source_span_ids: vec![SpanId::new("span_ec_001")],
    }
}

fn golden_patient_case() -> PatientCase {
    PatientCase {
        case_id: CaseId::new("case_sepsis_allergy"),
        case_type: CaseType::Synthetic,
        facts: vec![serde_json::json!({"type": "diagnosis", "code": "sepsis"})],
        events: vec![serde_json::json!({"type": "admission", "time": "2024-01-15T08:00:00Z"})],
        observations: vec![serde_json::json!({"type": "temperature", "value": 39.2})],
        medications: vec![],
        conditions: vec![serde_json::json!({"code": "sepsis"})],
        allergies: vec![serde_json::json!({"severity": "severe", "substance": "beta_lactam"})],
        time_origin: Some("2024-01-15T08:00:00Z".into()),
        source_span_ids: vec![SpanId::new("span_case_001")],
        privacy_status: "synthetic".into(),
    }
}

fn golden_execution_witness() -> ExecutionWitness {
    ExecutionWitness {
        witness_id: WitnessId::new("witness_conflict_001"),
        bundle_id: BundleId::new("bundle_sepsis_toy"),
        case_id: Some(CaseId::new("case_sepsis_allergy")),
        context_facts: vec![serde_json::json!({"fact": "dx_sepsis", "value": true})],
        trace: vec![
            serde_json::json!({"result": "applicable", "rule": "rule_recommend", "step": 1}),
        ],
        applicable_rules: vec![RuleId::new("rule_sepsis_bl_001")],
        defeated_rules: vec![],
        violated_constraints: vec![],
        models: vec![serde_json::json!({"solver": "z3", "status": "sat"})],
        unsat_cores: vec![],
        source_span_ids: vec![SpanId::new("span_gl_s1")],
        certificate_ids: vec![],
    }
}

fn golden_conflict() -> Conflict {
    Conflict {
        conflict_id: ConflictId::new("conflict_bl_001"),
        conflict_type: "norm_contradiction".into(),
        severity: Severity::High,
        confidence: 0.95,
        minimal_artifact_set: vec![ContentHash(
            "sha256:aa00000000000000000000000000000000000000000000000000000000000001".into(),
        )],
        source_spans: vec![SpanId::new("span_gl_s1"), SpanId::new("span_gl_s2")],
        normalized_view: serde_json::json!({"type": "norm_contradiction"}),
        witness: Some(WitnessId::new("witness_conflict_001")),
        repair_candidates: vec![serde_json::json!({"type": "add_priority"})],
        solver_evidence: vec![serde_json::json!({"solver": "z3", "status": "sat"})],
        argument_graph_id: Some(ArgumentGraphId::new("ag_bl_001")),
        human_review_question_ja: "推奨と禁忌が矛盾しています".into(),
        human_review_question_en: "Recommendation and contraindication conflict".into(),
        classification: ConflictClassification::TrueConflict,
    }
}

fn golden_argument_graph() -> ArgumentGraph {
    ArgumentGraph {
        argument_graph_id: ArgumentGraphId::new("ag_bl_001"),
        arguments: vec![serde_json::json!({"id": "arg_recommend"})],
        attack_edges: vec![serde_json::json!({"from": "arg_contra", "to": "arg_recommend"})],
        support_edges: vec![],
        undercut_edges: vec![],
        defeat_edges: vec![],
        extension_summaries: vec![serde_json::json!({"semantics": "grounded"})],
        source_span_ids: vec![SpanId::new("span_gl_s1")],
    }
}

fn golden_certificate() -> Certificate {
    Certificate {
        certificate_id: CertificateId::new("cert_z3_001"),
        certificate_class: CertificateClass::C4Executable,
        input_artifact_hashes: vec![ContentHash(
            "sha256:bb00000000000000000000000000000000000000000000000000000000000001".into(),
        )],
        compiler_hash: None,
        solver_or_checker: "z3".into(),
        command_manifest: serde_json::json!({"args": ["-smt2"], "command": "z3"}),
        result: "sat".into(),
        proof_artifact_hashes: vec![],
        replay_status: ReplayStatus::Passed,
        diagnostics: vec![],
    }
}

fn golden_assurance_node() -> AssuranceNode {
    AssuranceNode {
        node_id: AssuranceNodeId::new("goal_top_001"),
        node_type: "goal".into(),
        claim: "Accepted artifacts are source-grounded and formally checkable".into(),
        evidence_artifact_ids: vec![ContentHash(
            "sha256:cc00000000000000000000000000000000000000000000000000000000000001".into(),
        )],
        status: "supported".into(),
        children: vec![AssuranceNodeId::new("strategy_001")],
    }
}

fn golden_audit_trace() -> AuditTrace {
    AuditTrace {
        trace_id: AuditTraceId::new("trace_toy_001"),
        stage_spans: vec![serde_json::json!({"stage": "normalize", "status": "ok"})],
        model_invocations: vec![],
        retrieval_events: vec![],
        verifier_events: vec![serde_json::json!({"result": "sat", "verifier": "z3"})],
        artifact_hashes: vec![ContentHash(
            "sha256:dd00000000000000000000000000000000000000000000000000000000000001".into(),
        )],
        redaction_status: "none".into(),
        audit_export_refs: vec![],
    }
}

fn golden_artifact_kind() -> ArtifactKind {
    ArtifactKind::Rule
}

fn golden_artifact_meta() -> ArtifactMeta {
    ArtifactMeta {
        schema_version: "0.0.0".into(),
        producer_version: "ckc-core/0.0.0".into(),
        command_manifest: serde_json::json!({"command": "ckc", "args": ["normalize"]}),
        source_input_hashes: vec![ContentHash(
            "sha256:ee00000000000000000000000000000000000000000000000000000000000001".into(),
        )],
        parent_hashes: vec![],
        stage: "normalize".into(),
        semantic_profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
        content_hash: ContentHash(
            "sha256:ff00000000000000000000000000000000000000000000000000000000000001".into(),
        ),
        certificate_ids: vec![],
        replay_command: Some("ckc normalize --bundle test".into()),
    }
}

fn golden_artifact_envelope() -> ArtifactEnvelope {
    ArtifactEnvelope::wrap(ArtifactKind::Rule, &golden_rule(), golden_artifact_meta())
}

fn golden_symbol_mapping() -> SymbolMapping {
    SymbolMapping {
        ckc_node_id: "rule_sepsis_bl_recommend".into(),
        target_symbol: "recommend_administer_beta_lactam".into(),
        source_span_ids: vec![SpanId::new("span_rec_sepsis_bl")],
    }
}

fn golden_compilation_map() -> CompilationMap {
    CompilationMap(vec![
        golden_symbol_mapping(),
        SymbolMapping {
            ckc_node_id: "rule_bl_anaphylaxis_contra".into(),
            target_symbol: "prohibit_administer_beta_lactam".into(),
            source_span_ids: vec![
                SpanId::new("span_contra_bl_allergy"),
                SpanId::new("span_allergy_history"),
            ],
        },
    ])
}

fn golden_compile_diagnostic() -> CompileDiagnostic {
    CompileDiagnostic {
        code: "ckc_compile_norm_conflict".into(),
        message_ja: "推奨と禁忌が優先順位なしで衝突します".into(),
        message_en: "Recommendation and contraindication conflict without priority".into(),
        source_span_ids: vec![
            SpanId::new("span_rec_sepsis_bl"),
            SpanId::new("span_contra_bl_allergy"),
        ],
    }
}

fn golden_compiled_target() -> CompiledTarget {
    CompiledTarget {
        target_language: TargetLanguage::SmtLib,
        artifact_text: "(set-logic QF_UF)\n(check-sat)\n".into(),
        compilation_map: golden_compilation_map(),
        diagnostics: vec![golden_compile_diagnostic()],
        source_artifact_hashes: vec![golden_content_hash()],
        replay_command: "z3 -smt2 logic/smt/norm_conflict.smt2".into(),
        target_parse_ok: Some(true),
    }
}

// ---------------------------------------------------------------------------
// Test macro: for each type, verify canonical golden bytes, round-trip hash
// determinism, and JSON Schema stability.
// ---------------------------------------------------------------------------

macro_rules! golden_suite {
    ($mod_name:ident, $type:ty, $fixture_fn:ident, $stem:literal) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn canonical() {
                check_golden(&$fixture_fn(), $stem);
            }

            #[test]
            fn roundtrip() {
                check_roundtrip::<$type>(&$fixture_fn(), $stem);
            }

            #[test]
            fn schema() {
                check_schema::<$type>($stem);
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Suite invocations (34 types = 102 tests)
// ---------------------------------------------------------------------------

golden_suite!(
    gs_content_hash,
    ContentHash,
    golden_content_hash,
    "content_hash"
);
golden_suite!(gs_bbox, BBox, golden_bbox, "bbox");
golden_suite!(
    gs_table_cell_ref,
    TableCellRef,
    golden_table_cell_ref,
    "table_cell_ref"
);
golden_suite!(
    gs_extractor_vote,
    ExtractorVote,
    golden_extractor_vote,
    "extractor_vote"
);
golden_suite!(
    gs_confidence_interval,
    ConfidenceInterval,
    golden_confidence_interval,
    "confidence_interval"
);
golden_suite!(
    gs_corpus_document,
    CorpusDocument,
    golden_corpus_document,
    "corpus_document"
);
golden_suite!(
    gs_source_span,
    SourceSpan,
    golden_source_span,
    "source_span"
);
golden_suite!(
    gs_extracted_table,
    ExtractedTable,
    golden_extracted_table,
    "extracted_table"
);
golden_suite!(
    gs_terminology_binding,
    TerminologyBinding,
    golden_terminology_binding,
    "terminology_binding"
);
golden_suite!(gs_concept, Concept, golden_concept, "concept");
golden_suite!(gs_pico_frame, PICOFrame, golden_pico_frame, "pico_frame");
golden_suite!(gs_etd_frame, EtDFrame, golden_etd_frame, "etd_frame");
golden_suite!(
    gs_evidence_atom,
    EvidenceAtom,
    golden_evidence_atom,
    "evidence_atom"
);
golden_suite!(gs_action, Action, golden_action, "action");
golden_suite!(gs_norm, Norm, golden_norm, "norm");
golden_suite!(gs_rule, Rule, golden_rule, "rule");
golden_suite!(
    gs_clinical_claim,
    ClinicalClaim,
    golden_clinical_claim,
    "clinical_claim"
);
golden_suite!(
    gs_decision_row,
    DecisionRow,
    golden_decision_row,
    "decision_row"
);
golden_suite!(
    gs_decision_table,
    DecisionTable,
    golden_decision_table,
    "decision_table"
);
golden_suite!(
    gs_workflow_fragment,
    WorkflowFragment,
    golden_workflow_fragment,
    "workflow_fragment"
);
golden_suite!(
    gs_event_narrative,
    EventNarrative,
    golden_event_narrative,
    "event_narrative"
);
golden_suite!(
    gs_patient_case,
    PatientCase,
    golden_patient_case,
    "patient_case"
);
golden_suite!(
    gs_execution_witness,
    ExecutionWitness,
    golden_execution_witness,
    "execution_witness"
);
golden_suite!(gs_conflict, Conflict, golden_conflict, "conflict");
golden_suite!(
    gs_argument_graph,
    ArgumentGraph,
    golden_argument_graph,
    "argument_graph"
);
golden_suite!(
    gs_certificate,
    Certificate,
    golden_certificate,
    "certificate"
);
golden_suite!(
    gs_assurance_node,
    AssuranceNode,
    golden_assurance_node,
    "assurance_node"
);
golden_suite!(
    gs_audit_trace,
    AuditTrace,
    golden_audit_trace,
    "audit_trace"
);
golden_suite!(
    gs_artifact_kind,
    ArtifactKind,
    golden_artifact_kind,
    "artifact_kind"
);
golden_suite!(
    gs_artifact_meta,
    ArtifactMeta,
    golden_artifact_meta,
    "artifact_meta"
);
golden_suite!(
    gs_artifact_envelope,
    ArtifactEnvelope,
    golden_artifact_envelope,
    "artifact_envelope"
);
golden_suite!(
    gs_compilation_map,
    CompilationMap,
    golden_compilation_map,
    "compilation_map"
);
golden_suite!(
    gs_compile_diagnostic,
    CompileDiagnostic,
    golden_compile_diagnostic,
    "compile_diagnostic"
);
golden_suite!(
    gs_compiled_target,
    CompiledTarget,
    golden_compiled_target,
    "compiled_target"
);

// ---------------------------------------------------------------------------
// Regeneration: run `cargo test -p ckc-core --test golden -- --ignored` to
// update all golden canonical JSON and JSON Schema files.
// ---------------------------------------------------------------------------

fn write_type<T: Serialize + schemars::JsonSchema>(fixture: &T, stem: &str) {
    let g = golden_dir();
    let s = schema_dir();
    std::fs::write(g.join(format!("{stem}.json")), to_canonical_bytes(fixture)).unwrap();
    let schema = schemars::schema_for!(T);
    std::fs::write(
        s.join(format!("{stem}.schema.json")),
        serde_json::to_string_pretty(&schema).unwrap() + "\n",
    )
    .unwrap();
}

#[test]
#[ignore]
fn regenerate() {
    std::fs::create_dir_all(golden_dir()).unwrap();

    write_type(&golden_content_hash(), "content_hash");
    write_type(&golden_bbox(), "bbox");
    write_type(&golden_table_cell_ref(), "table_cell_ref");
    write_type(&golden_extractor_vote(), "extractor_vote");
    write_type(&golden_confidence_interval(), "confidence_interval");
    write_type(&golden_corpus_document(), "corpus_document");
    write_type(&golden_source_span(), "source_span");
    write_type(&golden_extracted_table(), "extracted_table");
    write_type(&golden_terminology_binding(), "terminology_binding");
    write_type(&golden_concept(), "concept");
    write_type(&golden_pico_frame(), "pico_frame");
    write_type(&golden_etd_frame(), "etd_frame");
    write_type(&golden_evidence_atom(), "evidence_atom");
    write_type(&golden_action(), "action");
    write_type(&golden_norm(), "norm");
    write_type(&golden_rule(), "rule");
    write_type(&golden_clinical_claim(), "clinical_claim");
    write_type(&golden_decision_row(), "decision_row");
    write_type(&golden_decision_table(), "decision_table");
    write_type(&golden_workflow_fragment(), "workflow_fragment");
    write_type(&golden_event_narrative(), "event_narrative");
    write_type(&golden_patient_case(), "patient_case");
    write_type(&golden_execution_witness(), "execution_witness");
    write_type(&golden_conflict(), "conflict");
    write_type(&golden_argument_graph(), "argument_graph");
    write_type(&golden_certificate(), "certificate");
    write_type(&golden_assurance_node(), "assurance_node");
    write_type(&golden_audit_trace(), "audit_trace");
    write_type(&golden_artifact_kind(), "artifact_kind");
    write_type(&golden_artifact_meta(), "artifact_meta");
    write_type(&golden_artifact_envelope(), "artifact_envelope");
    write_type(&golden_compilation_map(), "compilation_map");
    write_type(&golden_compile_diagnostic(), "compile_diagnostic");
    write_type(&golden_compiled_target(), "compiled_target");
}
