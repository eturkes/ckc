//! Task 0.11.5 gate: `pipeline::run_substrate` emits the Phase-0 substrate
//! artifacts under a fresh output dir — the SKOS/Turtle terminology export, the
//! SHACL rule report, and the sparse-retrieval results — with three
//! `substrate`-stage manifest entries and cross-run determinism.
//!
//! Comparison modes mirror how each committed golden is stored. `terminology.ttl`
//! is raw Turtle and byte-identical. The JSON artifacts are emitted canonical
//! (SPEC 5.2, matching the verify stage) while the committed goldens are
//! pretty-printed for readability, so they are compared structurally
//! (deserialize-and-equal) — the same approach `ckc-retrieve/tests/persistence.rs`
//! takes for `retrieval_results.json`. The retrieval comparison stays structural
//! (query set + per-hit `span_id` order + fingerprints) rather than re-hashing
//! the deserialized snapshot, sidestepping the serde_json f64 round-trip
//! asymmetry; cross-run `content_hash` stability is the determinism gate below.

use std::fs;
use std::path::PathBuf;

use ckc_cli::pipeline::{load_bundle, run_substrate};
use ckc_cli::{RetrievalResult, ShaclReport, content_hash, validate_rules};

/// Repository root, two levels above this crate's manifest, so the committed
/// `examples/research_kernel/fixtures/*` goldens resolve.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn run_substrate_emits_terminology_shacl_retrieval() {
    let bundle = load_bundle("examples/research_kernel").expect("load toy bundle");
    let out = tempfile::tempdir().expect("tempdir");
    let entries = run_substrate(&bundle, out.path()).expect("run_substrate");
    let root = workspace_root();

    // Three substrate entries in rdf -> shacl -> retrieval order.
    assert_eq!(entries.len(), 3);
    let kinds: Vec<&str> = entries.iter().map(|e| e.artifact_kind.as_str()).collect();
    assert_eq!(kinds, ["rdf_export", "shacl_report", "retrieval_result"]);
    let paths: Vec<&str> = entries.iter().map(|e| e.artifact_path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "terminology.ttl",
            "shacl_report.json",
            "retrieval/retrieval_results.json",
        ]
    );
    for e in &entries {
        assert_eq!(e.stage, "substrate");
    }

    // (a) terminology.ttl: raw Turtle, byte-identical to the committed export.
    let ttl = fs::read(out.path().join("terminology.ttl")).expect("read emitted ttl");
    let committed_ttl = fs::read(root.join("examples/research_kernel/fixtures/terminology.ttl"))
        .expect("read committed ttl");
    assert_eq!(
        ttl, committed_ttl,
        "terminology.ttl drifted from committed export"
    );
    let ttl_str = String::from_utf8(ttl).expect("ttl is utf-8");
    assert_eq!(entries[0].content_hash, content_hash(&ttl_str));

    // (b) shacl_report.json: canonical on disk; structural compare to the
    // pretty committed golden and to a fresh in-memory recompute, plus the
    // entry hash (no f64 — exact round-trip).
    let shacl_bytes = fs::read(out.path().join("shacl_report.json")).expect("read emitted shacl");
    let emitted: ShaclReport = serde_json::from_slice(&shacl_bytes).expect("emitted shacl parses");
    let committed: ShaclReport = serde_json::from_slice(
        &fs::read(root.join("examples/research_kernel/fixtures/shacl_report.json"))
            .expect("read committed shacl"),
    )
    .expect("committed shacl parses");
    let expected = validate_rules(&bundle.rules);
    assert_eq!(
        emitted, expected,
        "emitted SHACL report drifted from validate_rules"
    );
    assert_eq!(
        emitted, committed,
        "emitted SHACL report drifted from committed golden"
    );
    assert_eq!(entries[1].content_hash, content_hash(&expected));

    // (c) retrieval_results.json: canonical on disk; structural compare to the
    // committed snapshot (query set + per-hit span_id order + index/corpus
    // hashes), avoiding the serde_json f64 round-trip asymmetry.
    let retr_bytes = fs::read(out.path().join("retrieval/retrieval_results.json"))
        .expect("read emitted retrieval");
    let emitted: Vec<RetrievalResult> =
        serde_json::from_slice(&retr_bytes).expect("emitted retrieval parses");
    let committed: Vec<RetrievalResult> = serde_json::from_slice(
        &fs::read(root.join("examples/research_kernel/fixtures/retrieval_results.json"))
            .expect("read committed retrieval"),
    )
    .expect("committed retrieval parses");
    assert_eq!(emitted.len(), committed.len(), "query count drift");
    for e in &emitted {
        let c = committed
            .iter()
            .find(|c| c.query.query_id == e.query.query_id)
            .unwrap_or_else(|| {
                panic!(
                    "query {} present in committed snapshot",
                    e.query.query_id.as_str()
                )
            });
        let e_ids: Vec<&str> = e.hits.iter().map(|h| h.span_id.as_str()).collect();
        let c_ids: Vec<&str> = c.hits.iter().map(|h| h.span_id.as_str()).collect();
        assert_eq!(
            e_ids,
            c_ids,
            "ranked span_id drift for {}",
            e.query.query_id.as_str()
        );
        assert_eq!(
            e.index_fingerprint,
            c.index_fingerprint,
            "index_fingerprint drift for {}",
            e.query.query_id.as_str()
        );
        assert_eq!(
            e.corpus_hash,
            c.corpus_hash,
            "corpus_hash drift for {}",
            e.query.query_id.as_str()
        );
    }
}

#[test]
fn run_substrate_is_deterministic_across_runs() {
    let bundle = load_bundle("examples/research_kernel").expect("load toy bundle");
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    let ea = run_substrate(&bundle, a.path()).expect("run a");
    let eb = run_substrate(&bundle, b.path()).expect("run b");
    assert_eq!(ea, eb, "manifest entries identical across runs");
    for entry in &ea {
        let rel = &entry.artifact_path;
        let fa = fs::read(a.path().join(rel)).unwrap_or_else(|e| panic!("read a {rel}: {e}"));
        let fb = fs::read(b.path().join(rel)).unwrap_or_else(|e| panic!("read b {rel}: {e}"));
        assert_eq!(fa, fb, "{rel} differs across the two runs");
    }
}
