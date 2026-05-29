//! Argument-graph assembly gate for task 0.10.4.
//!
//! `build_argument_graph` loads the authored `ag_sepsis_bl_conflict` oracle,
//! grounds it in the toy bundle (every `source_rule` resolves to a real rule),
//! and normalizes it: the three arguments, the single `rebut` attack, the single
//! `unresolved` defeat, and the `grounded` extension that rejects the recommend
//! argument survive, the graph carries a content-derived `nf-…` id, and two builds
//! agree under canonical hashing.

use ckc_conflict::argument::build_argument_graph;
use ckc_conflict::{CompileBundle, content_hash};
use serde_json::Value;

#[test]
fn graph_has_three_arguments() {
    let graph = build_argument_graph(&CompileBundle::load_toy());
    assert_eq!(graph.arguments.len(), 3);
}

#[test]
fn single_rebut_attack_contra_over_recommend() {
    let graph = build_argument_graph(&CompileBundle::load_toy());
    assert_eq!(graph.attack_edges.len(), 1);
    let attack = &graph.attack_edges[0];
    assert_eq!(attack["attack_type"], "rebut");
    assert_eq!(attack["from"], "arg_contraindicate_bl");
    assert_eq!(attack["to"], "arg_recommend_bl");
}

#[test]
fn single_unresolved_defeat_edge() {
    let graph = build_argument_graph(&CompileBundle::load_toy());
    assert_eq!(graph.defeat_edges.len(), 1);
    assert_eq!(graph.defeat_edges[0]["defeat_status"], "unresolved");
}

#[test]
fn grounded_extension_rejects_recommend() {
    let graph = build_argument_graph(&CompileBundle::load_toy());
    let grounded = graph
        .extension_summaries
        .iter()
        .find(|e| e["semantics"] == "grounded")
        .expect("a grounded extension summary");
    let rejected = grounded["rejected"]
        .as_array()
        .expect("grounded extension carries a `rejected` array");
    assert!(
        rejected.iter().any(|v| v == "arg_recommend_bl"),
        "grounded extension must reject the unprioritized recommendation"
    );
}

#[test]
fn argument_graph_id_is_normalized() {
    let graph = build_argument_graph(&CompileBundle::load_toy());
    assert!(
        graph.argument_graph_id.as_str().starts_with("nf-"),
        "normalize_all must assign a content-derived nf-… id"
    );
}

#[test]
fn every_source_rule_resolves_to_a_bundle_rule() {
    let bundle = CompileBundle::load_toy();
    let graph = build_argument_graph(&bundle);
    for argument in &graph.arguments {
        if let Some(source_rule) = argument.get("source_rule").and_then(Value::as_str) {
            assert!(
                bundle.rules.iter().any(|r| r.rule_id.as_str() == source_rule),
                "argument source_rule {source_rule} is not a bundle rule id"
            );
        }
    }
}

#[test]
fn build_is_deterministic() {
    let bundle = CompileBundle::load_toy();
    assert_eq!(
        content_hash(&build_argument_graph(&bundle)),
        content_hash(&build_argument_graph(&bundle)),
        "two builds of the same graph must produce equal content hashes"
    );
}
