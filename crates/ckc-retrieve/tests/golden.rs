use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::Serialize;

use ckc_core::canonical::{content_hash, to_canonical_bytes, ContentHash};
use ckc_core::enums::Language;
use ckc_core::id::{QueryId, SpanId};
use ckc_retrieve::{AnalyzerConfig, QrelJudgment, RetrievalHit, RetrievalQuery, RetrievalResult};

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn schema_dir() -> PathBuf {
    workspace_root().join("schemas")
}

fn golden_dir() -> PathBuf {
    workspace_root().join("schemas").join("golden")
}

// ---------------------------------------------------------------------------
// Assertion helpers (same contract as ckc-core/tests/golden.rs)
// ---------------------------------------------------------------------------

fn check_golden<T: Serialize>(fixture: &T, stem: &str) {
    let bytes = to_canonical_bytes(fixture);
    let path = golden_dir().join(format!("{stem}.json"));
    let golden =
        std::fs::read(&path).unwrap_or_else(|e| panic!("read golden {}: {e}", path.display()));
    assert!(
        bytes == golden,
        "canonical bytes mismatch for {stem} (got {} bytes, golden {} bytes)",
        bytes.len(),
        golden.len()
    );
}

fn check_roundtrip<T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug>(
    fixture: &T,
    stem: &str,
) {
    let bytes1 = to_canonical_bytes(fixture);
    let hash1 = content_hash(fixture);
    let rt: T = serde_json::from_slice(&bytes1)
        .unwrap_or_else(|e| panic!("deserialize {stem}: {e}"));
    let bytes2 = to_canonical_bytes(&rt);
    let hash2 = content_hash(&rt);
    assert_eq!(bytes1, bytes2, "bytes differ after roundtrip for {stem}");
    assert_eq!(hash1, hash2, "hash differs after roundtrip for {stem}");
    assert_eq!(*fixture, rt, "value differs after roundtrip for {stem}");
}

fn check_schema<T: schemars::JsonSchema>(stem: &str) {
    let schema = schemars::schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap() + "\n";
    let path = schema_dir().join(format!("{stem}.schema.json"));
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read schema {}: {e}", path.display()));
    assert!(json == golden, "schema mismatch for {stem}");
}

// ---------------------------------------------------------------------------
// Golden fixture constructors
// ---------------------------------------------------------------------------

fn golden_analyzer_config() -> AnalyzerConfig {
    AnalyzerConfig {
        name: "lindera_ipadic".into(),
        dictionary: "ipadic".into(),
        mode: "normal".into(),
    }
}

fn golden_retrieval_query() -> RetrievalQuery {
    RetrievalQuery {
        query_id: QueryId::new("q_sepsis_bl"),
        query_text: "敗血症 βラクタム 投与".into(),
        language: Language::Ja,
        analyzer_config: golden_analyzer_config(),
    }
}

fn golden_retrieval_hit() -> RetrievalHit {
    RetrievalHit {
        span_id: SpanId::new("span_rec_sepsis_bl"),
        score: 12.34,
        rank: 1,
    }
}

fn golden_retrieval_result() -> RetrievalResult {
    RetrievalResult {
        query: golden_retrieval_query(),
        hits: vec![
            golden_retrieval_hit(),
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

fn golden_qrel_judgment() -> QrelJudgment {
    QrelJudgment {
        query_id: QueryId::new("q_sepsis_bl"),
        span_id: SpanId::new("span_rec_sepsis_bl"),
        relevance: 2,
    }
}

// ---------------------------------------------------------------------------
// Test macro
// ---------------------------------------------------------------------------

macro_rules! golden_suite {
    ($mod_name:ident, $type:ty, $fixture_fn:ident, $stem:literal) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn canonical() {
                check_golden(&$fixture_fn(), $stem);
            }

            #[test]
            fn roundtrip() {
                check_roundtrip::<$type>(&$fixture_fn(), $stem);
            }

            #[test]
            fn schema() {
                check_schema::<$type>($stem);
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Suite invocations (5 types = 15 tests)
// ---------------------------------------------------------------------------

golden_suite!(gs_analyzer_config, AnalyzerConfig, golden_analyzer_config, "analyzer_config");
golden_suite!(gs_retrieval_query, RetrievalQuery, golden_retrieval_query, "retrieval_query");
golden_suite!(gs_retrieval_hit, RetrievalHit, golden_retrieval_hit, "retrieval_hit");
golden_suite!(gs_retrieval_result, RetrievalResult, golden_retrieval_result, "retrieval_result");
golden_suite!(gs_qrel_judgment, QrelJudgment, golden_qrel_judgment, "qrel_judgment");

// ---------------------------------------------------------------------------
// Cross-type: all 5 retrieval golden fixtures produce distinct content hashes
// ---------------------------------------------------------------------------

#[test]
fn all_retrieval_golden_fixtures_produce_distinct_hashes() {
    let hashes: Vec<ContentHash> = vec![
        content_hash(&golden_analyzer_config()),
        content_hash(&golden_retrieval_query()),
        content_hash(&golden_retrieval_hit()),
        content_hash(&golden_retrieval_result()),
        content_hash(&golden_qrel_judgment()),
    ];
    for (i, a) in hashes.iter().enumerate() {
        for (j, b) in hashes.iter().enumerate().skip(i + 1) {
            assert_ne!(a, b, "hash collision between retrieval golden fixtures {i} and {j}");
        }
    }
}

// ---------------------------------------------------------------------------
// Regeneration
// ---------------------------------------------------------------------------

fn write_type<T: Serialize + schemars::JsonSchema>(fixture: &T, stem: &str) {
    let g = golden_dir();
    let s = schema_dir();
    std::fs::write(g.join(format!("{stem}.json")), to_canonical_bytes(fixture)).unwrap();
    let schema = schemars::schema_for!(T);
    std::fs::write(
        s.join(format!("{stem}.schema.json")),
        serde_json::to_string_pretty(&schema).unwrap() + "\n",
    )
    .unwrap();
}

#[test]
#[ignore]
fn regenerate() {
    std::fs::create_dir_all(golden_dir()).unwrap();

    write_type(&golden_analyzer_config(), "analyzer_config");
    write_type(&golden_retrieval_query(), "retrieval_query");
    write_type(&golden_retrieval_hit(), "retrieval_hit");
    write_type(&golden_retrieval_result(), "retrieval_result");
    write_type(&golden_qrel_judgment(), "qrel_judgment");
}
