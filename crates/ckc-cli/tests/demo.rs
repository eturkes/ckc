//! Task 0.11.6 gate: `pipeline::run_demo` orchestrates the full Phase-0
//! pipeline under a fresh output dir — compile ++ verify ++ conflicts ++
//! substrate ++ report — assembles the deterministic [`RunManifest`], writes
//! `run_manifest.json`, and (with `--replay`) re-runs and hash-compares a second
//! pass. Per-artifact byte identity lives in the per-stage gates
//! (compile/verify/conflicts/substrate/report); this gate covers orchestration:
//! stage composition, file presence, cross-location determinism, and the binary
//! path.

use std::fs;

use ckc_cli::manifest::RunManifest;
use ckc_cli::pipeline::{load_bundle, run_demo};
use ckc_cli::{
    conflict_manifest, content_hash, portfolio_manifest, verification_manifest, verify_all,
};

/// Substrate-stage entry count: `rdf_export`, `shacl_report`, `retrieval_result`,
/// and `egraph_equivalence` (SPEC 13.5). The substrate stage has no manifest
/// function (unlike compile/verify/conflicts); its count is locked by the
/// `run_substrate` gate, so this mirror is the expected contribution to the demo
/// manifest.
const SUBSTRATE_ENTRIES: usize = 4;

/// Report-stage entry count: the single `report.json` artifact. Like the
/// substrate stage, the report stage has no manifest function; its count is
/// locked by the `run_report` gate, so this mirror is its expected contribution
/// to the demo manifest (bringing the whole to 9+24+4+4+1 = 42).
const REPORT_ENTRIES: usize = 1;

#[test]
fn run_demo_assembles_manifest_over_all_stages() {
    let out = tempfile::tempdir().expect("tempdir");
    let manifest = run_demo("research-kernel", true, out.path()).expect("run_demo");

    // Per-stage entry counts sum to the whole, validating the five-stage
    // concatenation against each stage's own manifest (substrate via its mirror).
    let bundle = load_bundle("examples/research_kernel").expect("bundle");
    let report = verify_all(&bundle);
    let n_compile = portfolio_manifest(&bundle).0.len();
    let n_verify = verification_manifest(&bundle).0.len();
    let n_conflicts = conflict_manifest(&bundle, &report).0.len();
    let count = |stage: &str| manifest.entries.iter().filter(|e| e.stage == stage).count();
    assert_eq!(count("compile"), n_compile, "compile-stage entry count");
    assert_eq!(count("verify"), n_verify, "verify-stage entry count");
    assert_eq!(
        count("conflicts"),
        n_conflicts,
        "conflicts-stage entry count"
    );
    assert_eq!(
        count("substrate"),
        SUBSTRATE_ENTRIES,
        "substrate-stage entry count"
    );
    assert_eq!(count("report"), REPORT_ENTRIES, "report-stage entry count");
    assert_eq!(
        manifest.entries.len(),
        n_compile + n_verify + n_conflicts + SUBSTRATE_ENTRIES + REPORT_ENTRIES,
        "manifest length equals the sum of the five stage manifests"
    );

    // Every manifest entry's artifact is on disk under out_dir, plus the cvc5
    // proof (verify-stage evidence carrying no manifest entry) and the manifest.
    for e in &manifest.entries {
        assert!(
            out.path().join(&e.artifact_path).exists(),
            "missing emitted artifact {}",
            e.artifact_path
        );
    }
    assert!(
        out.path().join("certs/cvc5_norm_conflict.proof").exists(),
        "cvc5 proof evidence not emitted"
    );

    // run_manifest.json exists and reconstructs the returned manifest (no f64 —
    // exact round-trip), proving the written bytes are the in-memory manifest.
    let bytes = fs::read(out.path().join("run_manifest.json")).expect("read run_manifest.json");
    let on_disk: RunManifest = serde_json::from_slice(&bytes).expect("run_manifest.json parses");
    assert_eq!(
        on_disk, manifest,
        "written run_manifest.json drifted from the returned manifest"
    );
}

#[test]
fn run_demo_hashes_identically_across_tempdirs() {
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    let ma = run_demo("research-kernel", false, a.path()).expect("run a");
    let mb = run_demo("research-kernel", false, b.path()).expect("run b");
    assert_eq!(ma, mb, "manifest differs across output locations");
    assert_eq!(
        content_hash(&ma),
        content_hash(&mb),
        "manifest hash differs across output locations"
    );
}

#[test]
fn ckc_demo_binary_replays_and_writes_manifest() {
    let out = tempfile::tempdir().expect("tempdir");
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args(["demo", "research-kernel", "--replay", "--out"])
        .arg(out.path())
        .status()
        .expect("spawn ckc binary");
    assert!(
        status.success(),
        "ckc demo research-kernel --replay exited non-zero"
    );
    assert!(
        out.path().join("run_manifest.json").exists(),
        "binary did not write run_manifest.json"
    );
}
