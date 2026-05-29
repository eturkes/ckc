//! `detect_all` orchestration gate for task 0.10.8.
//!
//! Over the toy bundle and its verification report, `detect_all` composes the three
//! per-class detectors into one `ConflictReport`: three source-grounded conflicts
//! (norm-contradiction, decision-table-defect, temporal-violation), the single Dung
//! argument graph backing the defeasible conflicts, and zero backend-disagreement
//! diagnostics (the real portfolio agrees). Every conflict and the argument graph
//! carries a normalized `nf-…` id, and two independent runs agree on every
//! per-artifact content hash.
//!
//! Task 0.10.9 persists that report to committed `certs/` files
//! (`certs/conflicts/<conflict_id>.json`, `certs/argument_graphs/<argument_graph_id>.json`)
//! and gates that each committed file regenerates byte-identically from a live
//! `detect_all`.

use std::fs;
use std::path::PathBuf;

use ckc_conflict::{
    CompileBundle, ConflictReport, ContentHash, content_hash, detect_all, verify_all,
};
use ckc_core::canonical::to_canonical_bytes;

fn toy_report() -> ConflictReport {
    let bundle = CompileBundle::load_toy();
    let report = verify_all(&bundle);
    detect_all(&bundle, &report)
}

/// Every conflict hash (document order) followed by every argument-graph hash — the
/// per-artifact content-hash vector the determinism check compares across runs.
fn artifact_hashes(report: &ConflictReport) -> Vec<ContentHash> {
    let mut hashes: Vec<ContentHash> = report.conflicts.iter().map(content_hash).collect();
    hashes.extend(report.argument_graphs.iter().map(content_hash));
    hashes
}

#[test]
fn composes_three_conflicts_one_graph_no_diagnostics() {
    let report = toy_report();
    assert_eq!(
        report.conflicts.len(),
        3,
        "norm-contradiction + decision-table-defect + temporal-violation"
    );
    assert_eq!(report.argument_graphs.len(), 1);
    assert_eq!(
        report.diagnostics.len(),
        0,
        "the real portfolio agrees, so no backend-disagreement diagnostics pass through"
    );
}

#[test]
fn every_artifact_id_is_normalized() {
    let report = toy_report();
    for conflict in &report.conflicts {
        assert!(
            conflict.conflict_id.as_str().starts_with("nf-"),
            "each detector assigns a content-derived nf-… conflict id: {}",
            conflict.conflict_id.as_str()
        );
    }
    for graph in &report.argument_graphs {
        assert!(
            graph.argument_graph_id.as_str().starts_with("nf-"),
            "the argument graph carries a content-derived nf-… id: {}",
            graph.argument_graph_id.as_str()
        );
    }
}

#[test]
fn detection_is_deterministic() {
    assert_eq!(
        artifact_hashes(&toy_report()),
        artifact_hashes(&toy_report()),
        "two detect_all runs must produce equal per-artifact content hashes"
    );
}

// ---------------------------------------------------------------------------
// Task 0.10.9: committed conflict artifact files + byte-identical regen
// ---------------------------------------------------------------------------

/// Repository root, two levels above this crate's manifest, so the committed
/// `certs/` artifact files resolve.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every serialized `detect_all` artifact paired with its canonical repo-relative
/// `certs/` path: each conflict under `certs/conflicts/<conflict_id>.json`, then
/// each argument graph under `certs/argument_graphs/<argument_graph_id>.json` —
/// each as `to_canonical_bytes`, in `detect_all` order. The regenerator and the
/// byte-equality gate share this list, so they never diverge.
fn report_artifacts(report: &ConflictReport) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    for conflict in &report.conflicts {
        out.push((
            PathBuf::from("certs/conflicts")
                .join(format!("{}.json", conflict.conflict_id.as_str())),
            to_canonical_bytes(conflict),
        ));
    }
    for graph in &report.argument_graphs {
        out.push((
            PathBuf::from("certs/argument_graphs")
                .join(format!("{}.json", graph.argument_graph_id.as_str())),
            to_canonical_bytes(graph),
        ));
    }
    out
}

/// Gate for task 0.10.9: every committed `certs/conflicts/*` and
/// `certs/argument_graphs/*` file holds exactly the bytes a live `detect_all`
/// produces, so regeneration is a byte-identical no-op. Drift here means a
/// detector or the argument-graph builder changed without rerunning the
/// regenerator below.
#[test]
fn committed_conflict_files_match_detect_all() {
    let report = toy_report();
    let root = workspace_root();
    for (rel, bytes) in report_artifacts(&report) {
        let committed = fs::read(root.join(&rel))
            .unwrap_or_else(|e| panic!("read committed {}: {e}", rel.display()));
        assert_eq!(
            committed,
            bytes,
            "committed {} drifted from detect_all; rerun `cargo test -p ckc-conflict \
             --test detect_all -- --ignored regenerate_conflict_artifact_files`",
            rel.display()
        );
    }
}

/// Regenerator (ignored by default; run with `--ignored` after intentional
/// detector or argument-graph changes). Writes each `detect_all` artifact to its
/// canonical `certs/` path via `to_canonical_bytes`, creating parent directories
/// as needed.
#[test]
#[ignore = "regenerate-only; rewrites the committed certs/conflicts/* and certs/argument_graphs/* files"]
fn regenerate_conflict_artifact_files() {
    let report = toy_report();
    let root = workspace_root();
    for (rel, bytes) in report_artifacts(&report) {
        let path = root.join(&rel);
        fs::create_dir_all(path.parent().expect("certs artifact path has a parent dir"))
            .unwrap_or_else(|e| panic!("create dir for {}: {e}", rel.display()));
        fs::write(&path, &bytes).unwrap_or_else(|e| panic!("write {}: {e}", rel.display()));
    }
}
