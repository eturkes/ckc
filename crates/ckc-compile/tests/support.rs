//! Support gate for task 0.8.3: the shared emitter helpers are deterministic.
//! `sorted_lines` yields an order-independent block, and `build_target` fills
//! the emit-only `CompiledTarget` invariants while preserving its inputs.

use ckc_compile::{
    CompilationMap, CompiledTarget, ContentHash, SymbolMapping, TargetLanguage, build_target,
    replay_command, sorted_lines,
};

#[test]
fn sorted_lines_sorts_and_is_order_independent() {
    // Sorted, '\n'-joined, single trailing newline.
    assert_eq!(sorted_lines(vec!["b".into(), "a".into()]), "a\nb\n");
    // Two input orders produce the same block.
    let one = sorted_lines(vec!["a".into(), "b".into(), "c".into()]);
    let two = sorted_lines(vec!["c".into(), "a".into(), "b".into()]);
    assert_eq!(one, two);
    assert_eq!(one, "a\nb\nc\n");
}

#[test]
fn build_target_fills_emit_only_invariants() {
    let map = CompilationMap(vec![SymbolMapping {
        ckc_node_id: "rule_sepsis_bl_recommend".into(),
        target_symbol: "recommend_administer_beta_lactam".into(),
        source_span_ids: vec![],
    }]);
    let hashes = vec![ContentHash("sha256:abc".into())];
    let target: CompiledTarget = build_target(
        TargetLanguage::SmtLib,
        "(check-sat)\n".into(),
        map.clone(),
        hashes.clone(),
    );

    // Emit-only invariants.
    assert_eq!(
        target.replay_command,
        replay_command(TargetLanguage::SmtLib)
    );
    assert!(target.diagnostics.is_empty());
    assert_eq!(target.target_parse_ok, None);

    // Passed fields stored verbatim.
    assert_eq!(target.target_language, TargetLanguage::SmtLib);
    assert_eq!(target.artifact_text, "(check-sat)\n");
    assert_eq!(target.compilation_map, map);
    assert_eq!(target.source_artifact_hashes, hashes);
}
