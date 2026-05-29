//! Gate for task 0.8.12: the Alloy stub emitter is byte-deterministic, carries the
//! Rule-signature/priority_over/NoPriorityCycle skeleton, and grounds each
//! superiority endpoint to its Alloy atom and source spans.

use ckc_compile::modelcheck::emit_alloy_stub;
use ckc_compile::{CompileBundle, content_hash};

#[test]
fn emit_alloy_stub_is_byte_identical_across_runs() {
    let bundle = CompileBundle::load_toy();
    let a = emit_alloy_stub(&bundle);
    let b = emit_alloy_stub(&bundle);
    assert_eq!(a.artifact_text, b.artifact_text, "artifact text bytes");
    assert_eq!(content_hash(&a), content_hash(&b), "whole-artifact hash");
}

#[test]
fn emit_alloy_stub_has_priority_skeleton() {
    let target = emit_alloy_stub(&CompileBundle::load_toy());
    assert!(target.artifact_text.contains("sig Rule"), "Rule signature");
    assert!(
        target.artifact_text.contains("priority_over: set Rule"),
        "priority_over field"
    );
    assert!(
        target.artifact_text.contains("check NoPriorityCycle"),
        "acyclicity check"
    );
    // The sole toy edge populates the priority_over relation in the fact.
    assert!(
        target
            .artifact_text
            .contains("priority_over = rule_bl_anaphylaxis_contra -> rule_sepsis_bl_recommend"),
        "the superiority edge populates the priority_over relation"
    );
}

#[test]
fn emit_alloy_stub_grounds_both_endpoints() {
    let bundle = CompileBundle::load_toy();
    let target = emit_alloy_stub(&bundle);
    let map = &target.compilation_map.0;

    // Endpoints appear superior-first then inferior (first-seen order over the
    // single edge); each maps to its bare Alloy atom and its own spans.
    let ids: Vec<&str> = map.iter().map(|m| m.ckc_node_id.as_str()).collect();
    assert_eq!(
        ids,
        ["rule_bl_anaphylaxis_contra", "rule_sepsis_bl_recommend"]
    );

    let sup = map
        .iter()
        .find(|m| m.ckc_node_id == "rule_bl_anaphylaxis_contra")
        .expect("superior mapping");
    assert_eq!(sup.target_symbol, "rule_bl_anaphylaxis_contra");
    let sup_spans: Vec<&str> = sup.source_span_ids.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        sup_spans,
        ["span_contra_bl_allergy", "span_allergy_history"]
    );

    let inf = map
        .iter()
        .find(|m| m.ckc_node_id == "rule_sepsis_bl_recommend")
        .expect("inferior mapping");
    assert_eq!(inf.target_symbol, "rule_sepsis_bl_recommend");
    let inf_spans: Vec<&str> = inf.source_span_ids.iter().map(|s| s.as_str()).collect();
    assert_eq!(inf_spans, ["span_rec_sepsis_bl", "span_evidence_sepsis"]);

    // source_artifact_hashes hash both endpoint rules, in endpoint order.
    let contra = bundle
        .rules
        .iter()
        .find(|r| r.rule_id.as_str() == "rule_bl_anaphylaxis_contra")
        .unwrap();
    let recommend = bundle
        .rules
        .iter()
        .find(|r| r.rule_id.as_str() == "rule_sepsis_bl_recommend")
        .unwrap();
    assert_eq!(
        target.source_artifact_hashes,
        vec![content_hash(contra), content_hash(recommend)]
    );
}
