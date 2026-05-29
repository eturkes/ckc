//! Live solver invocation harness (SPEC 13).
//!
//! Spawns the installed verifiers and parses their output back into the recorded
//! oracle's vocabulary ([`crate::verdict`]). This module exists so the PATH-guarded
//! `tests/live_*.rs` can re-derive each Phase-0 verdict from a real solver run and
//! confirm it equals the committed oracle entry. The oracle stays the source of
//! truth for accepted artifacts; these runs are the validation that the oracle
//! still matches the solvers. [`solver_available`] is the binary guard that lets a
//! solver-less environment skip the live tests while the rest of the suite passes.

use std::path::{Path, PathBuf};
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

/// Run souffle over one Datalog artifact: `souffle <artifact_abs>`, executed from
/// `cwd`. souffle materializes each `.output` relation as `<relation>.csv` relative
/// to its working directory, so the caller passes a scratch dir to keep the repo
/// clean (the Phase-0 `priority.dl` emits `cycle.csv`). An acyclic priority graph
/// yields an empty `cycle` relation — a zero-byte `cycle.csv` — which is the
/// `empty_relation` verdict the caller re-derives from the file size.
pub fn run_souffle(artifact_abs: &Path, cwd: &Path) -> anyhow::Result<RunResult> {
    let mut cmd = Command::new("souffle");
    cmd.arg(artifact_abs).current_dir(cwd);
    capture(cmd)
}

/// Run lean over one source file: `lean <artifact_abs>` (lean resolves on the base
/// PATH via elan). The single import-free Phase-0 file kernel-checks standalone, so
/// no lakefile/lake project is needed. lean exits 0 and prints nothing when the
/// kernel accepts the file, and emits `error:` diagnostics (with a nonzero exit) on
/// any failure; a clean check — exit 0 with no `error:` and no `sorry`/`admit` — is
/// the `kernel_checked` (C7) verdict the caller re-derives.
pub fn run_lean(artifact_abs: &Path) -> anyhow::Result<RunResult> {
    let mut cmd = Command::new("lean");
    cmd.arg(artifact_abs);
    capture(cmd)
}

/// Resolve the bundled TLA+ tools jar at `$HOME/.local/share/tla/tla2tools.jar`
/// (it carries both SANY and TLC). There is no standalone SANY binary, so the
/// model-check live test guards on this path's `Path::exists` alongside
/// `solver_available("java")` rather than a `solver_available` shim.
pub fn tla_tools_jar() -> PathBuf {
    local_share().join("tla/tla2tools.jar")
}

/// Resolve the Alloy distribution jar at `$HOME/.local/share/alloy/alloy.jar`.
/// Like [`tla_tools_jar`], the model-check live test guards on its `Path::exists`
/// plus `solver_available("java")`.
pub fn alloy_jar() -> PathBuf {
    local_share().join("alloy/alloy.jar")
}

/// `$HOME/.local/share`, resolved through `HOME`. A missing `HOME` yields a
/// relative path that fails `Path::exists`, so the jar guards skip the model-check
/// live tests cleanly rather than erroring.
fn local_share() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
}

/// Run the TLA+ SANY syntax+semantic checker over one `.tla` module:
/// `java -cp <tla2tools.jar> tla2sany.SANY <artifact_abs>`. SANY prints its
/// `Parsing file …` / `Semantic processing of module …` progress to stdout and
/// exits 0 on a clean module, emitting explicit error lines (and a nonzero exit) on
/// any syntax/semantic fault. A clean exit-0 run carrying the
/// `Semantic processing of module <name>` line with no `error`/`Abort` is the
/// `semantic_check_passed` (C4) verdict the caller re-derives. The jar resolves via
/// [`tla_tools_jar`]; the caller guards on java + jar existence.
pub fn run_tla_sany(artifact_abs: &Path) -> anyhow::Result<RunResult> {
    let mut cmd = Command::new("java");
    cmd.arg("-cp")
        .arg(tla_tools_jar())
        .arg("tla2sany.SANY")
        .arg(artifact_abs);
    capture(cmd)
}

/// Run the Alloy Analyzer's headless CLI over one `.als` module:
/// `java -jar <alloy.jar> exec <artifact_abs>`, executed from `cwd`. `exec`
/// materializes a `<Module>/receipt.json` directory relative to its working dir, so
/// the caller passes a scratch CWD to keep the repo clean (the Phase-0
/// `Priority.als` writes `Priority/`). Alloy prints one result line per command;
/// for a `check`, a trailing `UNSAT` means no counterexample exists — the acyclic
/// priority graph — which is the `no_counterexample` (C4) verdict the caller
/// re-derives. The jar resolves via [`alloy_jar`]; the caller guards on java + jar
/// existence.
pub fn run_alloy(artifact_abs: &Path, cwd: &Path) -> anyhow::Result<RunResult> {
    let mut cmd = Command::new("java");
    cmd.arg("-jar")
        .arg(alloy_jar())
        .arg("exec")
        .arg(artifact_abs)
        .current_dir(cwd);
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
