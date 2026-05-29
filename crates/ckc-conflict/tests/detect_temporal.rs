//! Temporal-violation detector gate for task 0.10.7.
//!
//! Over the toy bundle and its verification report, `detect_temporal_violation`
//! reports exactly one source-grounded conflict. The detection scan agrees on the
//! Event-Calculus allergy-persistence violation — the single narrative shows a
//! fluent still holding when `administer_drug` fires (t = 10) with no terminating
//! event, and the recorded EC witness names `conflict_ec_allergy_persistence` in its
//! `violated_constraints`; the assembled conflict carries the computed
//! type/classification/severity, a normalized `nf-…` conflict id, and the real
//! `nf-…` EC witness (not the toy `witness_norm_conflict` placeholder); and two runs
//! agree under canonical hashing.

use ckc_conflict::detect::{
    allergy_persists_at_administration, detect_temporal_violation, ec_violated_constraints,
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
    assert_eq!(detect_temporal_violation(&bundle, &report).len(), 1);
}

#[test]
fn scan_agrees_on_ec_persistence_violation() {
    let (bundle, report) = toy();
    let narrative = bundle
        .event_narratives
        .first()
        .expect("the toy bundle carries one event narrative");
    assert!(
        allergy_persists_at_administration(narrative),
        "the fluent persists unterminated through the administer_drug time"
    );
    assert!(
        ec_violated_constraints(&report)
            .iter()
            .any(|c| c == "conflict_ec_allergy_persistence"),
        "the recorded EC witness names the allergy-persistence violation"
    );
}

#[test]
fn carries_computed_type_classification_severity() {
    let (bundle, report) = toy();
    let conflict = &detect_temporal_violation(&bundle, &report)[0];
    assert_eq!(conflict.conflict_type, "temporal_violation");
    assert_eq!(
        conflict.classification,
        ConflictClassification::TrueConflict
    );
    assert_eq!(conflict.severity, Severity::High);
}

#[test]
fn conflict_id_is_normalized() {
    let (bundle, report) = toy();
    let conflict = &detect_temporal_violation(&bundle, &report)[0];
    assert!(
        conflict.conflict_id.as_str().starts_with("nf-"),
        "normalize_all assigns a content-derived nf-… conflict id: {}",
        conflict.conflict_id.as_str()
    );
}

#[test]
fn witness_is_the_real_nf_witness() {
    let (bundle, report) = toy();
    let conflict = &detect_temporal_violation(&bundle, &report)[0];
    let witness = conflict
        .witness
        .as_ref()
        .expect("the temporal violation links the clingo EC witness");
    assert!(
        witness.as_str().starts_with("nf-"),
        "the linked witness is the real nf-… id, not the toy placeholder: {}",
        witness.as_str()
    );
}

#[test]
fn detection_is_deterministic() {
    let bundle = CompileBundle::load_toy();
    let a = detect_temporal_violation(&bundle, &verify_all(&bundle));
    let b = detect_temporal_violation(&bundle, &verify_all(&bundle));
    assert_eq!(a.len(), 1);
    assert_eq!(
        content_hash(&a[0]),
        content_hash(&b[0]),
        "two detection runs must produce equal conflict content hashes"
    );
}
