//! SPEC §8.3 verify processing_stage, adapter half: invoke Z3 per query as a
//! subprocess under a per-query wall-clock budget and capture the process
//! truth — raw stdout/stderr, exit fate, and the leading verdict token.
//!
//! §6 solvers row: Z3 by binary invocation, identity and version recorded
//! in manifests and verifier results — [`Z3Adapter`] construction probes
//! `--version` live and parses [`SolverIdentity`] from the reply, so the
//! code carries no version literal anywhere. §7.4 failure codes: a budget
//! expiry maps to `solver_timeout`, a spawn failure or nonzero exit to
//! `solver_execution_failure` ([`RunOutcome::failure_code`]) — diagnostic
//! codes, kept distinct from the §6 categories smt-verify.b derives.
//! Verdict/core/model s-expression parsing and [`VerifierResult`] assembly
//! live in `verdict`; this module never interprets output beyond the
//! leading token.
//!
//! [`VerifierResult`]: crate::VerifierResult

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ckc_core::{DiagnosticCode, Id, SolverIdentity};

use crate::SolverVerdict;

/// Wall-clock budget for the construction-time `--version` probe. A version
/// reply is instant from a healthy solver binary; a probe that outlives
/// this is a broken installation, not a hard query.
const VERSION_PROBE_BUDGET: Duration = Duration::from_secs(10);

/// Poll interval for the budget loop: short enough that kill-on-expiry
/// lands within a few milliseconds of the deadline.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Post-fate bound on draining the I/O threads. A dead process with no
/// orphans reaches EOF in microseconds; anything still unfinished at the
/// bound is a pipe held open by an orphaned grandchild, and the runner
/// snapshots partial output and detaches rather than wait it out.
const DRAIN_GRACE: Duration = Duration::from_secs(1);

/// SPEC §6/§8.3 Z3 adapter: one solver binary plus the [`SolverIdentity`]
/// parsed live from its `--version` reply at construction. Queries run via
/// [`Z3Adapter::invoke`], one subprocess per query (`z3 -in`, query text on
/// stdin), each under its own wall-clock budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Z3Adapter {
    program: PathBuf,
    identity: SolverIdentity,
}

impl Z3Adapter {
    /// Adapter over the `z3` binary on PATH (§6: Z3 required first).
    pub fn new() -> Result<Self, AdapterError> {
        Self::with_program("z3")
    }

    /// Adapter over an explicit binary path. The program must answer
    /// `--version` with a line carrying a token after the word `version`
    /// (Z3's shape: `Z3 version 4.13.3 - 64 bit`); construction fails
    /// otherwise, so a held adapter always carries a live-parsed identity.
    pub fn with_program(program: impl Into<PathBuf>) -> Result<Self, AdapterError> {
        let program = program.into();
        let probe = run_process(&program, &["--version"], b"", VERSION_PROBE_BUDGET);
        match probe.outcome {
            RunOutcome::Completed { .. } => {}
            RunOutcome::Timeout => {
                return Err(AdapterError::Probe {
                    detail: "version probe hit its wall-clock budget".to_owned(),
                });
            }
            RunOutcome::ExitFailure { code } => {
                return Err(AdapterError::Probe {
                    detail: format!(
                        "version probe exited with code {:?}: {}",
                        code,
                        probe.stderr.trim()
                    ),
                });
            }
            RunOutcome::SpawnFailure { error } => {
                return Err(AdapterError::Probe {
                    detail: format!("version probe failed to spawn: {error}"),
                });
            }
        }
        let line = probe.stdout.lines().next().unwrap_or("").trim().to_owned();
        let version = parse_version_line(&line).ok_or(AdapterError::VersionUnparsed { line })?;
        Ok(Z3Adapter {
            program,
            identity: SolverIdentity {
                solver_id: Id::new("z3").expect("static solver id"),
                version,
            },
        })
    }

    /// The §5 solver identity this adapter stamps into manifests and
    /// verifier results — the `--version` truth, never a code literal.
    pub fn identity(&self) -> &SolverIdentity {
        &self.identity
    }

    /// Run one query's SMT-LIB 2 text under `budget` wall-clock time
    /// (§8.3: invoke Z3 per query). The subprocess is killed on expiry;
    /// every fate — verdict, timeout, exit failure, spawn failure — comes
    /// back as data in the [`SolverRun`], with whatever stdout/stderr
    /// drained before the end.
    pub fn invoke(&self, smt2: &str, budget: Duration) -> SolverRun {
        run_process(&self.program, &["-in"], smt2.as_bytes(), budget)
    }
}

/// One per-query solver subprocess, raw (§6: solver diagnostics preserved
/// distinctly). `stdout`/`stderr` hold everything drained, lossily decoded,
/// partial when the budget kill landed first; result s-expressions stay
/// unparsed for smt-verify.b.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolverRun {
    pub outcome: RunOutcome,
    pub stdout: String,
    pub stderr: String,
}

/// Process fate of one solver invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    /// Exit status zero. `verdict` is the leading stdout token when it is
    /// one of `sat`/`unsat`/`unknown` — Z3 answers `unsupported` lines or
    /// nothing at all with status zero too, leaving `None`.
    Completed { verdict: Option<SolverVerdict> },
    /// The wall-clock budget expired and the process was killed
    /// (§6 verdict vocabulary: the budget-minted `timeout`, never a parsed
    /// token).
    Timeout,
    /// Nonzero exit status; `code` is `None` when a signal ended the
    /// process (or its status could not be collected).
    ExitFailure { code: Option<i32> },
    /// The process never started.
    SpawnFailure { error: String },
}

impl RunOutcome {
    /// SPEC §7.4 diagnostic code for a failed run, `None` for a completed
    /// one: budget expiry → `solver_timeout`, spawn failure or nonzero
    /// exit → `solver_execution_failure`. smt-verify.b folds these into
    /// DiagnosticRecords; they stay distinct from the §6 categories.
    pub fn failure_code(&self) -> Option<DiagnosticCode> {
        match self {
            RunOutcome::Completed { .. } => None,
            RunOutcome::Timeout => Some(DiagnosticCode::SolverTimeout),
            RunOutcome::ExitFailure { .. } | RunOutcome::SpawnFailure { .. } => {
                Some(DiagnosticCode::SolverExecutionFailure)
            }
        }
    }
}

/// Construction failed: the `--version` probe broke, or its reply carried
/// no version token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// The probe process timed out, exited nonzero, or failed to spawn.
    Probe { detail: String },
    /// The probe's first stdout line names no version token.
    VersionUnparsed { line: String },
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::Probe { detail } => write!(f, "solver version probe failed: {detail}"),
            AdapterError::VersionUnparsed { line } => {
                write!(f, "no version token in solver reply {line:?}")
            }
        }
    }
}

impl std::error::Error for AdapterError {}

/// Leading verdict token of solver stdout: the first line, trimmed, when it
/// is exactly `sat`/`unsat`/`unknown`. `timeout` is budget-minted by the
/// runner, never parsed, so the text `timeout` (like any other reply) gives
/// `None`.
pub(crate) fn leading_verdict(stdout: &str) -> Option<SolverVerdict> {
    match stdout.lines().next()?.trim() {
        "sat" => Some(SolverVerdict::Sat),
        "unsat" => Some(SolverVerdict::Unsat),
        "unknown" => Some(SolverVerdict::Unknown),
        _ => None,
    }
}

/// Version token from a `--version` reply line: the whitespace token after
/// the first case-insensitive `version`.
fn parse_version_line(line: &str) -> Option<String> {
    let mut tokens = line.split_whitespace();
    while let Some(token) = tokens.next() {
        if token.eq_ignore_ascii_case("version") {
            return tokens.next().map(str::to_owned);
        }
    }
    None
}

/// Reader thread draining one output pipe into a shared buffer until EOF
/// (write ends close at exit or kill), so neither pipe can fill and stall
/// the child. The buffer is shared so the runner can snapshot partial
/// output without joining: a killed process's orphans (a forking solver
/// wrapper) inherit the write ends and would hold a join hostage.
fn drain<R: Read + Send + 'static>(pipe: R) -> (JoinHandle<()>, Arc<Mutex<Vec<u8>>>) {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&buf);
    let handle = thread::spawn(move || {
        let mut pipe = pipe;
        let mut chunk = [0u8; 4096];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => sink
                    .lock()
                    .expect("drain readers never panic holding the lock")
                    .extend_from_slice(&chunk[..n]),
            }
        }
    });
    (handle, buf)
}

/// Spawn `program args`, feed `stdin_bytes`, and wait at most `budget`
/// wall-clock time, polling every [`POLL_INTERVAL`] and killing on expiry.
/// Stdin is written from its own thread (a write error means the child
/// stopped reading — already dead or never reading; the exit fate carries
/// the story), stdout/stderr drain concurrently into shared buffers, and
/// every fate returns as a [`SolverRun`]. After the fate is known the I/O
/// threads get [`DRAIN_GRACE`] to reach EOF, then the buffers are
/// snapshotted and laggards detached — a clean exit drains in microseconds,
/// while pipes held open past the kill (an orphaned grandchild) cannot
/// stall the runner.
fn run_process(program: &Path, args: &[&str], stdin_bytes: &[u8], budget: Duration) -> SolverRun {
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return SolverRun {
                outcome: RunOutcome::SpawnFailure {
                    error: error.to_string(),
                },
                stdout: String::new(),
                stderr: String::new(),
            };
        }
    };

    let mut stdin = child.stdin.take().expect("stdin piped above");
    let payload = stdin_bytes.to_vec();
    let stdin_thread = thread::spawn(move || {
        let _ = stdin.write_all(&payload);
    });
    let (stdout_thread, stdout_buf) = drain(child.stdout.take().expect("stdout piped above"));
    let (stderr_thread, stderr_buf) = drain(child.stderr.take().expect("stderr piped above"));

    /// How the wait loop ended: a collected status, a budget kill, or a
    /// wait that itself failed (killed and folded into a codeless exit
    /// failure).
    enum Fate {
        Exited(ExitStatus),
        TimedOut,
        WaitFailed,
    }

    let deadline = Instant::now() + budget;
    let fate = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Fate::Exited(status),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break Fate::TimedOut;
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break Fate::WaitFailed;
            }
        }
    };

    let grace_deadline = Instant::now() + DRAIN_GRACE;
    let io_threads = [stdin_thread, stdout_thread, stderr_thread];
    while io_threads.iter().any(|t| !t.is_finished()) && Instant::now() < grace_deadline {
        thread::sleep(POLL_INTERVAL);
    }
    for thread in io_threads {
        if thread.is_finished() {
            let _ = thread.join();
        }
    }
    let snapshot = |buf: &Mutex<Vec<u8>>| {
        String::from_utf8_lossy(
            &buf.lock()
                .expect("drain readers never panic holding the lock"),
        )
        .into_owned()
    };
    let stdout = snapshot(&stdout_buf);
    let stderr = snapshot(&stderr_buf);

    let outcome = match fate {
        Fate::TimedOut => RunOutcome::Timeout,
        Fate::WaitFailed => RunOutcome::ExitFailure { code: None },
        Fate::Exited(status) if status.success() => RunOutcome::Completed {
            verdict: leading_verdict(&stdout),
        },
        Fate::Exited(status) => RunOutcome::ExitFailure {
            code: status.code(),
        },
    };
    SolverRun {
        outcome,
        stdout,
        stderr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    /// Write an executable stub shell script answering `--version` like Z3
    /// before running `body`, named uniquely per test for parallel runs.
    fn stub(name: &str, body: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("ckc-verify-stub-{}-{name}", std::process::id()));
        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo \"Z3 version 9.9.9 - stub\"\n  exit 0\nfi\n{body}\n"
        );
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    // Live z3: construction parses the identity from `z3 --version` — a
    // dotted numeric token, no literal anywhere in code or assertions.
    #[test]
    fn live_identity_from_version_probe() {
        let adapter = Z3Adapter::new().unwrap();
        let identity = adapter.identity();
        assert_eq!(identity.solver_id.as_str(), "z3");
        assert!(
            identity.version.starts_with(|c: char| c.is_ascii_digit()),
            "version {:?} lacks a leading digit",
            identity.version
        );
        assert!(
            identity.version.contains('.'),
            "version {:?} lacks a dot",
            identity.version
        );
    }

    // Live z3 over tiny inline queries: sat and unsat verdict tokens, exit
    // zero, empty stderr.
    #[test]
    fn live_sat_and_unsat_verdicts() {
        let adapter = Z3Adapter::new().unwrap();
        let budget = Duration::from_secs(30);

        let sat = adapter.invoke(
            "(set-option :print-success false)\n(set-logic QF_LRA)\n(declare-const |q.x| Real)\n(assert (> |q.x| 0))\n(check-sat)\n",
            budget,
        );
        assert_eq!(
            sat.outcome,
            RunOutcome::Completed {
                verdict: Some(SolverVerdict::Sat)
            }
        );
        assert_eq!(sat.stdout, "sat\n");
        assert_eq!(sat.stderr, "");
        assert_eq!(sat.outcome.failure_code(), None);

        let unsat = adapter.invoke(
            "(set-option :print-success false)\n(declare-const |q.p| Bool)\n(assert (and |q.p| (not |q.p|)))\n(check-sat)\n",
            budget,
        );
        assert_eq!(
            unsat.outcome,
            RunOutcome::Completed {
                verdict: Some(SolverVerdict::Unsat)
            }
        );
        assert_eq!(unsat.stdout, "unsat\n");
    }

    // Live z3 recovers from an unknown command with `unsupported` and exit
    // zero (observed 4.13.3 behavior): completed, but no verdict token.
    #[test]
    fn live_unsupported_command_completes_without_verdict() {
        let adapter = Z3Adapter::new().unwrap();
        let run = adapter.invoke("(bogus)\n(check-sat)\n", Duration::from_secs(30));
        assert_eq!(run.outcome, RunOutcome::Completed { verdict: None });
        assert!(
            run.stdout.starts_with("unsupported"),
            "stdout {:?}",
            run.stdout
        );
        assert_eq!(run.outcome.failure_code(), None);
    }

    // Live z3 exits nonzero on an undeclared constant (observed 4.13.3
    // behavior: `(error ...)` printed, exit 1) — the §7.4
    // solver_execution_failure path on the real binary, raw capture kept.
    #[test]
    fn live_error_exit_maps_to_execution_failure() {
        let adapter = Z3Adapter::new().unwrap();
        let run = adapter.invoke(
            "(assert (and p (not p)))\n(check-sat)\n",
            Duration::from_secs(30),
        );
        assert_eq!(run.outcome, RunOutcome::ExitFailure { code: Some(1) });
        assert!(run.stdout.contains("(error"), "stdout {:?}", run.stdout);
        assert_eq!(
            run.outcome.failure_code(),
            Some(DiagnosticCode::SolverExecutionFailure)
        );
    }

    // Canned-text verdict-token parse: the three solver tokens, model text
    // following sat, and the non-verdicts — `timeout` included, which only
    // the budget mints.
    #[test]
    fn leading_verdict_parses_canned_text() {
        assert_eq!(leading_verdict("sat\n"), Some(SolverVerdict::Sat));
        assert_eq!(
            leading_verdict("sat\n((define-fun |q.x| () Real 1.0))\n"),
            Some(SolverVerdict::Sat)
        );
        assert_eq!(
            leading_verdict("unsat\n(a.r1 a.r2)\n"),
            Some(SolverVerdict::Unsat)
        );
        assert_eq!(leading_verdict("unknown\n"), Some(SolverVerdict::Unknown));
        assert_eq!(leading_verdict(" sat \n"), Some(SolverVerdict::Sat));
        assert_eq!(leading_verdict("timeout\n"), None);
        assert_eq!(leading_verdict("unsupported\nsat\n"), None);
        assert_eq!(leading_verdict("(error \"line 1\")\nsat\n"), None);
        assert_eq!(leading_verdict("satx\n"), None);
        assert_eq!(leading_verdict(""), None);
    }

    // Canned-text version-line parse: the live Z3 shape, a lowercase
    // variant, and lines with no version token.
    #[test]
    fn version_line_parses_canned_text() {
        assert_eq!(
            parse_version_line("Z3 version 4.13.3 - 64 bit"),
            Some("4.13.3".to_owned())
        );
        assert_eq!(parse_version_line("z3 VERSION 1.2"), Some("1.2".to_owned()));
        assert_eq!(parse_version_line("Z3 version"), None);
        assert_eq!(parse_version_line("no token here"), None);
        assert_eq!(parse_version_line(""), None);
    }

    // Budget kill: a stub sleeper outliving its budget is killed within
    // moments of the deadline, mints `timeout`, and maps to the §7.4
    // solver_timeout code. Also pins the stub identity parse.
    #[test]
    fn budget_kills_stub_sleeper() {
        let path = stub("sleeper", "sleep 30");
        let adapter = Z3Adapter::with_program(&path).unwrap();
        assert_eq!(adapter.identity().version, "9.9.9");

        let start = Instant::now();
        let run = adapter.invoke("(check-sat)\n", Duration::from_millis(200));
        let elapsed = start.elapsed();

        assert_eq!(run.outcome, RunOutcome::Timeout);
        assert_eq!(
            run.outcome.failure_code(),
            Some(DiagnosticCode::SolverTimeout)
        );
        assert!(
            elapsed < Duration::from_secs(10),
            "kill-on-expiry took {elapsed:?} against a 30s sleeper"
        );
        fs::remove_file(&path).unwrap();
    }

    // Nonzero stub exit: code and both streams captured, §7.4
    // solver_execution_failure.
    #[test]
    fn nonzero_exit_captures_streams() {
        let path = stub("exit3", "echo boom-out\necho boom-err 1>&2\nexit 3");
        let adapter = Z3Adapter::with_program(&path).unwrap();
        let run = adapter.invoke("(check-sat)\n", Duration::from_secs(30));
        assert_eq!(run.outcome, RunOutcome::ExitFailure { code: Some(3) });
        assert_eq!(run.stdout, "boom-out\n");
        assert_eq!(run.stderr, "boom-err\n");
        assert_eq!(
            run.outcome.failure_code(),
            Some(DiagnosticCode::SolverExecutionFailure)
        );
        fs::remove_file(&path).unwrap();
    }

    // Spawn failure: a binary that vanishes after construction fails to
    // spawn, as data, mapping to solver_execution_failure; a binary that
    // never existed fails construction at the probe.
    #[test]
    fn spawn_failure_maps_to_execution_failure() {
        let path = stub("vanishing", "exit 0");
        let adapter = Z3Adapter::with_program(&path).unwrap();
        fs::remove_file(&path).unwrap();

        let run = adapter.invoke("(check-sat)\n", Duration::from_secs(30));
        match &run.outcome {
            RunOutcome::SpawnFailure { error } => assert!(!error.is_empty()),
            other => panic!("expected spawn failure, got {other:?}"),
        }
        assert_eq!(run.stdout, "");
        assert_eq!(run.stderr, "");
        assert_eq!(
            run.outcome.failure_code(),
            Some(DiagnosticCode::SolverExecutionFailure)
        );

        match Z3Adapter::with_program("/nonexistent/ckc-z3") {
            Err(AdapterError::Probe { detail }) => {
                assert!(detail.contains("spawn"), "detail {detail:?}")
            }
            other => panic!("expected probe failure, got {other:?}"),
        }
    }

    // A program answering `--version` with no version token fails
    // construction with the line preserved.
    #[test]
    fn unparseable_version_fails_construction() {
        let path =
            std::env::temp_dir().join(format!("ckc-verify-stub-{}-noversion", std::process::id()));
        fs::write(&path, "#!/bin/sh\necho \"mystery solver build 7\"\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        match Z3Adapter::with_program(&path) {
            Err(AdapterError::VersionUnparsed { line }) => {
                assert_eq!(line, "mystery solver build 7")
            }
            other => panic!("expected unparsed version, got {other:?}"),
        }
        fs::remove_file(&path).unwrap();
    }
}
