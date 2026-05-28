use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::canonical::ContentHash;
use ckc_core::enums::Language;
use ckc_core::id::{QueryId, SpanId};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_analyzer_config() -> AnalyzerConfig {
        AnalyzerConfig {
            name: "lindera_ipadic".into(),
            dictionary: "ipadic".into(),
            mode: "normal".into(),
        }
    }

    fn fixture_retrieval_query() -> RetrievalQuery {
        RetrievalQuery {
            query_id: QueryId::new("q_sepsis_bl"),
            query_text: "敗血症 βラクタム 投与".into(),
            language: Language::Ja,
            analyzer_config: fixture_analyzer_config(),
        }
    }

    fn fixture_retrieval_hit() -> RetrievalHit {
        RetrievalHit {
            span_id: SpanId::new("span_rec_sepsis_bl"),
            score: 12.34,
            rank: 1,
        }
    }

    fn fixture_retrieval_result() -> RetrievalResult {
        RetrievalResult {
            query: fixture_retrieval_query(),
            hits: vec![
                fixture_retrieval_hit(),
                RetrievalHit {
                    span_id: SpanId::new("span_contra_bl_allergy"),
                    score: 8.76,
                    rank: 2,
                },
            ],
            index_fingerprint: ContentHash(
                "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                    .into(),
            ),
            corpus_hash: ContentHash(
                "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                    .into(),
            ),
        }
    }

    fn fixture_qrel_judgment() -> QrelJudgment {
        QrelJudgment {
            query_id: QueryId::new("q_sepsis_bl"),
            span_id: SpanId::new("span_rec_sepsis_bl"),
            relevance: 2,
        }
    }

    macro_rules! roundtrip_test {
        ($test_name:ident, $fixture_fn:ident, $ty:ty) => {
            #[test]
            fn $test_name() {
                let val = $fixture_fn();
                let json = serde_json::to_string(&val).unwrap();
                let rt: $ty = serde_json::from_str(&json).unwrap();
                assert_eq!(val, rt);
            }
        };
    }

    roundtrip_test!(analyzer_config_roundtrip, fixture_analyzer_config, AnalyzerConfig);
    roundtrip_test!(retrieval_query_roundtrip, fixture_retrieval_query, RetrievalQuery);
    roundtrip_test!(retrieval_hit_roundtrip, fixture_retrieval_hit, RetrievalHit);
    roundtrip_test!(retrieval_result_roundtrip, fixture_retrieval_result, RetrievalResult);
    roundtrip_test!(qrel_judgment_roundtrip, fixture_qrel_judgment, QrelJudgment);

    #[test]
    fn retrieval_result_empty_hits() {
        let val = RetrievalResult {
            hits: vec![],
            ..fixture_retrieval_result()
        };
        let json = serde_json::to_string(&val).unwrap();
        let rt: RetrievalResult = serde_json::from_str(&json).unwrap();
        assert_eq!(val, rt);
        assert!(rt.hits.is_empty());
    }

    #[test]
    fn qrel_relevance_grades() {
        for grade in [0, 1, 2, 3] {
            let j = QrelJudgment {
                relevance: grade,
                ..fixture_qrel_judgment()
            };
            let json = serde_json::to_string(&j).unwrap();
            let rt: QrelJudgment = serde_json::from_str(&json).unwrap();
            assert_eq!(j.relevance, rt.relevance);
        }
    }
}
