//! Command shell: the SPEC §3 CLI invariants wired once.
//!
//! Every `ckc` command runs inside one [`Shell`]. It owns the §3 boundary
//! invariants — every disk write goes through the containment-conditioned
//! [`Shell::write_under`]; §4.6 events and §7.4 diagnostics leave as
//! canonical JSONL (`logs/{events,diagnostics}.jsonl` under the output
//! directory per the §8.3 run layout, or an events stream for stderr when
//! the command has none); per-item outcomes fold by §4.4 severity; and
//! [`Shell::finish`] yields exactly one §4.4 [`TotalOperationResult`].
//! Dispatch (not the shell) parses and validates arguments.

use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ckc_core::{
    CanonError, DiagnosticRecord, EventRecord, Hash, Id, Outcome, TotalOperationResult,
    canonical_sort_key, content_hash, jsonl_line, write_jsonl,
};

/// Parse a compile-time-constant token as an [`Id`]; all call sites pass
/// literals from the M1 command surface, covered by tests.
pub(crate) fn static_id(token: &str) -> Id {
    token
        .parse()
        .expect("static id token matches the Id grammar")
}

/// §4.6 `run_id` sentinel for commands with no associated run directory
/// (run ids are runtime metadata, so the sentinel never reaches a hash).
pub(crate) fn run_none() -> Id {
    static_id("run.none")
}

/// Shell failure: the command body may still have succeeded, but the
/// invariant layer could not land its evidence or result.
#[derive(Debug)]
pub(crate) enum ShellError {
    /// Write target escapes the output directory, or no directory exists.
    Containment(String),
    Io(std::io::Error),
    Canon(CanonError),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::Containment(reason) => write!(f, "containment: {reason}"),
            ShellError::Io(e) => write!(f, "io: {e}"),
            ShellError::Canon(e) => write!(f, "canon: {e}"),
        }
    }
}

impl std::error::Error for ShellError {}

impl From<std::io::Error> for ShellError {
    fn from(e: std::io::Error) -> Self {
        ShellError::Io(e)
    }
}

impl From<CanonError> for ShellError {
    fn from(e: CanonError) -> Self {
        ShellError::Canon(e)
    }
}

/// A closed command: the one total result plus the evidence streams that
/// still need a channel (files already landed under the output directory).
pub(crate) struct FinishedCommand {
    pub result: TotalOperationResult,
    /// Canonical JSONL line of `result` for stdout.
    pub result_line: Vec<u8>,
    /// §4.6 events JSONL for stderr when the command has no output
    /// directory; `None` once `logs/events.jsonl` is on disk.
    pub streamed_events: Option<Vec<u8>>,
}

/// One command's invariant context. Open it after validation, record
/// outcomes, diagnostics, and processing_stage events while the command body runs,
/// then [`finish`].
///
/// [`finish`]: Shell::finish
pub(crate) struct Shell {
    operation_id: Id,
    run_id: Id,
    out_dir: Option<PathBuf>,
    started_at: String,
    started: Instant,
    outcome: Outcome,
    /// Command-scope diagnostics ([`Shell::diagnostic`]): they ride the
    /// closing command event. ProcessingStage-scope diagnostics ride their processing_stage
    /// event instead.
    diagnostics: Vec<DiagnosticRecord>,
    /// Every diagnostic raised during the command, append-ordered: the
    /// `logs/diagnostics.jsonl` stream and the result's
    /// `diagnostic_hashes` draw from here.
    ledger: Vec<DiagnosticRecord>,
    /// §4.6 processing_stage events in execution order ([`Shell::processing_stage_event`]);
    /// [`Shell::finish`] appends the command event after them.
    events: Vec<EventRecord>,
}

/// One §4.6 processing_stage execution, ready for the shell to number: the shell
/// assigns `event_id`/`event_sequence_number` (execution order), `run_id`, and
/// `log_level`; everything else is the processing_stage's to report.
pub(crate) struct ProcessingStageEvent {
    pub pipeline_id: Id,
    pub pipeline_step_id: Id,
    pub processing_stage: Id,
    pub started_at: String,
    pub ended_at: String,
    pub duration_ms: u64,
    pub input_hashes: Vec<Hash>,
    pub output_hashes: Vec<Hash>,
    pub outcome: Outcome,
    /// Diagnostics the processing_stage raised (§4.3 set: pass canonical order —
    /// wrapper fields already hold it). They extend the ledger here.
    pub diagnostics: Vec<DiagnosticRecord>,
    pub resource_counters: Vec<(Id, u64)>,
}

/// Wall-clock + monotonic capture opened at a processing_stage's start, closed into
/// the §4.6 event bounds.
pub(crate) struct ProcessingStageClock {
    started_at: String,
    started: Instant,
}

pub(crate) fn processing_stage_clock() -> ProcessingStageClock {
    ProcessingStageClock {
        started_at: rfc3339_utc(SystemTime::now()),
        started: Instant::now(),
    }
}

impl ProcessingStageClock {
    /// `(started_at, ended_at, duration_ms)` at processing_stage end.
    pub(crate) fn stop(self) -> (String, String, u64) {
        let ended_at = rfc3339_utc(SystemTime::now());
        let duration_ms = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        (self.started_at, ended_at, duration_ms)
    }
}

impl Shell {
    /// Open the shell. `out_dir`, when present, is the only writable root
    /// for the whole command; `run_id` is the run-directory name or
    /// [`run_none`].
    pub(crate) fn open(operation_id: Id, run_id: Id, out_dir: Option<PathBuf>) -> Shell {
        Shell {
            operation_id,
            run_id,
            out_dir,
            started_at: rfc3339_utc(SystemTime::now()),
            started: Instant::now(),
            outcome: Outcome::Ok,
            diagnostics: Vec::new(),
            ledger: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Severity-fold a per-item outcome into the total (§4.4 `max`
    /// aggregation; the enum order is the severity order).
    pub(crate) fn merge(&mut self, outcome: Outcome) {
        self.outcome = self.outcome.max(outcome);
    }

    /// The run id this shell stamps on every event — run-scoped processing_stage
    /// diagnostics cite it as their subject.
    pub(crate) fn run_id(&self) -> &Id {
        &self.run_id
    }

    /// The output directory as the dispatcher received it — the report
    /// processing_stage's `--out` token when reconstructing the §4.6 replay command.
    pub(crate) fn out_dir(&self) -> Option<&Path> {
        self.out_dir.as_deref()
    }

    /// Every §7.4 record raised so far, append-ordered — the report
    /// processing_stage's diagnostics-summary source.
    pub(crate) fn ledger(&self) -> &[DiagnosticRecord] {
        &self.ledger
    }

    /// Record a command-scope §7.4 diagnostic: its outcome folds into the
    /// total and the record rides the command event,
    /// `logs/diagnostics.jsonl`, and `diagnostic_hashes`.
    pub(crate) fn diagnostic(&mut self, diagnostic: DiagnosticRecord) {
        self.merge(diagnostic.outcome);
        self.ledger.push(diagnostic.clone());
        self.diagnostics.push(diagnostic);
    }

    /// Record one §4.6 processing_stage event: the outcome folds into the total, the
    /// processing_stage's diagnostics extend the ledger, and the event takes the next
    /// `event_id`/`event_sequence_number` slot ahead of the closing command event.
    pub(crate) fn processing_stage_event(&mut self, processing_stage: ProcessingStageEvent) {
        self.merge(processing_stage.outcome);
        self.ledger
            .extend(processing_stage.diagnostics.iter().cloned());
        let slot = self.events.len();
        self.events.push(EventRecord {
            event_id: event_id(slot),
            run_id: self.run_id.clone(),
            pipeline_id: processing_stage.pipeline_id,
            pipeline_step_id: processing_stage.pipeline_step_id,
            processing_stage: processing_stage.processing_stage,
            log_level: log_level_for(processing_stage.outcome),
            event_sequence_number: slot as u64,
            started_at: processing_stage.started_at,
            ended_at: processing_stage.ended_at,
            duration_ms: processing_stage.duration_ms,
            input_hashes: processing_stage.input_hashes,
            output_hashes: processing_stage.output_hashes,
            outcome: processing_stage.outcome,
            diagnostics: processing_stage.diagnostics,
            resource_counters: processing_stage.resource_counters,
        });
    }

    /// The single write primitive: every byte a command persists goes
    /// through here. `rel` must resolve inside the output directory — the
    /// guard is lexical (relative, normal components only) over a run
    /// directory the dispatcher created fresh. Parent directories are
    /// created on demand.
    pub(crate) fn write_under(&self, rel: &str, bytes: &[u8]) -> Result<PathBuf, ShellError> {
        let Some(root) = &self.out_dir else {
            return Err(ShellError::Containment(format!(
                "command has no output directory; refused write of {rel:?}"
            )));
        };
        let contained = !rel.is_empty()
            && Path::new(rel)
                .components()
                .all(|c| matches!(c, Component::Normal(_)));
        if !contained {
            return Err(ShellError::Containment(format!(
                "write target {rel:?} escapes the output directory"
            )));
        }
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, bytes)?;
        Ok(path)
    }

    /// Close the command: build the §4.6 command event (the last event,
    /// numbered after the processing_stage events; command-scope diagnostics ride it),
    /// land both JSONL streams, and return exactly one §4.4 total operation
    /// result.
    pub(crate) fn finish(self) -> Result<FinishedCommand, ShellError> {
        let ended_at = rfc3339_utc(SystemTime::now());
        let duration_ms = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);

        // diagnostic_hashes (over the whole ledger) and the command event's
        // diagnostics (command scope) are §4.3 sets: store them in canonical
        // sort order with byte-identical duplicates collapsed, so the
        // structs round-trip equal through strict reads.
        // logs/diagnostics.jsonl keeps append order (streams are evidence).
        let mut keyed_hashes: Vec<(Vec<u8>, Hash)> = Vec::with_capacity(self.ledger.len());
        for diagnostic in &self.ledger {
            let hash = content_hash(diagnostic)?;
            keyed_hashes.push((canonical_sort_key(&hash)?, hash));
        }
        keyed_hashes.sort_by(|a, b| a.0.cmp(&b.0));
        keyed_hashes.dedup_by(|a, b| a.0 == b.0);
        let mut keyed_diags: Vec<(Vec<u8>, DiagnosticRecord)> =
            Vec::with_capacity(self.diagnostics.len());
        for diagnostic in &self.diagnostics {
            keyed_diags.push((canonical_sort_key(diagnostic)?, diagnostic.clone()));
        }
        keyed_diags.sort_by(|a, b| a.0.cmp(&b.0));
        keyed_diags.dedup_by(|a, b| a.0 == b.0);

        let slot = self.events.len();
        let event = EventRecord {
            event_id: event_id(slot),
            run_id: self.run_id.clone(),
            pipeline_id: static_id("cli"),
            pipeline_step_id: format!("cli.{}", self.operation_id)
                .parse()
                .expect("cli.<operation-id> matches the Id grammar"),
            processing_stage: self.operation_id.clone(),
            log_level: log_level_for(self.outcome),
            event_sequence_number: slot as u64,
            started_at: self.started_at.clone(),
            ended_at,
            duration_ms,
            input_hashes: Vec::new(),
            output_hashes: Vec::new(),
            outcome: self.outcome,
            diagnostics: keyed_diags.into_iter().map(|(_, d)| d).collect(),
            resource_counters: Vec::new(),
        };
        let events_bytes = write_jsonl(self.events.iter().chain([&event]))?;

        let streamed_events = if self.out_dir.is_some() {
            let diagnostics_bytes = write_jsonl(self.ledger.iter())?;
            self.write_under("logs/events.jsonl", &events_bytes)?;
            self.write_under("logs/diagnostics.jsonl", &diagnostics_bytes)?;
            None
        } else {
            Some(events_bytes)
        };

        let result = TotalOperationResult {
            operation_id: self.operation_id,
            outcome: self.outcome,
            value_hashes: Vec::new(),
            diagnostic_hashes: keyed_hashes.into_iter().map(|(_, h)| h).collect(),
            residual_hashes: Vec::new(),
            ambiguity_hashes: Vec::new(),
            incoherence_hashes: Vec::new(),
        };
        let result_line = jsonl_line(&result)?;
        Ok(FinishedCommand {
            result,
            result_line,
            streamed_events,
        })
    }
}

/// §4.6 event id for execution slot `n`: `event.<n>`, matching
/// `event_sequence_number` so id order is event order.
fn event_id(slot: usize) -> Id {
    format!("event.{slot}")
        .parse()
        .expect("event.<decimal> matches the Id grammar")
}

/// §4.6 `log_level` token from the total outcome's severity band.
fn log_level_for(outcome: Outcome) -> Id {
    let token = match outcome {
        Outcome::Ok => "info",
        Outcome::Residual | Outcome::Ambiguity => "warn",
        Outcome::Unsupported | Outcome::Incoherence | Outcome::Invalid => "error",
    };
    static_id(token)
}

/// RFC 3339 UTC wall-clock text for §4.6 event bounds (runtime evidence,
/// excluded from content hashes). Whole-second precision; civil date by the
/// standard days-from-epoch conversion (Hinnant `civil_from_days`).
fn rfc3339_utc(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    let days = i64::try_from(secs / 86_400).expect("u64 seconds / 86400 fits i64");
    let (hh, mm, ss) = (secs % 86_400 / 3_600, secs % 3_600 / 60, secs % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{DiagnosticCode, read_jsonl};

    fn diag(code: DiagnosticCode, outcome: Outcome, text: &str) -> DiagnosticRecord {
        DiagnosticRecord {
            code,
            outcome,
            payload: vec![(static_id("reason"), text.to_owned())],
            region_ids: Vec::new(),
            artifact_hashes: Vec::new(),
        }
    }

    #[test]
    fn rfc3339_epoch_and_leap_day() {
        assert_eq!(rfc3339_utc(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        let leap = UNIX_EPOCH + Duration::from_secs(951_827_696);
        assert_eq!(rfc3339_utc(leap), "2000-02-29T12:34:56Z");
    }

    #[test]
    fn severity_folds_to_max() {
        let mut shell = Shell::open(static_id("run"), run_none(), None);
        shell.merge(Outcome::Residual);
        shell.diagnostic(diag(
            DiagnosticCode::SchemaInvalid,
            Outcome::Invalid,
            "boom",
        ));
        shell.merge(Outcome::Ambiguity);
        assert_eq!(shell.outcome, Outcome::Invalid);
    }

    #[test]
    fn write_under_guards_containment() {
        let root = tempfile::tempdir().unwrap();
        let shell = Shell::open(
            static_id("run"),
            static_id("m1"),
            Some(root.path().to_path_buf()),
        );
        for escape in ["../sibling", "/etc/passwd", "logs/../../x", ""] {
            let err = shell.write_under(escape, b"x").unwrap_err();
            assert!(matches!(err, ShellError::Containment(_)), "{escape:?}");
        }
        let landed = shell.write_under("logs/nested/file.txt", b"ok").unwrap();
        assert_eq!(std::fs::read(landed).unwrap(), b"ok");

        let homeless = Shell::open(static_id("trace"), run_none(), None);
        let err = homeless.write_under("logs/events.jsonl", b"x").unwrap_err();
        assert!(matches!(err, ShellError::Containment(_)));
    }

    #[test]
    fn finish_with_out_dir_lands_streams_and_result() {
        let root = tempfile::tempdir().unwrap();
        let mut shell = Shell::open(
            static_id("run"),
            static_id("m1"),
            Some(root.path().to_path_buf()),
        );
        let d = diag(DiagnosticCode::SchemaInvalid, Outcome::Invalid, "bad");
        shell.diagnostic(d.clone());
        shell.diagnostic(d.clone()); // byte-identical: collapses in set fields
        let finished = shell.finish().unwrap();

        assert!(finished.streamed_events.is_none());
        let events_bytes = std::fs::read(root.path().join("logs/events.jsonl")).unwrap();
        let events: Vec<EventRecord> = read_jsonl(&events_bytes).unwrap();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.event_id, static_id("event.0"));
        assert_eq!(event.run_id, static_id("m1"));
        assert_eq!(event.pipeline_id, static_id("cli"));
        assert_eq!(event.pipeline_step_id, static_id("cli.run"));
        assert_eq!(event.processing_stage, static_id("run"));
        assert_eq!(event.log_level, static_id("error"));
        assert_eq!(event.event_sequence_number, 0);
        assert_eq!(event.outcome, Outcome::Invalid);
        assert_eq!(event.diagnostics, vec![d.clone()]);

        // The diagnostics stream keeps append order (both records).
        let diag_bytes = std::fs::read(root.path().join("logs/diagnostics.jsonl")).unwrap();
        let stream: Vec<DiagnosticRecord> = read_jsonl(&diag_bytes).unwrap();
        assert_eq!(stream, vec![d.clone(), d.clone()]);

        assert_eq!(finished.result.operation_id, static_id("run"));
        assert_eq!(finished.result.outcome, Outcome::Invalid);
        assert_eq!(
            finished.result.diagnostic_hashes,
            vec![content_hash(&d).unwrap()]
        );
        let parsed: Vec<TotalOperationResult> = read_jsonl(&finished.result_line).unwrap();
        assert_eq!(parsed, vec![finished.result]);
    }

    #[test]
    fn finish_without_out_dir_streams_events() {
        let shell = Shell::open(static_id("registry.check"), run_none(), None);
        let finished = shell.finish().unwrap();
        let events: Vec<EventRecord> =
            read_jsonl(finished.streamed_events.as_deref().unwrap()).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].run_id, run_none());
        assert_eq!(events[0].pipeline_step_id, static_id("cli.registry.check"));
        assert_eq!(events[0].log_level, static_id("info"));
        assert_eq!(events[0].outcome, Outcome::Ok);
        assert_eq!(finished.result.outcome, Outcome::Ok);
        assert!(finished.result.diagnostic_hashes.is_empty());
    }

    // ProcessingStage events take slots 0..n in execution order; the command event
    // closes the file at slot n carrying only command-scope diagnostics,
    // while the result's hashes and the diagnostics stream cover the whole
    // ledger (processing_stage diagnostics included).
    #[test]
    fn processing_stage_events_precede_the_command_event() {
        let root = tempfile::tempdir().unwrap();
        let mut shell = Shell::open(
            static_id("run"),
            static_id("m1"),
            Some(root.path().to_path_buf()),
        );
        let processing_stage_diag = diag(DiagnosticCode::SolverTimeout, Outcome::Residual, "slow");
        let command_diag = diag(DiagnosticCode::SchemaInvalid, Outcome::Invalid, "late");
        let (started_at, ended_at, duration_ms) = processing_stage_clock().stop();
        shell.processing_stage_event(ProcessingStageEvent {
            pipeline_id: static_id("pipe.p"),
            pipeline_step_id: static_id("processing_stage.s"),
            processing_stage: static_id("extract"),
            started_at,
            ended_at,
            duration_ms,
            input_hashes: Vec::new(),
            output_hashes: Vec::new(),
            outcome: Outcome::Residual,
            diagnostics: vec![processing_stage_diag.clone()],
            resource_counters: Vec::new(),
        });
        shell.diagnostic(command_diag.clone());
        let finished = shell.finish().unwrap();

        let events: Vec<EventRecord> =
            read_jsonl(&std::fs::read(root.path().join("logs/events.jsonl")).unwrap()).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_id, static_id("event.0"));
        assert_eq!(events[0].event_sequence_number, 0);
        assert_eq!(events[0].pipeline_id, static_id("pipe.p"));
        assert_eq!(events[0].pipeline_step_id, static_id("processing_stage.s"));
        assert_eq!(events[0].processing_stage, static_id("extract"));
        assert_eq!(events[0].log_level, static_id("warn"));
        assert_eq!(events[0].outcome, Outcome::Residual);
        assert_eq!(events[0].diagnostics, vec![processing_stage_diag.clone()]);
        assert_eq!(events[1].event_id, static_id("event.1"));
        assert_eq!(events[1].event_sequence_number, 1);
        assert_eq!(events[1].processing_stage, static_id("run"));
        assert_eq!(events[1].outcome, Outcome::Invalid);
        assert_eq!(events[1].diagnostics, vec![command_diag.clone()]);

        let stream: Vec<DiagnosticRecord> =
            read_jsonl(&std::fs::read(root.path().join("logs/diagnostics.jsonl")).unwrap())
                .unwrap();
        assert_eq!(
            stream,
            vec![processing_stage_diag.clone(), command_diag.clone()]
        );

        assert_eq!(finished.result.outcome, Outcome::Invalid);
        let mut expected = vec![
            content_hash(&processing_stage_diag).unwrap(),
            content_hash(&command_diag).unwrap(),
        ];
        expected.sort_by_key(|h| canonical_sort_key(h).unwrap());
        assert_eq!(finished.result.diagnostic_hashes, expected);
    }

    #[test]
    fn diagnostic_hashes_sort_canonically() {
        let mut shell = Shell::open(static_id("run"), run_none(), None);
        let a = diag(DiagnosticCode::SchemaInvalid, Outcome::Invalid, "alpha");
        let b = diag(DiagnosticCode::SolverTimeout, Outcome::Residual, "beta");
        shell.diagnostic(a.clone());
        shell.diagnostic(b.clone());
        let finished = shell.finish().unwrap();
        let mut expected = vec![content_hash(&a).unwrap(), content_hash(&b).unwrap()];
        expected.sort_by_key(|h| canonical_sort_key(h).unwrap());
        assert_eq!(finished.result.diagnostic_hashes, expected);
    }
}
