//! Gate for task 0.8.6: the SMT-LIB MaxSMT repair emitter is byte-deterministic
//! and grounds both repair candidates to their soft-constraint symbols and the
//! conflict's source spans.

use ckc_compile::smt::emit_repair_maxsmt;
use ckc_compile::{CompileBundle, content_hash};

#[test]
fn emit_repair_maxsmt_is_byte_identical_across_runs() {
    let bundle = CompileBundle::load_toy();
    let a = emit_repair_maxsmt(&bundle);
    let b = emit_repair_maxsmt(&bundle);
    assert_eq!(a.artifact_text, b.artifact_text, "artifact text bytes");
    assert_eq!(content_hash(&a), content_hash(&b), "whole-artifact hash");
}

#[test]
fn emit_repair_maxsmt_artifact_has_maxsmt_skeleton() {
    let target = emit_repair_maxsmt(&CompileBundle::load_toy());
    assert!(target.artifact_text.contains("(set-logic QF_UF)"));
    assert!(target.artifact_text.contains("assert-soft"));
    assert!(target.artifact_text.contains("(check-sat)"));
    assert!(target.artifact_text.contains("(get-objectives)"));
}

#[test]
fn emit_repair_maxsmt_grounds_both_repairs() {
    let bundle = CompileBundle::load_toy();
    let target = emit_repair_maxsmt(&bundle);
    let map = &target.compilation_map.0;

    // The conflict's source spans are attached to every repair candidate.
    let conflict = bundle
        .conflicts
        .iter()
        .find(|c| c.conflict_id.as_str() == "conflict_norm_bl_contradiction")
        .unwrap();
    let conflict_spans: Vec<&str> = conflict.source_spans.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        conflict_spans,
        ["span_rec_sepsis_bl", "span_contra_bl_allergy"]
    );

    // One mapping per repair candidate: the candidate `type` is the ckc node id,
    // `repair_<type>` is the soft-constraint symbol, both carry the conflict
    // spans, and the symbol appears verbatim in the emitted program.
    let expected = [
        ("add_priority", "repair_add_priority"),
        ("add_exception", "repair_add_exception"),
    ];
    assert_eq!(
        map.len(),
        expected.len(),
        "one mapping per repair candidate"
    );
    for (repair_type, sym) in expected {
        let entry = map
            .iter()
            .find(|m| m.ckc_node_id == repair_type)
            .unwrap_or_else(|| panic!("mapping for {repair_type}"));
        assert_eq!(entry.target_symbol, sym, "soft-constraint symbol");
        let spans: Vec<&str> = entry.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(spans, conflict_spans, "conflict spans for {repair_type}");
        assert!(
            target.artifact_text.contains(sym),
            "{sym} appears in the program"
        );
    }

    // source_artifact_hashes is the conflict's content hash — confirms the
    // emitter hashed the conflict, not the underlying rules.
    assert_eq!(target.source_artifact_hashes, vec![content_hash(conflict)]);
}
