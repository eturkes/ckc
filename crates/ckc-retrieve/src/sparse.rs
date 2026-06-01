//! In-memory Tantivy + Lindera (IPADIC) BM25 sparse index over `SourceSpan`.
//!
//! Subtask 0.7.2 deliverable. The index is the Phase-0 sparse-retrieval
//! baseline over the toy corpus -- the reference that future dense/
//! late-interaction retrievers must beat or match on the Japanese qrels.
//!
//! Determinism guarantees:
//! - The `index_fingerprint` is the SHA-256 content hash (RFC 8785 canonical
//!   JSON) of the lex-sorted `(span_id, content_hash(span))` pairs and is
//!   therefore invariant under the order in which spans are passed in.
//! - BM25 ranking is deterministic across independent index builds because
//!   `search` fetches every matching document, applies a post-collector stable
//!   sort (primary key score descending, secondary key `span_id` ascending),
//!   and only then truncates to `top_k`. Sorting before truncating makes both
//!   the ORDER and the SELECTION of survivors a function of that total order,
//!   dominating any Tantivy-internal tie-breaking that can otherwise vary
//!   across builds (multi-threaded indexers may assign different doc ids to the
//!   same spans even under identical insertion order, breaking the implicit
//!   `TopDocs` "doc-id-ascending" tie-break that would otherwise decide both
//!   order and, at the `top_k` boundary, membership).

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow};

use ckc_core::canonical::{ContentHash, content_hash};
use ckc_core::id::SpanId;
use ckc_core::source::SourceSpan;

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera_tantivy::tokenizer::LinderaTokenizer;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{
    Field, IndexRecordOption, STORED, Schema, TextFieldIndexing, TextOptions, Value,
};
use tantivy::{Index, IndexReader, TantivyDocument, doc};

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
    /// described in [`SchemaFields`]. Input order influences only Tantivy's
    /// internal doc-id assignment; `search` applies a post-collector stable
    /// sort (score DESC, span_id ASC) that delivers cross-run rank stability
    /// regardless of input order, so callers need not pre-sort `spans`.
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

        // Fetch every matching document, not just Tantivy's own top_k:
        // `TopDocs::with_limit` truncates by (score, internal doc-id) BEFORE the
        // post-collector stable sort runs, so an exact f32 score tie straddling
        // the top_k boundary would let Tantivy's nondeterministic doc-id
        // assignment decide which spans survive. Over-fetching and truncating
        // AFTER the (score DESC, span_id ASC) sort makes survivor selection -- not
        // just ordering -- a deterministic function of the total order. Phase-0
        // corpora are tiny, so fetching all matches is free.
        let fetch_limit = (searcher.num_docs() as usize).max(1);
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(fetch_limit))
            .context("execute BM25 search")?;

        let mut hits = Vec::with_capacity(top_docs.len());
        for (score, addr) in top_docs.into_iter() {
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
                rank: 0,
            });
        }

        // Deterministic tie-break: primary score DESC, secondary span_id ASC.
        // Without this, two independently-built indices over the same span
        // set can return tied hits in opposite orders because Tantivy assigns
        // doc ids based on indexer-thread interleaving rather than purely on
        // insertion order. `partial_cmp` over BM25 scores always returns
        // `Some` (BM25 produces finite non-NaN f32), so `unwrap_or(Equal)` is
        // defensive only.
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.span_id.as_str().cmp(b.span_id.as_str()))
        });
        hits.truncate(top_k);
        for (i, h) in hits.iter_mut().enumerate() {
            h.rank = (i as u32) + 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::enums::Language;
    use ckc_core::id::DocId;

    /// Minimal span carrying `text` in every text slot, for tokenizer-driven
    /// scoring tests that need synthetic content the toy corpus lacks.
    fn span(id: &str, text: &str) -> SourceSpan {
        SourceSpan {
            span_id: SpanId::new(id),
            doc_id: DocId::new("doc_tie"),
            section_path: Vec::new(),
            cq_id: None,
            page: None,
            bbox: None,
            table_cell: None,
            raw_text: text.to_owned(),
            nfkc_text: text.to_owned(),
            search_text: text.to_owned(),
            display_text: text.to_owned(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: Vec::new(),
            confidence: 1.0,
        }
    }

    /// Five spans share identical text, so a matching query scores them all
    /// identically -- an exact BM25 tie wider than `top_k`. Survivor selection
    /// must then be the lexicographically-smallest `span_id`s and must be
    /// identical across two independent index builds, since Tantivy's internal
    /// doc-id tie-break is not stable across builds. Exercises the `>top_k`
    /// truncation path the toy corpus never reaches, guarding the over-fetch-
    /// then-truncate determinism contract.
    #[test]
    fn truncation_selects_deterministic_set_under_score_ties() {
        let spans: Vec<SourceSpan> = (0..5)
            .map(|i| span(&format!("span_tie_{i}"), "発熱 体温 上昇"))
            .collect();

        let run = || {
            SparseIndex::build_from_spans(&spans)
                .expect("index builds")
                .search("発熱", 2)
                .expect("search succeeds")
        };
        let first = run();
        let second = run();

        // Two independent builds agree on both selection and order.
        assert_eq!(first, second);
        // The two lexicographically-smallest span_ids survive truncation.
        let ids: Vec<&str> = first.iter().map(|h| h.span_id.as_str()).collect();
        assert_eq!(ids, vec!["span_tie_0", "span_tie_1"]);
        // Ranks are reassigned 1..=2 over the survivors, which share the tie score.
        assert_eq!((first[0].rank, first[1].rank), (1, 2));
        assert_eq!(first[0].score, first[1].score);
    }
}
