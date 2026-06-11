//! Binary-level shell test: the SPEC §3 CLI invariants observed from
//! outside the process — exactly one §4.4 result line on stdout, §4.6
//! events on stderr or under the run directory, writes confined to the
//! output directory, exit code mapped from the outcome.

use std::path::Path;
use std::process::{Command, Output};

use ckc_core::{EventRecord, Outcome, TotalOperationResult, read_jsonl};

fn ckc(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args(args)
        .output()
        .unwrap()
}

fn single_result(stdout: &[u8]) -> TotalOperationResult {
    let results: Vec<TotalOperationResult> = read_jsonl(stdout).unwrap();
    assert_eq!(results.len(), 1, "stdout carries exactly one result line");
    results.into_iter().next().unwrap()
}

fn sorted_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

// §8.5 item 2 end-to-end: `ckc registry check` from the repository root
// passes against the committed registry set, streaming events to stderr and
// exactly one ok result to stdout.
#[test]
fn registry_check_passes_from_the_repo_root() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args(["registry", "check"])
        .current_dir(repo_root)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let result = single_result(&out.stdout);
    assert_eq!(result.operation_id, "registry.check".parse().unwrap());
    assert_eq!(result.outcome, Outcome::Ok);
    assert!(result.diagnostic_hashes.is_empty());
    let events: Vec<EventRecord> = read_jsonl(&out.stderr).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, Outcome::Ok);
    assert_eq!(events[0].run_id, "run.none".parse().unwrap());
}

// The same command from a root without registry files fails invalid (exit
// 2) with one diagnostic per unreadable core file.
#[test]
fn registry_check_fails_invalid_off_root() {
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args(["registry", "check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let result = single_result(&out.stdout);
    assert_eq!(result.outcome, Outcome::Invalid);
    assert_eq!(result.diagnostic_hashes.len(), 3);
    let events: Vec<EventRecord> = read_jsonl(&out.stderr).unwrap();
    assert_eq!(events[0].diagnostics.len(), 3);
}

// `ckc run` from the repository root completes the §8.3 chain through
// verify: document and group artifacts land under the §8.3 layout, the run
// exits ok, and every write is confined to the output directory.
#[test]
fn run_writes_only_under_its_out_dir() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let out_dir = tmp.path().join("m1");
    let out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args([
            "run",
            "--experiment",
            "exp.m1_spine",
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .current_dir(repo_root)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty(), "events landed in logs/, not stderr");
    assert_eq!(single_result(&out.stdout).outcome, Outcome::Ok);

    assert_eq!(sorted_entries(tmp.path()), ["m1"]);
    assert_eq!(sorted_entries(&out_dir), ["artifacts", "groups", "logs"]);
    assert_eq!(
        sorted_entries(&out_dir.join("logs")),
        ["diagnostics.jsonl", "events.jsonl"]
    );
    assert_eq!(
        sorted_entries(&out_dir.join("artifacts/fixture.m1_guideline_a")),
        [
            "ir_bundle.json",
            "normalization.json",
            "segments.json",
            "source_graph.json"
        ]
    );
    assert_eq!(
        sorted_entries(&out_dir.join("groups")),
        ["group.m1_conflict", "group.m1_null"]
    );
    assert_eq!(
        sorted_entries(&out_dir.join("groups/group.m1_conflict")),
        ["compiled.json", "smt", "verifier_results.json"]
    );
    assert_eq!(
        sorted_entries(&out_dir.join("groups/group.m1_conflict/smt")),
        [
            "q.m1_conflict.pair1.deontic.smt2",
            "q.m1_conflict.pair1.overlap.smt2"
        ]
    );
    let events: Vec<EventRecord> =
        read_jsonl(&std::fs::read(out_dir.join("logs/events.jsonl")).unwrap()).unwrap();
    assert_eq!(events.len(), 17);
    assert_eq!(events[0].run_id, "m1".parse().unwrap());
}

#[test]
fn invalid_arguments_exit_two_with_diagnostic() {
    let out = ckc(&["run", "--experiment", "exp.x"]);
    assert_eq!(out.status.code(), Some(2));
    let result = single_result(&out.stdout);
    assert_eq!(result.outcome, Outcome::Invalid);
    assert_eq!(result.diagnostic_hashes.len(), 1);
    let events: Vec<EventRecord> = read_jsonl(&out.stderr).unwrap();
    assert_eq!(events.len(), 1);
}
