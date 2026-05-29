//! Gate for task 0.8.10: the Lean norm-conflict theorem emitter is
//! byte-deterministic, emits a sorry/admit-free theorem, and grounds both
//! conflicting rules to their Lean norm-predicate identifiers and source spans.

use ckc_compile::lean::emit_norm_conflict_theorem;
use ckc_compile::{CompileBundle, content_hash};

#[test]
fn emit_norm_conflict_theorem_is_byte_identical_across_runs() {
    let bundle = CompileBundle::load_toy();
    let a = emit_norm_conflict_theorem(&bundle);
    let b = emit_norm_conflict_theorem(&bundle);
    assert_eq!(a.artifact_text, b.artifact_text, "artifact text bytes");
    assert_eq!(content_hash(&a), content_hash(&b), "whole-artifact hash");
}

#[test]
fn emit_norm_conflict_theorem_is_sorry_admit_free() {
    let target = emit_norm_conflict_theorem(&CompileBundle::load_toy());
    assert!(
        !target.artifact_text.contains("sorry"),
        "emitted Lean carries no sorry"
    );
    assert!(
        !target.artifact_text.contains("admit"),
        "emitted Lean carries no admit"
    );
    assert!(
        target
            .artifact_text
            .contains("theorem unprioritized_norm_conflict"),
        "emitted Lean states the norm-conflict theorem"
    );
}

#[test]
fn emit_norm_conflict_theorem_grounds_both_rules() {
    let bundle = CompileBundle::load_toy();
    let target = emit_norm_conflict_theorem(&bundle);
    let map = &target.compilation_map.0;

    // Each rule maps to the Lean norm predicate derived from its deontic
    // projection (`recommended` -> recommend, `prohibited` -> prohibit), grounded
    // in the rule's own spans.
    let rec = map
        .iter()
        .find(|m| m.ckc_node_id == "rule_sepsis_bl_recommend")
        .expect("recommend mapping");
    assert_eq!(rec.target_symbol, "recommend");
    let rec_spans: Vec<&str> = rec.source_span_ids.iter().map(|s| s.as_str()).collect();
    assert_eq!(rec_spans, ["span_rec_sepsis_bl", "span_evidence_sepsis"]);

    let contra = map
        .iter()
        .find(|m| m.ckc_node_id == "rule_bl_anaphylaxis_contra")
        .expect("contra mapping");
    assert_eq!(contra.target_symbol, "prohibit");
    let contra_spans: Vec<&str> = contra.source_span_ids.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        contra_spans,
        ["span_contra_bl_allergy", "span_allergy_history"]
    );

    // source_artifact_hashes are the two consumed rules' content hashes, in emit
    // order — confirms the emitter hashed the rules, not the conflict.
    let rule_recommend = bundle
        .rules
        .iter()
        .find(|r| r.rule_id.as_str() == "rule_sepsis_bl_recommend")
        .unwrap();
    let rule_contra = bundle
        .rules
        .iter()
        .find(|r| r.rule_id.as_str() == "rule_bl_anaphylaxis_contra")
        .unwrap();
    assert_eq!(
        target.source_artifact_hashes,
        vec![content_hash(rule_recommend), content_hash(rule_contra)]
    );
}
