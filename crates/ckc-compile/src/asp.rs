//! CKC → ASP/Clingo emitters (SPEC 13.3, 14).
//!
//! Phase-0 task 0.8: the defeasible + argumentation encoding
//! (`emit_defeasible`). `CompileBundle` carries no argument graph, so the
//! Dung-style argumentation view is derived here from the toy rules' superiority
//! relation. Emit-only — the clingo run, stable-model recovery, and
//! grounded-extension extraction belong to task 0.9.

use crate::{
    CompilationMap, CompileBundle, CompiledTarget, SymbolMapping, TargetLanguage, build_target,
    content_hash, find_rule, sorted_lines,
};

/// Emit the ASP/Clingo defeasible + argumentation program for
/// `conflict_norm_bl_contradiction` (SPEC 13.3: defeasible priority and a
/// Dung-style argumentation view exported as a diagnostic).
///
/// The two defeasible norm rules `rule_sepsis_bl_recommend` and
/// `rule_bl_anaphylaxis_contra` and their superiority
/// (`rule_bl_anaphylaxis_contra` ≻ `rule_sepsis_bl_recommend`, read from
/// `priority_over`) compile to two coherent views:
///
/// * defeasible layer — `applicable/1` + `superior/2` facts and a `defeated/1`
///   derivation that marks the dominated rule;
/// * argumentation layer — one `argument/1` per rule and one `attacks/2` edge
///   per superiority, with the grounded extension labelled by `in/1` and
///   `defeated_arg/1`. The derived attack graph is acyclic, so the program is
///   locally stratified and clingo yields a unique stable model — the grounded
///   extension `{arg_rule_bl_anaphylaxis_contra}`.
///
/// Facts and derivation rules go through [`sorted_lines`] so their bytes depend
/// only on content; the `#show` footer stays in fixed order. The clingo run is
/// task 0.9.
pub fn emit_defeasible(bundle: &CompileBundle) -> CompiledTarget {
    const RULE_RECOMMEND: &str = "rule_sepsis_bl_recommend";
    const RULE_CONTRA: &str = "rule_bl_anaphylaxis_contra";

    const HEADER: &str = "\
% CKC -> ASP/Clingo defeasible + argumentation: conflict_norm_bl_contradiction
% Two defeasible norm rules and their superiority, as a Dung argumentation view:
%   rule_bl_anaphylaxis_contra > rule_sepsis_bl_recommend
% defeasible layer: applicable/1 + superior/2 facts, defeated/1 derivation.
% argumentation layer: argument/1 + attacks/2, grounded extension in/1 + defeated_arg/1.
% The derived attack graph is acyclic -> locally stratified -> unique stable model.
% Emit-only: the clingo run is task 0.9.
";

    const FOOTER: &str = "\
#show in/1.
#show defeated/1.
#show defeated_arg/1.
";

    // The two defeasible norm rules participating in the Dung view, in fixed
    // clinical order. rule_incomplete_provenance is strict and ungrounded, so it
    // stays outside the defeasible/argumentation encoding.
    let rule_recommend = find_rule(bundle, RULE_RECOMMEND);
    let rule_contra = find_rule(bundle, RULE_CONTRA);
    let rules = [rule_recommend, rule_contra];

    // Facts, data-driven from the rules and their superiority. Each rule yields
    // an applicable/1 and an argument/1 fact; each priority_over edge whose
    // endpoints are both in view yields a superior/2 fact and the corresponding
    // attacks/2 edge over the arg_<rule_id> arguments.
    let mut facts = Vec::new();
    for rule in &rules {
        let id = rule.rule_id.as_str();
        facts.push(format!("applicable({id})."));
        facts.push(format!("argument(arg_{id})."));
    }
    for sup in &rules {
        let sup_id = sup.rule_id.as_str();
        for inf in &sup.priority_over {
            let inf_id = inf.as_str();
            if rules.iter().any(|r| r.rule_id.as_str() == inf_id) {
                facts.push(format!("superior({sup_id},{inf_id})."));
                facts.push(format!("attacks(arg_{sup_id},arg_{inf_id})."));
            }
        }
    }
    let facts = sorted_lines(facts);

    // Derivation rules: the defeasible defeated/1 view, and the grounded-extension
    // labelling (in/1, defeated_arg/1) over the derived attack graph. An argument
    // is in when no undefeated attacker remains; an argument is defeated_arg when
    // an in argument attacks it.
    let derivation = sorted_lines(vec![
        "defeated(R) :- superior(S,R), applicable(S), applicable(R).".to_string(),
        "in(A) :- argument(A), not attacked_by_undefeated(A).".to_string(),
        "attacked_by_undefeated(A) :- attacks(B,A), not defeated_arg(B).".to_string(),
        "defeated_arg(A) :- attacks(B,A), in(B).".to_string(),
    ]);

    let artifact_text = format!("{HEADER}{facts}{derivation}{FOOTER}");

    // Each rule maps to its applicable/1 atom, grounded in the rule's own spans.
    let compilation_map = CompilationMap(
        rules
            .iter()
            .map(|rule| {
                let id = rule.rule_id.as_str();
                SymbolMapping {
                    ckc_node_id: id.to_string(),
                    target_symbol: format!("applicable({id})"),
                    source_span_ids: rule.source_span_ids.clone(),
                }
            })
            .collect(),
    );

    let source_artifact_hashes = vec![content_hash(rule_recommend), content_hash(rule_contra)];

    build_target(
        TargetLanguage::Asp,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}
