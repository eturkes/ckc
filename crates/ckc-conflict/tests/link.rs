//! Evidence-linkage gate for task 0.10.2.
//!
//! Over `verify_all(&CompileBundle::load_toy())`, `witness_for_cert` resolves the
//! real `nf-…` execution-witness id behind a certificate — replacing the toy
//! `conflicts.json` `witness_norm_conflict` placeholder — and `minimal_artifact_set`
//! resolves the real certificate content hashes behind a set of cert keys,
//! replacing the `sha256:a00…` placeholder. Both stay stable across independent
//! verification runs.

use ckc_conflict::link::{minimal_artifact_set, witness_for_cert};
use ckc_conflict::{CompileBundle, VerificationReport, verify_all};

/// The four norm-contradiction certificate keys (SPEC 15.1 #1/#2): the z3, cvc5,
/// and Lean checks over the norm-conflict target plus the clingo defeasible check —
/// the deterministic `cert_<solver>_<stem>` ids the 0.9 builder emits.
const NORM_CERT_KEYS: [&str; 4] = [
    "cert_z3_norm_conflict",
    "cert_cvc5_norm_conflict",
    "cert_lean_norm_conflict",
    "cert_clingo_defeasible",
];

fn toy_report() -> VerificationReport {
    verify_all(&CompileBundle::load_toy())
}

#[test]
fn witness_for_cert_resolves_real_nf_witness() {
    let report = toy_report();
    let witness = witness_for_cert(&report, "cert_z3_norm_conflict")
        .expect("a witness is checked by the z3 norm-conflict cert");
    assert!(
        witness.as_str().starts_with("nf-"),
        "resolved witness id is the real nf-… id, not the toy placeholder: {}",
        witness.as_str()
    );
}

#[test]
fn minimal_artifact_set_resolves_distinct_cert_hashes() {
    let report = toy_report();
    let set = minimal_artifact_set(&report, &NORM_CERT_KEYS);
    assert!(
        set.len() >= 3,
        "the norm-conflict cert keys resolve to at least 3 distinct hashes, got {}",
        set.len()
    );
    for h in &set {
        assert!(
            h.as_str().starts_with("sha256:"),
            "every minimal-artifact-set entry is a sha256: content hash: {}",
            h.as_str()
        );
    }
    let mut deduped = set.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(
        set, deduped,
        "the result is already sorted and deduplicated"
    );
}

#[test]
fn linkage_is_deterministic() {
    let a = toy_report();
    let b = toy_report();
    assert_eq!(
        witness_for_cert(&a, "cert_z3_norm_conflict"),
        witness_for_cert(&b, "cert_z3_norm_conflict"),
        "witness linkage differs across independent verification runs"
    );
    assert_eq!(
        minimal_artifact_set(&a, &NORM_CERT_KEYS),
        minimal_artifact_set(&b, &NORM_CERT_KEYS),
        "minimal artifact set differs across independent verification runs"
    );
}
