//! Gate for task 0.9.4: live cvc5 re-derivation of the proof-carrying SMT verdict.
//!
//! Runs the installed cvc5 with proof production over the committed `norm_conflict`
//! SMT-LIB target and confirms the recorded C6 proof object: the leading verdict
//! token is `unsat`, the dumped proof block carries `:rule` steps, cvc5 exits 0,
//! and the recorded cvc5 outcome is `proof_present` at class `C6-ProofObject`.
//! Structural presence only — the committed `cvc5_norm_conflict.proof` is recorded
//! evidence, not a byte-equality target (cvc5 proof text is not guaranteed
//! byte-stable). PATH-guarded: when cvc5 is absent the test `eprintln!`s and
//! returns, so a solver-less environment stays green.

use std::path::PathBuf;

use ckc_verify::runner::{
    assert_matches_recorded, cvc5_proof_present, parse_smt_status, run_cvc5_proof, solver_available,
};
use ckc_verify::{
    CertificateClass, RecordedOutcomes, SolverId, VerdictStatus, VerifierOutcome,
    load_recorded_outcomes,
};

/// Workspace root: two parents above this crate's manifest dir (mirrors
/// `tests/live_z3.rs`). Recorded `artifact_path`s are repo-relative.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

#[test]
fn cvc5_proof_verdict_matches_recorded() {
    if !solver_available("cvc5") {
        eprintln!("skipping live_cvc5: cvc5 not found on PATH");
        return;
    }

    let root = workspace_root();
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();

    // The single cvc5 outcome: a proof object over the norm_conflict target (z3
    // checks the same file as a plain C4 executable, so filter by solver, not
    // artifact path).
    let recorded = outcomes
        .iter()
        .find(|o| o.solver == SolverId::Cvc5)
        .expect("oracle records one cvc5 outcome");
    assert!(
        recorded.proof_present,
        "recorded cvc5 outcome carries a proof"
    );
    assert_eq!(recorded.certificate_class, CertificateClass::C6ProofObject);

    // Live cvc5 reproduces the proof-carrying unsat verdict.
    let abs = root.join(&recorded.artifact_path);
    let run = run_cvc5_proof(&abs).expect("cvc5 run completes");

    assert_eq!(run.exit_code, 0, "cvc5 exits 0 on the unsat proof");
    let status = parse_smt_status(&run.stdout)
        .unwrap_or_else(|| panic!("cvc5 reports sat/unsat for {}", recorded.artifact_path));
    assert_eq!(
        status,
        VerdictStatus::Unsat,
        "cvc5 leading token is unsat for {}",
        recorded.artifact_path
    );
    assert!(
        cvc5_proof_present(&run.stdout),
        "cvc5 dumps an Alethe proof block (:rule steps)"
    );

    // Solver + status agree with the oracle (shared assertion path with live_z3).
    let live = VerifierOutcome {
        status,
        ..recorded.clone()
    };
    assert_matches_recorded(&live, recorded);
}
