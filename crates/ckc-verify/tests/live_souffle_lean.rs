//! Gate for task 0.9.6: live souffle + lean re-derivation of the Datalog and
//! kernel verdicts.
//!
//! Runs the installed souffle over the committed `priority.dl` from a throwaway
//! scratch CWD and re-derives the recorded C4 `empty_relation` verdict: the acyclic
//! toy priority graph yields a zero-byte `cycle.csv`. Runs lean over the committed
//! `NormConflict.lean` and re-derives the recorded C7 `kernel_checked` verdict:
//! exit 0 with no `error:` diagnostic means the kernel accepted the file
//! sorry/admit-free. The two solvers are PATH-guarded independently — an
//! environment missing either one `eprintln!`s and skips just that test, so a
//! solver-less environment stays green. Each verdict is a solver side-effect (the
//! empty relation file) or exit-condition (a clean kernel check), not a parsed
//! stdout token, so the live re-derivation is the file/exit assertion and the
//! oracle entry's `(status, certificate_class)` is asserted directly.

use std::fs;
use std::path::PathBuf;

use ckc_verify::runner::{run_lean, run_souffle, solver_available};
use ckc_verify::{
    CertificateClass, RecordedOutcomes, SolverId, VerdictStatus, load_recorded_outcomes,
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
fn souffle_datalog_verdict_matches_recorded() {
    if !solver_available("souffle") {
        eprintln!("skipping live_souffle_lean::souffle: souffle not found on PATH");
        return;
    }

    let root = workspace_root();
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();
    let recorded = outcomes
        .iter()
        .find(|o| o.solver == SolverId::Souffle)
        .expect("oracle records one souffle outcome");
    assert_eq!(recorded.status, VerdictStatus::EmptyRelation);
    assert_eq!(recorded.certificate_class, CertificateClass::C4Executable);

    // souffle writes cycle.csv relative to CWD; run it from a throwaway tempdir so
    // the repo stays untouched.
    let scratch = tempfile::tempdir().expect("create scratch CWD");
    let abs = root.join(&recorded.artifact_path);
    let run = run_souffle(&abs, scratch.path()).expect("souffle run completes");
    assert_eq!(
        run.exit_code, 0,
        "souffle exits 0 on {}: stderr={}",
        recorded.artifact_path, run.stderr
    );

    // The acyclic toy priority graph yields an empty cycle relation -> a zero-byte
    // cycle.csv; the file size re-derives the recorded empty_relation verdict.
    let cycle_csv = scratch.path().join("cycle.csv");
    let len = fs::metadata(&cycle_csv)
        .expect("souffle emits cycle.csv")
        .len();
    assert_eq!(
        len, 0,
        "souffle cycle relation is empty (acyclic priority graph) for {}",
        recorded.artifact_path
    );
}

#[test]
fn lean_kernel_verdict_matches_recorded() {
    if !solver_available("lean") {
        eprintln!("skipping live_souffle_lean::lean: lean not found on PATH");
        return;
    }

    let root = workspace_root();
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();
    let recorded = outcomes
        .iter()
        .find(|o| o.solver == SolverId::Lean)
        .expect("oracle records one lean outcome");
    assert_eq!(recorded.status, VerdictStatus::KernelChecked);
    assert_eq!(recorded.certificate_class, CertificateClass::C7Kernel);

    let abs = root.join(&recorded.artifact_path);
    let run = run_lean(&abs).expect("lean run completes");
    assert_eq!(
        run.exit_code, 0,
        "lean kernel-checks {}: stderr={}",
        recorded.artifact_path, run.stderr
    );

    // No error: diagnostic in either stream -> the kernel accepted the file with no
    // sorry/admit; this re-derives the recorded C7 kernel_checked verdict.
    assert!(
        !run.stdout.contains("error:") && !run.stderr.contains("error:"),
        "lean reports no error: for {} (stdout={}, stderr={})",
        recorded.artifact_path,
        run.stdout,
        run.stderr
    );
}
