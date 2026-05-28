use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::canonical::ContentHash;
use ckc_core::enums::Language;
use ckc_core::id::{QueryId, SpanId};

pub mod metrics;
pub mod sparse;
pub use metrics::{ndcg_at_k, recall_at_k, reciprocal_rank};
pub use sparse::{SparseIndex, compute_index_fingerprint, ipadic_tokens};

/// Morphological analyzer configuration for Japanese text retrieval.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzerConfig {
    pub name: String,
    pub dictionary: String,
    pub mode: String,
}

/// A retrieval query with analyzer context.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RetrievalQuery {
    pub query_id: QueryId,
    pub query_text: String,
    pub language: Language,
    pub analyzer_config: AnalyzerConfig,
}

/// A single ranked retrieval hit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RetrievalHit {
    pub span_id: SpanId,
    pub score: f64,
    pub rank: u32,
}

/// Full retrieval result for a single query: ranked hits plus index provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RetrievalResult {
    pub query: RetrievalQuery,
    pub hits: Vec<RetrievalHit>,
    pub index_fingerprint: ContentHash,
    pub corpus_hash: ContentHash,
}

/// A single relevance judgment for retrieval evaluation (qrel entry).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct QrelJudgment {
    pub query_id: QueryId,
    pub span_id: SpanId,
    pub relevance: u32,
}
