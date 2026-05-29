//! Gate for task 0.8.7: the ASP/Clingo defeasible + argumentation emitter is
//! byte-deterministic and grounds both defeasible rules to their `applicable/1`
//! atoms and source spans.

use ckc_compile::asp::emit_defeasible;
use ckc_compile::{CompileBundle, content_hash};

#[test]
fn emit_defeasible_is_byte_identical_across_runs() {
    let bundle = CompileBundle::load_toy();
    let a = emit_defeasible(&bundle);
    let b = emit_defeasible(&bundle);
    assert_eq!(a.artifact_text, b.artifact_text, "artifact text bytes");
    assert_eq!(content_hash(&a), content_hash(&b), "whole-artifact hash");
}

#[test]
fn emit_defeasible_artifact_has_superiority_and_attack() {
    let target = emit_defeasible(&CompileBundle::load_toy());
    assert!(
        target
            .artifact_text
            .contains("superior(rule_bl_anaphylaxis_contra,rule_sepsis_bl_recommend)."),
        "superiority fact present"
    );
    assert!(
        target.artifact_text.contains("attacks("),
        "an attacks/2 atom present"
    );
}

#[test]
fn emit_defeasible_grounds_both_rules() {
    let bundle = CompileBundle::load_toy();
    let target = emit_defeasible(&bundle);
    let map = &target.compilation_map.0;

    let rec = map
        .iter()
        .find(|m| m.ckc_node_id == "rule_sepsis_bl_recommend")
        .expect("recommend mapping");
    assert_eq!(rec.target_symbol, "applicable(rule_sepsis_bl_recommend)");
    let rec_spans: Vec<&str> = rec.source_span_ids.iter().map(|s| s.as_str()).collect();
    assert_eq!(rec_spans, ["span_rec_sepsis_bl", "span_evidence_sepsis"]);

    let contra = map
        .iter()
        .find(|m| m.ckc_node_id == "rule_bl_anaphylaxis_contra")
        .expect("contra mapping");
    assert_eq!(
        contra.target_symbol,
        "applicable(rule_bl_anaphylaxis_contra)"
    );
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
