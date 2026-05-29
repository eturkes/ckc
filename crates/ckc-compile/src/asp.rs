//! CKC → ASP/Clingo emitters (SPEC 13.3, 14).
//!
//! Phase-0 task 0.8: the defeasible + argumentation encoding (`emit_defeasible`)
//! and the Event Calculus persistence/clipping narrative
//! (`emit_event_calculus`). `CompileBundle` carries no argument graph, so the
//! Dung-style argumentation view is derived here from the toy rules' superiority
//! relation. Emit-only — the clingo run, stable-model recovery,
//! grounded-extension extraction, and holdsAt/violation witness recovery belong
//! to task 0.9.

use std::collections::BTreeMap;

use serde_json::Value;

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

/// Read a required string field from an open-JSON narrative entry. Panics when a
/// committed fixture stops carrying it — a build-time bug, mirroring
/// [`crate::find_rule`].
fn str_field<'a>(entry: &'a Value, key: &str) -> &'a str {
    entry
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("event-narrative entry must carry string `{key}`"))
}

/// Read a required integer field (a discrete Event Calculus time point) from an
/// open-JSON narrative entry. Panics on a malformed committed fixture.
fn i64_field(entry: &Value, key: &str) -> i64 {
    entry
        .get(key)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("event-narrative entry must carry integer `{key}`"))
}

/// Emit the ASP/Clingo Event Calculus program for `conflict_ec_allergy_persistence`
/// (SPEC 13.4 event layer; SPEC 15.1 unsafe-persistence Event Calculus conflict).
///
/// The single `EventNarrative` records an allergy detected at t=0 and a
/// beta-lactam administered at t=10. Discrete Event Calculus persistence/clipping
/// axioms over a `happens`/`initiates`/`terminates` narrative carry a fluent
/// forward: `holdsAt(F,T2)` derives from an initiating event at `T1 < T2` when no
/// terminating event `stoppedIn` the open interval clips it. `allergy_known` is
/// initiated at t=0 and the narrative's `terminates` set is empty, so
/// `holdsAt(allergy_known,10)` derives. The ground violation query fires because
/// `administer_drug` happens at t=10 while that contraindicating fluent still
/// holds — the persistence-conflict witness.
///
/// The `time/1` domain is data-driven from `happens`, so `T2` ranges only over
/// narrated instants. `happens`/`initiates` facts are data-driven from the
/// narrative and go through [`sorted_lines`]; the `#defined terminates/3`
/// declaration (silencing the empty-relation warning), the Event Calculus
/// axioms, the ground violation query, and the `#show` footer stay in fixed
/// order. The clingo run / holdsAt + violation recovery is task 0.9.
pub fn emit_event_calculus(bundle: &CompileBundle) -> CompiledTarget {
    let narrative = bundle
        .event_narratives
        .first()
        .expect("toy bundle must contain an event narrative");

    const HEADER: &str = "\
% CKC -> ASP/Clingo Event Calculus: conflict_ec_allergy_persistence
% Allergy detected at t=0, beta-lactam administered at t=10.
% Discrete Event Calculus persistence/clipping: holdsAt derives from an initiating
% event when no terminating event clips the fluent in the open interval.
% allergy_known is initiated at t=0 and never terminated, so holdsAt(allergy_known,10)
% derives; the violation fires as administer_drug at t=10 meets the persisted allergy.
% Emit-only: the clingo run is task 0.9.
";

    // Fixed-order Event Calculus axioms. `#defined terminates/3` declares the
    // possibly-empty narrative relation so clingo stays quiet when no terminating
    // event exists. The violation query is ground at the toy conflict instant
    // (administer_drug at t=10 vs holdsAt(allergy_known,10)).
    const AXIOMS: &str = "\
#defined terminates/3.
time(T) :- happens(_,T).
holdsAt(F,T2) :- happens(E,T1), initiates(E,F,T1), time(T2), T1 < T2, not stoppedIn(T1,F,T2).
stoppedIn(T1,F,T2) :- happens(E,T), terminates(E,F,T), time(T1), time(T2), T1 < T, T < T2.
violation(conflict_ec_allergy_persistence) :- happens(administer_drug,10), holdsAt(allergy_known,10).
";

    const FOOTER: &str = "\
#show holdsAt/2.
#show violation/1.
";

    // happens/2 + initiates/3 facts, data-driven from the narrative. Each event
    // maps to its happens atom and each fluent to its initiates atom — the ground
    // fact that introduces the entity — for the compilation map.
    let mut facts = Vec::new();
    let mut event_atom: BTreeMap<String, String> = BTreeMap::new();
    for h in &narrative.happens {
        let event = str_field(h, "event");
        let time = i64_field(h, "time");
        let atom = format!("happens({event},{time})");
        facts.push(format!("{atom}."));
        event_atom.insert(event.to_string(), atom);
    }
    let mut fluent_atom: BTreeMap<String, String> = BTreeMap::new();
    for i in &narrative.initiates {
        let event = str_field(i, "event");
        let fluent = str_field(i, "fluent");
        let time = i64_field(i, "time");
        let atom = format!("initiates({event},{fluent},{time})");
        facts.push(format!("{atom}."));
        fluent_atom.insert(fluent.to_string(), atom);
    }
    let facts = sorted_lines(facts);

    let artifact_text = format!("{HEADER}{facts}{AXIOMS}{FOOTER}");

    // Vocabulary grounding: events (event_types order) then fluents (fluent_types
    // order), each to its introducing atom and the narrative's shared spans.
    let mut symbol_mappings = Vec::new();
    for event in &narrative.event_types {
        let target_symbol = event_atom
            .get(event.as_str())
            .unwrap_or_else(|| panic!("event {event} must have a happens fact"))
            .clone();
        symbol_mappings.push(SymbolMapping {
            ckc_node_id: event.clone(),
            target_symbol,
            source_span_ids: narrative.source_span_ids.clone(),
        });
    }
    for fluent in &narrative.fluent_types {
        let target_symbol = fluent_atom
            .get(fluent.as_str())
            .unwrap_or_else(|| panic!("fluent {fluent} must have an initiates fact"))
            .clone();
        symbol_mappings.push(SymbolMapping {
            ckc_node_id: fluent.clone(),
            target_symbol,
            source_span_ids: narrative.source_span_ids.clone(),
        });
    }
    let compilation_map = CompilationMap(symbol_mappings);

    let source_artifact_hashes = vec![content_hash(narrative)];

    build_target(
        TargetLanguage::Asp,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}
