//! Gate for task 0.9.3: live z3 re-derivation of the SMT verdicts.
//!
//! Runs the installed z3 over the three committed SMT-LIB targets
//! (`norm_conflict`, `decision_table`, `repair_maxsmt`) and asserts each live
//! verdict equals its recorded oracle entry: norm -> unsat, decision-table -> sat,
//! repair -> sat with objective 1. PATH-guarded — when z3 is absent the test
//! `eprintln!`s and returns, so a solver-less environment stays green.

use std::path::PathBuf;

use ckc_verify::runner::{
    assert_matches_recorded, parse_smt_status, parse_z3_objective, run_z3, solver_available,
};
use ckc_verify::{RecordedOutcomes, SolverId, VerifierOutcome, load_recorded_outcomes};

/// Workspace root: two parents above this crate's manifest dir (mirrors
/// `tests/recorded.rs`). Recorded `artifact_path`s are repo-relative.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

#[test]
fn z3_smt_verdicts_match_recorded() {
    if !solver_available("z3") {
        eprintln!("skipping live_z3: z3 not found on PATH");
        return;
    }

    let root = workspace_root();
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();

    // The three z3 SMT outcomes (cvc5 over the same norm_conflict is a separate
    // solver). Filtering by solver keeps the test independent of array position.
    let z3_outcomes: Vec<&VerifierOutcome> = outcomes
        .iter()
        .filter(|o| o.solver == SolverId::Z3)
        .collect();
    assert_eq!(z3_outcomes.len(), 3, "oracle must record 3 z3 SMT outcomes");

    for recorded in z3_outcomes {
        let abs = root.join(&recorded.artifact_path);
        let run = run_z3(&abs).expect("z3 run completes");
        let status = parse_smt_status(&run.stdout)
            .unwrap_or_else(|| panic!("z3 reports sat/unsat for {}", recorded.artifact_path));
        let objective = parse_z3_objective(&run.stdout);
        let live = VerifierOutcome {
            status,
            objective,
            ..recorded.clone()
        };
        assert_matches_recorded(&live, recorded);
    }
}
