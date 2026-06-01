//! Task 0.13.4 gate: the `ckc replay` subcommand drives `replay::run_replay`
//! end to end through the compiled binary. The success path replays the
//! committed `run_manifest.json` golden (task 0.13.1) and exits zero with a
//! `Passed` `replay_report.json`; the bail path feeds a tampered manifest and
//! proves the binary persists the `Failed` diagnostic *and* exits non-zero —
//! `run_replay` writes the report before its mismatch `bail!`. The pure diff and
//! per-artifact determinism live in `tests/replay.rs`; this gate covers only the
//! clap wiring and the process exit code.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ckc_cli::manifest::RunManifest;
use ckc_cli::{ContentHash, ReplayReport, to_canonical_bytes};
use ckc_core::enums::ReplayStatus;

/// Repository root, two levels above this crate's manifest, so the committed
/// `schemas/golden/run_manifest.json` oracle resolves (as in replay.rs).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Path to the committed demo manifest golden (task 0.13.1).
fn committed_manifest_path() -> PathBuf {
    workspace_root().join("schemas/golden/run_manifest.json")
}

/// Deserialize `out_dir/replay_report.json`, asserting both its presence and its
/// shape — `fs::read` fails if the binary never persisted the diagnostic.
fn read_report(out_dir: &Path) -> ReplayReport {
    let bytes = fs::read(out_dir.join("replay_report.json")).expect("read replay_report.json");
    serde_json::from_slice(&bytes).expect("replay_report.json parses")
}

#[test]
fn ckc_replay_binary_passes_on_committed_manifest() {
    let out = tempfile::tempdir().expect("tempdir");
    let status = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .arg("replay")
        .arg(committed_manifest_path())
        .arg("--out")
        .arg(out.path())
        .status()
        .expect("spawn ckc binary");
    assert!(
        status.success(),
        "ckc replay over the golden exited non-zero"
    );
    assert_eq!(
        read_report(out.path()).status,
        ReplayStatus::Passed,
        "replay did not pass"
    );
}

#[test]
fn ckc_replay_binary_bails_on_tampered_manifest() {
    // Clone the committed golden and flip one entry's content_hash. The `command`
    // field is left untouched, so it still equals DEMO_COMMAND and clears the
    // command gate — the divergence is purely the tampered hash.
    let bytes = fs::read(committed_manifest_path()).expect("read run_manifest.json golden");
    let mut tampered: RunManifest =
        serde_json::from_slice(&bytes).expect("committed run_manifest.json parses");
    tampered.entries[0].content_hash = ContentHash("sha256:tampered".to_string());

    let manifest_dir = tempfile::tempdir().expect("manifest tempdir");
    let tampered_path = manifest_dir.path().join("run_manifest.json");
    fs::write(&tampered_path, to_canonical_bytes(&tampered)).expect("write tampered manifest");

    let out = tempfile::tempdir().expect("out tempdir");
    let status = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .arg("replay")
        .arg(&tampered_path)
        .arg("--out")
        .arg(out.path())
        .status()
        .expect("spawn ckc binary");
    assert!(
        !status.success(),
        "ckc replay over a tampered manifest must exit non-zero"
    );
    // run_replay persists the diagnostic before its mismatch bail!, so the Failed
    // report is on disk despite the non-zero exit.
    assert_eq!(
        read_report(out.path()).status,
        ReplayStatus::Failed,
        "tamper went undetected"
    );
}
