//! Task 0.11.4 gate: `pipeline::run_conflicts` emits the Phase-0 conflict
//! artifact set under a fresh output dir — 3 conflicts + 1 argument graph (each
//! byte-identical to its committed `certs/{conflicts,argument_graphs}/*` `nf-…`
//! counterpart) — with 4 manifest entries carrying the `conflict_manifest`
//! content hashes and two runs byte-identical.

use std::fs;
use std::path::PathBuf;

use ckc_cli::conflict_manifest;
use ckc_cli::pipeline::{load_bundle, run_conflicts};
use ckc_cli::verify_all;

/// Repository root, two levels above this crate's manifest, so the committed
/// `certs/*` conflict artifacts resolve.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn run_conflicts_emits_4_artifacts_matching_committed_certs() {
    let bundle = load_bundle("examples/research_kernel").expect("load toy bundle");
    let report = verify_all(&bundle);
    let out = tempfile::tempdir().expect("tempdir");
    let entries = run_conflicts(&bundle, &report, out.path()).expect("run_conflicts");

    let manifest = conflict_manifest(&bundle, &report);
    assert_eq!(entries.len(), 4, "3 conflicts + 1 argument graph");
    assert_eq!(
        entries.len(),
        manifest.0.len(),
        "one entry per manifest row"
    );

    let root = workspace_root();
    for (i, entry) in entries.iter().enumerate() {
        let row = &manifest.0[i];
        assert_eq!(entry.stage, "conflicts");
        assert_eq!(entry.artifact_kind, row.artifact_kind);
        assert_eq!(entry.artifact_path, row.artifact_path);
        assert_eq!(
            entry.content_hash, row.content_hash,
            "entry {i} hash equals the conflict-manifest hash, in order"
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
}

#[test]
fn run_conflicts_is_deterministic_across_runs() {
    let bundle = load_bundle("examples/research_kernel").expect("load toy bundle");
    let report = verify_all(&bundle);
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    let ea = run_conflicts(&bundle, &report, a.path()).expect("run a");
    let eb = run_conflicts(&bundle, &report, b.path()).expect("run b");
    assert_eq!(ea, eb, "manifest entries identical across runs");
    for entry in &ea {
        let rel = &entry.artifact_path;
        let fa = fs::read(a.path().join(rel)).unwrap_or_else(|e| panic!("read a {rel}: {e}"));
        let fb = fs::read(b.path().join(rel)).unwrap_or_else(|e| panic!("read b {rel}: {e}"));
        assert_eq!(fa, fb, "{rel} differs across the two runs");
    }
}
