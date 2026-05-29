//! CKC → Datalog/Soufflé emitter (SPEC 13.6 static analysis, 14).
//!
//! Phase-0 task 0.8: the rule-superiority priority-cycle analysis
//! (`emit_priority_analysis`). Each rule's `priority_over` edge becomes a
//! `priority_over` input fact; a transitive-closure relation `reaches` and a
//! `cycle` query witness whether the superiority graph is cyclic. The toy set is
//! a single edge, so `cycle` is empty — a witnessed-acyclicity result. Emit-only
//! — the Soufflé parse/run belongs to task 0.9.

use crate::{
    CompilationMap, CompileBundle, CompiledTarget, SymbolMapping, TargetLanguage, build_target,
    content_hash, find_rule, sorted_lines,
};

/// Emit the Datalog/Soufflé priority-cycle program over the toy rules'
/// superiority relation (SPEC 13.6 Datalog/Soufflé static analysis; SPEC 15.1
/// cyclic or contradictory priority relations).
///
/// Each rule's `priority_over` entry becomes one `priority_over(sup, inf)` input
/// fact; the sole toy edge is
/// `rule_bl_anaphylaxis_contra ≻ rule_sepsis_bl_recommend`. The transitive
/// closure `reaches` plus the query `cycle(R) :- reaches(R, R)` derive a rule
/// that reaches itself iff the superiority graph has a cycle. For the toy
/// single-edge set `cycle` is empty — the acyclicity witness.
///
/// The `priority_over` facts go through [`sorted_lines`] so the fact block's
/// bytes depend only on its contents; the relation declarations, the
/// transitive-closure rules, the `cycle` derivation, and the `.output` footer
/// stay in fixed order. The Soufflé parse/run and `cycle.csv` recovery are
/// task 0.9.
pub fn emit_priority_analysis(bundle: &CompileBundle) -> CompiledTarget {
    const HEADER: &str = "\
// CKC -> Datalog/Souffle priority-cycle analysis: conflict_norm_bl_contradiction priority graph
// One priority_over(sup,inf) input fact per rule superiority edge:
//   rule_bl_anaphylaxis_contra > rule_sepsis_bl_recommend
// reaches is the transitive closure of priority_over; cycle(R) holds when a rule
// reaches itself. The toy superiority is a single edge, so cycle is empty -> the
// priority graph is acyclic (SPEC 15.1 cyclic/contradictory priority relations).
// Emit-only: the souffle parse/run is task 0.9.
.decl priority_over(sup:symbol, inf:symbol)
";

    const BODY: &str = "\
.decl reaches(src:symbol, dst:symbol)
reaches(X,Y) :- priority_over(X,Y).
reaches(X,Z) :- priority_over(X,Y), reaches(Y,Z).
.decl cycle(r:symbol)
cycle(R) :- reaches(R,R).
.output cycle
";

    // priority_over(sup,inf) input facts, data-driven from each rule's
    // superiority edges. Rules that carry an edge are the hashed sources; the
    // distinct endpoints (sup then inf, in first-seen order) are the mapped CKC
    // nodes. rule_incomplete_provenance and rule_sepsis_bl_recommend carry no
    // priority_over field, so neither is a carrier.
    let mut facts = Vec::new();
    let mut endpoint_ids: Vec<&str> = Vec::new();
    let mut carriers = Vec::new();
    for rule in &bundle.rules {
        if rule.priority_over.is_empty() {
            continue;
        }
        carriers.push(rule);
        let sup = rule.rule_id.as_str();
        if !endpoint_ids.contains(&sup) {
            endpoint_ids.push(sup);
        }
        for inf in &rule.priority_over {
            let inf = inf.as_str();
            facts.push(format!("priority_over(\"{sup}\",\"{inf}\")."));
            if !endpoint_ids.contains(&inf) {
                endpoint_ids.push(inf);
            }
        }
    }
    let facts = sorted_lines(facts);

    let artifact_text = format!("{HEADER}{facts}{BODY}");

    // Each endpoint rule maps to its Soufflé symbol constant (the quoted form it
    // takes in priority_over), grounded in that rule's own source spans.
    let compilation_map = CompilationMap(
        endpoint_ids
            .iter()
            .map(|&id| SymbolMapping {
                ckc_node_id: id.to_string(),
                target_symbol: format!("\"{id}\""),
                source_span_ids: find_rule(bundle, id).source_span_ids.clone(),
            })
            .collect(),
    );

    let source_artifact_hashes = carriers.iter().map(|r| content_hash(*r)).collect();

    build_target(
        TargetLanguage::Datalog,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}
