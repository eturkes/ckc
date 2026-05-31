//! Task 0.12.5 gate: the demo pipeline's report stage emits `report.json`,
//! byte-identical to the committed `examples/research_kernel/fixtures/report.json`
//! golden (task 0.12.4), and reproduces the same report `content_hash` across
//! independent runs. Per-field card assembly is locked in `ckc-report`'s own
//! golden; this gate covers the CLI emission and its cross-run determinism.

use std::fs;
use std::path::PathBuf;

use ckc_cli::pipeline::run_demo;

/// Repository root, two levels above this crate's manifest, so the committed
/// `examples/research_kernel/fixtures/report.json` golden resolves.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn run_demo_emits_report_matching_committed_golden() {
    let out = tempfile::tempdir().expect("tempdir");
    run_demo("research-kernel", false, out.path()).expect("run_demo");

    let emitted = out.path().join("report.json");
    assert!(
        emitted.exists(),
        "report.json not emitted by the demo pipeline"
    );
    let written = fs::read(&emitted).expect("read emitted report.json");
    let committed =
        fs::read(workspace_root().join("examples/research_kernel/fixtures/report.json"))
            .expect("read committed report.json golden");
    assert_eq!(
        written, committed,
        "emitted report.json drifted from the committed golden"
    );
}

#[test]
fn run_demo_report_hashes_identically_across_runs() {
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    let ma = run_demo("research-kernel", false, a.path()).expect("run a");
    let mb = run_demo("research-kernel", false, b.path()).expect("run b");

    let ha = ma
        .entries
        .iter()
        .find(|e| e.stage == "report")
        .expect("manifest a carries a report-stage entry");
    let hb = mb
        .entries
        .iter()
        .find(|e| e.stage == "report")
        .expect("manifest b carries a report-stage entry");
    assert_eq!(
        ha.content_hash, hb.content_hash,
        "report content_hash differs across independent runs"
    );
}
