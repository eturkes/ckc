//! Retrieval-evaluation metrics: Recall@k, reciprocal rank, nDCG@k.
//!
//! Subtask 0.7.4 deliverable. Pure functions over the rank-ordered
//! [`RetrievalHit`] slice produced by [`crate::SparseIndex::search`]. They
//! make no assumption about how the slice was produced, only that:
//!
//! - `hits[i].rank` is the 1-indexed position of `hits[i].span_id` in the
//!   ranked output (the convention used by `SparseIndex::search`);
//! - the slice is sorted by ascending rank (i.e. score-descending), which is
//!   what `SparseIndex::search` already guarantees;
//! - each `span_id` appears at most once.
//!
//! The aggregating pipeline (subtask 0.7.5) calls these per query and then
//! averages across queries.
//!
//! Conventions for degenerate inputs (matches typical IR evaluation
//! libraries and the explicit roadmap guidance for nDCG):
//!
//! - Empty `hits` slice → every metric is `0.0`.
//! - Empty relevant set / empty `graded` map → `recall_at_k` and
//!   `reciprocal_rank` are `0.0`; `ndcg_at_k` is `0.0` (ideal DCG = 0).
//! - `k == 0` → `recall_at_k` and `ndcg_at_k` are `0.0` (top-0 retrieves
//!   nothing).
//! - `k > hits.len()` → the metric is computed over the full slice; no
//!   padding is added.

use std::collections::{HashMap, HashSet};

use ckc_core::id::SpanId;

use crate::RetrievalHit;

/// Recall@k: fraction of relevant documents that appear in the top-k hits.
///
/// `relevant` is the unordered set of all known-relevant span ids for the
/// query (binary relevance). The denominator is `|relevant|`, so removing a
/// relevant item from the corpus changes recall even when the retriever
/// could never have returned it.
pub fn recall_at_k(hits: &[RetrievalHit], relevant: &HashSet<SpanId>, k: usize) -> f64 {
    if relevant.is_empty() || k == 0 {
        return 0.0;
    }
    let cutoff = k.min(hits.len());
    let mut found = 0_usize;
    for h in &hits[..cutoff] {
        if relevant.contains(&h.span_id) {
            found += 1;
        }
    }
    (found as f64) / (relevant.len() as f64)
}

/// Reciprocal rank: `1 / rank` of the first relevant hit, or `0.0` if no
/// hit in the slice is relevant.
///
/// `rank` is taken from `RetrievalHit::rank` (1-indexed), not the slice
/// index, so a truncated `hits` slice still reports the original rank.
pub fn reciprocal_rank(hits: &[RetrievalHit], relevant: &HashSet<SpanId>) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    for h in hits {
        if relevant.contains(&h.span_id) {
            return 1.0 / (h.rank as f64);
        }
    }
    0.0
}

/// Normalized DCG at k with graded relevance.
///
/// DCG@k uses the standard formulation
/// `DCG@k = Σ_{i=1..=k} rel_i / log2(rank_i + 1)`, where `rel_i` is the
/// grade of the hit at the i-th position (zero if the span is absent from
/// `graded`). Ideal DCG is computed over the top-k of `graded` values
/// sorted descending. Returns `0.0` when ideal DCG is zero (no relevant
/// items exist at all).
pub fn ndcg_at_k(hits: &[RetrievalHit], graded: &HashMap<SpanId, u32>, k: usize) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let cutoff = k.min(hits.len());
    let mut dcg = 0.0_f64;
    for h in &hits[..cutoff] {
        let rel = graded.get(&h.span_id).copied().unwrap_or(0) as f64;
        if rel == 0.0 {
            continue;
        }
        dcg += rel / ((h.rank as f64) + 1.0).log2();
    }

    let mut ideal: Vec<u32> = graded.values().copied().collect();
    ideal.sort_unstable_by(|a, b| b.cmp(a));
    let mut idcg = 0.0_f64;
    for (i, &rel) in ideal.iter().take(k).enumerate() {
        if rel == 0 {
            break;
        }
        let rank = (i as f64) + 1.0;
        idcg += (rel as f64) / (rank + 1.0).log2();
    }

    if idcg == 0.0 { 0.0 } else { dcg / idcg }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Tolerance for nDCG comparisons that bottom out at f64 log2 values.
    const TOL: f64 = 1e-12;

    fn hit(span_id: &str, rank: u32) -> RetrievalHit {
        RetrievalHit {
            span_id: SpanId::new(span_id),
            // Score is unused by every metric in this module; pin to a
            // monotone-decreasing placeholder so the slice still reads as a
            // valid ranked list.
            score: 1.0 / (rank as f64),
            rank,
        }
    }

    fn relevant(ids: &[&str]) -> HashSet<SpanId> {
        ids.iter().map(|s| SpanId::new(*s)).collect()
    }

    fn graded(pairs: &[(&str, u32)]) -> HashMap<SpanId, u32> {
        pairs.iter().map(|(s, g)| (SpanId::new(*s), *g)).collect()
    }

    // -----------------------------------------------------------------
    // recall_at_k
    // -----------------------------------------------------------------

    // Shared 4-hit slice for the recall walk-through. Relevant set
    // {B, D}, so:
    //   k=1 → top {A}                → 0/2 = 0.0
    //   k=2 → top {A, B}             → 1/2 = 0.5
    //   k=3 → top {A, B, C}          → 1/2 = 0.5
    //   k=4 → top {A, B, C, D}       → 2/2 = 1.0
    //   k=5 → same as k=4 (cap)      → 2/2 = 1.0
    fn recall_walkthrough() -> (Vec<RetrievalHit>, HashSet<SpanId>) {
        let hits = vec![hit("A", 1), hit("B", 2), hit("C", 3), hit("D", 4)];
        let rel = relevant(&["B", "D"]);
        (hits, rel)
    }

    #[test]
    fn recall_at_1_misses_when_top_is_irrelevant() {
        let (hits, rel) = recall_walkthrough();
        assert_eq!(recall_at_k(&hits, &rel, 1), 0.0);
    }

    #[test]
    fn recall_at_2_recovers_first_relevant() {
        let (hits, rel) = recall_walkthrough();
        // |{B}| / |{B, D}| = 1/2 = 0.5
        assert_eq!(recall_at_k(&hits, &rel, 2), 0.5);
    }

    #[test]
    fn recall_at_3_is_unchanged_when_next_hit_is_irrelevant() {
        let (hits, rel) = recall_walkthrough();
        // |{B}| / |{B, D}| = 1/2 = 0.5
        assert_eq!(recall_at_k(&hits, &rel, 3), 0.5);
    }

    #[test]
    fn recall_at_4_recovers_all_relevant() {
        let (hits, rel) = recall_walkthrough();
        // |{B, D}| / |{B, D}| = 2/2 = 1.0
        assert_eq!(recall_at_k(&hits, &rel, 4), 1.0);
    }

    #[test]
    fn recall_at_k_caps_to_hits_len() {
        let (hits, rel) = recall_walkthrough();
        // k=5 equals k=4: positions beyond the available hits are absent, so
        // the denominator stays capped at the number of hits.
        assert_eq!(recall_at_k(&hits, &rel, 5), 1.0);
    }

    #[test]
    fn recall_on_empty_hits_is_zero() {
        let rel = relevant(&["A"]);
        assert_eq!(recall_at_k(&[], &rel, 5), 0.0);
    }

    #[test]
    fn recall_with_zero_relevant_is_zero() {
        let (hits, _) = recall_walkthrough();
        assert_eq!(recall_at_k(&hits, &HashSet::new(), 3), 0.0);
    }

    #[test]
    fn recall_when_every_hit_is_relevant_and_set_equals_hits() {
        // Set membership matches the entire hit list → every k ≥ |hits|
        // saturates to 1.0.
        let hits = vec![hit("A", 1), hit("B", 2)];
        let rel = relevant(&["A", "B"]);
        assert_eq!(recall_at_k(&hits, &rel, 2), 1.0);
        assert_eq!(recall_at_k(&hits, &rel, 10), 1.0);
    }

    #[test]
    fn recall_with_k_zero_is_zero() {
        let (hits, rel) = recall_walkthrough();
        assert_eq!(recall_at_k(&hits, &rel, 0), 0.0);
    }

    // -----------------------------------------------------------------
    // reciprocal_rank
    // -----------------------------------------------------------------

    #[test]
    fn rr_picks_first_relevant_rank() {
        // First relevant is B at rank 2 → 1/2 = 0.5.
        let hits = vec![hit("A", 1), hit("B", 2), hit("C", 3), hit("D", 4)];
        let rel = relevant(&["B", "D"]);
        assert_eq!(reciprocal_rank(&hits, &rel), 0.5);
    }

    #[test]
    fn rr_is_one_when_top_hit_is_relevant() {
        // Top hit A is relevant → 1/1 = 1.0.
        let hits = vec![hit("A", 1), hit("B", 2)];
        let rel = relevant(&["A"]);
        assert_eq!(reciprocal_rank(&hits, &rel), 1.0);
    }

    #[test]
    fn rr_uses_hit_rank_not_slice_index() {
        // Truncated slice: caller dropped ranks 1-2 but the remaining
        // entries still carry their original rank. First relevant is C
        // at rank 3 → 1/3.
        let hits = vec![hit("C", 3), hit("D", 4)];
        let rel = relevant(&["D", "C"]);
        // 1/3 = 0.333… (exact f64 value); assert with tolerance to remain
        // robust to representation rounding.
        let expected = 1.0_f64 / 3.0;
        assert!((reciprocal_rank(&hits, &rel) - expected).abs() < TOL);
    }

    #[test]
    fn rr_on_empty_hits_is_zero() {
        let rel = relevant(&["A"]);
        assert_eq!(reciprocal_rank(&[], &rel), 0.0);
    }

    #[test]
    fn rr_with_zero_relevant_is_zero() {
        let hits = vec![hit("A", 1)];
        assert_eq!(reciprocal_rank(&hits, &HashSet::new()), 0.0);
    }

    #[test]
    fn rr_when_no_hit_is_relevant_is_zero() {
        let hits = vec![hit("A", 1), hit("B", 2)];
        let rel = relevant(&["X"]);
        assert_eq!(reciprocal_rank(&hits, &rel), 0.0);
    }

    // -----------------------------------------------------------------
    // ndcg_at_k
    // -----------------------------------------------------------------

    #[test]
    fn ndcg_perfect_ranking_is_one() {
        // hits = [A, B, C] ranks 1-3, graded = {A:3, B:2, C:1}.
        // DCG@3  = 3/log2(2) + 2/log2(3) + 1/log2(4)
        //        = 3 + 2/log2(3) + 0.5
        // IDCG@3 = same (already in ideal descending order)
        // nDCG@3 = 1.0
        let hits = vec![hit("A", 1), hit("B", 2), hit("C", 3)];
        let g = graded(&[("A", 3), ("B", 2), ("C", 1)]);
        assert!((ndcg_at_k(&hits, &g, 3) - 1.0).abs() < TOL);
    }

    #[test]
    fn ndcg_single_relevant_at_rank_3_with_clean_log2() {
        // hits = [A, B, C, D] ranks 1-4. Only C is relevant with grade 8.
        // log2(rank+1) at rank 3 is log2(4) = 2 (rational).
        // DCG@4  = 0 + 0 + 8/log2(4) + 0 = 8/2 = 4
        // IDCG@4 = 8/log2(2) + 0 + 0 + 0 = 8/1 = 8 (ideal places C at rank 1)
        // nDCG@4 = 4 / 8 = 0.5
        let hits = vec![hit("A", 1), hit("B", 2), hit("C", 3), hit("D", 4)];
        let g = graded(&[("C", 8)]);
        assert_eq!(ndcg_at_k(&hits, &g, 4), 0.5);
    }

    #[test]
    fn ndcg_irrelevant_top_with_single_relevant_at_rank_2() {
        // hits = [B, A, C] ranks 1-3, graded = {A: 1}; B and C absent → 0.
        // DCG@3  = 0/log2(2) + 1/log2(3) + 0/log2(4) = 1/log2(3)
        // IDCG@3 = 1/log2(2) + 0 + 0 = 1
        // nDCG@3 = 1/log2(3) ≈ 0.6309297535714574
        let hits = vec![hit("B", 1), hit("A", 2), hit("C", 3)];
        let g = graded(&[("A", 1)]);
        let expected = 1.0_f64 / 3.0_f64.log2();
        assert!((ndcg_at_k(&hits, &g, 3) - expected).abs() < TOL);
    }

    #[test]
    fn ndcg_on_empty_hits_is_zero() {
        let g = graded(&[("A", 3)]);
        assert_eq!(ndcg_at_k(&[], &g, 5), 0.0);
    }

    #[test]
    fn ndcg_with_zero_relevant_is_zero() {
        // Empty graded map → IDCG=0 → return 0.0 by convention.
        let hits = vec![hit("A", 1), hit("B", 2)];
        assert_eq!(ndcg_at_k(&hits, &HashMap::new(), 2), 0.0);
    }

    #[test]
    fn ndcg_with_only_zero_grades_is_zero() {
        // Map populated but every grade is 0 → IDCG=0 → return 0.0.
        let hits = vec![hit("A", 1), hit("B", 2)];
        let g = graded(&[("A", 0), ("B", 0)]);
        assert_eq!(ndcg_at_k(&hits, &g, 2), 0.0);
    }

    #[test]
    fn ndcg_k_larger_than_hits_matches_k_equal_to_hits() {
        // hits = [A], graded = {A: 4}. DCG@1 = 4/log2(2) = 4;
        // IDCG@1 = 4/log2(2) = 4; nDCG@1 = 1.0. k=5 must produce the same
        // value because the slice exposes one position and no padding is
        // added.
        let hits = vec![hit("A", 1)];
        let g = graded(&[("A", 4)]);
        let small = ndcg_at_k(&hits, &g, 1);
        let large = ndcg_at_k(&hits, &g, 5);
        assert_eq!(small, large);
        assert!((small - 1.0).abs() < TOL);
    }

    #[test]
    fn ndcg_with_k_zero_is_zero() {
        let hits = vec![hit("A", 1)];
        let g = graded(&[("A", 4)]);
        assert_eq!(ndcg_at_k(&hits, &g, 0), 0.0);
    }
}
