//! Portfolio agreement and backend-disagreement gates (task 0.9.11).
//!
//! Over the real recorded oracle: exactly two `C5-Portfolio` agreement records
//! (norm-conflict and priority-acyclicity) and zero diagnostics. Over a
//! synthetic divergent pair on one claim: exactly one `backend_disagreement`
//! diagnostic and no agreement.

use ckc_core::enums::CertificateClass;

use ckc_verify::{SolverId, VerdictStatus, load_recorded_outcomes, portfolio_check};

#[test]
fn real_outcomes_yield_two_agreements_and_no_disagreement() {
    let outcomes = load_recorded_outcomes().0;
    let (agreements, diagnostics) = portfolio_check(&outcomes);

    assert!(
        diagnostics.is_empty(),
        "the real recorded oracle has no backend disagreement, found {diagnostics:?}"
    );
    assert_eq!(
        agreements.len(),
        2,
        "norm-conflict and priority-acyclicity are the two Phase-0 portfolios"
    );
    assert!(
        agreements
            .iter()
            .all(|record| record.certificate_class == CertificateClass::C5Portfolio)
    );

    let claim_ids: Vec<&str> = agreements
        .iter()
        .map(|record| record.claim_id.as_str())
        .collect();
    assert!(claim_ids.contains(&"conflict_norm_bl_contradiction"));
    assert!(claim_ids.contains(&"priority_acyclicity"));

    // The norm-conflict portfolio cross-checks three backends; all agree the
    // claim is established, and the backend set is deterministically ordered.
    let norm = agreements
        .iter()
        .find(|record| record.claim_id == "conflict_norm_bl_contradiction")
        .expect("norm-conflict agreement present");
    assert_eq!(norm.agreed_result, "established");
    assert_eq!(
        norm.backends,
        vec![SolverId::Cvc5, SolverId::Lean, SolverId::Z3]
    );
}

#[test]
fn divergent_backends_yield_one_disagreement() {
    let real_z3_unsat = load_recorded_outcomes()
        .0
        .into_iter()
        .find(|outcome| {
            outcome.solver == SolverId::Z3
                && outcome.artifact_path == "logic/smt/norm_conflict.smt2"
        })
        .expect("recorded oracle has the z3 norm-conflict unsat outcome");

    // Fabricate a second backend on the same claim that contradicts it.
    let mut fabricated = real_z3_unsat.clone();
    fabricated.status = VerdictStatus::Sat;

    let (agreements, diagnostics) = portfolio_check(&[real_z3_unsat, fabricated]);

    assert!(
        agreements.is_empty(),
        "a contradicted claim earns no agreement record"
    );
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "backend_disagreement");
}
