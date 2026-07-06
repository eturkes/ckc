//! SPEC §3 command surface: parse → validate → execute → finish.
//!
//! ```text
//! ckc registry check
//! ckc run --experiment <experiment-id> --out runs/<run-id>
//! ckc replay runs/<run-id>
//! ckc trace --run runs/<run-id> --finding <finding-id>
//! ```
//!
//! Every invocation — including unparseable input — ends in exactly one §4.4
//! total operation result on stdout. Argument and validation failures report
//! a `schema_invalid` diagnostic under outcome `invalid` (§4.4 "command
//! validation fails"). `ckc trace` precedes the result line with its
//! chain text (§8.5 item 7) and `ckc replay` with its match report (§8.5
//! item 8), the two stdout bodies. Exit code maps from the
//! outcome: `ok` 0, `invalid` 2, every other outcome 1. There is no human
//! usage text: the result line, command body, and event stream are the
//! interface.

use std::path::{Path, PathBuf};

use ckc_core::{DiagnosticCode, DiagnosticRecord, Id, Outcome, TotalOperationResult, jsonl_line};

use crate::shell::{Shell, run_none, static_id};

const OP_CLI: &str = "cli";
const OP_REGISTRY_CHECK: &str = "registry.check";
const OP_RUN: &str = "run";
const OP_REPLAY: &str = "replay";
const OP_TRACE: &str = "trace";

/// One completed invocation: what `main` emits plus the process exit code.
/// `result_line` is the §4.4 result as one canonical JSONL line for stdout;
/// `streamed_events` is the §4.6 events stream for stderr whenever no output
/// directory took it as `logs/events.jsonl`; `command_output` is the
/// command body's stdout text ahead of the result line (the `ckc trace`
/// chain, the `ckc replay` match report), present only beside an ok
/// result.
pub struct CliExit {
    pub result: TotalOperationResult,
    pub result_line: Vec<u8>,
    pub streamed_events: Option<Vec<u8>>,
    pub command_output: Option<Vec<u8>>,
    pub exit_code: i32,
}

/// Run one `ckc` invocation (`args` excludes the binary name). Total: every
/// path ends in exactly one §4.4 result.
pub fn run_cli(args: &[String]) -> CliExit {
    let command = match parse(args).and_then(validate) {
        Ok(command) => command,
        Err(fail) => return invalid_exit(fail),
    };
    let mut shell = shell_for(&command);
    let command_output = execute(&command, &mut shell);
    finish_exit(command_op(&command), shell, command_output)
}

/// Argument-shape failure or input-validation failure: `op` is the §4.4
/// operation id the failure is attributed to (`cli` when no command parsed).
struct Fail {
    op: Id,
    reason: String,
}

impl Fail {
    fn new(op: Id, reason: String) -> Fail {
        Fail { op, reason }
    }
}

/// Shape-parsed command: flags accounted for, values still raw text.
#[derive(Debug)]
enum RawCommand {
    RegistryCheck,
    Run {
        experiment: String,
        out: String,
        record: bool,
    },
    Replay {
        run_dir: String,
    },
    Trace {
        run_dir: String,
        finding: String,
    },
}

/// Validated command.
#[derive(Debug)]
enum Command {
    RegistryCheck,
    Run {
        experiment: Id,
        out: PathBuf,
        run_id: Id,
        record: bool,
    },
    Replay {
        run_dir: PathBuf,
        run_id: Id,
    },
    Trace {
        run_dir: PathBuf,
        run_id: Id,
        finding: Id,
    },
}

fn parse(args: &[String]) -> Result<RawCommand, Fail> {
    let Some((head, rest)) = args.split_first() else {
        return Err(Fail::new(
            static_id(OP_CLI),
            "missing command; expected one of: registry run replay trace".to_owned(),
        ));
    };
    match head.as_str() {
        "registry" => {
            let op = static_id(OP_REGISTRY_CHECK);
            match rest {
                [sub] if sub.as_str() == "check" => Ok(RawCommand::RegistryCheck),
                [] => Err(Fail::new(
                    op,
                    "missing subcommand; expected `registry check`".to_owned(),
                )),
                _ => Err(Fail::new(
                    op,
                    format!("unexpected arguments after `registry`: {rest:?}"),
                )),
            }
        }
        "run" => {
            // `--record` is a bare boolean; pull it out before the value-flag
            // pass, which accounts for exactly `--experiment` and `--out`.
            let (record, rest) = take_bool_flag(OP_RUN, "--record", rest)?;
            let [experiment, out] = take_flags(OP_RUN, ["--experiment", "--out"], &rest)?;
            Ok(RawCommand::Run {
                experiment,
                out,
                record,
            })
        }
        "replay" => {
            let op = static_id(OP_REPLAY);
            match rest {
                [dir] if !dir.starts_with('-') => Ok(RawCommand::Replay {
                    run_dir: dir.clone(),
                }),
                [] => Err(Fail::new(
                    op,
                    "missing run directory; expected `replay runs/<run-id>`".to_owned(),
                )),
                _ => Err(Fail::new(
                    op,
                    format!("expected exactly one run directory path, got {rest:?}"),
                )),
            }
        }
        "trace" => {
            let [run_dir, finding] = take_flags(OP_TRACE, ["--run", "--finding"], rest)?;
            Ok(RawCommand::Trace { run_dir, finding })
        }
        other => Err(Fail::new(
            static_id(OP_CLI),
            format!("unknown command {other:?}; expected one of: registry run replay trace"),
        )),
    }
}

/// Pull an optional bare boolean flag out of `args`, returning whether it was
/// present and the remaining tokens for the value-flag pass. A repeat is a
/// duplicate error; a value-bearing form (`--flag=x`) is rejected here as
/// taking no value, so the outcome is the same wherever the flag sits.
fn take_bool_flag(op: &str, name: &str, args: &[String]) -> Result<(bool, Vec<String>), Fail> {
    let mut present = false;
    let mut rest = Vec::with_capacity(args.len());
    for token in args {
        if token.as_str() == name {
            if present {
                return Err(Fail::new(static_id(op), format!("duplicate {name}")));
            }
            present = true;
        } else if token
            .strip_prefix(name)
            .is_some_and(|tail| tail.starts_with('='))
        {
            // A bare boolean takes no value: reject `--flag=x` here so the
            // outcome is position-independent, rather than leaking it to the
            // value-flag pass (where a leading token errors "unexpected" but a
            // token in value position is swallowed as some flag's value).
            return Err(Fail::new(static_id(op), format!("{name} takes no value")));
        } else {
            rest.push(token.clone());
        }
    }
    Ok((present, rest))
}

/// Collect exactly the named flags, each once with one value, no extras.
fn take_flags<const N: usize>(
    op: &str,
    names: [&str; N],
    args: &[String],
) -> Result<[String; N], Fail> {
    let mut values: [Option<String>; N] = std::array::from_fn(|_| None);
    let mut tokens = args.iter();
    while let Some(token) = tokens.next() {
        let Some(slot) = names.iter().position(|name| token.as_str() == *name) else {
            return Err(Fail::new(
                static_id(op),
                format!("unexpected argument {token:?}"),
            ));
        };
        if values[slot].is_some() {
            return Err(Fail::new(static_id(op), format!("duplicate {token}")));
        }
        let Some(value) = tokens.next() else {
            return Err(Fail::new(
                static_id(op),
                format!("missing value for {token}"),
            ));
        };
        values[slot] = Some(value.clone());
    }
    let mut out: [String; N] = std::array::from_fn(|_| String::new());
    for (slot, value) in values.into_iter().enumerate() {
        match value {
            Some(value) => out[slot] = value,
            None => {
                return Err(Fail::new(
                    static_id(op),
                    format!("missing required {}", names[slot]),
                ));
            }
        }
    }
    Ok(out)
}

/// Input validation (§3 "validates inputs"): Id grammar on identifiers and
/// run-directory names, existence for consumed run directories, freshness
/// plus creation for `run`'s output directory.
fn validate(raw: RawCommand) -> Result<Command, Fail> {
    match raw {
        RawCommand::RegistryCheck => Ok(Command::RegistryCheck),
        RawCommand::Run {
            experiment,
            out,
            record,
        } => {
            let op = static_id(OP_RUN);
            let experiment = parse_id(&op, "--experiment", &experiment)?;
            let out = PathBuf::from(out);
            let run_id = dir_run_id(&op, &out)?;
            if out.exists() {
                return Err(Fail::new(
                    op,
                    format!("output directory already exists: {}", out.display()),
                ));
            }
            std::fs::create_dir_all(&out).map_err(|e| {
                Fail::new(
                    op.clone(),
                    format!("creating output directory {}: {e}", out.display()),
                )
            })?;
            Ok(Command::Run {
                experiment,
                out,
                run_id,
                record,
            })
        }
        RawCommand::Replay { run_dir } => {
            let op = static_id(OP_REPLAY);
            let run_dir = existing_run_dir(&op, run_dir)?;
            let run_id = dir_run_id(&op, &run_dir)?;
            Ok(Command::Replay { run_dir, run_id })
        }
        RawCommand::Trace { run_dir, finding } => {
            let op = static_id(OP_TRACE);
            let finding = parse_id(&op, "--finding", &finding)?;
            let run_dir = existing_run_dir(&op, run_dir)?;
            let run_id = dir_run_id(&op, &run_dir)?;
            Ok(Command::Trace {
                run_dir,
                run_id,
                finding,
            })
        }
    }
}

fn parse_id(op: &Id, what: &str, raw: &str) -> Result<Id, Fail> {
    raw.parse()
        .map_err(|e| Fail::new(op.clone(), format!("{what} {raw:?}: {e}")))
}

/// §4.6 run id from the run directory's name (`runs/<run-id>`, §3).
fn dir_run_id(op: &Id, dir: &Path) -> Result<Id, Fail> {
    let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
        return Err(Fail::new(
            op.clone(),
            format!("run directory {} has no usable name", dir.display()),
        ));
    };
    name.parse()
        .map_err(|e| Fail::new(op.clone(), format!("run id (directory name) {name:?}: {e}")))
}

fn existing_run_dir(op: &Id, raw: String) -> Result<PathBuf, Fail> {
    let dir = PathBuf::from(raw);
    if dir.is_dir() {
        Ok(dir)
    } else {
        Err(Fail::new(
            op.clone(),
            format!("run directory not found: {}", dir.display()),
        ))
    }
}

fn command_op(command: &Command) -> Id {
    let token = match command {
        Command::RegistryCheck => OP_REGISTRY_CHECK,
        Command::Run { .. } => OP_RUN,
        Command::Replay { .. } => OP_REPLAY,
        Command::Trace { .. } => OP_TRACE,
    };
    static_id(token)
}

fn shell_for(command: &Command) -> Shell {
    match command {
        Command::RegistryCheck => Shell::open(command_op(command), run_none(), None),
        Command::Run { out, run_id, .. } => {
            Shell::open(command_op(command), run_id.clone(), Some(out.clone()))
        }
        // replay/trace consume an existing run directory and stream their
        // events: trace writes nothing, and replay's re-execution writes
        // only through the core's internal shell into the scratch layout.
        Command::Replay { run_id, .. } | Command::Trace { run_id, .. } => {
            Shell::open(command_op(command), run_id.clone(), None)
        }
    }
}

/// Command bodies. `registry check`, `run`, and `replay`'s re-execution
/// work against the working directory (§3 anchors `registry/` and corpus
/// paths at the repository root); `trace` and `replay` resolve their
/// validated run directories in place and are the bodies returning stdout
/// text (the §8.5 item 7 chain, the §8.5 item 8 match report).
fn execute(command: &Command, shell: &mut Shell) -> Option<Vec<u8>> {
    match command {
        Command::RegistryCheck => {
            crate::registry_check::check(Path::new("."), shell);
            None
        }
        Command::Run {
            experiment, record, ..
        } => {
            crate::run::execute(Path::new("."), experiment, *record, shell);
            None
        }
        Command::Trace {
            run_dir, finding, ..
        } => crate::trace::command::execute(run_dir, finding, shell),
        Command::Replay { run_dir, .. } => {
            crate::replay::command::execute(Path::new("."), run_dir, shell)
        }
    }
}

fn exit_code(outcome: Outcome) -> i32 {
    match outcome {
        Outcome::Ok => 0,
        Outcome::Invalid => 2,
        _ => 1,
    }
}

/// Failure before any command body ran: an out-dir-less shell still emits
/// the event stream and the single invalid result.
fn invalid_exit(fail: Fail) -> CliExit {
    let mut shell = Shell::open(fail.op.clone(), run_none(), None);
    shell.diagnostic(DiagnosticRecord {
        code: DiagnosticCode::SchemaInvalid,
        outcome: Outcome::Invalid,
        payload: vec![(static_id("reason"), fail.reason)],
        region_ids: Vec::new(),
        artifact_hashes: Vec::new(),
    });
    finish_exit(fail.op, shell, None)
}

fn finish_exit(op: Id, shell: Shell, command_output: Option<Vec<u8>>) -> CliExit {
    match shell.finish() {
        Ok(finished) => CliExit {
            exit_code: exit_code(finished.result.outcome),
            result_line: finished.result_line,
            streamed_events: finished.streamed_events,
            command_output,
            result: finished.result,
        },
        Err(err) => {
            // Last-resort path: the invariant layer itself failed (logs I/O,
            // canonical emission). Keep the exactly-one-result requirements with
            // a bare invalid result; the cause goes straight to stderr since
            // no event stream could be built.
            eprintln!("ckc: {op}: {err}");
            let result = TotalOperationResult {
                operation_id: op,
                outcome: Outcome::Invalid,
                value_hashes: Vec::new(),
                diagnostic_hashes: Vec::new(),
                residual_hashes: Vec::new(),
                ambiguity_hashes: Vec::new(),
                incoherence_hashes: Vec::new(),
            };
            let result_line =
                jsonl_line(&result).expect("bare invalid result emission cannot fail");
            CliExit {
                exit_code: exit_code(Outcome::Invalid),
                result_line,
                streamed_events: None,
                command_output: None,
                result,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{EventRecord, read_jsonl};

    fn args(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|t| (*t).to_owned()).collect()
    }

    fn streamed(exit: &CliExit) -> Vec<EventRecord> {
        read_jsonl(exit.streamed_events.as_deref().unwrap()).unwrap()
    }

    /// Reason text of the single schema_invalid diagnostic in the stream.
    fn invalid_reason(exit: &CliExit) -> String {
        assert_eq!(exit.result.outcome, Outcome::Invalid);
        assert_eq!(exit.exit_code, 2);
        assert_eq!(exit.result.diagnostic_hashes.len(), 1);
        let events = streamed(exit);
        assert_eq!(events.len(), 1);
        let diagnostic = &events[0].diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::SchemaInvalid);
        assert_eq!(diagnostic.payload[0].0, static_id("reason"));
        diagnostic.payload[0].1.clone()
    }

    fn assert_one_result_line(exit: &CliExit) {
        let parsed: Vec<TotalOperationResult> = read_jsonl(&exit.result_line).unwrap();
        assert_eq!(parsed, vec![exit.result.clone()]);
    }

    // Dispatch wires the registry_check body: from this test's working
    // directory (the crate dir, no registry/ tree) the checker reports the
    // three unreadable files. The ok path runs in registry_check::tests and
    // the binary-level test, both rooted at the repository.
    #[test]
    fn registry_check_dispatches_into_the_checker() {
        let exit = run_cli(&args(&["registry", "check"]));
        assert_eq!(exit.result.operation_id, static_id(OP_REGISTRY_CHECK));
        assert_eq!(exit.result.outcome, Outcome::Invalid);
        assert_eq!(exit.exit_code, 2);
        assert_eq!(exit.result.diagnostic_hashes.len(), 3);
        let events = streamed(&exit);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].outcome, Outcome::Invalid);
        assert_eq!(events[0].diagnostics.len(), 3);
        assert_one_result_line(&exit);
    }

    // Dispatch wires the run body: from this test's working directory (the
    // crate dir, no registry/ tree) resolution reports the three unreadable
    // registry files; the out dir was created and took the streams. The ok
    // path runs in run::tests and the binary-level test, rooted at the
    // repository.
    #[test]
    fn run_creates_out_dir_and_lands_logs() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("m1");
        let exit = run_cli(&args(&[
            "run",
            "--experiment",
            "exp.m1_scaffold",
            "--out",
            out.to_str().unwrap(),
        ]));
        assert_eq!(exit.result.operation_id, static_id(OP_RUN));
        assert_eq!(exit.result.outcome, Outcome::Invalid);
        assert_eq!(exit.exit_code, 2);
        assert!(exit.streamed_events.is_none());
        let events: Vec<EventRecord> =
            read_jsonl(&std::fs::read(out.join("logs/events.jsonl")).unwrap()).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].run_id, static_id("m1"));
        assert_eq!(events[0].processing_stage, static_id(OP_RUN));
        let diagnostics: Vec<DiagnosticRecord> =
            read_jsonl(&std::fs::read(out.join("logs/diagnostics.jsonl")).unwrap()).unwrap();
        assert_eq!(diagnostics.len(), 3);
        assert_eq!(exit.result.diagnostic_hashes.len(), 3);
        assert_one_result_line(&exit);
    }

    #[test]
    fn run_rejects_existing_out_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("m1");
        std::fs::create_dir_all(&out).unwrap();
        let exit = run_cli(&args(&[
            "run",
            "--experiment",
            "exp.m1_scaffold",
            "--out",
            out.to_str().unwrap(),
        ]));
        assert!(invalid_reason(&exit).contains("already exists"));
    }

    #[test]
    fn run_argument_shape_failures() {
        let tmp = tempfile::tempdir().unwrap();
        let fresh = tmp.path().join("m2");
        let fresh_str = fresh.to_str().unwrap();
        let cases: Vec<(Vec<String>, &str)> = vec![
            (
                args(&["run", "--experiment", "exp.x"]),
                "missing required --out",
            ),
            (
                args(&["run", "--out", fresh_str]),
                "missing required --experiment",
            ),
            (
                args(&[
                    "run",
                    "--experiment",
                    "a",
                    "--experiment",
                    "b",
                    "--out",
                    fresh_str,
                ]),
                "duplicate --experiment",
            ),
            (
                args(&[
                    "run",
                    "--experiment",
                    "exp.x",
                    "--out",
                    fresh_str,
                    "--bogus",
                    "v",
                ]),
                "unexpected argument",
            ),
            (
                args(&["run", "--experiment"]),
                "missing value for --experiment",
            ),
            (
                args(&["run", "--experiment", "EXP", "--out", fresh_str]),
                "--experiment \"EXP\"",
            ),
        ];
        for (argv, needle) in cases {
            let exit = run_cli(&argv);
            let reason = invalid_reason(&exit);
            assert!(reason.contains(needle), "{argv:?} -> {reason}");
            assert_eq!(exit.result.operation_id, static_id(OP_RUN));
        }
        // Validation failed before creation in every case.
        assert!(!fresh.exists());
    }

    #[test]
    fn run_record_flag_parses() {
        // No `--record` → record false.
        let Ok(RawCommand::Run { record, .. }) =
            parse(&args(&["run", "--experiment", "e", "--out", "o"]))
        else {
            panic!("expected a run command");
        };
        assert!(!record);

        // Bare `--record` → record true, position-independent, other flags intact.
        let Ok(RawCommand::Run {
            record,
            experiment,
            out,
        }) = parse(&args(&[
            "run",
            "--record",
            "--experiment",
            "e",
            "--out",
            "o",
        ]))
        else {
            panic!("expected a run command");
        };
        assert!(record);
        assert_eq!(experiment, "e");
        assert_eq!(out, "o");

        // Duplicate `--record` → reject.
        let err = parse(&args(&[
            "run",
            "--record",
            "--record",
            "--experiment",
            "e",
            "--out",
            "o",
        ]))
        .unwrap_err();
        assert!(err.reason.contains("duplicate --record"), "{}", err.reason);

        // Value-bearing `--record=x` → rejected as taking no value, position
        // independent (whether it leads or sits where a value would).
        for argv in [
            args(&["run", "--record=x", "--experiment", "e", "--out", "o"]),
            args(&["run", "--experiment", "e", "--record=x", "--out", "o"]),
        ] {
            let err = parse(&argv).unwrap_err();
            assert!(
                err.reason.contains("--record takes no value"),
                "{argv:?} -> {}",
                err.reason
            );
        }
    }

    #[test]
    fn replay_validates_run_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("nope");
        let exit = run_cli(&args(&["replay", missing.to_str().unwrap()]));
        assert!(invalid_reason(&exit).contains("run directory not found"));

        // The body runs against the validated directory: an empty run dir
        // fails the replay_manifest.json strict read (the command's own
        // tests cover the live paths, rooted at the repository).
        let run_dir = tmp.path().join("r1");
        std::fs::create_dir_all(&run_dir).unwrap();
        let exit = run_cli(&args(&["replay", run_dir.to_str().unwrap()]));
        assert_eq!(exit.result.operation_id, static_id(OP_REPLAY));
        assert_eq!(exit.result.outcome, Outcome::Invalid);
        assert_eq!(exit.exit_code, 2);
        assert!(exit.command_output.is_none());
        let events = streamed(&exit);
        assert_eq!(events[0].run_id, static_id("r1"));
        let diagnostic = &events[0].diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::SchemaInvalid);
        assert_eq!(diagnostic.payload[0].1, "replay_manifest.json");

        let exit = run_cli(&args(&["replay"]));
        assert!(invalid_reason(&exit).contains("missing run directory"));
        let exit = run_cli(&args(&["replay", "a", "b"]));
        assert!(invalid_reason(&exit).contains("exactly one run directory"));
        let exit = run_cli(&args(&["replay", "--flag"]));
        assert!(invalid_reason(&exit).contains("exactly one run directory"));
    }

    #[test]
    fn trace_validates_run_dir_and_finding() {
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = tmp.path().join("r1");
        std::fs::create_dir_all(&run_dir).unwrap();
        let run_dir_str = run_dir.to_str().unwrap();

        // The body runs against the validated directory: an empty run dir
        // fails the trace_bundle.json strict read (the command's own tests
        // cover resolution; the ok path runs in the binary-level test).
        let exit = run_cli(&args(&[
            "trace",
            "--run",
            run_dir_str,
            "--finding",
            "finding.group.m1_conflict.1",
        ]));
        assert_eq!(exit.result.operation_id, static_id(OP_TRACE));
        assert_eq!(exit.result.outcome, Outcome::Invalid);
        assert_eq!(exit.exit_code, 2);
        assert!(exit.command_output.is_none());
        let events = streamed(&exit);
        assert_eq!(events[0].run_id, static_id("r1"));
        let diagnostic = &events[0].diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::SchemaInvalid);
        assert_eq!(diagnostic.payload[0].1, "trace_bundle.json");

        let exit = run_cli(&args(&["trace", "--run", run_dir_str, "--finding", "Bad!"]));
        assert!(invalid_reason(&exit).contains("--finding"));

        let exit = run_cli(&args(&["trace", "--finding", "f.1"]));
        assert!(invalid_reason(&exit).contains("missing required --run"));
    }

    #[test]
    fn registry_and_unknown_command_shapes() {
        let exit = run_cli(&args(&["registry"]));
        assert!(invalid_reason(&exit).contains("registry check"));
        assert_eq!(exit.result.operation_id, static_id(OP_REGISTRY_CHECK));

        let exit = run_cli(&args(&["registry", "check", "extra"]));
        assert!(invalid_reason(&exit).contains("unexpected arguments"));

        let exit = run_cli(&args(&["frobnicate"]));
        assert!(invalid_reason(&exit).contains("unknown command"));
        assert_eq!(exit.result.operation_id, static_id(OP_CLI));

        let exit = run_cli(&[]);
        assert!(invalid_reason(&exit).contains("missing command"));
        assert_eq!(exit.result.operation_id, static_id(OP_CLI));
        assert_one_result_line(&exit);
    }
}
