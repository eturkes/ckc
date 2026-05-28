use ckc_core::artifact::*;
use ckc_core::canonical::{ContentHash, to_canonical_bytes};
use ckc_core::clinical::*;
use ckc_core::enums::*;
use ckc_core::id::*;
use ckc_core::nf::normalize_all;
use ckc_core::profile::SemanticProfile;
use ckc_core::source::*;
use ckc_core::verify::*;
use std::path::PathBuf;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("schemas")
        .join("golden_nf")
}

macro_rules! golden_nf_test {
    ($mod_name:ident, $fixture_fn:ident, $file:expr) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn canonical_bytes_match_golden() {
                let mut value = $fixture_fn();
                normalize_all(&mut value);
                let bytes = to_canonical_bytes(&value);

                let path = golden_dir().join($file);
                if !path.exists() {
                    panic!(
                        "Golden NF file missing: {}. Run `cargo test -p ckc-core --test golden_nf -- regenerate --ignored` to create it.",
                        path.display()
                    );
                }
                let expected = std::fs::read(&path).unwrap();
                assert_eq!(
                    bytes, expected,
                    "NF canonical bytes for {} diverged from golden file {}",
                    stringify!($mod_name),
                    $file
                );
            }

            #[test]
            fn idempotent_hash() {
                let mut value = $fixture_fn();
                normalize_all(&mut value);
                let hash1 = ckc_core::canonical::content_hash(&value);
                normalize_all(&mut value);
                let hash2 = ckc_core::canonical::content_hash(&value);
                assert_eq!(hash1, hash2, "NF(NF(x)) must produce identical hash for {}", stringify!($mod_name));
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Fixture functions: each returns a pre-normalization instance
// ---------------------------------------------------------------------------

fn fixture_rule() -> Rule {
    Rule {
        rule_id: RuleId::new("rule_sepsis_bl_001"),
        profiles: vec![SemanticProfile::Defeasible, SemanticProfile::Norm],
        kind: RuleKind::Defeasible,
        context: "sepsis AND adult_patient".into(),
        antecedent: "(dx sepsis) AND (adult patient)".into(),
        consequent: "(administer beta_lactam)".into(),
        norm: Some(Norm {
            context: "sepsis in adult patients".into(),
            direction: RecommendationDirection::For,
            action: Action {
                action_type: "Administer".into(),
                target_concept: ConceptId::new("concept_beta_lactam"),
                parameters: serde_json::json!({"route": "iv"}),
                temporal_constraints: serde_json::json!({"onset": "immediate"}),
                quantity_constraints: serde_json::json!({"volume": {"value": 500, "unit": "ml"}}),
            },
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: "投与を推奨する".into(),
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "beta-lactam allergy contraindicates".into(),
            prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
        }),
        priority_over: vec![],
        exceptions: vec!["z_exception".into(), "a_exception".into()],
        temporal_scope: Some("acute_phase".into()),
        population_scope: Some("adult".into()),
        source_span_ids: vec![SpanId::new("span_s2"), SpanId::new("span_s1")],
        provenance: "guideline_sepsis_2024_cq1".into(),
        certificate_ids: vec![],
    }
}

fn fixture_clinical_claim() -> ClinicalClaim {
    ClinicalClaim {
        claim_id: ClaimId::new("claim_sepsis_beta_lactam"),
        claim_type: "recommendation".into(),
        profiles: vec![SemanticProfile::Norm, SemanticProfile::Evidence],
        source_span_ids: vec![SpanId::new("span_s2"), SpanId::new("span_s1")],
        pico: None,
        etd: None,
        evidence_atoms: vec![],
        rule_ids: vec![RuleId::new("rule_z"), RuleId::new("rule_a")],
        decision_table_ids: vec![],
        workflow_fragment_ids: vec![],
        gloss_ja: "βラクタム系\u{3000}抗菌薬の投与を強く推奨する".into(),
        gloss_en: "Beta-lactam  antibiotics  are  strongly  recommended".into(),
        status: "candidate".into(),
    }
}

fn fixture_concept() -> Concept {
    Concept {
        concept_id: ConceptId::new("concept_beta_lactam"),
        label_ja: "βラクタム系\u{3000}抗菌薬".into(),
        label_en: Some("Beta-Lactam  Antibiotics".into()),
        semantic_type: "drug_class".into(),
        terminology_bindings: vec![TerminologyBinding {
            system: "MEDIS".into(),
            code: Some("M001".into()),
            version: Some("2024".into()),
            label: "βラクタム系\u{3000}抗菌薬".into(),
            status: BindingStatus::Exact,
            mapping_relation: "equivalent".into(),
            provenance: "test".into(),
            confidence: 1.0,
            license_status: "permitted".into(),
            valid_from: None,
            valid_to: None,
        }],
        egraph_class_id: None,
        source_span_ids: vec![SpanId::new("span_s2"), SpanId::new("span_s1")],
    }
}

fn fixture_decision_table() -> DecisionTable {
    DecisionTable {
        table_id: DecisionTableId::new("dt_vitals"),
        hit_policy: HitPolicy::Unique,
        input_columns: vec!["体温\u{3000}（℃）".into(), "心拍数".into()],
        output_columns: vec!["アラート\u{3000}レベル".into()],
        rows: vec![
            DecisionRow {
                row_id: DecisionRowId::new("row_z"),
                conditions: vec![
                    serde_json::json!({"field": "temp", "unit": "degC", "op": ">=", "value": 38.0}),
                ],
                outputs: vec![serde_json::json!({"dose": {"value": 500, "unit": "ml"}})],
                priority: None,
                source_span_ids: vec![],
                cell_refs: vec![],
            },
            DecisionRow {
                row_id: DecisionRowId::new("row_a"),
                conditions: vec![
                    serde_json::json!({"field": "temp", "unit": "Cel", "value": 37.0}),
                ],
                outputs: vec![serde_json::json!({"action": "normal"})],
                priority: None,
                source_span_ids: vec![],
                cell_refs: vec![],
            },
        ],
        source_table_id: None,
        dmn_export_id: None,
        certificate_ids: vec![],
    }
}

fn fixture_workflow_fragment() -> WorkflowFragment {
    WorkflowFragment {
        workflow_id: WorkflowId::new("wf_sepsis"),
        workflow_type: "epath".into(),
        states: vec![
            serde_json::json!({"id": "monitoring", "label": "経過観察"}),
            serde_json::json!({"id": "triage", "label": "トリアージ"}),
            serde_json::json!({"id": "abx_admin", "label": "抗菌薬投与"}),
        ],
        transitions: vec![
            serde_json::json!({"from": "abx_admin", "to": "monitoring"}),
            serde_json::json!({"from": "triage", "to": "abx_admin"}),
        ],
        outcomes: vec![serde_json::json!({"id": "recovery"})],
        assessments: vec![],
        tasks: vec![],
        variance_rules: vec![],
        source_span_ids: vec![SpanId::new("span_p2"), SpanId::new("span_p1")],
    }
}

fn fixture_argument_graph() -> ArgumentGraph {
    ArgumentGraph {
        argument_graph_id: ArgumentGraphId::new("ag_sepsis_001"),
        arguments: vec![
            serde_json::json!({"id": "arg_contraindicate", "claim": "withhold"}),
            serde_json::json!({"id": "arg_recommend", "claim": "administer"}),
        ],
        attack_edges: vec![
            serde_json::json!({"from": "arg_contraindicate", "to": "arg_recommend"}),
        ],
        support_edges: vec![],
        undercut_edges: vec![],
        defeat_edges: vec![],
        extension_summaries: vec![],
        source_span_ids: vec![SpanId::new("span_s2"), SpanId::new("span_s1")],
    }
}

fn fixture_conflict() -> Conflict {
    Conflict {
        conflict_id: ConflictId::new("conflict_001"),
        conflict_type: "norm_contradiction".into(),
        severity: Severity::High,
        confidence: 0.95,
        minimal_artifact_set: vec![
            ContentHash("sha256:bbbb0001".into()),
            ContentHash("sha256:aaaa0001".into()),
        ],
        source_spans: vec![SpanId::new("span_s2"), SpanId::new("span_s1")],
        normalized_view: serde_json::json!({"type": "contradiction"}),
        witness: None,
        repair_candidates: vec![],
        solver_evidence: vec![],
        argument_graph_id: None,
        human_review_question_ja: "βラクタムアレルギー患者への\u{3000}投与について".into(),
        human_review_question_en: "Regarding  administration  to  beta-lactam  allergic  patients"
            .into(),
        classification: ConflictClassification::TrueConflict,
    }
}

fn fixture_patient_case() -> PatientCase {
    PatientCase {
        case_id: CaseId::new("case_sepsis_allergy"),
        case_type: CaseType::Synthetic,
        facts: vec![
            serde_json::json!({"type": "z_fact"}),
            serde_json::json!({"type": "a_fact"}),
        ],
        events: vec![serde_json::json!({"time": 1, "type": "admission"})],
        observations: vec![],
        medications: vec![],
        conditions: vec![],
        allergies: vec![serde_json::json!({"substance": "beta_lactam"})],
        time_origin: None,
        source_span_ids: vec![SpanId::new("span_c2"), SpanId::new("span_c1")],
        privacy_status: "synthetic".into(),
    }
}

fn fixture_execution_witness() -> ExecutionWitness {
    ExecutionWitness {
        witness_id: WitnessId::new("witness_001"),
        bundle_id: BundleId::new("bundle_test"),
        case_id: None,
        context_facts: vec![
            serde_json::json!({"fact": "z_fact"}),
            serde_json::json!({"fact": "a_fact"}),
        ],
        trace: vec![serde_json::json!({"step": 1})],
        applicable_rules: vec![RuleId::new("rule_z"), RuleId::new("rule_a")],
        defeated_rules: vec![],
        violated_constraints: vec!["z_constraint".into(), "a_constraint".into()],
        models: vec![],
        unsat_cores: vec![],
        source_span_ids: vec![SpanId::new("span_w2"), SpanId::new("span_w1")],
        certificate_ids: vec![],
    }
}

// ---------------------------------------------------------------------------
// Golden NF tests (one module per type)
// ---------------------------------------------------------------------------

golden_nf_test!(gnf_rule, fixture_rule, "rule.json");
golden_nf_test!(
    gnf_clinical_claim,
    fixture_clinical_claim,
    "clinical_claim.json"
);
golden_nf_test!(gnf_concept, fixture_concept, "concept.json");
golden_nf_test!(
    gnf_decision_table,
    fixture_decision_table,
    "decision_table.json"
);
golden_nf_test!(
    gnf_workflow_fragment,
    fixture_workflow_fragment,
    "workflow_fragment.json"
);
golden_nf_test!(
    gnf_argument_graph,
    fixture_argument_graph,
    "argument_graph.json"
);
golden_nf_test!(gnf_conflict, fixture_conflict, "conflict.json");
golden_nf_test!(gnf_patient_case, fixture_patient_case, "patient_case.json");
golden_nf_test!(
    gnf_execution_witness,
    fixture_execution_witness,
    "execution_witness.json"
);

// ---------------------------------------------------------------------------
// Regeneration (ignored by default; run manually to update golden files)
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn regenerate() {
    let dir = golden_dir();
    std::fs::create_dir_all(&dir).unwrap();

    type Fixture = (&'static str, Box<dyn FnOnce() -> Vec<u8>>);
    let fixtures: Vec<Fixture> = vec![
        (
            "rule.json",
            Box::new(|| {
                let mut v = fixture_rule();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
        (
            "clinical_claim.json",
            Box::new(|| {
                let mut v = fixture_clinical_claim();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
        (
            "concept.json",
            Box::new(|| {
                let mut v = fixture_concept();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
        (
            "decision_table.json",
            Box::new(|| {
                let mut v = fixture_decision_table();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
        (
            "workflow_fragment.json",
            Box::new(|| {
                let mut v = fixture_workflow_fragment();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
        (
            "argument_graph.json",
            Box::new(|| {
                let mut v = fixture_argument_graph();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
        (
            "conflict.json",
            Box::new(|| {
                let mut v = fixture_conflict();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
        (
            "patient_case.json",
            Box::new(|| {
                let mut v = fixture_patient_case();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
        (
            "execution_witness.json",
            Box::new(|| {
                let mut v = fixture_execution_witness();
                normalize_all(&mut v);
                to_canonical_bytes(&v)
            }),
        ),
    ];

    for (name, make) in fixtures {
        let bytes = make();
        std::fs::write(dir.join(name), &bytes).unwrap();
        eprintln!("wrote golden NF: {name} ({} bytes)", bytes.len());
    }
}
