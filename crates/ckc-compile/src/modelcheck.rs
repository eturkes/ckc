//! CKC → model-checker meta-spec emitters (SPEC 13.4 workflow/temporal, 14).
//!
//! Phase-0 task 0.8 generated model-checker meta-spec stubs:
//! - [`emit_tlaplus_stub`] reflects the Event Calculus allergy-persistence conflict
//!   as a bounded boolean state machine whose safety invariant the persisted-allergy
//!   trace violates.
//! - [`emit_alloy_stub`] reflects the rule-superiority priority graph as `Rule`
//!   atoms with a `priority_over` relation and a `NoPriorityCycle` acyclicity check;
//!   the toy single-edge superiority is acyclic, so no counterexample exists.
//!
//! Both are stub-only — the TLC/Apalache and Alloy Analyzer runs are task 0.9.

use crate::{
    CompilationMap, CompileBundle, CompiledTarget, SymbolMapping, TargetLanguage, build_target,
    content_hash, find_rule, sorted_lines,
};

/// Emit the generated TLA+ meta-spec stub for `conflict_ec_allergy_persistence`
/// (SPEC 13.4 protocol transition systems; SPEC 15.1 unsafe-persistence conflict).
///
/// The single `EventNarrative`'s boolean fluents (`allergy_known`, `drug_active`)
/// become TLA+ `VARIABLES`. `Init` starts both fluents `FALSE` (matching the
/// narrative's `initially`); the `Next` skeleton lets each fluent rise, so a trace
/// can set `allergy_known` and then `drug_active`. The safety invariant
/// `AllergyContraindication == allergy_known => ~drug_active` is the
/// unsafe-persistence conflict (SPEC 15.1): once the allergy fluent holds, an
/// active beta-lactam administration breaks it. A future TLC/Apalache run would
/// return that violating trace as a counterexample.
///
/// The `VARIABLES` declaration and the `Init` conjunction are data-driven from the
/// fluents, taken in sorted order so their bytes depend only on the fluent set,
/// not on fixture discovery order. The header comment, the `Next` skeleton, the
/// invariant, and the module footer stay in fixed order — the invariant encodes
/// the directional contraindication semantics specific to this conflict, mirroring
/// the fixed violation query of [`crate::asp::emit_event_calculus`]. The
/// TLC/Apalache run is task 0.9.
pub fn emit_tlaplus_stub(bundle: &CompileBundle) -> CompiledTarget {
    let narrative = bundle
        .event_narratives
        .first()
        .expect("toy bundle must contain an event narrative");

    // Fluents in sorted order: the declaration/Init bytes then depend only on the
    // fluent set. The toy set is {allergy_known, drug_active}, already sorted.
    let mut fluents: Vec<&str> = narrative.fluent_types.iter().map(String::as_str).collect();
    fluents.sort_unstable();

    let variables = fluents.join(", ");
    let init_block = fluents
        .iter()
        .map(|f| format!("  /\\ {f} = FALSE"))
        .collect::<Vec<_>>()
        .join("\n");

    let artifact_text = format!(
        "\
---- MODULE Conflict ----
\\* CKC -> TLA+ meta-spec stub: conflict_ec_allergy_persistence (SPEC 13.4, 15.1).
\\* Generated stub reflecting the Event Calculus allergy-persistence conflict as a
\\* bounded boolean state machine over the narrative's fluents: once allergy_known
\\* holds, an active beta-lactam administration (drug_active) violates the
\\* contraindication invariant AllergyContraindication (SPEC 15.1 unsafe persistence).
\\* Stub only: no TLC/Apalache run (task 0.9).

VARIABLES {variables}

Init ==
{init_block}

Next ==
  \\/ /\\ allergy_known' = TRUE
     /\\ UNCHANGED drug_active
  \\/ /\\ drug_active' = TRUE
     /\\ UNCHANGED allergy_known

AllergyContraindication == allergy_known => ~drug_active
====
"
    );

    // Each fluent maps to the TLA+ variable of the same name, grounded in the
    // narrative's shared source spans.
    let compilation_map = CompilationMap(
        fluents
            .iter()
            .map(|&f| SymbolMapping {
                ckc_node_id: f.to_string(),
                target_symbol: f.to_string(),
                source_span_ids: narrative.source_span_ids.clone(),
            })
            .collect(),
    );

    let source_artifact_hashes = vec![content_hash(narrative)];

    build_target(
        TargetLanguage::TlaPlus,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}

/// Emit the generated Alloy meta-spec stub for the rule-superiority priority
/// graph (SPEC 14 Alloy/Forge target; SPEC 15.1 cyclic/contradictory priority
/// relations).
///
/// Each endpoint of a `priority_over` edge becomes a `one sig … extends Rule`
/// atom; the `Superiority` fact pins the whole `priority_over` relation to the
/// toy edges, the sole one being
/// `rule_bl_anaphylaxis_contra -> rule_sepsis_bl_recommend`. The
/// `check NoPriorityCycle` predicate searches for a rule reachable from itself
/// through the transitive closure `^priority_over`. The toy superiority is a
/// single edge, so no such rule exists — the acyclicity witness, mirroring the
/// empty `cycle` relation of [`crate::datalog::emit_priority_analysis`].
///
/// The `one sig` atom block goes through [`sorted_lines`] and the relation's
/// union of `sup -> inf` tuples is sorted, so the data-driven bytes depend only
/// on the edge set; the `Rule` signature, the fact wrapper, and the `check`
/// predicate stay in fixed order. Stub only — the Alloy Analyzer run is task 0.9.
pub fn emit_alloy_stub(bundle: &CompileBundle) -> CompiledTarget {
    // Walk the superiority edges. A rule with a non-empty priority_over is a
    // superior endpoint; each id it lists is an inferior endpoint. The endpoints
    // (superior-first, first-seen) are both the mapped CKC nodes and the hashed
    // sources — the rules that appear in the priority graph. rule_sepsis_bl_recommend
    // and rule_incomplete_provenance carry no priority_over field, so neither is a
    // superior; the former still appears as an inferior endpoint.
    let mut endpoint_ids: Vec<&str> = Vec::new();
    let mut edges: Vec<String> = Vec::new();
    for rule in &bundle.rules {
        if rule.priority_over.is_empty() {
            continue;
        }
        let sup = rule.rule_id.as_str();
        if !endpoint_ids.contains(&sup) {
            endpoint_ids.push(sup);
        }
        for inf in &rule.priority_over {
            let inf = inf.as_str();
            edges.push(format!("{sup} -> {inf}"));
            if !endpoint_ids.contains(&inf) {
                endpoint_ids.push(inf);
            }
        }
    }

    // One singleton atom per endpoint; sorted so the block's bytes depend only on
    // the endpoint set.
    let sig_block = sorted_lines(
        endpoint_ids
            .iter()
            .map(|id| format!("one sig {id} extends Rule {{}}"))
            .collect(),
    );

    // The whole priority_over relation as a union of sup -> inf tuples, sorted so
    // the fact body depends only on the edge set. The toy set is a single edge.
    edges.sort();
    let superiority = edges.join(" + ");

    let artifact_text = format!(
        "\
// CKC -> Alloy meta-spec stub: conflict_norm_bl_contradiction priority graph (SPEC 14, 15.1).
// Each Rule atom carries priority_over superiority edges; the sole toy edge is
//   rule_bl_anaphylaxis_contra -> rule_sepsis_bl_recommend.
// check NoPriorityCycle searches for a rule reachable from itself through the
// transitive closure ^priority_over. The toy superiority is a single edge, so no
// counterexample exists -> the priority graph is acyclic (SPEC 15.1
// cyclic/contradictory priority relations). Stub only: no Alloy Analyzer run (task 0.9).

abstract sig Rule {{
  priority_over: set Rule
}}

{sig_block}
fact Superiority {{
  priority_over = {superiority}
}}

check NoPriorityCycle {{ no r: Rule | r in r.^priority_over }}
"
    );

    // Each endpoint rule maps to its bare Alloy atom (the `one sig` name, which is
    // the rule_id itself), grounded in that rule's own source spans.
    let compilation_map = CompilationMap(
        endpoint_ids
            .iter()
            .map(|&id| SymbolMapping {
                ckc_node_id: id.to_string(),
                target_symbol: id.to_string(),
                source_span_ids: find_rule(bundle, id).source_span_ids.clone(),
            })
            .collect(),
    );

    let source_artifact_hashes = endpoint_ids
        .iter()
        .map(|&id| content_hash(find_rule(bundle, id)))
        .collect();

    build_target(
        TargetLanguage::Alloy,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}
