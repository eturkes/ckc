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

/// Decode z3's leading verdict token into a [`VerdictStatus`]: `unsat` -> `Unsat`,
/// `sat` -> `Sat`. Returns `None` for `unknown` or any other first token.
pub fn parse_z3_status(stdout: &str) -> Option<VerdictStatus> {
    match stdout.split_whitespace().next()? {
        "unsat" => Some(VerdictStatus::Unsat),
        "sat" => Some(VerdictStatus::Sat),
        _ => None,
    }
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
