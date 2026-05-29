//! Gate for task 0.8.8: the ASP/Clingo Event Calculus emitter is
//! byte-deterministic, derives the persisted-allergy violation witness, and
//! grounds every event/fluent to its introducing atom and source spans.

use ckc_compile::asp::emit_event_calculus;
use ckc_compile::{CompileBundle, content_hash};

#[test]
fn emit_event_calculus_is_byte_identical_across_runs() {
    let bundle = CompileBundle::load_toy();
    let a = emit_event_calculus(&bundle);
    let b = emit_event_calculus(&bundle);
    assert_eq!(a.artifact_text, b.artifact_text, "artifact text bytes");
    assert_eq!(content_hash(&a), content_hash(&b), "whole-artifact hash");
}

#[test]
fn emit_event_calculus_has_persistence_and_violation() {
    let target = emit_event_calculus(&CompileBundle::load_toy());
    assert!(
        target.artifact_text.contains("holdsAt(allergy_known,10)"),
        "persisted allergy fluent at the administration instant"
    );
    assert!(
        target
            .artifact_text
            .contains("happens(administer_drug,10)."),
        "administration event fact"
    );
}

#[test]
fn emit_event_calculus_grounds_events_and_fluents() {
    let bundle = CompileBundle::load_toy();
    let target = emit_event_calculus(&bundle);
    let map = &target.compilation_map.0;

    // Events first (event_types order), then fluents (fluent_types order); each
    // name maps to the ground fact that introduces it.
    let ids: Vec<&str> = map.iter().map(|m| m.ckc_node_id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "detect_allergy",
            "administer_drug",
            "allergy_known",
            "drug_active"
        ]
    );
    let symbols: Vec<&str> = map.iter().map(|m| m.target_symbol.as_str()).collect();
    assert_eq!(
        symbols,
        [
            "happens(detect_allergy,0)",
            "happens(administer_drug,10)",
            "initiates(detect_allergy,allergy_known,0)",
            "initiates(administer_drug,drug_active,10)"
        ]
    );

    // Every mapping carries the narrative's shared source spans.
    for m in map {
        let spans: Vec<&str> = m.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(spans, ["span_allergy_history", "span_contra_bl_allergy"]);
    }

    // source_artifact_hashes is the single EventNarrative's content hash.
    let narrative = bundle.event_narratives.first().unwrap();
    assert_eq!(target.source_artifact_hashes, vec![content_hash(narrative)]);
}
