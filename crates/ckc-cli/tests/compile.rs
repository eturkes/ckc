//! Task 0.11.2 gate: `pipeline::run_compile` emits the nine-target SPEC-14
//! portfolio under a fresh output dir — every artifact byte-identical to its
//! committed `logic/*` + `lean/Ckc/*` counterpart, every manifest entry carrying
//! the `portfolio_manifest` content hash in order, and two runs byte-identical.

use std::fs;
use std::path::PathBuf;

use ckc_cli::pipeline::{load_bundle, run_compile};
use ckc_cli::{ARTIFACT_PATHS, portfolio_manifest};

/// Repository root, two levels above this crate's manifest, so the committed
/// target-artifact files resolve under their `logic/`/`lean/` paths.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn run_compile_emits_nine_files_matching_committed_artifacts() {
    let bundle = load_bundle("examples/research_kernel").expect("load toy bundle");
    let out = tempfile::tempdir().expect("tempdir");
    let entries = run_compile(&bundle, out.path()).expect("run_compile");

    assert_eq!(
        entries.len(),
        ARTIFACT_PATHS.len(),
        "one manifest entry per emitted target"
    );

    let root = workspace_root();
    let manifest = portfolio_manifest(&bundle);
    for (i, entry) in entries.iter().enumerate() {
        let rel = ARTIFACT_PATHS[i];
        assert_eq!(entry.stage, "compile");
        assert_eq!(entry.artifact_kind, "compiled_target");
        assert_eq!(entry.artifact_path, rel);
        assert_eq!(
            entry.content_hash, manifest.0[i].content_hash,
            "entry {i} hash equals the portfolio-manifest hash, in order"
        );
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
fn run_compile_is_deterministic_across_runs() {
    let bundle = load_bundle("examples/research_kernel").expect("load toy bundle");
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    let ea = run_compile(&bundle, a.path()).expect("run a");
    let eb = run_compile(&bundle, b.path()).expect("run b");
    assert_eq!(ea, eb, "manifest entries identical across runs");
    for rel in ARTIFACT_PATHS {
        let fa = fs::read(a.path().join(rel)).unwrap_or_else(|e| panic!("read a {rel}: {e}"));
        let fb = fs::read(b.path().join(rel)).unwrap_or_else(|e| panic!("read b {rel}: {e}"));
        assert_eq!(fa, fb, "{rel} differs across the two runs");
    }
}
