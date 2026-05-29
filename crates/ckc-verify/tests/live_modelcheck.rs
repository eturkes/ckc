//! Gate for task 0.9.7: live TLA+ SANY + Alloy re-derivation of the model-check
//! verdicts.
//!
//! Runs the installed `java` over the bundled `tla2tools.jar` (SANY) and
//! `alloy.jar` (the Alloy Analyzer CLI) to re-derive the two recorded C4 verdicts.
//! SANY over the committed `Conflict.tla` re-derives `semantic_check_passed`: a
//! clean exit-0 run whose stdout carries `Semantic processing of module Conflict`
//! with no `error`/`Abort`. Alloy `exec` over the committed `Priority.als` from a
//! throwaway scratch CWD re-derives `no_counterexample`: the `check NoPriorityCycle`
//! result line ends in `UNSAT`, meaning no counterexample to acyclicity exists.
//! Each tool is guarded by `solver_available("java")` plus the jar's `Path::exists`
//! — there is no standalone SANY/Alloy binary — so an environment missing java or a
//! jar `eprintln!`s and skips just that test, staying green. Each verdict is a
//! structural signal (a progress line / a check result token), not a parsed SMT
//! token, so the live re-derivation is the structural assertion and the oracle
//! entry's `(status, certificate_class)` is asserted directly (mirroring the
//! souffle/lean gate of 0.9.6).

use std::path::PathBuf;

use ckc_verify::runner::{alloy_jar, run_alloy, run_tla_sany, solver_available, tla_tools_jar};
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
fn tla_sany_verdict_matches_recorded() {
    if !solver_available("java") || !tla_tools_jar().exists() {
        eprintln!("skipping live_modelcheck::tla_sany: java or tla2tools.jar not found");
        return;
    }

    let root = workspace_root();
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();
    let recorded = outcomes
        .iter()
        .find(|o| o.solver == SolverId::TlaSany)
        .expect("oracle records one tla_sany outcome");
    assert_eq!(recorded.status, VerdictStatus::SemanticCheckPassed);
    assert_eq!(recorded.certificate_class, CertificateClass::C4Executable);

    let abs = root.join(&recorded.artifact_path);
    let run = run_tla_sany(&abs).expect("SANY run completes");
    assert_eq!(
        run.exit_code, 0,
        "SANY exits 0 on {}: stderr={}",
        recorded.artifact_path, run.stderr
    );

    // SANY emits its `Semantic processing of module <name>` progress line only after
    // a clean semantic pass; with no `error`/`Abort`, this re-derives the recorded
    // semantic_check_passed verdict.
    assert!(
        run.stdout
            .contains("Semantic processing of module Conflict"),
        "SANY semantically processes module Conflict for {} (stdout={})",
        recorded.artifact_path,
        run.stdout
    );
    assert!(
        !run.stdout.contains("error") && !run.stdout.contains("Abort"),
        "SANY reports no error/Abort for {} (stdout={})",
        recorded.artifact_path,
        run.stdout
    );
}

#[test]
fn alloy_verdict_matches_recorded() {
    if !solver_available("java") || !alloy_jar().exists() {
        eprintln!("skipping live_modelcheck::alloy: java or alloy.jar not found");
        return;
    }

    let root = workspace_root();
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();
    let recorded = outcomes
        .iter()
        .find(|o| o.solver == SolverId::Alloy)
        .expect("oracle records one alloy outcome");
    assert_eq!(recorded.status, VerdictStatus::NoCounterexample);
    assert_eq!(recorded.certificate_class, CertificateClass::C4Executable);

    // alloy `exec` writes a <Module>/ dir relative to CWD; run it from a throwaway
    // tempdir so the repo stays untouched.
    let scratch = tempfile::tempdir().expect("create scratch CWD");
    let abs = root.join(&recorded.artifact_path);
    let run = run_alloy(&abs, scratch.path()).expect("alloy run completes");

    // Alloy `exec` prints one result line per command to *stderr* (stdout stays
    // empty), so search stderr first, then stdout as a fallback. The NoPriorityCycle
    // check's line ends in UNSAT, meaning no counterexample to acyclicity exists ->
    // the recorded no_counterexample verdict over the acyclic toy priority graph.
    let check_line = run
        .stderr
        .lines()
        .chain(run.stdout.lines())
        .find(|l| l.contains("check NoPriorityCycle"))
        .unwrap_or_else(|| {
            panic!(
                "alloy reports a check NoPriorityCycle line for {} (stdout={}, stderr={})",
                recorded.artifact_path, run.stdout, run.stderr
            )
        });
    assert!(
        check_line.trim_end().ends_with("UNSAT"),
        "alloy NoPriorityCycle check is UNSAT (no counterexample) for {}: line={:?}",
        recorded.artifact_path,
        check_line
    );
}
