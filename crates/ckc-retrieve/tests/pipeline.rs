//! Subtask 0.7.5: in-memory retrieval pipeline + evaluation over the
//! Phase-0 toy corpus.
//!
//! Builds the Tantivy + Lindera sparse index from `spans.json`, runs each
//! `queries.json` query through `SparseIndex::search` (top_k=5), assembles
//! a [`RetrievalResult`] per query, and evaluates the ranked hits against
//! `qrels.json` using the 0.7.4 metric module (Recall@{1,3,5}, reciprocal
//! rank, nDCG@5). Per-query rank assertions and aggregate MRR guard the
//! retrieval-quality acceptance gate.
//!
//! Determinism note: spans are sorted by `span_id` before indexing so that
//! Tantivy's tie-breaking-by-ascending-doc-id behaviour produces a stable
//! ranking across runs (see `crates/ckc-retrieve/src/sparse.rs` docs).

use std::collections::{HashMap, HashSet};

use ckc_core::canonical::{ContentHash, content_hash};
use ckc_core::id::{QueryId, SpanId};
use ckc_core::source::SourceSpan;
use ckc_retrieve::{
    QrelJudgment, RetrievalHit, RetrievalQuery, RetrievalResult, SparseIndex,
    compute_index_fingerprint, ndcg_at_k, recall_at_k, reciprocal_rank,
};

// ---------------------------------------------------------------------------
// Embedded Phase-0 fixtures (single source of truth lives in the fixture
// files; the pipeline test never mutates them).
// ---------------------------------------------------------------------------

const TOY_SPANS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/spans.json");
const TOY_QUERIES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/queries.json");
const TOY_QRELS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/qrels.json");

fn load_spans_sorted() -> Vec<SourceSpan> {
    let mut spans: Vec<SourceSpan> =
        serde_json::from_str(TOY_SPANS_JSON).expect("toy spans.json must deserialize");
    spans.sort_by(|a, b| a.span_id.as_str().cmp(b.span_id.as_str()));
    spans
}

fn load_queries() -> Vec<RetrievalQuery> {
    serde_json::from_str(TOY_QUERIES_JSON).expect("toy queries.json must deserialize")
}

fn load_qrels() -> Vec<QrelJudgment> {
    serde_json::from_str(TOY_QRELS_JSON).expect("toy qrels.json must deserialize")
}

/// Corpus content hash over the span set.
///
/// Phase-0 has no project-wide "corpus hash" helper (verified by `grep -rn
/// 'corpus_hash' crates/`), so per the 0.7.5 roadmap directive we compute
/// `content_hash` over the span vec sorted by `span_id`. Sorting before
/// hashing keeps the value invariant under insertion order, mirroring the
/// determinism guarantee of [`compute_index_fingerprint`].
fn corpus_hash_for(spans: &[SourceSpan]) -> ContentHash {
    let mut sorted: Vec<&SourceSpan> = spans.iter().collect();
    sorted.sort_by(|a, b| a.span_id.as_str().cmp(b.span_id.as_str()));
    content_hash(&sorted)
}

// ---------------------------------------------------------------------------
// Qrel pivots: per-query relevant set (binary, grade > 0) and graded map
// (raw grades) used by the metric functions.
// ---------------------------------------------------------------------------

struct QrelIndex {
    relevant: HashMap<QueryId, HashSet<SpanId>>,
    graded: HashMap<QueryId, HashMap<SpanId, u32>>,
}

fn build_qrel_index(qrels: &[QrelJudgment]) -> QrelIndex {
    let mut relevant: HashMap<QueryId, HashSet<SpanId>> = HashMap::new();
    let mut graded: HashMap<QueryId, HashMap<SpanId, u32>> = HashMap::new();
    for j in qrels {
        graded
            .entry(j.query_id.clone())
            .or_default()
            .insert(j.span_id.clone(), j.relevance);
        if j.relevance > 0 {
            relevant
                .entry(j.query_id.clone())
                .or_default()
                .insert(j.span_id.clone());
        }
    }
    QrelIndex { relevant, graded }
}

// ---------------------------------------------------------------------------
// Pipeline runner: build index, search each query, return a RetrievalResult
// per query. Used by every test in this file.
// ---------------------------------------------------------------------------

const TOP_K: usize = 5;

fn run_pipeline() -> (Vec<SourceSpan>, Vec<RetrievalQuery>, Vec<RetrievalResult>) {
    let spans = load_spans_sorted();
    let queries = load_queries();

    let index = SparseIndex::build_from_spans(&spans).expect("build SparseIndex");
    let fingerprint = compute_index_fingerprint(&spans);
    // Sanity: the index reports the same fingerprint as the free helper
    // (the sparse_index suite already covers this directly; doubling up
    // here keeps the pipeline self-checking.)
    assert_eq!(index.fingerprint(), &fingerprint);
    let corpus = corpus_hash_for(&spans);

    let results: Vec<RetrievalResult> = queries
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
        .collect();

    (spans, queries, results)
}

fn find_result<'a>(results: &'a [RetrievalResult], query_id: &str) -> &'a RetrievalResult {
    results
        .iter()
        .find(|r| r.query.query_id.as_str() == query_id)
        .unwrap_or_else(|| panic!("pipeline produced no result for {query_id}"))
}

fn top_span_ids(hits: &[RetrievalHit], k: usize) -> Vec<&str> {
    hits.iter().take(k).map(|h| h.span_id.as_str()).collect()
}

// ---------------------------------------------------------------------------
// Per-query rank assertions (gate-mandated).
// ---------------------------------------------------------------------------

#[test]
fn q_sepsis_bl_returns_recommendation_in_top_3() {
    let (_, _, results) = run_pipeline();
    let r = find_result(&results, "q_sepsis_bl");
    let top3 = top_span_ids(&r.hits, 3);
    assert!(
        top3.contains(&"span_rec_sepsis_bl"),
        "expected span_rec_sepsis_bl in top-3 for q_sepsis_bl; got {top3:?}",
    );
}

#[test]
fn q_contra_allergy_returns_contraindication_in_top_3() {
    let (_, _, results) = run_pipeline();
    let r = find_result(&results, "q_contra_allergy");
    let top3 = top_span_ids(&r.hits, 3);
    assert!(
        top3.contains(&"span_contra_bl_allergy"),
        "expected span_contra_bl_allergy in top-3 for q_contra_allergy; got {top3:?}",
    );
}

#[test]
fn q_vitals_temp_returns_a_vitals_cell_in_top_5() {
    let (_, _, results) = run_pipeline();
    let r = find_result(&results, "q_vitals_temp");
    let top5 = top_span_ids(&r.hits, 5);
    let vitals_cell_ids = ["span_cell_r0c0", "span_cell_r1c0"];
    assert!(
        vitals_cell_ids.iter().any(|id| top5.contains(id)),
        "expected a vitals temperature cell (span_cell_r0c0 or span_cell_r1c0) in top-5 \
         for q_vitals_temp; got {top5:?}",
    );
}

// ---------------------------------------------------------------------------
// Aggregate metrics (gate-mandated MRR > 0.5).
// ---------------------------------------------------------------------------

/// Per-query reciprocal rank, summary statistics, and aggregate MRR.
struct AggregateMetrics {
    per_query_rr: Vec<(String, f64)>,
    mrr: f64,
    mean_recall_at_1: f64,
    mean_recall_at_3: f64,
    mean_recall_at_5: f64,
    mean_ndcg_at_5: f64,
}

fn evaluate(results: &[RetrievalResult], qrel: &QrelIndex) -> AggregateMetrics {
    let mut rrs: Vec<(String, f64)> = Vec::new();
    let mut recall_1 = Vec::new();
    let mut recall_3 = Vec::new();
    let mut recall_5 = Vec::new();
    let mut ndcg_5 = Vec::new();

    for r in results {
        let qid = &r.query.query_id;
        let empty_rel: HashSet<SpanId> = HashSet::new();
        let empty_graded: HashMap<SpanId, u32> = HashMap::new();
        let rel = qrel.relevant.get(qid).unwrap_or(&empty_rel);
        let graded = qrel.graded.get(qid).unwrap_or(&empty_graded);

        rrs.push((qid.as_str().to_owned(), reciprocal_rank(&r.hits, rel)));
        recall_1.push(recall_at_k(&r.hits, rel, 1));
        recall_3.push(recall_at_k(&r.hits, rel, 3));
        recall_5.push(recall_at_k(&r.hits, rel, 5));
        ndcg_5.push(ndcg_at_k(&r.hits, graded, 5));
    }

    let mean = |xs: &[f64]| -> f64 {
        if xs.is_empty() {
            0.0
        } else {
            xs.iter().sum::<f64>() / (xs.len() as f64)
        }
    };

    let mrr = mean(&rrs.iter().map(|(_, v)| *v).collect::<Vec<_>>());
    AggregateMetrics {
        per_query_rr: rrs,
        mrr,
        mean_recall_at_1: mean(&recall_1),
        mean_recall_at_3: mean(&recall_3),
        mean_recall_at_5: mean(&recall_5),
        mean_ndcg_at_5: mean(&ndcg_5),
    }
}

#[test]
fn aggregate_mrr_exceeds_threshold() {
    let (_, _, results) = run_pipeline();
    let qrel = build_qrel_index(&load_qrels());
    let m = evaluate(&results, &qrel);

    // Aggregate MRR > 0.5 means the average first-relevant rank across all
    // queries is better than 2. Print per-query RR on failure so a future
    // session can see which query regressed without rerunning manually.
    assert!(
        m.mrr > 0.5,
        "aggregate MRR must exceed 0.5; got {:.4}. per-query RR: {:?}",
        m.mrr,
        m.per_query_rr,
    );
}

#[test]
fn aggregate_metrics_are_within_valid_ranges() {
    // Sanity: every aggregate metric is a finite probability/score in [0, 1].
    // Failure here would indicate a metric implementation regression rather
    // than a retrieval-quality regression.
    let (_, _, results) = run_pipeline();
    let qrel = build_qrel_index(&load_qrels());
    let m = evaluate(&results, &qrel);

    for (name, v) in [
        ("MRR", m.mrr),
        ("R@1", m.mean_recall_at_1),
        ("R@3", m.mean_recall_at_3),
        ("R@5", m.mean_recall_at_5),
        ("nDCG@5", m.mean_ndcg_at_5),
    ] {
        assert!(
            v.is_finite() && (0.0..=1.0).contains(&v),
            "{name} out of [0,1] range or non-finite: {v}",
        );
    }
    // Monotonicity: recall is non-decreasing in k.
    assert!(m.mean_recall_at_1 <= m.mean_recall_at_3 + 1e-12);
    assert!(m.mean_recall_at_3 <= m.mean_recall_at_5 + 1e-12);
}

// ---------------------------------------------------------------------------
// Structural assertions on the assembled RetrievalResult set.
// ---------------------------------------------------------------------------

#[test]
fn pipeline_returns_one_result_per_query() {
    let (_, queries, results) = run_pipeline();
    assert_eq!(
        results.len(),
        queries.len(),
        "expected exactly one RetrievalResult per query",
    );

    let result_qids: HashSet<&str> = results.iter().map(|r| r.query.query_id.as_str()).collect();
    let query_qids: HashSet<&str> = queries.iter().map(|q| q.query_id.as_str()).collect();
    assert_eq!(result_qids, query_qids);
}

#[test]
fn every_result_carries_the_shared_corpus_identity() {
    // The pipeline indexes one corpus once, so every RetrievalResult must
    // carry the same index_fingerprint and corpus_hash. A drift here would
    // mean the search loop accidentally re-derived these per query.
    let (spans, _, results) = run_pipeline();
    let expected_fp = compute_index_fingerprint(&spans);
    let expected_corpus = corpus_hash_for(&spans);

    for r in &results {
        assert_eq!(
            r.index_fingerprint, expected_fp,
            "RetrievalResult for {} drifted from the shared index_fingerprint",
            r.query.query_id,
        );
        assert_eq!(
            r.corpus_hash, expected_corpus,
            "RetrievalResult for {} drifted from the shared corpus_hash",
            r.query.query_id,
        );
    }
}

#[test]
fn every_result_has_at_most_top_k_hits_with_contiguous_ranks() {
    let (_, _, results) = run_pipeline();
    for r in &results {
        assert!(
            r.hits.len() <= TOP_K,
            "result for {} returned more than top_k hits: {}",
            r.query.query_id,
            r.hits.len(),
        );
        for (i, h) in r.hits.iter().enumerate() {
            assert_eq!(
                h.rank as usize,
                i + 1,
                "rank must be 1-indexed and contiguous for {}",
                r.query.query_id,
            );
        }
    }
}
