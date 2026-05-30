//! Task 0.11.1 scaffold gate: the run-manifest type round-trips through serde,
//! and `emit::write_artifact` writes exact bytes under a fresh output dir.

use ckc_cli::ContentHash;
use ckc_cli::manifest::{RunManifest, RunManifestEntry};

#[test]
fn run_manifest_round_trips_through_serde() {
    let manifest = RunManifest {
        command: "ckc demo research-kernel --out runs/research".to_string(),
        producer_version: env!("CARGO_PKG_VERSION").to_string(),
        entries: vec![RunManifestEntry {
            stage: "compile".to_string(),
            artifact_kind: "compiled_target".to_string(),
            artifact_path: "logic/smt/norm_conflict.smt2".to_string(),
            content_hash: ContentHash("sha256:00".to_string()),
        }],
    };
    let bytes = serde_json::to_vec(&manifest).expect("serialize");
    let back: RunManifest = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(manifest, back);
}

#[test]
fn write_artifact_writes_exact_bytes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bytes: &[u8] = b"(rule beta_lactam)\n";
    ckc_cli::emit::write_artifact(dir.path(), "logic/smt/x.smt2", bytes).expect("write");
    let written = std::fs::read(dir.path().join("logic/smt/x.smt2")).expect("read back");
    assert_eq!(written, bytes.to_vec());
}
