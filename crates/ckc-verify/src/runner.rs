//! Live solver invocation harness (SPEC 13).
//!
//! Spawns the installed verifiers and parses their output back into the recorded
//! oracle's vocabulary ([`crate::verdict`]). This module exists so the PATH-guarded
//! `tests/live_*.rs` can re-derive each Phase-0 verdict from a real solver run and
//! confirm it equals the committed oracle entry. The oracle stays the source of
//! truth for accepted artifacts; these runs are the validation that the oracle
//! still matches the solvers. [`solver_available`] is the binary guard that lets a
//! solver-less environment skip the live tests while the rest of the suite passes.

use std::path::Path;
use std::process::Command;

use crate::verdict::{VerdictStatus, VerifierOutcome};

/// Captured result of one solver invocation: decoded stdout/stderr and the
/// process exit code (`-1` when terminated by signal).
#[derive(Clone, Debug)]
pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Return whether `bin` can be spawned at all (the repo's binary guard). Spawns
/// `bin --version` and reports success when the process launches, regardless of
/// its exit status — only a spawn error (binary absent) yields `false`. Output is
/// captured, so nothing reaches the terminal. The live tests call this first and
/// skip cleanly when it returns `false`, keeping solver-less CI green.
pub fn solver_available(bin: &str) -> bool {
    Command::new(bin).arg("--version").output().is_ok()
}

/// Run a prepared command to completion and capture its output as a [`RunResult`].
fn capture(mut cmd: Command) -> anyhow::Result<RunResult> {
    let out = cmd.output()?;
    Ok(RunResult {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        exit_code: out.status.code().unwrap_or(-1),
    })
}

/// Run z3 over one SMT-LIB artifact: `z3 <artifact_abs>`. The artifact carries its
/// own `(check-sat)`/`(get-objectives)`, so no extra flags are needed.
pub fn run_z3(artifact_abs: &Path) -> anyhow::Result<RunResult> {
    let mut cmd = Command::new("z3");
    cmd.arg(artifact_abs);
    capture(cmd)
}

/// Run cvc5 over one SMT-LIB artifact with proof production:
/// `cvc5 --produce-proofs --dump-proofs <artifact_abs>`. cvc5 prints the standard
/// SMT-LIB verdict token (`unsat`) followed by an Alethe-style proof whose steps
/// carry `:rule` annotations; [`parse_smt_status`] decodes the token and
/// [`cvc5_proof_present`] confirms the proof block (the C6 proof-object signal).
pub fn run_cvc5_proof(artifact_abs: &Path) -> anyhow::Result<RunResult> {
    let mut cmd = Command::new("cvc5");
    cmd.args(["--produce-proofs", "--dump-proofs"])
        .arg(artifact_abs);
    capture(cmd)
}

/// Run clingo over one ASP artifact: `clingo <artifact_abs>`. clingo prints its
/// answer set(s) then a standalone `SATISFIABLE`/`UNSATISFIABLE` line. It encodes
/// the result in its exit code as the clasp bitmask (SAT = 10, EXHAUST = 20, so a
/// satisfiable+enumerated program exits 30), never 0 — so callers read the verdict
/// from stdout via [`parse_clingo_status`] and treat exit `{10, 30}` as success.
pub fn run_clingo(artifact_abs: &Path) -> anyhow::Result<RunResult> {
    let mut cmd = Command::new("clingo");
    cmd.arg(artifact_abs);
    capture(cmd)
}

/// Decode an SMT-LIB solver's leading `(check-sat)` response token into a
/// [`VerdictStatus`]: `unsat` -> `Unsat`, `sat` -> `Sat`. Returns `None` for
/// `unknown` or any other first token. Shared by the z3 and cvc5 runners, which
/// both print the standard SMT-LIB verdict token first.
pub fn parse_smt_status(stdout: &str) -> Option<VerdictStatus> {
    match stdout.split_whitespace().next()? {
        "unsat" => Some(VerdictStatus::Unsat),
        "sat" => Some(VerdictStatus::Sat),
        _ => None,
    }
}

/// Decode clingo's solve-result line into a [`VerdictStatus`]. clingo prints a
/// standalone `SATISFIABLE`/`UNSATISFIABLE` token after its answer sets; this
/// returns `Some(Satisfiable)` on an exact `SATISFIABLE` token (splitting on
/// whitespace guards against the `UNSATISFIABLE` superstring) and `None`
/// otherwise. Both Phase-0 ASP targets are satisfiable.
pub fn parse_clingo_status(stdout: &str) -> Option<VerdictStatus> {
    stdout
        .split_whitespace()
        .any(|tok| tok == "SATISFIABLE")
        .then_some(VerdictStatus::Satisfiable)
}

/// Extract the optimization objective from a z3 `(get-objectives)` block. The
/// MaxSMT repair target prints `(objectives\n ( 1)\n)`; this returns the first
/// integer token inside that block (`Some(1)` there, `None` when no block exists,
/// as for the plain sat/unsat targets).
pub fn parse_z3_objective(stdout: &str) -> Option<i64> {
    let block = stdout.split_once("(objectives")?.1;
    block
        .replace(['(', ')'], " ")
        .split_whitespace()
        .find_map(|tok| tok.parse::<i64>().ok())
}

/// Report whether cvc5's dumped proof carries explicit derivation steps. cvc5
/// prints each Alethe proof step with a `:rule` annotation, so the substring's
/// presence is the structural proof-object signal recorded as `proof_present`
/// (certificate class C6).
pub fn cvc5_proof_present(stdout: &str) -> bool {
    stdout.contains(":rule")
}

/// Assert that a live-derived outcome matches its recorded oracle entry: the
/// solver and verdict status must agree, and the optimization objective must agree
/// whenever the recorded entry carries one (the MaxSMT repair target). Panics with
/// a located message on any divergence — that panic is the live test's failure.
pub fn assert_matches_recorded(live: &VerifierOutcome, recorded: &VerifierOutcome) {
    assert_eq!(
        live.solver, recorded.solver,
        "solver mismatch: live {:?} vs recorded {:?}",
        live.solver, recorded.solver
    );
    assert_eq!(
        live.status, recorded.status,
        "status mismatch for {}: live {:?} vs recorded {:?}",
        recorded.artifact_path, live.status, recorded.status
    );
    if recorded.objective.is_some() {
        assert_eq!(
            live.objective, recorded.objective,
            "objective mismatch for {}: live {:?} vs recorded {:?}",
            recorded.artifact_path, live.objective, recorded.objective
        );
    }
}
