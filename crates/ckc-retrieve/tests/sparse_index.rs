//! Subtask 0.7.2 acceptance tests for the Tantivy + Lindera (IPADIC) sparse
//! index, exercised against the Phase-0 toy span fixture.
//!
//! The fixture is loaded via `include_str!` to make the test self-contained
//! and to keep it compiled-in even when the workspace cwd shifts. The 16
//! toy spans (sepsis recommendation, β-lactam allergy contraindication,
//! vital-sign table cells, etc.) are documented in
//! `examples/research_kernel/fixtures/spans.json` and built by the
//! `toy_source_corpus` regen test in `ckc-core`.

use ckc_core::source::SourceSpan;
use ckc_retrieve::{RetrievalHit, SparseIndex, compute_index_fingerprint, ipadic_tokens};

const TOY_SPANS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/spans.json");

fn load_toy_spans() -> Vec<SourceSpan> {
    serde_json::from_str(TOY_SPANS_JSON).expect("toy spans.json must deserialize")
}

fn span_ids(hits: &[RetrievalHit]) -> Vec<&str> {
    hits.iter().map(|h| h.span_id.as_str()).collect()
}

// ---------------------------------------------------------------------------
// Index construction
// ---------------------------------------------------------------------------

#[test]
fn index_builds_over_all_toy_spans() {
    let spans = load_toy_spans();
    // The Phase-0 fixture currently has 16 spans; the test should still pass
    // if that count drifts up or down, so we only require a non-trivial
    // corpus and that every span is indexable.
    assert!(
        spans.len() >= 12,
        "toy span fixture should be non-trivial; got {}",
        spans.len()
    );

    let index = SparseIndex::build_from_spans(&spans).expect("build SparseIndex");
    let fp = index.fingerprint().as_str();
    assert!(
        fp.starts_with("sha256:") && fp.len() == 7 + 64,
        "fingerprint must be sha256:<64-hex>; got {fp}",
    );
}

// ---------------------------------------------------------------------------
// Query: sepsis + β-lactam recommendation
// ---------------------------------------------------------------------------

#[test]
fn query_sepsis_betalactam_administration_ranks_recommendation_in_top_3() {
    let spans = load_toy_spans();
    let index = SparseIndex::build_from_spans(&spans).expect("build SparseIndex");

    let hits = index
        .search("敗血症 βラクタム 投与", 3)
        .expect("search executes");
    assert!(!hits.is_empty(), "expected non-empty hits");

    let ids = span_ids(&hits);
    assert!(
        ids.contains(&"span_rec_sepsis_bl"),
        "expected span_rec_sepsis_bl in top-3 for '敗血症 βラクタム 投与'; got {ids:?}",
    );

    // Rank/score sanity: results are in score-descending order and rank is
    // 1-indexed contiguous.
    for (i, h) in hits.iter().enumerate() {
        assert_eq!(
            h.rank as usize,
            i + 1,
            "rank must be 1-indexed and contiguous"
        );
    }
    let scores: Vec<f64> = hits.iter().map(|h| h.score).collect();
    let mut descending = scores.clone();
    descending.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert_eq!(scores, descending, "hits must be score-descending");
}

// ---------------------------------------------------------------------------
// Query: 体温 retrieves vital-sign table-cell spans
// ---------------------------------------------------------------------------

#[test]
fn query_taion_retrieves_vitals_temperature_cells() {
    let spans = load_toy_spans();
    let index = SparseIndex::build_from_spans(&spans).expect("build SparseIndex");

    // "体温" appears in two vital-sign table cells: span_cell_r0c0 ("体温 ≧
    // 38.0℃") and span_cell_r1c0 ("体温 ≧ 38.5℃"). At least one must
    // surface in the top-5; both should be retrievable in a larger top-k.
    let hits = index.search("体温", 5).expect("search executes");
    let ids = span_ids(&hits);
    let temp_cell_ids = ["span_cell_r0c0", "span_cell_r1c0"];
    let matched: Vec<&&str> = temp_cell_ids.iter().filter(|id| ids.contains(id)).collect();
    assert!(
        !matched.is_empty(),
        "expected at least one '体温' table-cell span in top-5; got {ids:?}",
    );
}

// ---------------------------------------------------------------------------
// Query: アナフィラキシー 禁忌 retrieves the contraindication span
// ---------------------------------------------------------------------------

#[test]
fn query_anaphylaxis_contraindication_retrieves_contra_span() {
    let spans = load_toy_spans();
    let index = SparseIndex::build_from_spans(&spans).expect("build SparseIndex");

    let hits = index
        .search("アナフィラキシー 禁忌", 5)
        .expect("search executes");
    let ids = span_ids(&hits);
    assert!(
        ids.contains(&"span_contra_bl_allergy"),
        "expected span_contra_bl_allergy in top-5 for 'アナフィラキシー 禁忌'; got {ids:?}",
    );
}

// ---------------------------------------------------------------------------
// Fingerprint determinism
// ---------------------------------------------------------------------------

#[test]
fn fingerprint_is_deterministic_across_insertion_orders() {
    let spans = load_toy_spans();

    let mut reversed = spans.clone();
    reversed.reverse();

    let mut shuffled = spans.clone();
    // Deterministic non-trivial shuffle: swap halves so the order really
    // changes but the test stays reproducible.
    let mid = shuffled.len() / 2;
    shuffled.rotate_left(mid);

    let idx_a = SparseIndex::build_from_spans(&spans).expect("build A");
    let idx_b = SparseIndex::build_from_spans(&reversed).expect("build B");
    let idx_c = SparseIndex::build_from_spans(&shuffled).expect("build C");

    assert_eq!(
        idx_a.fingerprint(),
        idx_b.fingerprint(),
        "fingerprint must be order-invariant (original vs reversed)",
    );
    assert_eq!(
        idx_a.fingerprint(),
        idx_c.fingerprint(),
        "fingerprint must be order-invariant (original vs rotated)",
    );

    // The free function exposes the same algorithm and must agree.
    assert_eq!(
        compute_index_fingerprint(&spans),
        *idx_a.fingerprint(),
        "compute_index_fingerprint and SparseIndex must agree",
    );
}

#[test]
fn fingerprint_distinguishes_distinct_corpora() {
    let full = load_toy_spans();
    let mut subset = full.clone();
    subset.pop();
    assert_ne!(
        compute_index_fingerprint(&full),
        compute_index_fingerprint(&subset),
        "removing a span must change the fingerprint",
    );

    // Modifying a span's content (without changing the set membership) must
    // also change the fingerprint, because the per-span content_hash is
    // included in the fingerprint input.
    let mut mutated = full.clone();
    mutated[0].search_text.push_str("追加テキスト");
    assert_ne!(
        compute_index_fingerprint(&full),
        compute_index_fingerprint(&mutated),
        "mutating a span body must change the fingerprint",
    );
}

// ---------------------------------------------------------------------------
// Lindera IPADIC morphological analysis
// ---------------------------------------------------------------------------

#[test]
fn ipadic_tokenizer_segments_japanese_phrases() {
    // A single concatenated Japanese phrase must produce multiple morphemes
    // under IPADIC. We do not pin exact morpheme strings (IPADIC versions
    // can shift surface forms), but we assert non-trivial segmentation.
    let tokens = ipadic_tokens("敗血症患者にβラクタム系抗菌薬を投与する").expect("tokenize");
    assert!(
        tokens.len() >= 4,
        "expected non-trivial IPADIC segmentation; got {tokens:?}",
    );
    // Key surface terms used by our toy queries must appear as morphemes.
    assert!(
        tokens.iter().any(|t| t == "敗血症"),
        "IPADIC should yield '敗血症' as a morpheme; got {tokens:?}",
    );
    assert!(
        tokens.iter().any(|t| t == "投与"),
        "IPADIC should yield '投与' as a morpheme; got {tokens:?}",
    );
}

#[test]
fn ipadic_tokenizer_idempotent_across_calls() {
    let t1 = ipadic_tokens("体温は38度です").expect("tokenize call 1");
    let t2 = ipadic_tokens("体温は38度です").expect("tokenize call 2");
    assert_eq!(t1, t2, "IPADIC tokenization must be deterministic");
}
