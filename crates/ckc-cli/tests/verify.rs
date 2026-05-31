//! Task 0.11.3 gate: `pipeline::run_verify` emits the Phase-0 verification
//! artifact set under a fresh output dir — 11 certificates, 11 witnesses, the
//! certificate graph, and the assurance seed (each byte-identical to its committed
//! `certs/*` counterpart) plus the recorded cvc5 proof — with 24 manifest entries
//! carrying the `verification_manifest` content hashes and two runs byte-identical.

use std::fs;
use std::path::PathBuf;

use ckc_cli::pipeline::{load_bundle, run_verify};
use ckc_cli::verification_manifest;

/// Repository root, two levels above this crate's manifest, so the committed
/// `certs/*` verification artifacts resolve.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The recorded cvc5 proof copy: `run_verify` writes it, but as recorded evidence
/// reached through `cert_cvc5_norm_conflict.proof_artifact_hashes` it carries no
/// manifest entry.
const CVC5_PROOF_REL: &str = "certs/cvc5_norm_conflict.proof";

#[test]
fn run_verify_emits_24_artifacts_matching_committed_certs() {
    let bundle = load_bundle("examples/research_kernel").expect("load toy bundle");
    let out = tempfile::tempdir().expect("tempdir");
    let (_report, entries) = run_verify(&bundle, out.path()).expect("run_verify");

    let manifest = verification_manifest(&bundle);
    assert_eq!(
        entries.len(),
        24,
        "11 certificates + 11 witnesses + the graph + the assurance seed"
    );
    assert_eq!(
        entries.len(),
        manifest.0.len(),
        "one entry per manifest row"
    );

    let root = workspace_root();
    for (i, entry) in entries.iter().enumerate() {
        let row = &manifest.0[i];
        assert_eq!(entry.stage, "verify");
        assert_eq!(entry.artifact_kind, row.artifact_kind);
        assert_eq!(entry.artifact_path, row.artifact_path);
        assert_eq!(
            entry.content_hash, row.content_hash,
            "entry {i} hash equals the verification-manifest hash, in order"
        );
        let rel = &entry.artifact_path;
        let written =
            fs::read(out.path().join(rel)).unwrap_or_else(|e| panic!("read emitted {rel}: {e}"));
        let committed =
            fs::read(root.join(rel)).unwrap_or_else(|e| panic!("read committed {rel}: {e}"));
        assert_eq!(
            written, committed,
            "emitted {rel} drifted from the committed artifact"
        );
    }

    // The recorded cvc5 proof is written beside the certs but is no manifest entry.
    let written_proof = fs::read(out.path().join(CVC5_PROOF_REL)).expect("read emitted cvc5 proof");
    let committed_proof = fs::read(root.join(CVC5_PROOF_REL)).expect("read committed cvc5 proof");
    assert_eq!(
        written_proof, committed_proof,
        "emitted {CVC5_PROOF_REL} drifted from the committed proof"
    );
    assert!(
        entries.iter().all(|e| e.artifact_path != CVC5_PROOF_REL),
        "the recorded cvc5 proof must carry no manifest entry",
    );
}

#[test]
fn run_verify_is_deterministic_across_runs() {
    let bundle = load_bundle("examples/research_kernel").expect("load toy bundle");
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    let (_ra, ea) = run_verify(&bundle, a.path()).expect("run a");
    let (_rb, eb) = run_verify(&bundle, b.path()).expect("run b");
    assert_eq!(ea, eb, "manifest entries identical across runs");
    for entry in &ea {
        let rel = &entry.artifact_path;
        let fa = fs::read(a.path().join(rel)).unwrap_or_else(|e| panic!("read a {rel}: {e}"));
        let fb = fs::read(b.path().join(rel)).unwrap_or_else(|e| panic!("read b {rel}: {e}"));
        assert_eq!(fa, fb, "{rel} differs across the two runs");
    }
    let pa = fs::read(a.path().join(CVC5_PROOF_REL)).expect("read a proof");
    let pb = fs::read(b.path().join(CVC5_PROOF_REL)).expect("read b proof");
    assert_eq!(pa, pb, "{CVC5_PROOF_REL} differs across the two runs");
}
