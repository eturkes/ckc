//! Gate for task 0.13.2: the replay diagnostic types (SPEC 18) round-trip
//! through serde for both the `Passed` (no-mismatch) and `Failed`
//! (one-`ReplayMismatch`) shapes, and the `ReplayReport` JSON Schema stays
//! byte-stable against the committed `schemas/replay_report.schema.json`.
//! Mirrors the hand-built serde checks of `scaffold.rs` and the schema-only
//! `check_schema` + `#[ignore] regenerate` idiom of `ckc-report/tests/schema.rs`
//! (0.12.1). `replay_report.json` is a run artifact, not a committed golden, so
//! this gate locks the schema only; the value golden is `run_manifest.json`.

use std::path::PathBuf;

use ckc_cli::ContentHash;
use ckc_cli::replay::{ReplayMismatch, ReplayReport};
use ckc_core::enums::ReplayStatus;

fn schema_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("schemas")
}

#[test]
fn passed_report_round_trips_through_serde() {
    let report = ReplayReport {
        manifest_command: "ckc demo research-kernel --out runs/research".to_string(),
        status: ReplayStatus::Passed,
        n_entries: 41,
        n_matched: 41,
        mismatches: vec![],
    };
    let bytes = serde_json::to_vec(&report).expect("serialize");
    let back: ReplayReport = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(report, back);
}

#[test]
fn failed_report_round_trips_through_serde() {
    let report = ReplayReport {
        manifest_command: "ckc demo research-kernel --out runs/research".to_string(),
        status: ReplayStatus::Failed,
        n_entries: 41,
        n_matched: 40,
        mismatches: vec![ReplayMismatch {
            artifact_path: "logic/smt/norm_conflict.smt2".to_string(),
            expected_hash: Some(ContentHash("sha256:00".to_string())),
            actual_hash: Some(ContentHash("sha256:11".to_string())),
        }],
    };
    let bytes = serde_json::to_vec(&report).expect("serialize");
    let back: ReplayReport = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(report, back);
}

#[test]
fn schema() {
    let schema = schemars::schema_for!(ReplayReport);
    let json = serde_json::to_string_pretty(&schema).unwrap() + "\n";
    let path = schema_dir().join("replay_report.schema.json");
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read schema {}: {e}", path.display()));
    assert!(json == golden, "schema mismatch for replay_report");
}

// Regeneration: `cargo test -p ckc-cli --test replay_schema -- --ignored`
#[test]
#[ignore]
fn regenerate() {
    let schema = schemars::schema_for!(ReplayReport);
    std::fs::write(
        schema_dir().join("replay_report.schema.json"),
        serde_json::to_string_pretty(&schema).unwrap() + "\n",
    )
    .unwrap();
}
