//! Certificate-builder gate for task 0.9.8.
//!
//! `certificates(&CompileBundle::load_toy())` yields the 10 Phase-0 certs — the 9
//! `compile_all` targets zipped with their recorded outcomes, plus the standalone
//! cvc5 proof-object cert over the norm-conflict target. This gate locks the
//! certificate-class distribution (one C6 cvc5 cert with a single proof hash, one
//! C7 Lean kernel cert, and eight C4-Executable certs) and the builder's
//! determinism across repeated calls.

use ckc_compile::CompileBundle;
use ckc_verify::{Certificate, CertificateClass, certificates, content_hash};

fn toy_certificates() -> Vec<Certificate> {
    certificates(&CompileBundle::load_toy())
}

#[test]
fn certificate_set_has_expected_class_distribution() {
    let certs = toy_certificates();
    assert_eq!(
        certs.len(),
        10,
        "10 certs: 9 compile targets + standalone cvc5"
    );

    let cvc5 = certs
        .iter()
        .find(|c| c.certificate_id.as_str() == "cert_cvc5_norm_conflict")
        .expect("a cvc5 norm-conflict cert");
    assert_eq!(cvc5.certificate_class, CertificateClass::C6ProofObject);
    assert_eq!(
        cvc5.proof_artifact_hashes.len(),
        1,
        "cvc5 cert carries exactly one proof hash"
    );

    let lean = certs
        .iter()
        .find(|c| c.certificate_id.as_str() == "cert_lean_norm_conflict")
        .expect("a lean norm-conflict cert");
    assert_eq!(lean.certificate_class, CertificateClass::C7Kernel);

    // Everything else is C4-Executable: z3 (×3), clingo (×2), souffle, tla, alloy.
    let c4 = certs
        .iter()
        .filter(|c| c.certificate_class == CertificateClass::C4Executable)
        .count();
    assert_eq!(c4, 8, "the other 8 certs are C4-Executable");

    // The MaxSMT repair cert appends its optimization objective to the verdict
    // token; the plain sat/unsat certs carry the bare token.
    let repair = certs
        .iter()
        .find(|c| c.certificate_id.as_str() == "cert_z3_repair_maxsmt")
        .expect("a z3 repair_maxsmt cert");
    assert_eq!(repair.result, "sat:1", "repair result appends objective 1");
}

#[test]
fn certificates_are_deterministic() {
    let a = toy_certificates();
    let b = toy_certificates();
    assert_eq!(a.len(), b.len());
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            content_hash(x),
            content_hash(y),
            "cert {i} content_hash differs across calls"
        );
    }
}
