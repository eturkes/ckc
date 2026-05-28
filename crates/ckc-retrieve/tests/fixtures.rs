//! Subtask 0.7.3: queries / qrels fixtures for the Phase-0 toy corpus.
//!
//! The fixtures sit next to the existing span/document fixtures under
//! `examples/research_kernel/fixtures/` and drive the retrieval evaluation
//! tests in later subtasks (0.7.5 pipeline, 0.7.6 persistence).
//!
//! Relevance grades follow the TREC convention:
//!   3 = highly relevant (direct target of the query)
//!   2 = relevant (supporting evidence / closely related)
//!   1 = marginally relevant (peripheral mention)
//!   0 = not relevant (omitted from the file)
//!
//! Constructor functions are the single source of truth; the committed JSON
//! files are regenerated from them via the ignored `regen_fixtures` test.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use ckc_core::canonical::to_canonical_bytes;
use ckc_core::enums::Language;
use ckc_core::id::{QueryId, SpanId};
use ckc_core::source::SourceSpan;
use ckc_retrieve::{AnalyzerConfig, QrelJudgment, RetrievalQuery};

// ---------------------------------------------------------------------------
// Phase-0 spans loaded from the existing committed fixture.
// ---------------------------------------------------------------------------

const TOY_SPANS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/spans.json");
const TOY_QUERIES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/queries.json");
const TOY_QRELS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/qrels.json");

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/research_kernel/fixtures")
}

// ---------------------------------------------------------------------------
// Canonical analyzer config (Lindera IPADIC, normal mode) used by every query.
// ---------------------------------------------------------------------------

fn ipadic_normal() -> AnalyzerConfig {
    AnalyzerConfig {
        name: "lindera_ipadic".into(),
        dictionary: "ipadic".into(),
        mode: "normal".into(),
    }
}

fn query(id: &str, text: &str) -> RetrievalQuery {
    RetrievalQuery {
        query_id: QueryId::new(id),
        query_text: text.into(),
        language: Language::Ja,
        analyzer_config: ipadic_normal(),
    }
}

fn judgment(query_id: &str, span_id: &str, relevance: u32) -> QrelJudgment {
    QrelJudgment {
        query_id: QueryId::new(query_id),
        span_id: SpanId::new(span_id),
        relevance,
    }
}

// ---------------------------------------------------------------------------
// Fixture constructors (single source of truth).
// ---------------------------------------------------------------------------

fn toy_queries() -> Vec<RetrievalQuery> {
    vec![
        query("q_sepsis_bl", "敗血症 βラクタム 投与"),
        query("q_contra_allergy", "アナフィラキシー 禁忌"),
        query("q_allergy_history", "薬物アレルギー歴 確認 既往"),
        query("q_vitals_temp", "体温"),
        query("q_provenance", "ガイドライン 第5版 発行"),
        query("q_term_variants", "βラクタム ベータラクタム 化学構造"),
    ]
}

fn toy_qrels() -> Vec<QrelJudgment> {
    vec![
        // q_sepsis_bl: direct recommendation + supporting evidence span.
        judgment("q_sepsis_bl", "span_rec_sepsis_bl", 3),
        judgment("q_sepsis_bl", "span_evidence_sepsis", 2),
        // q_contra_allergy: contraindication is the direct target; allergy
        // history span carries adjacent clinical context.
        judgment("q_contra_allergy", "span_contra_bl_allergy", 3),
        judgment("q_contra_allergy", "span_allergy_history", 2),
        // q_allergy_history: history-check span is the target; the
        // contraindication span is supportive.
        judgment("q_allergy_history", "span_allergy_history", 3),
        judgment("q_allergy_history", "span_contra_bl_allergy", 2),
        // q_vitals_temp: temperature cells are the targets; caption is
        // peripheral context.
        judgment("q_vitals_temp", "span_cell_r0c0", 3),
        judgment("q_vitals_temp", "span_cell_r1c0", 3),
        judgment("q_vitals_temp", "span_vitals_caption", 1),
        // q_provenance: only one span carries publisher/edition metadata.
        judgment("q_provenance", "span_provenance_meta", 3),
        // q_term_variants: both terminology gloss spans are equally
        // relevant; allergy-history span mentions β-ラクタム as a marginal
        // variant occurrence.
        judgment("q_term_variants", "span_term_bl_greek", 3),
        judgment("q_term_variants", "span_term_bl_katakana", 3),
        judgment("q_term_variants", "span_allergy_history", 1),
    ]
}

// ---------------------------------------------------------------------------
// Loaders for committed fixture files.
// ---------------------------------------------------------------------------

fn load_committed_spans() -> Vec<SourceSpan> {
    serde_json::from_str(TOY_SPANS_JSON).expect("toy spans.json must deserialize")
}

fn load_committed_queries() -> Vec<RetrievalQuery> {
    serde_json::from_str(TOY_QUERIES_JSON).expect("toy queries.json must deserialize")
}

fn load_committed_qrels() -> Vec<QrelJudgment> {
    serde_json::from_str(TOY_QRELS_JSON).expect("toy qrels.json must deserialize")
}

// ---------------------------------------------------------------------------
// Cross-referential integrity (the gate for subtask 0.7.3).
// ---------------------------------------------------------------------------

#[test]
fn query_count_is_within_planned_range() {
    let qs = load_committed_queries();
    assert!(
        (6..=8).contains(&qs.len()),
        "expected 6-8 queries; got {}",
        qs.len()
    );
}

#[test]
fn every_qrel_query_id_resolves_to_a_query() {
    let queries = load_committed_queries();
    let qrels = load_committed_qrels();

    let known: HashSet<&str> = queries.iter().map(|q| q.query_id.as_str()).collect();
    for j in &qrels {
        assert!(
            known.contains(j.query_id.as_str()),
            "qrel references unknown query_id {}",
            j.query_id,
        );
    }
}

#[test]
fn every_qrel_span_id_exists_in_toy_spans() {
    let spans = load_committed_spans();
    let qrels = load_committed_qrels();

    let known: HashSet<&str> = spans.iter().map(|s| s.span_id.as_str()).collect();
    for j in &qrels {
        assert!(
            known.contains(j.span_id.as_str()),
            "qrel references unknown span_id {} (query {})",
            j.span_id,
            j.query_id,
        );
    }
}

#[test]
fn every_query_has_at_least_one_qrel() {
    // A query with no judgments would silently contribute 0 to every
    // retrieval metric; flag it as a fixture error early.
    let queries = load_committed_queries();
    let qrels = load_committed_qrels();

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for j in &qrels {
        *counts.entry(j.query_id.as_str()).or_default() += 1;
    }
    for q in &queries {
        let n = counts.get(q.query_id.as_str()).copied().unwrap_or(0);
        assert!(n >= 1, "query {} has no qrel judgments", q.query_id);
    }
}

#[test]
fn qrel_relevance_grades_are_within_spec_range() {
    // Spec: grades 0-3 (TREC convention). The file omits 0-grades but the
    // bound itself must hold defensively against fixture drift.
    for j in &load_committed_qrels() {
        assert!(
            j.relevance <= 3,
            "qrel relevance out of range for {}/{}: {}",
            j.query_id,
            j.span_id,
            j.relevance,
        );
    }
}

// ---------------------------------------------------------------------------
// Constructor / committed-file parity (catches accidental drift between the
// Rust source of truth and the JSON fixtures). Mirrors the
// `committed_*_match` pattern used by ckc-core's research_source_corpus test.
// ---------------------------------------------------------------------------

#[test]
fn committed_queries_match_constructor() {
    let expected = to_canonical_bytes(&toy_queries());
    let actual = std::fs::read(fixtures_dir().join("queries.json"))
        .expect("queries.json fixture must be readable");
    assert_eq!(
        actual, expected,
        "committed queries.json differs from constructor output; \
         regenerate with: cargo test -p ckc-retrieve --test fixtures regen_fixtures -- --ignored"
    );
}

#[test]
fn committed_qrels_match_constructor() {
    let expected = to_canonical_bytes(&toy_qrels());
    let actual = std::fs::read(fixtures_dir().join("qrels.json"))
        .expect("qrels.json fixture must be readable");
    assert_eq!(
        actual, expected,
        "committed qrels.json differs from constructor output; \
         regenerate with: cargo test -p ckc-retrieve --test fixtures regen_fixtures -- --ignored"
    );
}

// ---------------------------------------------------------------------------
// Regeneration (manual). Run with: --ignored
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn regen_fixtures() {
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).expect("create fixtures dir");
    std::fs::write(dir.join("queries.json"), to_canonical_bytes(&toy_queries()))
        .expect("write queries.json");
    std::fs::write(dir.join("qrels.json"), to_canonical_bytes(&toy_qrels()))
        .expect("write qrels.json");
}
