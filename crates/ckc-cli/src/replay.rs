//! SPEC §4.6 replay core (cli-runner.4.2a): re-execute a recorded run from
//! its `replay_manifest.json` over the same inputs and compare canonical
//! content hashes for all accepted artifacts. Runtime metadata — run ids,
//! timestamps, the JSONL logs — never enters the comparison: content
//! hashes cover canonical payload bytes only, so the exclusion holds by
//! construction. No shell contact: [`execute`] takes paths and returns
//! values; the `ckc replay` command (cli-runner.4.2b) owns the CLI
//! surface. The re-execution itself drives the full §8.3 pipeline through
//! [`crate::run::execute`] under a replay-owned internal shell, so the
//! scratch directory ends up a complete run layout, inspectable after any
//! verdict.
//!
//! Failure currency is the §7.4 record. An unreadable or ill-shaped
//! manifest and a stale scratch directory are `schema_invalid`; a missing
//! external solver is `replay_identity_unsupported`, probed before
//! re-execution so the §4.6 code wins over the pipeline's own
//! `solver_execution_failure`; a re-execution that lands no readable §5
//! manifest, or one whose accepted-artifact hash set diverges from the
//! §4.6 expectation, is `replay_mismatch` — the record carries the
//! symmetric difference, every diverging hash an artifact ref. Matching
//! hashes are the §4.6 standing idempotency property: re-run equals prior.

use std::path::Path;

use ckc_core::{
    CanonRead, DiagnosticCode, DiagnosticRecord, Hash, Id, Outcome, ReplayManifest, RunManifest,
    read_canonical,
};
use ckc_smt::{AdapterError, Z3Adapter};

use crate::registry_check::invalid_diagnostic;
use crate::shell::{Shell, static_id};

/// §8.3 attestation record replay consumes from the prior run directory.
const REPLAY_MANIFEST: &str = "replay_manifest.json";

/// §8.3 record the re-execution's accepted-artifact set is read from (§5:
/// `output_hashes`, mirrored verbatim into the §4.6 expectation at
/// assembly, so this comparison covers both manifests).
const RUN_MANIFEST: &str = "manifest.json";

/// One replay comparison: the prior run's §4.6 expectation against the
/// re-execution's §5 attestation, plus their symmetric difference. All
/// four vecs are §4.3 canonical sets (sorted, deduped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCheck {
    /// Prior run's `expected_output_hashes` (§4.6).
    pub expected: Vec<Hash>,
    /// Re-execution's `output_hashes` (§5).
    pub actual: Vec<Hash>,
    /// `expected \ actual`: recorded artifacts the re-execution did not
    /// reproduce.
    pub missing: Vec<Hash>,
    /// `actual \ expected`: re-execution artifacts the record never
    /// attested.
    pub unexpected: Vec<Hash>,
    /// The re-execution's §4.4 total outcome — runtime evidence beside
    /// the hash verdict, never part of it.
    pub rerun_outcome: Outcome,
}

impl ReplayCheck {
    /// §4.6 verdict: every accepted artifact's content hash matches.
    pub fn matched(&self) -> bool {
        self.missing.is_empty() && self.unexpected.is_empty()
    }

    /// §7.4 `replay_mismatch` carrying the symmetric difference: each
    /// nonempty direction one payload entry of space-joined hash text,
    /// every diverging hash an artifact ref. `None` when hashes match.
    pub fn mismatch_diagnostic(&self) -> Option<DiagnosticRecord> {
        if self.matched() {
            return None;
        }
        let mut payload = Vec::with_capacity(3);
        if !self.missing.is_empty() {
            payload.push((static_id("missing"), join(&self.missing)));
        }
        payload.push((
            static_id("rerun_outcome"),
            self.rerun_outcome.as_str().to_owned(),
        ));
        if !self.unexpected.is_empty() {
            payload.push((static_id("unexpected"), join(&self.unexpected)));
        }
        let mut artifact_hashes: Vec<Hash> = self
            .missing
            .iter()
            .chain(&self.unexpected)
            .cloned()
            .collect();
        artifact_hashes.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        Some(DiagnosticRecord {
            code: DiagnosticCode::ReplayMismatch,
            outcome: Outcome::Invalid,
            payload,
            region_ids: vec![],
            artifact_hashes,
        })
    }
}

/// Replay the run recorded at `run_dir`: strict-read its §4.6 manifest,
/// re-execute the recorded command's experiment from `root` (the §3
/// invocation root carrying `registry/` and `corpus/` — the same inputs)
/// into `scratch`, and compare accepted-artifact content hashes. `run_dir`
/// is read-only; `scratch` must be absent or empty and ends up a full §8.3
/// run layout. `Ok` is a reached comparison — including a mismatching one
/// ([`ReplayCheck::mismatch_diagnostic`]); `Err` is the §7.4 record for a
/// replay that never reached it.
pub fn execute(
    root: &Path,
    run_dir: &Path,
    scratch: &Path,
) -> Result<ReplayCheck, DiagnosticRecord> {
    let manifest: ReplayManifest =
        read_record(&run_dir.join(REPLAY_MANIFEST)).map_err(|reason| {
            invalid_diagnostic(vec![
                (static_id("file"), REPLAY_MANIFEST.to_owned()),
                (static_id("reason"), reason),
            ])
        })?;
    let experiment = experiment_from_command(&manifest.command)?;
    // §4.6: a missing external tool is identity-unsupported, decided
    // before the pipeline could fail on it mid-run.
    Z3Adapter::new().map_err(|e| identity_unsupported(&e))?;
    prepare_scratch(scratch)?;

    // The run id is runtime metadata (§4.6): a fixed token never reaches
    // any hash, and the §5 record carries no run id at all.
    let mut shell = Shell::open(
        static_id("run"),
        static_id("replay"),
        Some(scratch.to_path_buf()),
    );
    crate::run::execute(root, &experiment, &mut shell);
    let rerun_outcome = match shell.finish() {
        Ok(finished) => finished.result.outcome,
        Err(e) => {
            return Err(invalid_diagnostic(vec![(
                static_id("reason"),
                format!("re-execution shell failed to close: {e}"),
            )]));
        }
    };

    let rerun: RunManifest =
        read_record(&scratch.join(RUN_MANIFEST)).map_err(|reason| DiagnosticRecord {
            code: DiagnosticCode::ReplayMismatch,
            outcome: Outcome::Invalid,
            payload: vec![
                (
                    static_id("reason"),
                    format!("re-execution landed no {RUN_MANIFEST}: {reason}"),
                ),
                (
                    static_id("rerun_outcome"),
                    rerun_outcome.as_str().to_owned(),
                ),
            ],
            region_ids: vec![],
            artifact_hashes: vec![],
        })?;

    let expected = manifest.expected_output_hashes;
    let actual = rerun.output_hashes;
    let missing = difference(&expected, &actual);
    let unexpected = difference(&actual, &expected);
    Ok(ReplayCheck {
        expected,
        actual,
        missing,
        unexpected,
        rerun_outcome,
    })
}

/// §4.6: missing external tools emit `replay_identity_unsupported` — the
/// recorded solver identity cannot be re-established, so no comparison is
/// attempted.
fn identity_unsupported(error: &AdapterError) -> DiagnosticRecord {
    DiagnosticRecord {
        code: DiagnosticCode::ReplayIdentityUnsupported,
        outcome: Outcome::Unsupported,
        payload: vec![(static_id("reason"), format!("solver adapter: {error}"))],
        region_ids: vec![],
        artifact_hashes: vec![],
    }
}

/// The §4.6 recorded argv names the run to re-execute. M1 records exactly
/// one shape — `ckc run --experiment <id> --out <dir>` — and replay
/// re-points `--out` at the scratch directory, so only the experiment
/// token is consumed.
fn experiment_from_command(command: &[String]) -> Result<Id, DiagnosticRecord> {
    let fail = |reason: String| {
        invalid_diagnostic(vec![
            (static_id("command"), command.join(" ")),
            (static_id("reason"), reason),
        ])
    };
    match command {
        [ckc, run, exp_flag, experiment, out_flag, _out]
            if ckc == "ckc"
                && run == "run"
                && exp_flag == "--experiment"
                && out_flag == "--out" =>
        {
            experiment
                .parse()
                .map_err(|e| fail(format!("experiment {experiment:?}: {e}")))
        }
        _ => Err(fail(
            "command is not the recorded `ckc run --experiment <id> --out <dir>` shape".to_owned(),
        )),
    }
}

/// The re-execution writes a full run layout: `scratch` is created on
/// demand and must start empty, so a stale prior attempt can never pose as
/// this replay's evidence.
fn prepare_scratch(scratch: &Path) -> Result<(), DiagnosticRecord> {
    let fail = |reason: String| {
        invalid_diagnostic(vec![
            (static_id("reason"), reason),
            (static_id("scratch"), scratch.display().to_string()),
        ])
    };
    std::fs::create_dir_all(scratch).map_err(|e| fail(format!("create scratch directory: {e}")))?;
    let mut entries =
        std::fs::read_dir(scratch).map_err(|e| fail(format!("read scratch directory: {e}")))?;
    if entries.next().is_some() {
        return Err(fail("scratch directory is not empty".to_owned()));
    }
    Ok(())
}

/// Strict-read one bare canonical record (the manifests attest envelopes;
/// nothing envelopes them), failures as reason text for the caller's §7.4
/// record.
fn read_record<T: CanonRead>(path: &Path) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read: {e}"))?;
    read_canonical(&bytes).map_err(|e| format!("strict read: {e}"))
}

/// `left \ right` over §4.3 canonical-set vecs; the §4.6 symmetric
/// difference is the two directed calls. Output keeps `left`'s sorted
/// order.
fn difference(left: &[Hash], right: &[Hash]) -> Vec<Hash> {
    left.iter()
        .filter(|h| !right.contains(h))
        .cloned()
        .collect()
}

/// Space-joined hash text for a §7.4 payload value.
fn join(hashes: &[Hash]) -> String {
    hashes
        .iter()
        .map(Hash::as_str)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use ckc_core::{SolverIdentity, canonical_payload_bytes};

    /// Repository root: two levels above the ckc-cli manifest, where the
    /// §3 `registry/` and `corpus/` trees live.
    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("crates/ckc-cli sits two levels under the repo root")
            .to_path_buf()
    }

    /// Execute `exp.m1_spine` into `<tmp>/m1` — the prior run every live
    /// replay test re-executes — and require it clean.
    fn fixture_run(root: &Path, tmp: &Path) -> PathBuf {
        let out = tmp.join("m1");
        std::fs::create_dir_all(&out).unwrap();
        let mut shell = Shell::open(static_id("run"), static_id("m1"), Some(out.clone()));
        crate::run::execute(root, &static_id("exp.m1_spine"), &mut shell);
        let finished = shell.finish().unwrap();
        assert_eq!(finished.result.outcome, Outcome::Ok);
        out
    }

    fn hash(seed: char) -> Hash {
        Hash::new(format!("sha256:{}", seed.to_string().repeat(64))).unwrap()
    }

    /// A canonical-storage §4.6 record around an arbitrary `command`, for
    /// the pre-execution failure paths.
    fn synthetic_manifest(command: &[&str]) -> ReplayManifest {
        ReplayManifest {
            command: command.iter().map(|t| (*t).to_owned()).collect(),
            input_hashes: vec![hash('1')],
            lexicon_hash: hash('2'),
            corpus_hash: hash('3'),
            toolchain_manifest_hash: hash('4'),
            environment_profile: vec![(static_id("os"), "linux".to_owned())],
            lockfile_hashes: vec![(static_id("cargo.lock"), hash('5'))],
            solver_identity: SolverIdentity {
                solver_id: static_id("z3"),
                version: "4".to_owned(),
            },
            expected_output_hashes: vec![hash('6')],
        }
    }

    // §4.6 re-run-equals-prior over a live fixture run — the §8.5 item 8
    // property, asserted at the core: matching hash sets, and the §5
    // record reproduced byte-equal (runtime metadata differs across the
    // two runs; nothing hashed does).
    #[test]
    fn replay_of_a_fixture_run_matches_all_accepted_hashes() {
        let root = repo_root();
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = fixture_run(&root, tmp.path());
        let scratch = tmp.path().join("scratch");

        let check = execute(&root, &run_dir, &scratch).unwrap();
        assert!(check.matched());
        assert_eq!(check.missing, Vec::new());
        assert_eq!(check.unexpected, Vec::new());
        assert_eq!(check.rerun_outcome, Outcome::Ok);
        assert_eq!(check.mismatch_diagnostic(), None);

        // The comparison consumed exactly the two §8.3 records.
        let prior: ReplayManifest = read_record(&run_dir.join(REPLAY_MANIFEST)).unwrap();
        assert_eq!(check.expected, prior.expected_output_hashes);
        assert_eq!(check.actual, check.expected);

        // The re-execution attests the §5 record byte-for-byte, and the
        // §4.6 record diverges only in the `--out` argv token.
        let prior_run: RunManifest = read_record(&run_dir.join(RUN_MANIFEST)).unwrap();
        let rerun_run: RunManifest = read_record(&scratch.join(RUN_MANIFEST)).unwrap();
        assert_eq!(rerun_run, prior_run);
        let rerun: ReplayManifest = read_record(&scratch.join(REPLAY_MANIFEST)).unwrap();
        assert_eq!(rerun.command[..5], prior.command[..5]);
        assert_eq!(rerun.command[5], scratch.display().to_string());
        let stripped = |m: &ReplayManifest| {
            let mut m = m.clone();
            m.command = vec![];
            m
        };
        assert_eq!(stripped(&rerun), stripped(&prior));
    }

    // A doctored expectation surfaces as the §7.4 symmetric difference:
    // the fake hash is missing from the rerun, the displaced real hash is
    // unexpected, and both ride the record as artifact refs.
    #[test]
    fn doctored_expectation_yields_symmetric_difference() {
        let root = repo_root();
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = fixture_run(&root, tmp.path());

        let path = run_dir.join(REPLAY_MANIFEST);
        let mut manifest: ReplayManifest = read_record(&path).unwrap();
        let displaced = manifest.expected_output_hashes.pop().unwrap();
        let fake = hash('f');
        manifest.expected_output_hashes.push(fake.clone());
        manifest
            .expected_output_hashes
            .sort_by(|a, b| a.as_str().cmp(b.as_str()));
        std::fs::write(&path, canonical_payload_bytes(&manifest).unwrap()).unwrap();

        let check = execute(&root, &run_dir, &tmp.path().join("scratch")).unwrap();
        assert!(!check.matched());
        assert_eq!(check.missing, vec![fake.clone()]);
        assert_eq!(check.unexpected, vec![displaced.clone()]);
        assert_eq!(check.rerun_outcome, Outcome::Ok);

        let diagnostic = check.mismatch_diagnostic().unwrap();
        assert_eq!(diagnostic.code, DiagnosticCode::ReplayMismatch);
        assert_eq!(diagnostic.outcome, Outcome::Invalid);
        assert_eq!(
            diagnostic.payload,
            vec![
                (static_id("missing"), fake.as_str().to_owned()),
                (static_id("rerun_outcome"), "ok".to_owned()),
                (static_id("unexpected"), displaced.as_str().to_owned()),
            ]
        );
        let mut refs = vec![fake, displaced];
        refs.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        assert_eq!(diagnostic.artifact_hashes, refs);
        assert!(diagnostic.region_ids.is_empty());
    }

    // Pre-comparison failures are §7.4 records, never comparisons.
    #[test]
    fn missing_or_corrupt_manifest_is_schema_invalid() {
        let root = repo_root();
        let tmp = tempfile::tempdir().unwrap();

        let diagnostic = execute(&root, tmp.path(), &tmp.path().join("s1")).unwrap_err();
        assert_eq!(diagnostic.code, DiagnosticCode::SchemaInvalid);
        assert_eq!(diagnostic.outcome, Outcome::Invalid);
        assert_eq!(
            diagnostic.payload[0],
            (static_id("file"), REPLAY_MANIFEST.to_owned())
        );

        std::fs::write(tmp.path().join(REPLAY_MANIFEST), b"{not canonical}").unwrap();
        let diagnostic = execute(&root, tmp.path(), &tmp.path().join("s2")).unwrap_err();
        assert_eq!(diagnostic.code, DiagnosticCode::SchemaInvalid);
        assert!(diagnostic.payload[1].1.contains("strict read"));
    }

    #[test]
    fn unrecorded_command_shape_is_schema_invalid() {
        let root = repo_root();
        let tmp = tempfile::tempdir().unwrap();
        for command in [
            &["ckc", "trace", "--experiment", "exp.m1_spine", "--out", "x"][..],
            &["ckc", "run", "--experiment", "exp m1", "--out", "x"][..],
            &["ckc", "run", "--experiment", "exp.m1_spine"][..],
        ] {
            let manifest = synthetic_manifest(command);
            std::fs::write(
                tmp.path().join(REPLAY_MANIFEST),
                canonical_payload_bytes(&manifest).unwrap(),
            )
            .unwrap();
            let diagnostic = execute(&root, tmp.path(), &tmp.path().join("s")).unwrap_err();
            assert_eq!(
                diagnostic.code,
                DiagnosticCode::SchemaInvalid,
                "{command:?}"
            );
            assert_eq!(diagnostic.payload[0].0, static_id("command"), "{command:?}");
        }
    }

    // The empty-scratch guard fires before any re-execution: a stale
    // prior attempt can never pose as this replay's evidence.
    #[test]
    fn stale_scratch_is_rejected() {
        let root = repo_root();
        let tmp = tempfile::tempdir().unwrap();
        let manifest =
            synthetic_manifest(&["ckc", "run", "--experiment", "exp.m1_spine", "--out", "x"]);
        std::fs::write(
            tmp.path().join(REPLAY_MANIFEST),
            canonical_payload_bytes(&manifest).unwrap(),
        )
        .unwrap();
        let scratch = tmp.path().join("scratch");
        std::fs::create_dir_all(&scratch).unwrap();
        std::fs::write(scratch.join("stale.json"), b"x").unwrap();

        let diagnostic = execute(&root, tmp.path(), &scratch).unwrap_err();
        assert_eq!(diagnostic.code, DiagnosticCode::SchemaInvalid);
        assert_eq!(
            diagnostic.payload[0],
            (
                static_id("reason"),
                "scratch directory is not empty".to_owned()
            )
        );
    }

    // The §4.6 missing-tool code, pinned through a real adapter error
    // from an absent binary; [`execute`] wires the same mapping ahead of
    // the re-execution.
    #[test]
    fn missing_solver_maps_to_replay_identity_unsupported() {
        let error = Z3Adapter::with_program("/nonexistent/ckc-replay-probe-z3").unwrap_err();
        let diagnostic = identity_unsupported(&error);
        assert_eq!(diagnostic.code, DiagnosticCode::ReplayIdentityUnsupported);
        assert_eq!(diagnostic.outcome, Outcome::Unsupported);
        assert_eq!(diagnostic.payload[0].0, static_id("reason"));
        assert!(diagnostic.payload[0].1.contains("spawn"));
        assert!(diagnostic.region_ids.is_empty() && diagnostic.artifact_hashes.is_empty());
    }
}
