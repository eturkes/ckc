//! Norm-contradiction detector gate for task 0.10.5.
//!
//! Over the toy bundle and its verification report, `detect_norm_contradiction`
//! reports exactly one source-grounded conflict. The detection scan agrees on the
//! defeasible for/against pair (`rule_sepsis_bl_recommend` vs
//! `rule_bl_anaphylaxis_contra`, both over `concept_beta_lactam`); the assembled
//! conflict carries the computed type/classification/severity, a normalized `nf-…`
//! conflict id and argument-graph id, the real `nf-…` witness (not the toy
//! `witness_norm_conflict` placeholder), and a `sha256:` minimal artifact set; and
//! two runs agree under canonical hashing.

use ckc_conflict::detect::{detect_norm_contradiction, norm_contradiction_pair};
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
    assert_eq!(detect_norm_contradiction(&bundle, &report).len(), 1);
}

#[test]
fn scan_agrees_on_for_against_pair() {
    let bundle = CompileBundle::load_toy();
    let (for_rule, against_rule) = norm_contradiction_pair(&bundle)
        .expect("the toy bundle carries a defeasible for/against pair");
    assert_eq!(for_rule.rule_id.as_str(), "rule_sepsis_bl_recommend");
    assert_eq!(against_rule.rule_id.as_str(), "rule_bl_anaphylaxis_contra");
}

#[test]
fn carries_computed_type_classification_severity() {
    let (bundle, report) = toy();
    let conflict = &detect_norm_contradiction(&bundle, &report)[0];
    assert_eq!(conflict.conflict_type, "norm_contradiction");
    assert_eq!(
        conflict.classification,
        ConflictClassification::TrueConflict
    );
    assert_eq!(conflict.severity, Severity::High);
}

#[test]
fn conflict_and_argument_graph_ids_are_normalized() {
    let (bundle, report) = toy();
    let conflict = &detect_norm_contradiction(&bundle, &report)[0];
    assert!(
        conflict.conflict_id.as_str().starts_with("nf-"),
        "normalize_all assigns a content-derived nf-… conflict id: {}",
        conflict.conflict_id.as_str()
    );
    let argument_graph_id = conflict
        .argument_graph_id
        .as_ref()
        .expect("the norm contradiction links its argument graph");
    assert!(
        argument_graph_id.as_str().starts_with("nf-"),
        "the linked argument graph carries a normalized nf-… id: {}",
        argument_graph_id.as_str()
    );
}

#[test]
fn witness_is_the_real_nf_witness() {
    let (bundle, report) = toy();
    let conflict = &detect_norm_contradiction(&bundle, &report)[0];
    let witness = conflict
        .witness
        .as_ref()
        .expect("the norm contradiction links the z3 norm-conflict witness");
    assert!(
        witness.as_str().starts_with("nf-"),
        "the linked witness is the real nf-… id, not the toy placeholder: {}",
        witness.as_str()
    );
}

#[test]
fn minimal_artifact_set_is_content_hashes() {
    let (bundle, report) = toy();
    let conflict = &detect_norm_contradiction(&bundle, &report)[0];
    assert!(
        !conflict.minimal_artifact_set.is_empty(),
        "the norm-conflict certs resolve to a non-empty minimal artifact set"
    );
    for h in &conflict.minimal_artifact_set {
        assert!(
            h.as_str().starts_with("sha256:"),
            "every minimal-artifact-set entry is a sha256: content hash: {}",
            h.as_str()
        );
    }
}

#[test]
fn detection_is_deterministic() {
    let bundle = CompileBundle::load_toy();
    let a = detect_norm_contradiction(&bundle, &verify_all(&bundle));
    let b = detect_norm_contradiction(&bundle, &verify_all(&bundle));
    assert_eq!(a.len(), 1);
    assert_eq!(
        content_hash(&a[0]),
        content_hash(&b[0]),
        "two detection runs must produce equal conflict content hashes"
    );
}
