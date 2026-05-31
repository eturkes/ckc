//! Gate for task 0.12.4: one byte-golden over the assembled Phase-0
//! `report.json` locks the full bilingual report (SPEC 21, 23) — the summary
//! tallies plus every SPEC-21 conflict card — to its committed canonical bytes.
//! `Report` composes already-normalized inputs, so it is compared only by
//! canonical bytes and `content_hash`, never a deserialize -> `PartialEq`
//! roundtrip: serde_json's f64 parse/format asymmetry (agent-memory 2026-05-29)
//! can break that at ULP boundaries inside the embedded witness/normalized_view
//! `serde_json::Value`s. Regenerate with
//! `cargo test -p ckc-report --test golden -- --ignored regenerate`.

use std::path::PathBuf;

use ckc_core::canonical::{content_hash, to_canonical_bytes};
use ckc_report::assemble_toy_report;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn report_golden_path() -> PathBuf {
    workspace_root()
        .join("examples")
        .join("research_kernel")
        .join("fixtures")
        .join("report.json")
}

#[test]
fn report_matches_committed_golden() {
    let bytes = to_canonical_bytes(&assemble_toy_report());
    let path = report_golden_path();
    let golden =
        std::fs::read(&path).unwrap_or_else(|e| panic!("read golden {}: {e}", path.display()));
    assert!(
        bytes == golden,
        "canonical bytes mismatch for report.json (got {} bytes, golden {} bytes)",
        bytes.len(),
        golden.len()
    );
}

#[test]
fn report_structure_is_sane() {
    let report = assemble_toy_report();
    assert_eq!(report.conflict_cards.len(), 3);
    assert_eq!(report.summary.n_conflicts, 3);
}

#[test]
fn report_assembly_is_deterministic() {
    let first = assemble_toy_report();
    let second = assemble_toy_report();
    assert_eq!(content_hash(&first), content_hash(&second));
    assert_eq!(to_canonical_bytes(&first), to_canonical_bytes(&second));
}

#[test]
#[ignore]
fn regenerate() {
    std::fs::write(
        report_golden_path(),
        to_canonical_bytes(&assemble_toy_report()),
    )
    .unwrap();
}
