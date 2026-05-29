//! Gate for task 0.8.11: the TLA+ stub emitter is byte-deterministic, carries the
//! module/VARIABLES/invariant skeleton reflecting the persistence conflict, and
//! grounds each fluent to its TLA+ variable and the narrative source spans.

use ckc_compile::modelcheck::emit_tlaplus_stub;
use ckc_compile::{CompileBundle, content_hash};

#[test]
fn emit_tlaplus_stub_is_byte_identical_across_runs() {
    let bundle = CompileBundle::load_toy();
    let a = emit_tlaplus_stub(&bundle);
    let b = emit_tlaplus_stub(&bundle);
    assert_eq!(a.artifact_text, b.artifact_text, "artifact text bytes");
    assert_eq!(content_hash(&a), content_hash(&b), "whole-artifact hash");
}

#[test]
fn emit_tlaplus_stub_has_module_skeleton() {
    let target = emit_tlaplus_stub(&CompileBundle::load_toy());
    assert!(
        target.artifact_text.contains("MODULE Conflict"),
        "module header"
    );
    assert!(
        target
            .artifact_text
            .contains("VARIABLES allergy_known, drug_active"),
        "fluents declared as TLA+ variables"
    );
    // The safety invariant is the unsafe-persistence conflict witness.
    assert!(
        target
            .artifact_text
            .contains("AllergyContraindication == allergy_known => ~drug_active"),
        "contraindication invariant reflecting conflict_ec_allergy_persistence"
    );
}

#[test]
fn emit_tlaplus_stub_grounds_fluents() {
    let bundle = CompileBundle::load_toy();
    let target = emit_tlaplus_stub(&bundle);
    let map = &target.compilation_map.0;

    // Each fluent maps to the TLA+ variable of the same name (sorted order).
    let ids: Vec<&str> = map.iter().map(|m| m.ckc_node_id.as_str()).collect();
    assert_eq!(ids, ["allergy_known", "drug_active"]);
    let symbols: Vec<&str> = map.iter().map(|m| m.target_symbol.as_str()).collect();
    assert_eq!(symbols, ["allergy_known", "drug_active"]);

    // Every mapping carries the narrative's shared source spans.
    for m in map {
        let spans: Vec<&str> = m.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(spans, ["span_allergy_history", "span_contra_bl_allergy"]);
    }

    // source_artifact_hashes is the single EventNarrative's content hash.
    let narrative = bundle.event_narratives.first().unwrap();
    assert_eq!(target.source_artifact_hashes, vec![content_hash(narrative)]);
}
