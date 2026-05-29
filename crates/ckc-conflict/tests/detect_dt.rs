//! Decision-table-defect detector gate for task 0.10.6.
//!
//! Over the toy bundle and its verification report, `detect_decision_table_defects`
//! reports exactly one source-grounded conflict. The detection scan agrees on the
//! overlapping unique-policy row pair (`row_temp_high` vs `row_temp_very_high` of
//! `dt_vitals_triage`, which both fire at `temperature ≥ 38.5` under differing
//! outputs) and finds a gap point firing no row; the assembled conflict carries the
//! computed type/classification/severity and a normalized `nf-…` conflict id; and
//! two runs agree under canonical hashing.

use ckc_conflict::detect::{
    decision_table_gap, decision_table_overlap, detect_decision_table_defects,
};
use ckc_conflict::{CompileBundle, VerificationReport, content_hash, verify_all};
use ckc_core::enums::{ConflictClassification, Severity};

fn toy() -> (CompileBundle, VerificationReport) {
    let bundle = CompileBundle::load_toy();
    let report = verify_all(&bundle);
    (bundle, report)
}

#[test]
fn detects_exactly_one_conflict() {
    let (bundle, report) = toy();
    assert_eq!(detect_decision_table_defects(&bundle, &report).len(), 1);
}

#[test]
fn scan_agrees_on_overlapping_row_pair() {
    let bundle = CompileBundle::load_toy();
    let (a, b) = decision_table_overlap(&bundle)
        .expect("the toy bundle carries a unique-policy overlapping row pair");
    assert_eq!(a.row_id.as_str(), "row_temp_high");
    assert_eq!(b.row_id.as_str(), "row_temp_very_high");
}

#[test]
fn scan_finds_a_gap_point() {
    let bundle = CompileBundle::load_toy();
    assert!(
        decision_table_gap(&bundle).is_some(),
        "the unique-policy table leaves an input region firing no row"
    );
}

#[test]
fn carries_computed_type_classification_severity() {
    let (bundle, report) = toy();
    let conflict = &detect_decision_table_defects(&bundle, &report)[0];
    assert_eq!(conflict.conflict_type, "decision_table_overlap");
    assert_eq!(
        conflict.classification,
        ConflictClassification::TrueConflict
    );
    assert_eq!(conflict.severity, Severity::Medium);
}

#[test]
fn conflict_id_is_normalized() {
    let (bundle, report) = toy();
    let conflict = &detect_decision_table_defects(&bundle, &report)[0];
    assert!(
        conflict.conflict_id.as_str().starts_with("nf-"),
        "normalize_all assigns a content-derived nf-… conflict id: {}",
        conflict.conflict_id.as_str()
    );
}

#[test]
fn carries_no_argument_graph() {
    let (bundle, report) = toy();
    let conflict = &detect_decision_table_defects(&bundle, &report)[0];
    assert!(
        conflict.argument_graph_id.is_none(),
        "a decision-table defect carries no argument graph"
    );
}

#[test]
fn detection_is_deterministic() {
    let bundle = CompileBundle::load_toy();
    let a = detect_decision_table_defects(&bundle, &verify_all(&bundle));
    let b = detect_decision_table_defects(&bundle, &verify_all(&bundle));
    assert_eq!(a.len(), 1);
    assert_eq!(
        content_hash(&a[0]),
        content_hash(&b[0]),
        "two detection runs must produce equal conflict content hashes"
    );
}
