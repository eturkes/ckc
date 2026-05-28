//! In-memory Tantivy + Lindera (IPADIC) BM25 sparse index over `SourceSpan`.
//!
//! Subtask 0.7.2 deliverable. The index is the sparse-retrieval oracle of the
//! Phase-0 toy corpus and is the baseline that future dense/late-interaction
//! retrievers must beat or match on the Japanese qrels.
//!
//! Determinism guarantees:
//! - The `index_fingerprint` is the SHA-256 content hash (RFC 8785 canonical
//!   JSON) of the lex-sorted `(span_id, content_hash(span))` pairs and is
//!   therefore invariant under the order in which spans are passed in.
//! - BM25 ranking itself is deterministic given a fixed corpus content, since
//!   Tantivy's `TopDocs` collector breaks score ties by ascending doc id.
//!   Span insertion order can still shift the doc-id assignment, so ranked
//!   results are deterministic only when spans are inserted in a stable order
//!   (callers that need cross-run rank stability must sort spans by `span_id`
//!   before calling `build_from_spans`).

use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};

use ckc_core::canonical::{content_hash, ContentHash};
use ckc_core::id::SpanId;
use ckc_core::source::SourceSpan;

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera_tantivy::tokenizer::LinderaTokenizer;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{
    Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, Value, STORED,
};
use tantivy::{doc, Index, IndexReader, TantivyDocument};

use crate::RetrievalHit;

/// Tokenizer name registered on the index for Japanese morphological analysis.
const LANG_JA_TOKENIZER: &str = "lang_ja";

/// Memory budget for the in-memory writer. Tantivy requires >= ~15 MB; 50 MB
/// is the upstream example default and is more than enough for Phase-0 corpus
/// sizes.
const WRITER_MEMORY_BUDGET_BYTES: usize = 50_000_000;

/// Field handles for the sparse-index schema.
///
/// `span_id`, `doc_id`, `section_path` are stored-only retrieval anchors.
/// `search_text` and `nfkc_text` are tokenized with Lindera (IPADIC) and
/// indexed with positions for BM25 ranking; only `search_text` is also
/// stored, so the document JSON remains compact while both fields remain
/// queryable.
#[derive(Clone, Copy, Debug)]
struct SchemaFields {
    span_id: Field,
    search_text: Field,
    nfkc_text: Field,
    doc_id: Field,
    section_path: Field,
}

/// Build the static Tantivy schema for the sparse index.
fn build_schema() -> (Schema, SchemaFields) {
    let mut sb = Schema::builder();

    let ja_indexed_stored = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(LANG_JA_TOKENIZER)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();

    let ja_indexed_only = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer(LANG_JA_TOKENIZER)
            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
    );

    let span_id = sb.add_text_field("span_id", STORED);
    let search_text = sb.add_text_field("search_text", ja_indexed_stored);
    let nfkc_text = sb.add_text_field("nfkc_text", ja_indexed_only);
    let doc_id = sb.add_text_field("doc_id", STORED);
    let section_path = sb.add_text_field("section_path", STORED);

    let schema = sb.build();
    let fields = SchemaFields {
        span_id,
        search_text,
        nfkc_text,
        doc_id,
        section_path,
    };
    (schema, fields)
}

/// Construct a fresh Lindera IPADIC tokenizer.
///
/// The IPADIC dictionary is embedded into the binary via the `embed-ipadic`
/// feature on `lindera-tantivy`; `load_dictionary("embedded://ipadic")`
/// returns it at runtime without filesystem access. This is the standard
/// upstream pattern (see `lindera-tantivy/examples/ipadic.rs`).
fn make_lindera_tokenizer() -> Result<LinderaTokenizer> {
    let dictionary = load_dictionary("embedded://ipadic")
        .map_err(|e| anyhow!("load embedded IPADIC dictionary: {e}"))?;
    let segmenter = Segmenter::new(Mode::Normal, dictionary, None);
    Ok(LinderaTokenizer::from_segmenter(segmenter))
}

/// Deterministic index fingerprint for a span set.
///
/// Sorts `(span_id, content_hash(span))` pairs lexicographically by `span_id`
/// and then content-hashes the resulting `Vec`. Two callers passing the same
/// span set in different orders produce the same fingerprint; any change to
/// a span's canonical content or span set membership changes the fingerprint.
pub fn compute_index_fingerprint(spans: &[SourceSpan]) -> ContentHash {
    // BTreeMap gives us de-duplication-by-span-id AND lex sorting in one pass.
    // If the same span_id appears twice we deliberately keep the last value
    // (matches BTreeMap insert semantics); the build path indexes every span
    // it is given, so this is a defensive choice for the fingerprint only.
    let mut by_id: BTreeMap<String, ContentHash> = BTreeMap::new();
    for span in spans {
        by_id.insert(span.span_id.as_str().to_owned(), content_hash(span));
    }
    let ordered: Vec<(String, ContentHash)> = by_id.into_iter().collect();
    content_hash(&ordered)
}

/// BM25 sparse-retrieval index over `SourceSpan` content, built in RAM.
///
/// Use [`SparseIndex::build_from_spans`] to construct one from a slice of
/// spans, and [`SparseIndex::search`] to issue a Japanese query and recover a
/// ranked list of [`RetrievalHit`]s. The fingerprint accessor
/// [`SparseIndex::fingerprint`] returns the content-addressed identity of the
/// indexed span set; persist it on every accepted `RetrievalResult` so that
/// downstream replay can detect corpus drift.
pub struct SparseIndex {
    index: Index,
    reader: IndexReader,
    fields: SchemaFields,
    fingerprint: ContentHash,
}

impl SparseIndex {
    /// Build a fresh in-memory index over the given spans.
    ///
    /// Each span produces one Tantivy document with the five-field schema
    /// described in [`SchemaFields`]. Order is preserved on the way in
    /// (matters for tie-breaking in `TopDocs`); callers that want cross-run
    /// rank stability should sort `spans` by `span_id` first.
    pub fn build_from_spans(spans: &[SourceSpan]) -> Result<Self> {
        let (schema, fields) = build_schema();
        let index = Index::create_in_ram(schema);

        // Register the Lindera tokenizer BEFORE writing documents: writes
        // resolve the tokenizer at indexing time, and an unregistered name
        // would cause `add_document` to fail.
        let tokenizer = make_lindera_tokenizer()?;
        index.tokenizers().register(LANG_JA_TOKENIZER, tokenizer);

        let mut writer = index
            .writer(WRITER_MEMORY_BUDGET_BYTES)
            .context("create tantivy IndexWriter")?;

        for span in spans {
            // section_path is a Vec<String>; collapse it with '/' so a single
            // stored text field is sufficient for downstream inspection. The
            // separator is not searched against — section_path is STORED only.
            let section_path = span.section_path.join("/");

            writer
                .add_document(doc!(
                    fields.span_id => span.span_id.as_str().to_owned(),
                    fields.search_text => span.search_text.clone(),
                    fields.nfkc_text => span.nfkc_text.clone(),
                    fields.doc_id => span.doc_id.as_str().to_owned(),
                    fields.section_path => section_path,
                ))
                .with_context(|| format!("index span {}", span.span_id.as_str()))?;
        }
        writer.commit().context("commit tantivy IndexWriter")?;

        let reader = index.reader().context("open tantivy IndexReader")?;
        let fingerprint = compute_index_fingerprint(spans);

        Ok(SparseIndex {
            index,
            reader,
            fields,
            fingerprint,
        })
    }

    /// Content-addressed fingerprint of the indexed span set.
    pub fn fingerprint(&self) -> &ContentHash {
        &self.fingerprint
    }

    /// BM25-rank the indexed spans against `query_text` and return the top-k.
    ///
    /// The query is parsed by Tantivy's `QueryParser` over the two indexed
    /// fields (`search_text`, `nfkc_text`). The parser splits the raw query
    /// on whitespace, then tokenizes each surface term through the registered
    /// Lindera tokenizer; the default boolean operator is OR, which is the
    /// right default for Japanese recall where users mix concepts.
    ///
    /// Hits are returned in score-descending order; `rank` is 1-indexed so it
    /// matches retrieval-evaluation conventions (Recall@k, MRR, etc.).
    pub fn search(&self, query_text: &str, top_k: usize) -> Result<Vec<RetrievalHit>> {
        let searcher = self.reader.searcher();

        let qp = QueryParser::for_index(
            &self.index,
            vec![self.fields.search_text, self.fields.nfkc_text],
        );
        let query = qp
            .parse_query(query_text)
            .with_context(|| format!("parse query: {query_text:?}"))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(top_k))
            .context("execute BM25 search")?;

        let mut hits = Vec::with_capacity(top_docs.len());
        for (rank_idx, (score, addr)) in top_docs.into_iter().enumerate() {
            let doc: TantivyDocument = searcher
                .doc(addr)
                .with_context(|| format!("fetch stored doc at {addr:?}"))?;
            let span_id_str = doc
                .get_first(self.fields.span_id)
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("indexed document missing span_id field"))?
                .to_owned();
            hits.push(RetrievalHit {
                span_id: SpanId::new(span_id_str),
                score: score as f64,
                rank: (rank_idx as u32) + 1,
            });
        }
        Ok(hits)
    }
}

/// Run the Lindera IPADIC tokenizer over a piece of text and return the
/// surface-form tokens.
///
/// Exposed for tests and tooling that need to inspect tokenization directly.
/// The index itself does not call this — Tantivy drives the same tokenizer
/// through its own `TextAnalyzer` pipeline at index/query time.
pub fn ipadic_tokens(text: &str) -> Result<Vec<String>> {
    use tantivy::tokenizer::{TokenStream, Tokenizer};

    let mut tok = make_lindera_tokenizer()?;
    let mut stream = tok.token_stream(text);
    let mut out = Vec::new();
    while stream.advance() {
        out.push(stream.token().text.clone());
    }
    Ok(out)
}
