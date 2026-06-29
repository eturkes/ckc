//! SPEC §9 recorded-subprocess model runtime, adapter half: invoke the
//! environment-supplied local-model runtime as a subprocess under a
//! wall-clock budget and capture the process truth — the raw generated
//! bytes, stderr, and the spawn/timeout/exit fate.
//!
//! Mirrors `ckc_smt::verify`'s `Z3Adapter` (the §6 recorded-subprocess
//! precedent): a bare command name resolved on PATH, a `--version`-style
//! probe parsing the runtime's self-reported [`ModelIdentity`] at
//! construction (so the code carries no identity literal), and one
//! subprocess per call. The §9 deliverable names no engine, so the concrete
//! runtime wrapper is environment-supplied outside git; the committed
//! contract is only the CLI shape — the probe flag, the generation args,
//! and the stdin/stdout byte streams.
//!
//! This is the invoke SKELETON (`model-adapter.1`): it resolves the runtime,
//! probes the identity, and runs one budgeted call feeding the prompt on
//! stdin under the constraint-path + seed args, capturing the outcome. Real
//! constrained decoding (the route's grammar/JSON-Schema) and k-sampling
//! land in `model-adapter.2`; diagnostic mapping of a process failure is the
//! §7.4 model-fill stage's job, so the adapter returns raw outcome data.

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ckc_core::{Id, ModelIdentity};

/// Env var overriding the model-runtime command — a bare name resolved on
/// PATH, or a path. Absent or empty falls back to [`DEFAULT_MODEL_COMMAND`].
const MODEL_COMMAND_ENV: &str = "CKC_MODEL_COMMAND";

/// Committed default model-runtime command: a neutral ROLE name, never an
/// engine name or absolute path (§9 engine-agnostic deliverable). The
/// concrete wrapper binary is environment-supplied outside git and resolved
/// on PATH under this name unless [`MODEL_COMMAND_ENV`] overrides it.
const DEFAULT_MODEL_COMMAND: &str = "ckc-model-runtime";

/// Probe flag (the §6 `--version`-style call): the runtime self-reports its
/// [`ModelIdentity`] on stdout, one `key=value` line per field
/// (`model_id`/`quant`/`runtime_version`), so no identity sits in code.
const IDENTITY_PROBE_FLAG: &str = "--identity";

/// Generation arg naming the constraint schema/grammar file (the committed
/// CLI contract; the path comes from the exported `schemas/`).
const CONSTRAINT_FLAG: &str = "--constraint";

/// Generation arg naming the decoding seed (the committed CLI contract; a
/// fixed seed is the determinism lever for greedy decoding).
const SEED_FLAG: &str = "--seed";

/// Wall-clock budget for the construction-time identity probe. A healthy
/// runtime answers instantly; a probe that outlives this is a broken
/// installation, not a hard generation.
const IDENTITY_PROBE_BUDGET: Duration = Duration::from_secs(10);

/// Poll interval for the budget loop: short enough that kill-on-expiry
/// lands within a few milliseconds of the deadline.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Post-fate bound on draining the I/O threads. A dead process with no
/// orphans reaches EOF in microseconds; anything still unfinished at the
/// bound is a pipe held open by an orphaned grandchild, and the runner
/// snapshots partial output and detaches rather than wait it out.
const DRAIN_GRACE: Duration = Duration::from_secs(1);

/// ETXTBSY grace for spawning. A freshly written wrapper or just-linked
/// binary can briefly carry a write fd inherited by a sibling fork in a
/// multithreaded spawn, so `execve` races to `ExecutableFileBusy`; the
/// window clears in microseconds, so the spawn is retried under this bound.
/// A missing or unexecutable binary fails with a different kind and returns
/// at once, and a busy state outliving the grace surfaces as the failure it
/// is.
const SPAWN_BUSY_GRACE: Duration = Duration::from_millis(250);

/// SPEC §9 model-runtime adapter: one runtime command plus the
/// [`ModelIdentity`] parsed live from its [`IDENTITY_PROBE_FLAG`] reply at
/// construction. Generations run via [`ModelAdapter::invoke`], one
/// subprocess per call (the prompt on stdin, constraint path and seed as
/// args), each under its own wall-clock budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelAdapter {
    program: PathBuf,
    identity: ModelIdentity,
}

impl ModelAdapter {
    /// Adapter over the environment's model-runtime command: the
    /// [`MODEL_COMMAND_ENV`] override, else the neutral
    /// [`DEFAULT_MODEL_COMMAND`], resolved on PATH like the §6 `z3` default.
    pub fn new() -> Result<Self, ModelAdapterError> {
        Self::with_command(command_name())
    }

    /// Adapter over an explicit runtime command. A bare name (no path
    /// separator) is resolved on PATH by [`Command`]; a path is used
    /// directly. The command must answer [`IDENTITY_PROBE_FLAG`] with the
    /// three labeled identity fields (see [`parse_identity`]); construction
    /// fails otherwise, so a held adapter always carries a live-parsed
    /// identity.
    pub fn with_command(command: impl Into<PathBuf>) -> Result<Self, ModelAdapterError> {
        let program = command.into();
        let probe = run_process(
            &program,
            &[OsStr::new(IDENTITY_PROBE_FLAG)],
            b"",
            IDENTITY_PROBE_BUDGET,
        );
        match &probe.outcome {
            ModelOutcome::Completed { .. } => {}
            ModelOutcome::Timeout => {
                return Err(ModelAdapterError::Probe {
                    detail: "identity probe hit its wall-clock budget".to_owned(),
                });
            }
            ModelOutcome::ExitFailure { code } => {
                return Err(ModelAdapterError::Probe {
                    detail: format!(
                        "identity probe exited with code {:?}: {}",
                        code,
                        probe.stderr.trim()
                    ),
                });
            }
            ModelOutcome::SpawnFailure { error } => {
                return Err(ModelAdapterError::Probe {
                    detail: format!("identity probe failed to spawn: {error}"),
                });
            }
        }
        let reply = std::str::from_utf8(&probe.stdout_bytes).map_err(|_| {
            ModelAdapterError::IdentityUnparsed {
                detail: "identity reply is not valid UTF-8".to_owned(),
            }
        })?;
        let identity = parse_identity(reply)
            .map_err(|detail| ModelAdapterError::IdentityUnparsed { detail })?;
        Ok(ModelAdapter { program, identity })
    }

    /// The §9 model identity this adapter stamps into manifests and report
    /// rows — the probe truth, never a code literal.
    pub fn identity(&self) -> &ModelIdentity {
        &self.identity
    }

    /// Run one generation under `budget` wall-clock time (§9 recorded
    /// subprocess). The committed CLI contract: `prompt` on stdin, the
    /// `constraint` schema/grammar path and `seed` as args, generated bytes
    /// back on stdout. The subprocess is killed on expiry; every fate —
    /// completed bytes, timeout, exit failure, spawn failure — comes back as
    /// data in the [`ModelRun`], with whatever stdout/stderr drained before
    /// the end. This skeleton passes the constraint path verbatim — argv
    /// carries it as `OsStr`, lossless even for a non-UTF-8 path — so
    /// compiling it to the runtime's constraint format is the wrapper's job
    /// (and the live constrained decoding wiring is `model-adapter.2`).
    pub fn invoke(&self, prompt: &str, constraint: &Path, seed: u64, budget: Duration) -> ModelRun {
        let seed = seed.to_string();
        let args = [
            OsStr::new(CONSTRAINT_FLAG),
            constraint.as_os_str(),
            OsStr::new(SEED_FLAG),
            OsStr::new(&seed),
        ];
        run_process(&self.program, &args, prompt.as_bytes(), budget)
    }
}

/// One model-runtime subprocess, raw. `stdout_bytes` holds the stdout
/// drained within the run's budget plus the post-fate [`DRAIN_GRACE`] —
/// kept as bytes (never lossy-decoded) because the generated output is
/// recorded byte-for-byte for the cassette and its determinism is
/// byte-stability. It is the complete output for a well-behaved runtime
/// (one that closes stdout before exiting), but partial when a budget kill
/// landed first or a descendant outlived the exit still holding the pipe
/// open past the grace. `stderr` is the diagnostic stream, lossily decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRun {
    pub outcome: ModelOutcome,
    pub stdout_bytes: Vec<u8>,
    pub stderr: String,
}

/// Process fate of one model invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelOutcome {
    /// Exit status zero. `bytes` is the stdout drained before exit closed
    /// the pipe: the full output for a well-behaved runtime (one that closes
    /// stdout before exiting), but only the pre-[`DRAIN_GRACE`] prefix if a
    /// descendant outlives the exit still holding the pipe open (see
    /// [`ModelRun::stdout_bytes`]). Mirrors `Z3Adapter`'s success-only
    /// payload — the adapter parses nothing (the §7.4 schema/grounding parse
    /// is the model-fill stage's job), so a completed run's "result" is
    /// simply its output bytes, also in [`ModelRun::stdout_bytes`].
    /// Guaranteeing completeness against a pipe-holding descendant (a
    /// process-group kill or an EOF-gated capture outcome) lands with the
    /// cassette recording in `model-adapter.2`/`model-cassette`, where these
    /// bytes become byte-stability load-bearing.
    Completed { bytes: Vec<u8> },
    /// The wall-clock budget expired and the process was killed.
    Timeout,
    /// Nonzero exit status; `code` is `None` when a signal ended the
    /// process (or its status could not be collected).
    ExitFailure { code: Option<i32> },
    /// The process never started.
    SpawnFailure { error: String },
}

/// Construction failed: the identity probe broke, or its reply did not name
/// the three identity fields. Mirrors `ckc_smt::verify`'s `AdapterError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelAdapterError {
    /// The probe process timed out, exited nonzero, or failed to spawn.
    Probe { detail: String },
    /// The probe completed but its stdout did not yield a valid
    /// [`ModelIdentity`] (a missing field, an empty value, or an
    /// ungrammatical `model_id`).
    IdentityUnparsed { detail: String },
}

impl std::fmt::Display for ModelAdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelAdapterError::Probe { detail } => {
                write!(f, "model runtime identity probe failed: {detail}")
            }
            ModelAdapterError::IdentityUnparsed { detail } => {
                write!(f, "model runtime identity reply unparsed: {detail}")
            }
        }
    }
}

impl std::error::Error for ModelAdapterError {}

/// The effective model-runtime command: the [`MODEL_COMMAND_ENV`] value when
/// set and non-empty, else [`DEFAULT_MODEL_COMMAND`].
pub fn command_name() -> String {
    resolve_command(std::env::var(MODEL_COMMAND_ENV).ok())
}

/// Pure env-resolution: a non-empty override wins, else the neutral default.
/// Split out so the policy is tested without mutating process env (the crate
/// forbids the `unsafe` `set_var`).
fn resolve_command(env_override: Option<String>) -> String {
    match env_override {
        Some(value) if !value.is_empty() => value,
        _ => DEFAULT_MODEL_COMMAND.to_owned(),
    }
}

/// Parse a [`ModelIdentity`] from a probe reply: one `key=value` per line,
/// order-independent, extra lines ignored, first occurrence of each key
/// kept. All three fields must be present and non-empty, and `model_id` must
/// be a grammatical [`Id`]; any miss is a descriptive error string.
fn parse_identity(reply: &str) -> Result<ModelIdentity, String> {
    let mut model_id = None;
    let mut quant = None;
    let mut runtime_version = None;
    for line in reply.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let slot = match key.trim() {
            "model_id" => &mut model_id,
            "quant" => &mut quant,
            "runtime_version" => &mut runtime_version,
            _ => continue,
        };
        if slot.is_none() {
            *slot = Some(value.trim().to_owned());
        }
    }
    let model_id = require_field(model_id, "model_id")?;
    let quant = require_field(quant, "quant")?;
    let runtime_version = require_field(runtime_version, "runtime_version")?;
    let model_id = Id::new(&model_id)
        .map_err(|e| format!("identity model_id {model_id:?} is not a valid id: {e}"))?;
    Ok(ModelIdentity {
        model_id,
        quant,
        runtime_version,
    })
}

/// A probe field: present and non-empty, or a descriptive error naming it.
fn require_field(value: Option<String>, name: &str) -> Result<String, String> {
    match value {
        Some(value) if !value.is_empty() => Ok(value),
        Some(_) => Err(format!("identity reply has empty {name}")),
        None => Err(format!("identity reply missing {name}")),
    }
}

/// Reader thread draining one output pipe into a shared buffer until EOF
/// (write ends close at exit or kill), so neither pipe can fill and stall
/// the child. The buffer is shared so the runner can snapshot partial
/// output without joining: a killed process's orphans inherit the write ends
/// and would hold a join hostage.
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

/// Spawn `program args` with all three stdio streams piped, retrying only
/// on `ExecutableFileBusy` (ETXTBSY) within [`SPAWN_BUSY_GRACE`]. Every
/// other error returns at once, preserving the caller's spawn-failure
/// mapping.
fn spawn_piped(program: &Path, args: &[&OsStr]) -> std::io::Result<Child> {
    let deadline = Instant::now() + SPAWN_BUSY_GRACE;
    loop {
        match Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Err(error)
                if error.kind() == std::io::ErrorKind::ExecutableFileBusy
                    && Instant::now() < deadline =>
            {
                thread::sleep(POLL_INTERVAL);
            }
            settled => return settled,
        }
    }
}

/// Spawn `program args`, feed `stdin_bytes`, and wait at most `budget`
/// wall-clock time, polling every [`POLL_INTERVAL`] and killing on expiry.
/// Stdin is written from its own thread (a write error means the child
/// stopped reading — already dead or never reading; the exit fate carries
/// the story), stdout/stderr drain concurrently into shared buffers, and
/// every fate returns as a [`ModelRun`]. After the fate is known the I/O
/// threads get [`DRAIN_GRACE`] to reach EOF, then the buffers are
/// snapshotted and laggards detached — a clean exit drains in microseconds,
/// while pipes held open past the kill (an orphaned grandchild) cannot
/// stall the runner. Mirrors `ckc_smt::verify`'s `run_process`, but the
/// stdout stays raw bytes and a clean exit yields [`ModelOutcome::Completed`]
/// rather than a parsed verdict.
fn run_process(program: &Path, args: &[&OsStr], stdin_bytes: &[u8], budget: Duration) -> ModelRun {
    let mut child = match spawn_piped(program, args) {
        Ok(child) => child,
        Err(error) => {
            return ModelRun {
                outcome: ModelOutcome::SpawnFailure {
                    error: error.to_string(),
                },
                stdout_bytes: Vec::new(),
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
        buf.lock()
            .expect("drain readers never panic holding the lock")
            .clone()
    };
    let stdout_bytes = snapshot(&stdout_buf);
    let stderr = String::from_utf8_lossy(&snapshot(&stderr_buf)).into_owned();

    let outcome = match fate {
        Fate::TimedOut => ModelOutcome::Timeout,
        Fate::WaitFailed => ModelOutcome::ExitFailure { code: None },
        Fate::Exited(status) if status.success() => ModelOutcome::Completed {
            bytes: stdout_bytes.clone(),
        },
        Fate::Exited(status) => ModelOutcome::ExitFailure {
            code: status.code(),
        },
    };
    ModelRun {
        outcome,
        stdout_bytes,
        stderr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;

    /// Write an executable stub script, named uniquely per test for parallel
    /// runs, and return its path.
    fn write_exec(tag: &str, script: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("ckc-model-stub-{}-{tag}", std::process::id()));
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// The committed model-runtime stub: a self-contained emulator of the
    /// `model-adapter.1` CLI contract. `--identity` self-reports the three
    /// identity fields; otherwise it honors `--constraint <path> --seed <n>`
    /// with the prompt on stdin and writes deterministic bytes to stdout
    /// (echoing the contract inputs). Two prompt sentinels drive the timeout
    /// and nonzero-exit fates so every [`ModelOutcome`] is exercised from one
    /// committed fixture.
    const COMMITTED_STUB: &str = r#"#!/bin/sh
if [ "$1" = "--identity" ]; then
  echo "model_id=model.stub"
  echo "quant=stub_quant"
  echo "runtime_version=0.0.0-stub"
  exit 0
fi
prompt=$(cat)
case "$prompt" in
  __TIMEOUT__) sleep 30 ;;
  __EXIT_FAIL__) printf 'partial-out'; printf 'boom-err' 1>&2; exit 4 ;;
  *) printf 'constraint=%s seed=%s prompt=%s' "$2" "$4" "$prompt" ;;
esac
"#;

    fn committed_stub(tag: &str) -> PathBuf {
        write_exec(tag, COMMITTED_STUB)
    }

    // Probe: construction parses the three labeled identity fields from the
    // committed stub's `--identity` reply.
    #[test]
    fn stub_identity_parses_from_probe() {
        let path = committed_stub("identity");
        let adapter = ModelAdapter::with_command(&path).unwrap();
        let identity = adapter.identity();
        assert_eq!(identity.model_id.as_str(), "model.stub");
        assert_eq!(identity.quant, "stub_quant");
        assert_eq!(identity.runtime_version, "0.0.0-stub");
        fs::remove_file(&path).unwrap();
    }

    // Invoke honors the CLI contract — constraint path and seed as args, the
    // prompt on stdin — and returns the generated bytes both as the
    // `Completed` payload and as the raw `stdout_bytes`; the same inputs
    // yield byte-identical output.
    #[test]
    fn invoke_honors_cli_contract_deterministically() {
        let path = committed_stub("invoke");
        let adapter = ModelAdapter::with_command(&path).unwrap();
        let constraint = Path::new("schemas/test.grammar");

        let run = adapter.invoke("hello", constraint, 7, Duration::from_secs(30));
        let expected = b"constraint=schemas/test.grammar seed=7 prompt=hello".to_vec();
        assert_eq!(
            run.outcome,
            ModelOutcome::Completed {
                bytes: expected.clone()
            }
        );
        assert_eq!(run.stdout_bytes, expected);
        assert_eq!(run.stderr, "");

        let again = adapter.invoke("hello", constraint, 7, Duration::from_secs(30));
        assert_eq!(again.stdout_bytes, run.stdout_bytes);
        fs::remove_file(&path).unwrap();
    }

    // The constraint path reaches the runtime verbatim even when it is not
    // valid UTF-8: argv carries it as `OsStr`, so a 0xFF byte survives
    // rather than being lossy-rewritten (which would break a wrapper opening
    // the path). Guards the committed verbatim contract against a
    // `to_string_lossy` regression.
    #[test]
    fn invoke_passes_non_utf8_constraint_path_verbatim() {
        let path = committed_stub("nonutf8path");
        let adapter = ModelAdapter::with_command(&path).unwrap();
        let constraint = Path::new(OsStr::from_bytes(b"schemas/g\xff.grammar"));

        let run = adapter.invoke("p", constraint, 7, Duration::from_secs(30));
        let expected = b"constraint=schemas/g\xff.grammar seed=7 prompt=p".to_vec();
        assert_eq!(
            run.outcome,
            ModelOutcome::Completed {
                bytes: expected.clone()
            }
        );
        assert_eq!(run.stdout_bytes, expected);
        fs::remove_file(&path).unwrap();
    }

    // Budget kill: a stub generation outliving its budget is killed within
    // moments of the deadline and mints `Timeout` — the orphaned `sleep`
    // holding the stdout pipe open is detached after the drain grace, so the
    // runner returns promptly.
    #[test]
    fn budget_kills_stub_generation() {
        let path = committed_stub("timeout");
        let adapter = ModelAdapter::with_command(&path).unwrap();

        let start = Instant::now();
        let run = adapter.invoke("__TIMEOUT__", Path::new("c"), 1, Duration::from_millis(200));
        let elapsed = start.elapsed();

        assert_eq!(run.outcome, ModelOutcome::Timeout);
        assert!(
            elapsed < Duration::from_secs(10),
            "kill-on-expiry took {elapsed:?} against a 30s sleeper"
        );
        fs::remove_file(&path).unwrap();
    }

    // Nonzero stub exit: code and both streams captured (partial stdout kept
    // as bytes).
    #[test]
    fn nonzero_exit_captures_streams() {
        let path = committed_stub("exit");
        let adapter = ModelAdapter::with_command(&path).unwrap();
        let run = adapter.invoke("__EXIT_FAIL__", Path::new("c"), 1, Duration::from_secs(30));
        assert_eq!(run.outcome, ModelOutcome::ExitFailure { code: Some(4) });
        assert_eq!(run.stdout_bytes, b"partial-out");
        assert_eq!(run.stderr, "boom-err");
        fs::remove_file(&path).unwrap();
    }

    // Spawn failure: a binary that vanishes after construction fails to
    // spawn, as data, with empty capture.
    #[test]
    fn spawn_failure_after_binary_vanishes() {
        let path = committed_stub("vanishing");
        let adapter = ModelAdapter::with_command(&path).unwrap();
        fs::remove_file(&path).unwrap();

        let run = adapter.invoke("hello", Path::new("c"), 1, Duration::from_secs(30));
        match &run.outcome {
            ModelOutcome::SpawnFailure { error } => assert!(!error.is_empty()),
            other => panic!("expected spawn failure, got {other:?}"),
        }
        assert!(run.stdout_bytes.is_empty());
        assert_eq!(run.stderr, "");
    }

    // A bare command name absent from PATH fails construction at the probe's
    // spawn — the PATH-resolution path of `with_command` exercised without
    // mutating process env.
    #[test]
    fn bare_command_name_probe_fails_to_spawn_when_absent() {
        let missing = format!("ckc-model-absent-{}", std::process::id());
        match ModelAdapter::with_command(&missing) {
            Err(ModelAdapterError::Probe { detail }) => {
                assert!(detail.contains("spawn"), "detail {detail:?}")
            }
            other => panic!("expected probe spawn failure, got {other:?}"),
        }
    }

    // A runtime answering `--identity` without the runtime_version field
    // fails construction with the missing field named.
    #[test]
    fn unparseable_identity_fails_construction() {
        let script = r#"#!/bin/sh
if [ "$1" = "--identity" ]; then
  echo "model_id=model.partial"
  echo "quant=q"
  exit 0
fi
"#;
        let path = write_exec("badidentity", script);
        match ModelAdapter::with_command(&path) {
            Err(ModelAdapterError::IdentityUnparsed { detail }) => {
                assert!(detail.contains("runtime_version"), "detail {detail:?}")
            }
            other => panic!("expected identity-unparsed, got {other:?}"),
        }
        fs::remove_file(&path).unwrap();
    }

    // A runtime whose --identity reply carries invalid UTF-8 fails
    // construction strictly, rather than lossy-decoding the recorded
    // identity into replacement characters.
    #[test]
    fn non_utf8_identity_fails_construction() {
        let script = r#"#!/bin/sh
if [ "$1" = "--identity" ]; then
  printf 'model_id=model.x\nquant=q\nruntime_version='
  printf '\377'
  printf '\n'
  exit 0
fi
"#;
        let path = write_exec("nonutf8identity", script);
        match ModelAdapter::with_command(&path) {
            Err(ModelAdapterError::IdentityUnparsed { detail }) => {
                assert!(detail.contains("UTF-8"), "detail {detail:?}")
            }
            other => panic!("expected identity-unparsed, got {other:?}"),
        }
        fs::remove_file(&path).unwrap();
    }

    // Env resolution: a non-empty override wins, an empty one and an absent
    // one both fall back to the neutral default.
    #[test]
    fn resolve_command_honors_env_then_default() {
        assert_eq!(
            resolve_command(Some("custom-runtime".to_owned())),
            "custom-runtime"
        );
        assert_eq!(resolve_command(Some(String::new())), DEFAULT_MODEL_COMMAND);
        assert_eq!(resolve_command(None), DEFAULT_MODEL_COMMAND);
    }

    // Canned-text identity parse: labeled fields, order-independence with
    // extra lines ignored, and the rejection shapes (missing field, empty
    // value, ungrammatical model_id).
    #[test]
    fn parse_identity_reads_and_rejects_canned_text() {
        let ok = parse_identity("model_id=model.x\nquant=q4\nruntime_version=1.2.3\n").unwrap();
        assert_eq!(ok.model_id.as_str(), "model.x");
        assert_eq!(ok.quant, "q4");
        assert_eq!(ok.runtime_version, "1.2.3");

        let reordered =
            parse_identity("runtime_version=9\nnote=ignored\nquant=q\nmodel_id=model.y\n").unwrap();
        assert_eq!(reordered.model_id.as_str(), "model.y");
        assert_eq!(reordered.runtime_version, "9");

        assert!(
            parse_identity("quant=q\nruntime_version=1\n")
                .unwrap_err()
                .contains("model_id")
        );
        assert!(
            parse_identity("model_id=model.z\nquant=\nruntime_version=1\n")
                .unwrap_err()
                .contains("quant")
        );
        assert!(
            parse_identity("model_id=bad id\nquant=q\nruntime_version=1\n")
                .unwrap_err()
                .contains("valid id")
        );
    }

    // ETXTBSY recover: an open write fd makes `execve` race to
    // `ExecutableFileBusy`, so spawn_piped's first attempt fails and it must
    // retry; releasing the fd well inside the grace lets a later attempt
    // win. A deterministic stand-in for the parallel fork/exec race the
    // retry exists to absorb.
    #[test]
    fn spawn_piped_retries_through_etxtbsy() {
        let path = write_exec("etxtbsy_recover", "#!/bin/sh\nexit 0\n");
        let writer = fs::OpenOptions::new().write(true).open(&path).unwrap();
        let probe_path = path.clone();
        let attempt = thread::spawn(move || spawn_piped(&probe_path, &[]));

        thread::sleep(Duration::from_millis(60));
        drop(writer);

        attempt
            .join()
            .unwrap()
            .expect("spawn must recover once the write fd closes")
            .wait()
            .unwrap();
        fs::remove_file(&path).unwrap();
    }

    // ETXTBSY surface: a write fd held for the whole grace keeps every
    // attempt busy, so spawn_piped stays bounded — it returns the
    // `ExecutableFileBusy` error after the grace rather than spinning on.
    #[test]
    fn spawn_piped_surfaces_persistent_etxtbsy() {
        let path = write_exec("etxtbsy_surface", "#!/bin/sh\nexit 0\n");
        let writer = fs::OpenOptions::new().write(true).open(&path).unwrap();

        let start = Instant::now();
        let result = spawn_piped(&path, &[]);
        let elapsed = start.elapsed();

        let error = result.expect_err("a persistently busy binary must not spawn");
        assert_eq!(error.kind(), std::io::ErrorKind::ExecutableFileBusy);
        assert!(
            elapsed >= SPAWN_BUSY_GRACE,
            "must retry up to the grace, gave up after {elapsed:?}"
        );
        drop(writer);
        fs::remove_file(&path).unwrap();
    }

    // Only ETXTBSY is retried: a missing binary surfaces its error at once,
    // well inside the busy grace, so unrelated spawn failures are not
    // delayed.
    #[test]
    fn spawn_piped_does_not_retry_other_errors() {
        let missing = std::env::temp_dir().join(format!("ckc-model-absent-{}", std::process::id()));
        let start = Instant::now();
        let result = spawn_piped(&missing, &[]);
        let elapsed = start.elapsed();

        let error = result.expect_err("a missing binary must fail to spawn");
        assert_ne!(error.kind(), std::io::ErrorKind::ExecutableFileBusy);
        assert!(
            elapsed < SPAWN_BUSY_GRACE,
            "non-busy errors must not retry, took {elapsed:?}"
        );
    }
}
