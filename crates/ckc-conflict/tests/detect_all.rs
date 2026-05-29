//! `detect_all` orchestration gate for task 0.10.8.
//!
//! Over the toy bundle and its verification report, `detect_all` composes the three
//! per-class detectors into one `ConflictReport`: three source-grounded conflicts
//! (norm-contradiction, decision-table-defect, temporal-violation), the single Dung
//! argument graph backing the defeasible conflicts, and zero backend-disagreement
//! diagnostics (the real portfolio agrees). Every conflict and the argument graph
//! carries a normalized `nf-…` id, and two independent runs agree on every
//! per-artifact content hash.

use ckc_conflict::{
    CompileBundle, ConflictReport, ContentHash, content_hash, detect_all, verify_all,
};

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
