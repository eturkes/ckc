use ckc_core::artifact::{DecisionRow, DecisionTable};
use ckc_core::canonical::{content_hash, to_canonical_bytes};
use ckc_core::clinical::{Action, ClinicalClaim, Norm, Rule};
use ckc_core::enums::*;
use ckc_core::id::*;
use ckc_core::nf::normalize_all;
use ckc_core::profile::SemanticProfile;
use ckc_core::verify::ArgumentGraph;
use proptest::prelude::*;

fn arb_span_ids() -> impl Strategy<Value = Vec<SpanId>> {
    prop::collection::vec("[a-z]{3,6}".prop_map(SpanId::new), 0..4)
}

fn arb_rule_kind() -> impl Strategy<Value = RuleKind> {
    prop_oneof![
        Just(RuleKind::Strict),
        Just(RuleKind::Defeasible),
        Just(RuleKind::Defeater),
    ]
}

fn arb_hit_policy() -> impl Strategy<Value = HitPolicy> {
    prop_oneof![
        Just(HitPolicy::Unique),
        Just(HitPolicy::Any),
        Just(HitPolicy::Priority),
        Just(HitPolicy::First),
        Just(HitPolicy::Collect),
    ]
}

fn arb_action_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("administer".to_string()),
        Just("ADMINISTER".to_string()),
        Just("Administer".to_string()),
        Just("contraindicate".to_string()),
        Just("CONTRAINDICATE".to_string()),
    ]
}

fn arb_unit() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("mL".to_string()),
        Just("ml".to_string()),
        Just("ML".to_string()),
        Just("Cel".to_string()),
        Just("degC".to_string()),
        Just("mg".to_string()),
    ]
}

fn arb_antecedent() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("(dx sepsis) AND (adult patient)".to_string()),
        Just("(adult patient) AND (dx sepsis)".to_string()),
        Just("(A) AND (B) AND (C)".to_string()),
        Just("(C) AND (A) AND (B)".to_string()),
        Just("single_predicate".to_string()),
        Just("X OR Y".to_string()),
        Just("Y OR X".to_string()),
    ]
}

fn arb_rule() -> impl Strategy<Value = Rule> {
    (
        arb_antecedent(),
        arb_antecedent(),
        arb_rule_kind(),
        arb_action_type(),
        arb_unit(),
        arb_span_ids(),
        prop::bool::ANY,
    )
        .prop_map(
            |(antecedent, context, kind, action_type, unit, span_ids, has_norm)| Rule {
                rule_id: RuleId::new("rule_proptest"),
                profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
                kind,
                context,
                antecedent,
                consequent: "(result)".into(),
                norm: if has_norm {
                    Some(Norm {
                        context: "test".into(),
                        direction: RecommendationDirection::For,
                        action: Action {
                            action_type,
                            target_concept: ConceptId::new("concept_test"),
                            parameters: serde_json::json!({"route": "iv"}),
                            temporal_constraints: serde_json::Value::Null,
                            quantity_constraints: serde_json::json!({"unit": unit}),
                        },
                        recommendation_strength: RecommendationStrength::Strong,
                        evidence_certainty: EvidenceCertainty::Moderate,
                        original_modality_phrase_ja: "投与を推奨する".into(),
                        deontic_projection: DeonticProjection::Recommended,
                        exception_policy: "none".into(),
                        prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
                    })
                } else {
                    None
                },
                priority_over: vec![],
                exceptions: vec!["exc_z".into(), "exc_a".into()],
                temporal_scope: None,
                population_scope: None,
                source_span_ids: span_ids,
                provenance: "proptest".into(),
                certificate_ids: vec![],
            },
        )
}

fn arb_clinical_claim() -> impl Strategy<Value = ClinicalClaim> {
    arb_span_ids().prop_map(|span_ids| ClinicalClaim {
        claim_id: ClaimId::new("claim_proptest"),
        claim_type: "recommendation".into(),
        profiles: vec![SemanticProfile::Norm, SemanticProfile::Evidence],
        source_span_ids: span_ids,
        pico: None,
        etd: None,
        evidence_atoms: vec![],
        rule_ids: vec![RuleId::new("rule_z"), RuleId::new("rule_a")],
        decision_table_ids: vec![],
        workflow_fragment_ids: vec![],
        gloss_ja: "βラクタム系\u{3000}抗菌薬".into(),
        gloss_en: "Beta-lactam  antibiotics".into(),
        status: "candidate".into(),
    })
}

fn arb_decision_table() -> impl Strategy<Value = DecisionTable> {
    (arb_hit_policy(), arb_unit()).prop_map(|(policy, unit)| DecisionTable {
        table_id: DecisionTableId::new("dt_proptest"),
        hit_policy: policy,
        input_columns: vec!["体温\u{3000}".into()],
        output_columns: vec!["action".into()],
        rows: vec![
            DecisionRow {
                row_id: DecisionRowId::new("row_z"),
                conditions: vec![serde_json::json!({"temp_unit": unit})],
                outputs: vec![serde_json::json!({"action": "alert"})],
                priority: None,
                source_span_ids: vec![],
                cell_refs: vec![],
            },
            DecisionRow {
                row_id: DecisionRowId::new("row_a"),
                conditions: vec![serde_json::json!({"value": 37})],
                outputs: vec![serde_json::json!({"action": "normal"})],
                priority: None,
                source_span_ids: vec![],
                cell_refs: vec![],
            },
        ],
        source_table_id: None,
        dmn_export_id: None,
        certificate_ids: vec![],
    })
}

fn arb_argument_graph() -> impl Strategy<Value = ArgumentGraph> {
    arb_span_ids().prop_map(|span_ids| ArgumentGraph {
        argument_graph_id: ArgumentGraphId::new("ag_proptest"),
        arguments: vec![
            serde_json::json!({"id": "z_arg", "claim": "recommend"}),
            serde_json::json!({"id": "a_arg", "claim": "contraindicate"}),
        ],
        attack_edges: vec![
            serde_json::json!({"from": "z_arg", "to": "a_arg"}),
            serde_json::json!({"from": "a_arg", "to": "z_arg"}),
        ],
        support_edges: vec![],
        undercut_edges: vec![],
        defeat_edges: vec![],
        extension_summaries: vec![],
        source_span_ids: span_ids,
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn nf_idempotent_rule(mut rule in arb_rule()) {
        normalize_all(&mut rule);
        let hash1 = content_hash(&rule);
        let bytes1 = to_canonical_bytes(&rule);

        normalize_all(&mut rule);
        let hash2 = content_hash(&rule);
        let bytes2 = to_canonical_bytes(&rule);

        prop_assert_eq!(hash1, hash2, "NF(NF(rule)) must equal NF(rule)");
        prop_assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn nf_idempotent_clinical_claim(mut claim in arb_clinical_claim()) {
        normalize_all(&mut claim);
        let hash1 = content_hash(&claim);
        let bytes1 = to_canonical_bytes(&claim);

        normalize_all(&mut claim);
        let hash2 = content_hash(&claim);
        let bytes2 = to_canonical_bytes(&claim);

        prop_assert_eq!(hash1, hash2, "NF(NF(claim)) must equal NF(claim)");
        prop_assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn nf_idempotent_decision_table(mut dt in arb_decision_table()) {
        normalize_all(&mut dt);
        let hash1 = content_hash(&dt);
        let bytes1 = to_canonical_bytes(&dt);

        normalize_all(&mut dt);
        let hash2 = content_hash(&dt);
        let bytes2 = to_canonical_bytes(&dt);

        prop_assert_eq!(hash1, hash2, "NF(NF(dt)) must equal NF(dt)");
        prop_assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn nf_idempotent_argument_graph(mut ag in arb_argument_graph()) {
        normalize_all(&mut ag);
        let hash1 = content_hash(&ag);
        let bytes1 = to_canonical_bytes(&ag);

        normalize_all(&mut ag);
        let hash2 = content_hash(&ag);
        let bytes2 = to_canonical_bytes(&ag);

        prop_assert_eq!(hash1, hash2, "NF(NF(ag)) must equal NF(ag)");
        prop_assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn stable_id_always_starts_with_nf_prefix(mut rule in arb_rule()) {
        normalize_all(&mut rule);
        prop_assert!(rule.rule_id.as_str().starts_with("nf-"));
        prop_assert_eq!(rule.rule_id.as_str().len(), 35);
    }
}
