//! Verdict-type gate for task 0.9.1: a `VerifierOutcome` round-trips through
//! serde, and its `content_hash` is stable across repeated computation.

use ckc_verify::{
    CertificateClass, ContentHash, SolverId, TargetLanguage, VerdictStatus, VerifierOutcome,
    content_hash,
};

fn sample() -> VerifierOutcome {
    VerifierOutcome {
        target_language: TargetLanguage::SmtLib,
        artifact_path: "logic/smt/norm_conflict.smt2".into(),
        solver: SolverId::Z3,
        status: VerdictStatus::Unsat,
        salient_atoms: vec![],
        objective: None,
        proof_present: false,
        certificate_class: CertificateClass::C4Executable,
    }
}

#[test]
fn verifier_outcome_roundtrips_through_serde() {
    let outcome = sample();
    let json = serde_json::to_string(&outcome).unwrap();
    let back: VerifierOutcome = serde_json::from_str(&json).unwrap();
    assert_eq!(outcome, back);
}

#[test]
fn content_hash_is_stable() {
    let outcome = sample();
    let a: ContentHash = content_hash(&outcome);
    let b = content_hash(&outcome);
    assert_eq!(a, b);
}
