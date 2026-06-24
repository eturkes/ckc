//! `ckc replay` (SPEC §8.5 item 8): the CLI surface over the §4.6 replay
//! core. The prior run at `<run-dir>` re-executes into the sibling scratch
//! layout `<run-dir>.replay` (created by the core, required empty — a
//! stale layout is rejected, never overwritten), and a reached match
//! renders as the command's stdout body: the run id, the rerun outcome
//! and layout, and every accepted artifact's content hash (§4.3 canonical
//! set order — §8.5 item 8's "matching canonical content hashes for all
//! accepted artifacts"). Every failure — the core's pre-comparison §7.4
//! records and the mismatch's symmetric-difference record alike — lands
//! as one command-scope diagnostic and withholds the body (dispatch:
//! stdout bodies print only beside an ok result).

use std::path::{Path, PathBuf};

use ckc_core::Id;

use super::ReplayCheck;
use crate::shell::Shell;

/// `ckc replay <run-dir>`: replay the recorded run from `root` (the §3
/// invocation root carrying `registry/` and `corpus/`) and return the
/// rendered match report for stdout — `None` after recording the failure
/// diagnostic.
pub(crate) fn execute(root: &Path, run_dir: &Path, shell: &mut Shell) -> Option<Vec<u8>> {
    let scratch = scratch_dir(run_dir);
    let check = match super::execute(root, run_dir, &scratch) {
        Ok(check) => check,
        Err(diagnostic) => {
            shell.diagnostic(diagnostic);
            return None;
        }
    };
    match check.mismatch_diagnostic() {
        None => Some(render(&check, shell.run_id(), &scratch)),
        Some(diagnostic) => {
            shell.diagnostic(diagnostic);
            None
        }
    }
}

/// The rerun's §8.3 layout root: the prior run's directory name suffixed
/// `.replay`, beside it. File-name append, so dotted run ids survive (an
/// extension swap would mangle `run.v2`).
fn scratch_dir(run_dir: &Path) -> PathBuf {
    let mut name = run_dir
        .file_name()
        .expect("validated run directory carries a name")
        .to_os_string();
    name.push(".replay");
    run_dir.with_file_name(name)
}

/// The matched §4.6 comparison as the stdout body: header, rerun outcome
/// and layout (runtime evidence beside the verdict), then the matched
/// hash set — on a match `expected` equals `actual`, both already in
/// §4.3 canonical set order.
fn render(check: &ReplayCheck, run_id: &Id, scratch: &Path) -> Vec<u8> {
    let mut out = String::new();
    out.push_str(&format!("replay run {run_id} matched\n"));
    out.push_str(&format!(
        "rerun outcome: {}\n",
        check.rerun_outcome.as_str()
    ));
    out.push_str(&format!("rerun layout: {}\n", scratch.display()));
    out.push_str(&format!("accepted artifacts: {}\n", check.expected.len()));
    for hash in &check.expected {
        out.push_str(hash.as_str());
        out.push('\n');
    }
    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::tests::{repo_root, test_source_run};
    use crate::shell::{FinishedCommand, static_id};
    use ckc_core::{
        DiagnosticCode, DiagnosticRecord, EventRecord, Hash, Outcome, ReplayManifest,
        canonical_payload_bytes, read_jsonl, read_strict_canonical,
    };

    /// The dispatch-shaped shell: operation `replay`, run id from the run
    /// directory's name, no write root (events stream).
    fn shell() -> Shell {
        Shell::open(static_id("replay"), static_id("m1"), None)
    }

    /// Close the shell and surface (outcome, diagnostics) from the command
    /// event — the observable failure surface.
    fn finished(shell: Shell) -> (Outcome, Vec<DiagnosticRecord>) {
        let FinishedCommand {
            result,
            streamed_events,
            ..
        } = shell.finish().unwrap();
        let events: Vec<EventRecord> = read_jsonl(streamed_events.as_deref().unwrap()).unwrap();
        assert_eq!(events.len(), 1);
        (result.outcome, events[0].diagnostics.clone())
    }

    fn read_manifest(run_dir: &Path) -> ReplayManifest {
        read_strict_canonical(&std::fs::read(run_dir.join("replay_manifest.json")).unwrap())
            .unwrap()
    }

    // §8.5 item 8 at the command surface, three paths over one test_source
    // run: the replay matches and renders the recorded hash set as the
    // body; the populated scratch layout then blocks a second attempt
    // (the core's empty-guard through the command's sibling layout
    // choice); a doctored expectation over a cleared scratch surfaces as
    // the §7.4 mismatch diagnostic and withholds the body.
    #[test]
    fn live_replay_matches_then_guards_then_mismatches() {
        let root = repo_root();
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = test_source_run(&root, tmp.path());

        let mut sh = shell();
        let body = execute(&root, &run_dir, &mut sh).unwrap();
        let scratch = tmp.path().join("m1.replay");
        assert!(scratch.join("manifest.json").is_file());
        assert!(scratch.join("report_en.md").is_file());
        let manifest = read_manifest(&run_dir);
        let mut expected = format!(
            "replay run m1 matched\nrerun outcome: ok\nrerun layout: {}\naccepted artifacts: {}\n",
            scratch.display(),
            manifest.expected_output_hashes.len()
        );
        for hash in &manifest.expected_output_hashes {
            expected.push_str(hash.as_str());
            expected.push('\n');
        }
        assert_eq!(String::from_utf8(body).unwrap(), expected);
        let (outcome, diagnostics) = finished(sh);
        assert_eq!(outcome, Outcome::Ok);
        assert!(diagnostics.is_empty());

        let mut sh = shell();
        assert_eq!(execute(&root, &run_dir, &mut sh), None);
        let (outcome, diagnostics) = finished(sh);
        assert_eq!(outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, DiagnosticCode::SchemaInvalid);
        assert_eq!(
            diagnostics[0].payload[0],
            (
                static_id("reason"),
                "scratch directory is not empty".to_owned()
            )
        );

        std::fs::remove_dir_all(&scratch).unwrap();
        let path = run_dir.join("replay_manifest.json");
        let mut doctored = read_manifest(&run_dir);
        let displaced = doctored.expected_output_hashes.pop().unwrap();
        let fake = Hash::new(format!("sha256:{}", "f".repeat(64))).unwrap();
        doctored.expected_output_hashes.push(fake.clone());
        doctored
            .expected_output_hashes
            .sort_by(|a, b| a.as_str().cmp(b.as_str()));
        std::fs::write(&path, canonical_payload_bytes(&doctored).unwrap()).unwrap();
        let mut sh = shell();
        assert_eq!(execute(&root, &run_dir, &mut sh), None);
        let (outcome, diagnostics) = finished(sh);
        assert_eq!(outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, DiagnosticCode::ReplayMismatch);
        assert_eq!(
            diagnostics[0].payload[0],
            (static_id("missing"), fake.as_str().to_owned())
        );
        assert_eq!(
            diagnostics[0].payload[2],
            (static_id("unexpected"), displaced.as_str().to_owned())
        );
        assert_eq!(diagnostics[0].artifact_hashes.len(), 2);
    }

    // Pre-comparison core failures land as command-scope diagnostics and
    // withhold the body; a replay that never re-executed creates no
    // scratch layout.
    #[test]
    fn missing_manifest_is_a_command_diagnostic() {
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = tmp.path().join("r1");
        std::fs::create_dir_all(&run_dir).unwrap();
        let mut sh = shell();
        assert_eq!(execute(&repo_root(), &run_dir, &mut sh), None);
        let (outcome, diagnostics) = finished(sh);
        assert_eq!(outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, DiagnosticCode::SchemaInvalid);
        assert_eq!(diagnostics[0].payload[0].1, "replay_manifest.json");
        assert!(!tmp.path().join("r1.replay").exists());
    }

    #[test]
    fn scratch_dir_is_the_suffixed_sibling() {
        assert_eq!(
            scratch_dir(Path::new("runs/m1")),
            Path::new("runs/m1.replay")
        );
        assert_eq!(
            scratch_dir(Path::new("runs/run.v2")),
            Path::new("runs/run.v2.replay")
        );
        assert_eq!(scratch_dir(Path::new("m1/")), Path::new("m1.replay"));
    }
}
