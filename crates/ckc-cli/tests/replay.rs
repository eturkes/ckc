//! Task 0.13.3 gate: `compare_manifests` is the pure manifest diff and
//! `run_replay` drives `ckc replay` — re-derive the Phase-0 pipeline, diff it
//! against a committed `RunManifest`, persist the `ReplayReport`, and fail on any
//! divergence. The positive path replays the committed `run_manifest.json` golden
//! (task 0.13.1) and expects a byte-for-byte `Passed`; the negative path proves a
//! single tampered `content_hash` surfaces as exactly one mismatch; the
//! determinism path proves two replays into separate tempdirs report identically.

use std::fs;
use std::path::PathBuf;

use ckc_cli::ContentHash;
use ckc_cli::manifest::RunManifest;
use ckc_cli::replay::{compare_manifests, run_replay};
use ckc_core::enums::ReplayStatus;

/// Repository root, two levels above this crate's manifest, so the committed
/// `schemas/golden/run_manifest.json` oracle resolves (as in report.rs).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Path to the committed demo manifest golden (task 0.13.1) — the oracle a fresh
/// `ckc replay` run is compared against.
fn committed_manifest_path() -> PathBuf {
    workspace_root().join("schemas/golden/run_manifest.json")
}

/// The committed demo manifest, deserialized for the pure-diff tests.
fn committed_manifest() -> RunManifest {
    let bytes = fs::read(committed_manifest_path()).expect("read run_manifest.json golden");
    serde_json::from_slice(&bytes).expect("committed run_manifest.json parses")
}

#[test]
fn run_replay_reproduces_committed_manifest() {
    let out = tempfile::tempdir().expect("tempdir");
    let report =
        run_replay(&committed_manifest_path(), out.path()).expect("run_replay over the golden");

    assert_eq!(report.status, ReplayStatus::Passed, "replay did not pass");
    assert!(report.mismatches.is_empty(), "unexpected mismatches");
    assert_eq!(report.n_entries, 42, "expected all 42 committed entries");
    assert_eq!(
        report.n_matched, report.n_entries,
        "not every entry reproduced byte-for-byte"
    );
    assert!(
        out.path().join("replay_report.json").exists(),
        "replay_report.json not persisted"
    );
}

#[test]
fn compare_manifests_flags_single_tampered_entry() {
    let committed = committed_manifest();
    let mut tampered = committed.clone();
    let target_path = tampered.entries[0].artifact_path.clone();
    // A real content hash is `sha256:<64 hex>`, so this sentinel cannot collide.
    tampered.entries[0].content_hash = ContentHash("sha256:tampered".to_string());

    let report = compare_manifests(&committed, &tampered);
    assert_eq!(
        report.status,
        ReplayStatus::Failed,
        "tamper went undetected"
    );
    assert_eq!(report.mismatches.len(), 1, "expected exactly one mismatch");
    let m = &report.mismatches[0];
    assert_eq!(m.artifact_path, target_path, "mismatch on the wrong path");
    assert_ne!(
        m.expected_hash, m.actual_hash,
        "a tampered entry must carry diverging hashes"
    );
    assert!(
        m.expected_hash.is_some() && m.actual_hash.is_some(),
        "both sides present for a tampered (not missing) entry"
    );
}

#[test]
fn run_replay_is_deterministic_across_tempdirs() {
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    let ra = run_replay(&committed_manifest_path(), a.path()).expect("replay a");
    let rb = run_replay(&committed_manifest_path(), b.path()).expect("replay b");
    assert_eq!(ra.status, ReplayStatus::Passed);
    assert_eq!(rb.status, ReplayStatus::Passed);
    assert_eq!(ra, rb, "replay reports differ across output locations");
}
