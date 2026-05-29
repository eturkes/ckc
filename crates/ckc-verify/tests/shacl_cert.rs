//! SHACL certificate + witness gate for task 0.9.10 (scenario 6: missing
//! provenance fails SHACL).
//!
//! `shacl_certificate(&CompileBundle::load_toy())` runs the in-process SHACL
//! rule-shape check, which flags exactly the two `rule_incomplete_provenance`
//! violations (missing `source_span_ids` via MinCount(1), empty `provenance` via
//! MinLength(1)), and wraps the report in a `C6-ProofObject` certificate plus its
//! execution witness. This gate locks the violation count, the certificate class
//! and its single proof hash, the witness's reference to the rule under review with
//! its two violated constraints, and builder determinism across repeated calls.

use ckc_compile::CompileBundle;
use ckc_term::shacl::validate_rules;
use ckc_verify::{CertificateClass, content_hash, shacl_certificate};

#[test]
fn shacl_report_flags_two_provenance_violations() {
    let bundle = CompileBundle::load_toy();
    let report = validate_rules(&bundle.rules);
    assert!(!report.conforms, "the incomplete rule breaks conformance");
    assert_eq!(
        report.violations.len(),
        2,
        "scenario 6: rule_incomplete_provenance violates MinCount(1) and MinLength(1)"
    );
    assert!(
        report
            .violations
            .iter()
            .all(|v| v.focus_node == "rule_incomplete_provenance"),
        "both violations localize to the incomplete rule"
    );
}

#[test]
fn shacl_certificate_is_c6_proof_object() {
    let bundle = CompileBundle::load_toy();
    let (cert, witness) = shacl_certificate(&bundle);

    assert_eq!(cert.certificate_id.as_str(), "cert_shacl_rules");
    assert_eq!(cert.certificate_class, CertificateClass::C6ProofObject);
    assert_eq!(
        cert.proof_artifact_hashes.len(),
        1,
        "the SHACL report is the single proof artifact"
    );
    assert_eq!(cert.result, "violations_found:2");
    assert_eq!(cert.solver_or_checker, "ckc-shacl");

    // scenario 6: the witness references the rule under review and both violated
    // constraints, and carries no source span (their absence is the violation).
    assert!(
        witness
            .applicable_rules
            .iter()
            .any(|r| r.as_str() == "rule_incomplete_provenance"),
        "the witness references the rule under review"
    );
    assert_eq!(
        witness.violated_constraints,
        vec![
            "rule_incomplete_provenance/provenance".to_string(),
            "rule_incomplete_provenance/source_span_ids".to_string(),
        ],
        "violated constraints match the recorded SHACL salient atoms"
    );
    assert!(
        witness.source_span_ids.is_empty(),
        "the rule under review has no spans — that absence is the violation"
    );
    assert!(
        witness
            .certificate_ids
            .iter()
            .any(|c| c.as_str() == "cert_shacl_rules"),
        "the witness links its certificate"
    );
}

#[test]
fn shacl_certificate_is_deterministic() {
    let bundle = CompileBundle::load_toy();
    let (c1, w1) = shacl_certificate(&bundle);
    let (c2, w2) = shacl_certificate(&bundle);
    assert_eq!(
        content_hash(&c1),
        content_hash(&c2),
        "certificate content_hash differs across calls"
    );
    assert_eq!(
        content_hash(&w1),
        content_hash(&w2),
        "witness content_hash differs across calls"
    );
}
