//! Subtask 0.7.6: CAS persistence + cross-run determinism for
//! `RetrievalResult` artifacts.
//!
//! Pipeline mirror: this file reconstructs the build → search step from
//! `tests/pipeline.rs`. Test binaries do not share helper modules, so the
//! small duplication is intentional and keeps each test file
//! self-contained.
//!
//! Determinism strategy: mirrors `cas_manifest_hash_is_stable` in
//! `crates/ckc-store/tests/research_fixture_bundle.rs`. Raw `StoreManifest`
//! bytes carry per-file `stored_at_epoch` and drift across runs by design,
//! so the cross-run gate compares per-artifact `envelope_hash` /
//! `content_hash` instead of raw manifest bytes.
//!
//! Fixture snapshot: the committed
//! `examples/research_kernel/fixtures/retrieval_results.json` is a
//! pretty-printed dump of the pipeline output. The snapshot test loads it
//! at runtime (not via `include_str!`) so the regenerator test can rebuild
//! it from a fresh checkout where the file may not yet exist; rerun
//! `cargo test -p ckc-retrieve --test persistence -- --ignored \
//! regenerate_retrieval_results_fixture` after any intentional pipeline
//! change.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use ckc_core::canonical::{ContentHash, content_hash};
use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind, ArtifactMeta};
use ckc_core::source::SourceSpan;
use ckc_retrieve::{RetrievalQuery, RetrievalResult, SparseIndex, compute_index_fingerprint};
use ckc_store::ContentStore;
use serde_json::json;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Embedded inputs + fixture path
// ---------------------------------------------------------------------------

const TOY_SPANS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/spans.json");
const TOY_QUERIES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/queries.json");

/// Top-k matches the 0.7.5 pipeline so the produced `RetrievalResult` set
/// is byte-identical to what the rest of Phase-0 sees.
const TOP_K: usize = 5;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/research_kernel/fixtures/retrieval_results.json")
}

// ---------------------------------------------------------------------------
// Pipeline rebuild (mirror of tests/pipeline.rs)
// ---------------------------------------------------------------------------

fn load_spans_sorted() -> Vec<SourceSpan> {
    let mut spans: Vec<SourceSpan> =
        serde_json::from_str(TOY_SPANS_JSON).expect("toy spans.json must deserialize");
    spans.sort_by(|a, b| a.span_id.as_str().cmp(b.span_id.as_str()));
    spans
}

fn load_queries() -> Vec<RetrievalQuery> {
    serde_json::from_str(TOY_QUERIES_JSON).expect("toy queries.json must deserialize")
}

fn corpus_hash_for(spans: &[SourceSpan]) -> ContentHash {
    let mut sorted: Vec<&SourceSpan> = spans.iter().collect();
    sorted.sort_by(|a, b| a.span_id.as_str().cmp(b.span_id.as_str()));
    content_hash(&sorted)
}

fn run_pipeline() -> Vec<RetrievalResult> {
    let spans = load_spans_sorted();
    let queries = load_queries();
    let index = SparseIndex::build_from_spans(&spans).expect("build SparseIndex");
    let fingerprint = compute_index_fingerprint(&spans);
    let corpus = corpus_hash_for(&spans);
    queries
        .iter()
        .map(|q| {
            let hits = index
                .search(&q.query_text, TOP_K)
                .unwrap_or_else(|e| panic!("search failed for {}: {e}", q.query_id));
            RetrievalResult {
                query: q.clone(),
                hits,
                index_fingerprint: fingerprint.clone(),
                corpus_hash: corpus.clone(),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Envelope wrapping (stage = "retrieve")
// ---------------------------------------------------------------------------

/// Template matches `envelope_meta` in
/// `crates/ckc-store/tests/research_fixture_bundle.rs`: identical
/// pipeline-metadata fields, distinguished by `stage = "retrieve"`. The
/// placeholder `content_hash` is overwritten by `ArtifactEnvelope::wrap`.
fn meta_retrieve() -> ArtifactMeta {
    ArtifactMeta {
        schema_version: "0.0.0".into(),
        producer_version: "ckc-retrieve/0.0.0".into(),
        command_manifest: json!({"command": "ckc", "args": ["demo", "research-kernel"]}),
        source_input_hashes: vec![],
        parent_hashes: vec![],
        stage: "retrieve".into(),
        semantic_profiles: vec![],
        content_hash: ContentHash(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
        ),
        certificate_ids: vec![],
        replay_command: Some("ckc demo research-kernel --replay --out runs/research".into()),
    }
}

fn wrap_results(results: &[RetrievalResult]) -> Vec<ArtifactEnvelope> {
    results
        .iter()
        .map(|r| ArtifactEnvelope::wrap(ArtifactKind::RetrievalResult, r, meta_retrieve()))
        .collect()
}

fn build_store_and_verify(envelopes: &[ArtifactEnvelope]) -> (TempDir, Vec<ContentHash>) {
    let tmp = TempDir::new().expect("create tempdir");
    let store = ContentStore::new(tmp.path());
    let hashes = store.put_batch(envelopes).expect("put_batch");
    for h in &hashes {
        assert!(store.exists(h), "envelope {h:?} must exist after put_batch");
        assert!(
            store.verify(h).expect("verify call"),
            "envelope {h:?} must verify",
        );
    }
    (tmp, hashes)
}

// ---------------------------------------------------------------------------
// Gate 1: every RetrievalResult is stored, verified, and content-hashed
// ---------------------------------------------------------------------------

#[test]
fn each_result_is_stored_verified_and_content_hashed() {
    let results = run_pipeline();
    let envelopes = wrap_results(&results);
    assert_eq!(envelopes.len(), results.len());

    let (_tmp, hashes) = build_store_and_verify(&envelopes);

    // Inner content_hash equals canonical hash of the RetrievalResult itself
    // — i.e. `wrap` correctly overwrote the placeholder hash with the
    // payload's true canonical digest.
    for (env, r) in envelopes.iter().zip(&results) {
        assert_eq!(
            env.meta.content_hash,
            content_hash(r),
            "envelope content_hash must match the RetrievalResult canonical hash for {}",
            r.query.query_id,
        );
    }

    // Distinct results yield distinct envelope hashes; catches accidental
    // payload aliasing (e.g. a stale clone losing the per-query hits).
    let unique: HashSet<&ContentHash> = hashes.iter().collect();
    assert_eq!(
        unique.len(),
        hashes.len(),
        "every RetrievalResult envelope must have a distinct store key",
    );
}

// ---------------------------------------------------------------------------
// Gate 2: per-artifact hashes match across two fully independent runs
// ---------------------------------------------------------------------------

#[test]
fn per_result_hashes_match_across_two_independent_runs() {
    let results1 = run_pipeline();
    let envelopes1 = wrap_results(&results1);
    let (_t1, hashes1) = build_store_and_verify(&envelopes1);

    let results2 = run_pipeline();
    let envelopes2 = wrap_results(&results2);
    let (_t2, hashes2) = build_store_and_verify(&envelopes2);

    assert_eq!(
        results1, results2,
        "pipeline output must be deterministic across runs",
    );
    for (e1, e2) in envelopes1.iter().zip(&envelopes2) {
        assert_eq!(
            e1.envelope_hash(),
            e2.envelope_hash(),
            "envelope_hash must be stable across runs for {}",
            e1.payload
                .get("query")
                .and_then(|q| q.get("query_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("?"),
        );
        assert_eq!(
            e1.meta.content_hash, e2.meta.content_hash,
            "inner content_hash must be stable across runs",
        );
    }
    assert_eq!(
        hashes1, hashes2,
        "per-result store keys must be identical across runs",
    );
}

// ---------------------------------------------------------------------------
// Gate 3: committed fixture deserializes and structurally matches the
// live pipeline (one result per query id; live ranks/scores not asserted)
// ---------------------------------------------------------------------------
//
// Why structural rather than struct-equality: serde_json's f64 deserializer
// uses a different decimal→f64 algorithm than its ryu-based serializer, and
// the two disagree at certain ULP boundaries. Concretely, the BM25 score
// `19.212745666503906_f64` (bits `4033367680000000`) prints as
// `"19.212745666503906"` via ryu, but `serde_json::from_str` re-parses that
// literal to `19.212745666503903_f64` (bits `403336767fffffff`, the
// neighboring f64). Round-trip fails by 1 ULP for some values. The
// per-run/per-artifact `content_hash` equality is enforced by Gate 2 in
// memory, which is the gate that actually matters for replay; on-disk
// strict struct equality would just relitigate this f64-parser asymmetry.

#[test]
fn committed_fixture_deserializes_and_covers_every_query() {
    let live = run_pipeline();
    let path = fixture_path();
    let on_disk_bytes = fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "retrieval_results.json must exist at {}: {e}; regenerate with \
             `cargo test -p ckc-retrieve --test persistence -- --ignored \
             regenerate_retrieval_results_fixture`",
            path.display(),
        )
    });
    let from_disk: Vec<RetrievalResult> = serde_json::from_slice(&on_disk_bytes).expect(
        "committed retrieval_results.json must deserialize as Vec<RetrievalResult>; \
         regenerate via `cargo test -p ckc-retrieve --test persistence -- --ignored \
         regenerate_retrieval_results_fixture` after model changes",
    );

    let live_qids: HashSet<&str> = live.iter().map(|r| r.query.query_id.as_str()).collect();
    let disk_qids: HashSet<&str> = from_disk
        .iter()
        .map(|r| r.query.query_id.as_str())
        .collect();
    assert_eq!(
        live_qids, disk_qids,
        "committed fixture must cover the same query set as the live pipeline",
    );

    // Index/corpus identity asserts the fixture was produced over the same
    // corpus snapshot the live pipeline reports. If span fixtures change,
    // both the live pipeline and the on-disk fingerprint shift in lockstep
    // and this stays green; if only one shifts, this fires.
    for live_r in &live {
        let disk_r = from_disk
            .iter()
            .find(|d| d.query.query_id == live_r.query.query_id)
            .expect("query id present in both");
        assert_eq!(
            live_r.index_fingerprint, disk_r.index_fingerprint,
            "index_fingerprint drift for {}",
            live_r.query.query_id,
        );
        assert_eq!(
            live_r.corpus_hash, disk_r.corpus_hash,
            "corpus_hash drift for {}",
            live_r.query.query_id,
        );
        // Per-hit span ids agree: this catches retrieval-order drift without
        // depending on f64 score round-trip.
        let live_ids: Vec<&str> = live_r.hits.iter().map(|h| h.span_id.as_str()).collect();
        let disk_ids: Vec<&str> = disk_r.hits.iter().map(|h| h.span_id.as_str()).collect();
        assert_eq!(
            live_ids, disk_ids,
            "ranked span_id sequence drift for {}",
            live_r.query.query_id,
        );
    }
}

// ---------------------------------------------------------------------------
// Regenerator (ignored by default; run with --ignored after intentional
// pipeline changes)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "regenerate-only; refreshes examples/research_kernel/fixtures/retrieval_results.json"]
fn regenerate_retrieval_results_fixture() {
    let live = run_pipeline();
    let mut json = serde_json::to_string_pretty(&live).expect("serialize Vec<RetrievalResult>");
    json.push('\n');
    fs::write(fixture_path(), json).expect("write retrieval_results.json");
}
