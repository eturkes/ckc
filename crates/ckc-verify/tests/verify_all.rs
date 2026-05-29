//! Orchestration gate for task 0.9.14.
//!
//! `verify_all(&CompileBundle::load_toy())` composes the Phase-0 verification
//! report from the recorded oracle: 11 certificates (the 10 solver/cvc5 certs
//! plus the in-process SHACL grounding cert), 11 mirrored witnesses, the two
//! C5-Portfolio agreements (norm-conflict, priority-acyclicity) with no backend
//! disagreement, the certificate graph, and the assurance seed. The determinism
//! check hashes every artifact field across two independent runs.

use ckc_compile::CompileBundle;
use ckc_core::canonical::{ContentHash, content_hash};

use ckc_verify::{VerificationReport, verify_all};

fn toy_report() -> VerificationReport {
    verify_all(&CompileBundle::load_toy())
}

#[test]
fn verify_all_has_expected_counts() {
    let report = toy_report();
    assert_eq!(
        report.certificates.len(),
        11,
        "11 certs: 10 solver/cvc5 + the SHACL grounding cert"
    );
    assert_eq!(
        report.witnesses.len(),
        11,
        "11 witnesses mirror the 11 certs"
    );
    assert_eq!(
        report.agreements.len(),
        2,
        "the norm-conflict and priority-acyclicity portfolios"
    );
    assert!(
        report.disagreements.is_empty(),
        "the recorded oracle has no backend disagreement, found {:?}",
        report.disagreements
    );
}

/// Per-artifact content hashes of a report, field by field in a fixed order — the
/// determinism fingerprint compared across runs.
fn field_hashes(report: &VerificationReport) -> Vec<ContentHash> {
    let mut hashes = Vec::new();
    hashes.extend(report.certificates.iter().map(content_hash));
    hashes.extend(report.witnesses.iter().map(content_hash));
    hashes.extend(report.agreements.iter().map(content_hash));
    hashes.extend(report.disagreements.iter().map(content_hash));
    hashes.push(content_hash(&report.graph));
    hashes.push(content_hash(&report.assurance));
    hashes
}

#[test]
fn verify_all_is_deterministic() {
    let a = field_hashes(&toy_report());
    let b = field_hashes(&toy_report());
    assert_eq!(
        a, b,
        "verify_all per-field content hashes differ across runs"
    );
}
