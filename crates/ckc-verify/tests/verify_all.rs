//! Orchestration gate for tasks 0.9.14 and 0.9.15.
//!
//! `verify_all(&CompileBundle::load_toy())` composes the Phase-0 verification
//! report from the recorded oracle: 11 certificates (the 10 solver/cvc5 certs
//! plus the in-process SHACL grounding cert), 11 mirrored witnesses, the two
//! C5-Portfolio agreements (norm-conflict, priority-acyclicity) with no backend
//! disagreement, the certificate graph, and the assurance seed. The determinism
//! check hashes every artifact field across two independent runs.
//!
//! Task 0.9.15 persists that report to committed `certs/*` files
//! (`certs/certificates/<id>.json`, `certs/witnesses/<id>.json`,
//! `certs/certificate_graph.json`, `certs/assurance_seed.json`) plus the copied
//! `certs/cvc5_norm_conflict.proof`, and gates that each committed file
//! regenerates byte-identically from a live `verify_all`.

use std::fs;
use std::path::PathBuf;

use ckc_compile::CompileBundle;
use ckc_core::canonical::{ContentHash, content_hash, to_canonical_bytes};

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

// ---------------------------------------------------------------------------
// Task 0.9.15: committed verification artifact files + byte-identical regen
// ---------------------------------------------------------------------------

/// Repository root, two levels above this crate's manifest, so the committed
/// `certs/` artifact files resolve.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The recorded cvc5 proof and its committed copy, placed beside the
/// `cert_cvc5_norm_conflict` certificate that references its content hash.
const CVC5_PROOF_SRC: &str = "examples/research_kernel/fixtures/cvc5_norm_conflict.proof";
const CVC5_PROOF_DST: &str = "certs/cvc5_norm_conflict.proof";

/// Every serialized `verify_all` artifact paired with its canonical repo-relative
/// `certs/` path: the 11 certificates under `certs/certificates/`, the 11
/// witnesses under `certs/witnesses/`, the certificate graph, and the assurance
/// seed — each as `to_canonical_bytes`. The regenerator and the byte-equality
/// gate share this list, so they never diverge.
fn report_artifacts(report: &VerificationReport) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    for cert in &report.certificates {
        out.push((
            PathBuf::from("certs/certificates")
                .join(format!("{}.json", cert.certificate_id.as_str())),
            to_canonical_bytes(cert),
        ));
    }
    for witness in &report.witnesses {
        out.push((
            PathBuf::from("certs/witnesses").join(format!("{}.json", witness.witness_id.as_str())),
            to_canonical_bytes(witness),
        ));
    }
    out.push((
        PathBuf::from("certs/certificate_graph.json"),
        to_canonical_bytes(&report.graph),
    ));
    out.push((
        PathBuf::from("certs/assurance_seed.json"),
        to_canonical_bytes(&report.assurance),
    ));
    out
}

/// Gate for task 0.9.15: every committed `certs/*` file holds exactly the bytes a
/// live `verify_all` produces, so regeneration is a byte-identical no-op. Drift
/// here means a builder changed without rerunning the regenerator below.
#[test]
fn committed_certs_files_match_verify_all() {
    let report = toy_report();
    let root = workspace_root();
    for (rel, bytes) in report_artifacts(&report) {
        let committed = fs::read(root.join(&rel))
            .unwrap_or_else(|e| panic!("read committed {}: {e}", rel.display()));
        assert_eq!(
            committed,
            bytes,
            "committed {} drifted from verify_all; rerun `cargo test -p ckc-verify \
             --test verify_all -- --ignored regenerate_certs_artifact_files`",
            rel.display()
        );
    }
    // The copied proof must match its recorded source byte-for-byte.
    let committed_proof = fs::read(root.join(CVC5_PROOF_DST))
        .unwrap_or_else(|e| panic!("read committed {CVC5_PROOF_DST}: {e}"));
    let source_proof = fs::read(root.join(CVC5_PROOF_SRC))
        .unwrap_or_else(|e| panic!("read source {CVC5_PROOF_SRC}: {e}"));
    assert_eq!(
        committed_proof, source_proof,
        "{CVC5_PROOF_DST} drifted from the recorded {CVC5_PROOF_SRC}"
    );
}

/// Regenerator (ignored by default; run with `--ignored` after intentional
/// builder changes). Writes each `verify_all` artifact to its canonical `certs/`
/// path via `to_canonical_bytes` and copies the recorded cvc5 proof beside them,
/// creating parent directories as needed.
#[test]
#[ignore = "regenerate-only; rewrites the committed certs/* verification artifact files"]
fn regenerate_certs_artifact_files() {
    let report = toy_report();
    let root = workspace_root();
    for (rel, bytes) in report_artifacts(&report) {
        let path = root.join(&rel);
        fs::create_dir_all(path.parent().expect("certs artifact path has a parent dir"))
            .unwrap_or_else(|e| panic!("create dir for {}: {e}", rel.display()));
        fs::write(&path, &bytes).unwrap_or_else(|e| panic!("write {}: {e}", rel.display()));
    }
    fs::copy(root.join(CVC5_PROOF_SRC), root.join(CVC5_PROOF_DST))
        .unwrap_or_else(|e| panic!("copy {CVC5_PROOF_SRC} -> {CVC5_PROOF_DST}: {e}"));
}
