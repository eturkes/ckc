//! Argument-graph assembly (task 0.10.4): SPEC 13.3 / 15.1 #13 Dung-style
//! argumentation over the defeasible rule pair, normalized to a stable graph.
//!
//! The argument graph is authored content, not derived from the rules: the
//! Dung-style arguments, the rebut attack, the unresolved defeat, and the
//! grounded extension that model the sepsis/anaphylaxis conflict live in
//! `argument_graphs.json` (the [`CompileBundle`] carries rules and conflicts but
//! no argument graphs). [`build_argument_graph`] loads that single
//! `ag_sepsis_bl_conflict` oracle, grounds it in the bundle — every `source_rule`
//! must resolve to a real rule and every source span must be a bundle span — then
//! runs CKC Normal Form so the graph carries a content-derived `nf-…` id and
//! sorted arrays.

use ckc_core::nf::normalize_all;
use serde_json::Value;

use crate::{ArgumentGraph, CompileBundle};

/// The authored Dung-style argument-graph oracle for the Phase-0
/// sepsis/beta-lactam conflict: a JSON array carrying the single
/// `ag_sepsis_bl_conflict` graph. The [`CompileBundle`] does not carry argument
/// graphs, so this committed fixture is the content `build_argument_graph`
/// grounds and normalizes.
const ARGUMENT_GRAPHS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/argument_graphs.json");

/// Assemble the normalized Dung-style argument graph for `bundle`'s defeasible
/// conflict (SPEC 13.3 / 15.1 #13).
///
/// Loads the authored `ag_sepsis_bl_conflict` oracle, then grounds it in the
/// bundle: every argument that names a `source_rule` must resolve to a
/// `bundle.rules` id (`arg_recommend_bl → rule_sepsis_bl_recommend`,
/// `arg_contraindicate_bl → rule_bl_anaphylaxis_contra`; the RCT-evidence argument
/// names no rule and is skipped) and every `source_span_ids` entry must be a
/// `bundle.spans` id. [`normalize_all`] then sorts the six graph arrays plus the
/// span set and reassigns a content-derived `nf-…` `argument_graph_id`. Panics
/// when the committed oracle stops resolving against the fixtures — a build-time
/// bug, mirroring [`crate::CompileBundle::load_toy`] and `ckc_compile::find_rule`.
pub fn build_argument_graph(bundle: &CompileBundle) -> ArgumentGraph {
    let mut graph: ArgumentGraph = serde_json::from_str::<Vec<ArgumentGraph>>(ARGUMENT_GRAPHS_JSON)
        .expect("argument_graphs.json must deserialize")
        .into_iter()
        .next()
        .expect("argument_graphs.json must carry the ag_sepsis_bl_conflict graph");

    // Ground every argument's source_rule in bundle.rules. Arguments without a
    // source_rule (the RCT-evidence argument) name no rule and are skipped.
    for argument in &graph.arguments {
        if let Some(source_rule) = argument.get("source_rule").and_then(Value::as_str) {
            assert!(
                bundle
                    .rules
                    .iter()
                    .any(|r| r.rule_id.as_str() == source_rule),
                "argument source_rule {source_rule} must resolve to a bundle rule"
            );
        }
    }

    // Ground every source span in bundle.spans.
    for span in &graph.source_span_ids {
        assert!(
            bundle
                .spans
                .iter()
                .any(|s| s.span_id.as_str() == span.as_str()),
            "argument graph source span {} must resolve to a bundle span",
            span.as_str()
        );
    }

    normalize_all(&mut graph);
    graph
}
