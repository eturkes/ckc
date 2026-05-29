//! Gate for task 0.9.5: live clingo re-derivation of the ASP verdicts.
//!
//! Runs the installed clingo over the two committed ASP targets (`defeasible`,
//! `event_calculus`) and asserts each live verdict equals its recorded oracle
//! entry: both are `satisfiable`, with every recorded salient model atom present
//! in the answer set. clingo encodes the result in its exit code (SAT = 10,
//! EXHAUST = 20, so a satisfiable program exits 30), never 0 — the test accepts
//! exit `{10, 30}` and reads the verdict from stdout. PATH-guarded — when clingo
//! is absent the test `eprintln!`s and returns, so a solver-less environment
//! stays green.

use std::path::PathBuf;

use ckc_verify::runner::{
    assert_matches_recorded, parse_clingo_status, run_clingo, solver_available,
};
use ckc_verify::{RecordedOutcomes, SolverId, VerifierOutcome, load_recorded_outcomes};

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
fn clingo_asp_verdicts_match_recorded() {
    if !solver_available("clingo") {
        eprintln!("skipping live_clingo: clingo not found on PATH");
        return;
    }

    let root = workspace_root();
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();

    // The two clingo ASP outcomes (defeasible + event_calculus). Filtering by
    // solver keeps the test independent of array position.
    let clingo_outcomes: Vec<&VerifierOutcome> = outcomes
        .iter()
        .filter(|o| o.solver == SolverId::Clingo)
        .collect();
    assert_eq!(
        clingo_outcomes.len(),
        2,
        "oracle must record 2 clingo ASP outcomes"
    );

    for recorded in clingo_outcomes {
        let abs = root.join(&recorded.artifact_path);
        let run = run_clingo(&abs).expect("clingo run completes");

        // clingo exits 30 on a satisfiable program (the 10|20 clasp bits), never
        // 0; accept {10, 30} and read the verdict from stdout.
        assert!(
            matches!(run.exit_code, 10 | 30),
            "clingo exit for {} is a SAT code (10|30), got {}",
            recorded.artifact_path,
            run.exit_code
        );
        let status = parse_clingo_status(&run.stdout)
            .unwrap_or_else(|| panic!("clingo reports SATISFIABLE for {}", recorded.artifact_path));

        // Every recorded salient atom appears in the answer set, re-deriving the
        // oracle's model atoms from the live run (order-independent containment —
        // clingo does not fix answer-set atom order across runs).
        for atom in &recorded.salient_atoms {
            assert!(
                run.stdout.contains(atom),
                "clingo model for {} contains {}",
                recorded.artifact_path,
                atom
            );
        }

        // Solver + status agree with the oracle (shared assertion path with
        // live_z3 / live_cvc5).
        let live = VerifierOutcome {
            status,
            ..recorded.clone()
        };
        assert_matches_recorded(&live, recorded);
    }
}
