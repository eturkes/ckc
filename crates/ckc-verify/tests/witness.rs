//! ExecutionWitness-builder gate for task 0.9.9.
//!
//! `witnesses(&CompileBundle::load_toy())` yields the 10 Phase-0 witnesses — the 9
//! `compile_all` targets paired with their recorded outcomes, plus the standalone
//! cvc5 witness over the norm-conflict target. This gate enforces the Phase-0
//! acceptance that every verifier output maps to source spans, the defeasible and
//! Event Calculus witnesses carry their expected reasoning, and the builder is
//! deterministic across repeated calls.

use ckc_compile::CompileBundle;
use ckc_verify::{ExecutionWitness, content_hash, witnesses};

fn toy_witnesses() -> Vec<ExecutionWitness> {
    witnesses(&CompileBundle::load_toy())
}

/// The witness checked by the certificate with id `cert_id` — a stable key
/// independent of position in the result vector.
fn by_cert<'a>(ws: &'a [ExecutionWitness], cert_id: &str) -> &'a ExecutionWitness {
    ws.iter()
        .find(|w| w.certificate_ids.iter().any(|c| c.as_str() == cert_id))
        .unwrap_or_else(|| panic!("a witness certified by {cert_id}"))
}

#[test]
fn every_witness_maps_to_source_spans() {
    let ws = toy_witnesses();
    assert_eq!(
        ws.len(),
        10,
        "10 witnesses: 9 compile targets + standalone cvc5"
    );
    for w in &ws {
        assert!(
            !w.source_span_ids.is_empty(),
            "witness {} maps to no source span",
            w.witness_id.as_str()
        );
    }
}

#[test]
fn defeasible_and_event_calculus_witnesses_carry_their_reasoning() {
    let ws = toy_witnesses();

    // The defeasible ASP witness defeats exactly the recommendation rule (read from
    // the model's `defeated(rule_sepsis_bl_recommend)` atom; the argumentation atom
    // `defeated_arg(arg_rule_sepsis_bl_recommend)` is excluded).
    let defeasible = by_cert(&ws, "cert_clingo_defeasible");
    assert_eq!(
        defeasible.defeated_rules.len(),
        1,
        "defeasible witness defeats exactly one rule"
    );
    assert_eq!(
        defeasible.defeated_rules[0].as_str(),
        "rule_sepsis_bl_recommend",
        "the recommendation rule is the defeated one"
    );

    // The Event Calculus witness records the allergy-persistence temporal violation.
    let ec = by_cert(&ws, "cert_clingo_event_calculus");
    assert!(
        ec.violated_constraints
            .iter()
            .any(|c| c == "conflict_ec_allergy_persistence"),
        "EC witness records the allergy-persistence violation"
    );
}

#[test]
fn witnesses_are_deterministic() {
    let a = toy_witnesses();
    let b = toy_witnesses();
    assert_eq!(a.len(), b.len());
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            content_hash(x),
            content_hash(y),
            "witness {i} content_hash differs across calls"
        );
    }
}
