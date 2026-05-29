//! Aggregate-type gate for task 0.10.1: an empty `ConflictReport` round-trips
//! through serde, and its `content_hash` is stable across repeated computation.

use ckc_conflict::{ConflictReport, ContentHash, content_hash};

fn empty_report() -> ConflictReport {
    ConflictReport {
        conflicts: vec![],
        argument_graphs: vec![],
        diagnostics: vec![],
    }
}

#[test]
fn conflict_report_roundtrips_through_serde() {
    let report = empty_report();
    let json = serde_json::to_string(&report).unwrap();
    let back: ConflictReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report, back);
}

#[test]
fn content_hash_is_stable() {
    let report = empty_report();
    let a: ContentHash = content_hash(&report);
    let b = content_hash(&report);
    assert_eq!(a, b);
}
