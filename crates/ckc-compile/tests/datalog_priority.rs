//! Gate for task 0.8.9: the Datalog/Soufflé priority-cycle emitter is
//! byte-deterministic, carries the `priority_over`/`cycle` skeleton, and grounds
//! each superiority endpoint to its Soufflé symbol and source spans.

use ckc_compile::datalog::emit_priority_analysis;
use ckc_compile::{CompileBundle, content_hash};

#[test]
fn emit_priority_analysis_is_byte_identical_across_runs() {
    let bundle = CompileBundle::load_toy();
    let a = emit_priority_analysis(&bundle);
    let b = emit_priority_analysis(&bundle);
    assert_eq!(a.artifact_text, b.artifact_text, "artifact text bytes");
    assert_eq!(content_hash(&a), content_hash(&b), "whole-artifact hash");
}

#[test]
fn emit_priority_analysis_artifact_has_datalog_skeleton() {
    let target = emit_priority_analysis(&CompileBundle::load_toy());
    assert!(target.artifact_text.contains(".decl priority_over"));
    assert!(target.artifact_text.contains(".output cycle"));
    // The sole toy edge is encoded as a single input fact.
    assert!(
        target.artifact_text.contains(
            "priority_over(\"rule_bl_anaphylaxis_contra\",\"rule_sepsis_bl_recommend\")."
        ),
        "the superiority edge is emitted as a priority_over fact"
    );
}

#[test]
fn emit_priority_analysis_grounds_both_endpoints() {
    let bundle = CompileBundle::load_toy();
    let target = emit_priority_analysis(&bundle);
    let map = &target.compilation_map.0;

    // Endpoints appear superior-first then inferior (first-seen order over the
    // single edge); each maps to its quoted Soufflé symbol and its own spans.
    let sup = map
        .iter()
        .find(|m| m.ckc_node_id == "rule_bl_anaphylaxis_contra")
        .expect("superior mapping");
    assert_eq!(sup.target_symbol, "\"rule_bl_anaphylaxis_contra\"");
    let sup_spans: Vec<&str> = sup.source_span_ids.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        sup_spans,
        ["span_contra_bl_allergy", "span_allergy_history"]
    );

    let inf = map
        .iter()
        .find(|m| m.ckc_node_id == "rule_sepsis_bl_recommend")
        .expect("inferior mapping");
    assert_eq!(inf.target_symbol, "\"rule_sepsis_bl_recommend\"");
    let inf_spans: Vec<&str> = inf.source_span_ids.iter().map(|s| s.as_str()).collect();
    assert_eq!(inf_spans, ["span_rec_sepsis_bl", "span_evidence_sepsis"]);

    // source_artifact_hashes are only the rules that CARRY a priority_over edge —
    // here just rule_bl_anaphylaxis_contra, not its inferior endpoint.
    let carrier = bundle
        .rules
        .iter()
        .find(|r| r.rule_id.as_str() == "rule_bl_anaphylaxis_contra")
        .unwrap();
    assert_eq!(target.source_artifact_hashes, vec![content_hash(carrier)]);
}
