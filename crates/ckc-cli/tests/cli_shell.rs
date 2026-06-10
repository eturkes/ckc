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

#[test]
fn registry_check_streams_events_and_one_result() {
    let out = ckc(&["registry", "check"]);
    assert_eq!(out.status.code(), Some(1));
    let result = single_result(&out.stdout);
    assert_eq!(result.operation_id, "registry.check".parse().unwrap());
    assert_eq!(result.outcome, Outcome::Unsupported);
    let events: Vec<EventRecord> = read_jsonl(&out.stderr).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, Outcome::Unsupported);
    assert_eq!(events[0].run_id, "run.none".parse().unwrap());
}

#[test]
fn run_writes_only_under_its_out_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let out_dir = tmp.path().join("v1");
    let out = ckc(&[
        "run",
        "--experiment",
        "exp.v1_spine",
        "--out",
        out_dir.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stderr.is_empty(), "events landed in logs/, not stderr");
    assert_eq!(single_result(&out.stdout).outcome, Outcome::Unsupported);

    assert_eq!(sorted_entries(tmp.path()), ["v1"]);
    assert_eq!(sorted_entries(&out_dir), ["logs"]);
    assert_eq!(
        sorted_entries(&out_dir.join("logs")),
        ["diagnostics.jsonl", "events.jsonl"]
    );
    let events: Vec<EventRecord> =
        read_jsonl(&std::fs::read(out_dir.join("logs/events.jsonl")).unwrap()).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].run_id, "v1".parse().unwrap());
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
