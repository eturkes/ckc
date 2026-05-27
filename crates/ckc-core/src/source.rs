use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::canonical::ContentHash;
use crate::enums::{BindingStatus, Language};
use crate::id::{ConceptId, CqId, DocId, EGraphClassId, ExtractedTableId, ManifestId, SpanId};

// ---------------------------------------------------------------------------
// Helper types
// ---------------------------------------------------------------------------

/// Page bounding box (PDF coordinates).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Reference to a cell within an extracted table.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct TableCellRef {
    pub table_id: ExtractedTableId,
    pub row: u32,
    pub col: u32,
}

/// A single extractor's output for an extraction vote.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExtractorVote {
    pub extractor: String,
    pub raw_text: String,
    pub confidence: f64,
}

// ---------------------------------------------------------------------------
// SPEC 10 types: source substrate and terminology
// ---------------------------------------------------------------------------

/// Registered source document with permission and version metadata (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CorpusDocument {
    pub doc_id: DocId,
    pub title_ja: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_en: Option<String>,
    pub source_type: String,
    pub publisher: String,
    pub society: String,
    pub edition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_date: Option<String>,
    pub license_status: String,
    pub content_hash: ContentHash,
    pub extraction_manifest_id: ManifestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<DocId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<DocId>,
}

/// Addressable textual unit from a source document (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SourceSpan {
    pub span_id: SpanId,
    pub doc_id: DocId,
    pub section_path: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cq_id: Option<CqId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<BBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_cell: Option<TableCellRef>,
    pub raw_text: String,
    pub nfkc_text: String,
    pub search_text: String,
    pub display_text: String,
    pub language: Language,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_span_id: Option<SpanId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_span_id: Option<SpanId>,
    pub extractor_votes: Vec<ExtractorVote>,
    pub confidence: f64,
}

/// Table extracted from a source document with cell-level addressability (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedTable {
    pub table_id: ExtractedTableId,
    pub doc_id: DocId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption_span_id: Option<SpanId>,
    pub cell_span_ids: Vec<SpanId>,
    pub row_headers: Vec<String>,
    pub column_headers: Vec<String>,
    pub reading_order: Vec<SpanId>,
    pub extraction_votes: Vec<ExtractorVote>,
    pub normalized_table_hash: ContentHash,
}

/// Domain concept with terminology bindings and e-graph membership (SPEC 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Concept {
    pub concept_id: ConceptId,
    pub label_ja: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_en: Option<String>,
    pub semantic_type: String,
    pub terminology_bindings: Vec<TerminologyBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egraph_class_id: Option<EGraphClassId>,
    pub source_span_ids: Vec<SpanId>,
}

/// Binding from a concept to an external terminology system (SPEC 6.2, 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TerminologyBinding {
    pub system: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub label: String,
    pub status: BindingStatus,
    pub mapping_relation: String,
    pub provenance: String,
    pub confidence: f64,
    pub license_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::{content_hash, to_canonical_bytes};

    fn fixture_terminology_binding() -> TerminologyBinding {
        TerminologyBinding {
            system: "MEDIS".into(),
            code: Some("M001".into()),
            version: Some("2024".into()),
            label: "敗血症".into(),
            status: BindingStatus::Exact,
            mapping_relation: "equivalent".into(),
            provenance: "medis_master_v2024".into(),
            confidence: 0.95,
            license_status: "permitted".into(),
            valid_from: Some("2024-01-01".into()),
            valid_to: None,
        }
    }

    fn fixture_corpus_document() -> CorpusDocument {
        CorpusDocument {
            doc_id: DocId::new("doc_sepsis_gl_2024"),
            title_ja: "日本版敗血症診療ガイドライン2024".into(),
            title_en: Some("Japanese Clinical Practice Guidelines for Sepsis 2024".into()),
            source_type: "guideline".into(),
            publisher: "日本集中治療医学会".into(),
            society: "JSICM".into(),
            edition: "2024".into(),
            publication_date: Some("2024-03-01".into()),
            access_date: Some("2026-05-01".into()),
            license_status: "permitted_research".into(),
            content_hash: ContentHash("sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
            extraction_manifest_id: ManifestId::new("manifest_sepsis_gl_2024"),
            supersedes: None,
            superseded_by: None,
        }
    }

    fn fixture_source_span() -> SourceSpan {
        SourceSpan {
            span_id: SpanId::new("span_s1"),
            doc_id: DocId::new("doc_sepsis_gl_2024"),
            section_path: vec!["CQ1".into(), "推奨".into()],
            cq_id: Some(CqId::new("cq_sepsis_abx_001")),
            page: Some(42),
            bbox: Some(BBox {
                x: 72.0,
                y: 200.0,
                width: 468.0,
                height: 14.0,
            }),
            table_cell: None,
            raw_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
            nfkc_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
            search_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
            display_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: Some(SpanId::new("span_s2")),
            extractor_votes: vec![ExtractorVote {
                extractor: "pymupdf".into(),
                raw_text: "敗血症にはβラクタム系抗菌薬の投与を推奨する".into(),
                confidence: 0.99,
            }],
            confidence: 0.99,
        }
    }

    fn fixture_extracted_table() -> ExtractedTable {
        ExtractedTable {
            table_id: ExtractedTableId::new("tbl_vitals_001"),
            doc_id: DocId::new("doc_sepsis_gl_2024"),
            caption_span_id: Some(SpanId::new("span_tbl_caption_001")),
            cell_span_ids: vec![
                SpanId::new("span_cell_r0c0"),
                SpanId::new("span_cell_r0c1"),
                SpanId::new("span_cell_r1c0"),
                SpanId::new("span_cell_r1c1"),
            ],
            row_headers: vec!["体温".into(), "血圧".into()],
            column_headers: vec!["項目".into(), "基準値".into()],
            reading_order: vec![
                SpanId::new("span_cell_r0c0"),
                SpanId::new("span_cell_r0c1"),
                SpanId::new("span_cell_r1c0"),
                SpanId::new("span_cell_r1c1"),
            ],
            extraction_votes: vec![ExtractorVote {
                extractor: "yomitoku".into(),
                raw_text: "体温|基準値\n血圧|基準値".into(),
                confidence: 0.92,
            }],
            normalized_table_hash: ContentHash(
                "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                    .into(),
            ),
        }
    }

    fn fixture_concept() -> Concept {
        Concept {
            concept_id: ConceptId::new("concept_sepsis"),
            label_ja: "敗血症".into(),
            label_en: Some("sepsis".into()),
            semantic_type: "diagnosis".into(),
            terminology_bindings: vec![fixture_terminology_binding()],
            egraph_class_id: Some(EGraphClassId::new("eclass_sepsis_001")),
            source_span_ids: vec![SpanId::new("span_s1")],
        }
    }

    // -- Serde round-trip tests --

    #[test]
    fn corpus_document_roundtrip() {
        let doc = fixture_corpus_document();
        let json = serde_json::to_string(&doc).unwrap();
        let rt: CorpusDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, rt);
    }

    #[test]
    fn source_span_roundtrip() {
        let span = fixture_source_span();
        let json = serde_json::to_string(&span).unwrap();
        let rt: SourceSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(span, rt);
    }

    #[test]
    fn extracted_table_roundtrip() {
        let table = fixture_extracted_table();
        let json = serde_json::to_string(&table).unwrap();
        let rt: ExtractedTable = serde_json::from_str(&json).unwrap();
        assert_eq!(table, rt);
    }

    #[test]
    fn concept_roundtrip() {
        let concept = fixture_concept();
        let json = serde_json::to_string(&concept).unwrap();
        let rt: Concept = serde_json::from_str(&json).unwrap();
        assert_eq!(concept, rt);
    }

    #[test]
    fn terminology_binding_roundtrip() {
        let binding = fixture_terminology_binding();
        let json = serde_json::to_string(&binding).unwrap();
        let rt: TerminologyBinding = serde_json::from_str(&json).unwrap();
        assert_eq!(binding, rt);
    }

    // -- Optional field omission --

    #[test]
    fn optional_fields_omitted_when_none() {
        let mut doc = fixture_corpus_document();
        doc.title_en = None;
        doc.publication_date = None;
        doc.access_date = None;
        let json = serde_json::to_string(&doc).unwrap();
        assert!(!json.contains("title_en"));
        assert!(!json.contains("publication_date"));
        assert!(!json.contains("access_date"));
        let rt: CorpusDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, rt);
    }

    #[test]
    fn source_span_minimal_options() {
        let mut span = fixture_source_span();
        span.cq_id = None;
        span.page = None;
        span.bbox = None;
        span.table_cell = None;
        span.previous_span_id = None;
        span.next_span_id = None;
        let json = serde_json::to_string(&span).unwrap();
        assert!(!json.contains("cq_id"));
        assert!(!json.contains("\"page\""));
        assert!(!json.contains("bbox"));
        assert!(!json.contains("table_cell"));
        let rt: SourceSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(span, rt);
    }

    // -- Canonical JSON byte stability --

    #[test]
    fn corpus_document_canonical_stability() {
        let doc = fixture_corpus_document();
        let bytes1 = to_canonical_bytes(&doc);
        let bytes2 = to_canonical_bytes(&doc);
        assert_eq!(bytes1, bytes2);
        let h1 = content_hash(&doc);
        let h2 = content_hash(&doc);
        assert_eq!(h1, h2);
    }

    #[test]
    fn source_span_canonical_stability() {
        let span = fixture_source_span();
        let bytes1 = to_canonical_bytes(&span);
        let bytes2 = to_canonical_bytes(&span);
        assert_eq!(bytes1, bytes2);
        let h1 = content_hash(&span);
        let h2 = content_hash(&span);
        assert_eq!(h1, h2);
    }

    #[test]
    fn extracted_table_canonical_stability() {
        let table = fixture_extracted_table();
        let bytes1 = to_canonical_bytes(&table);
        let bytes2 = to_canonical_bytes(&table);
        assert_eq!(bytes1, bytes2);
        let h1 = content_hash(&table);
        let h2 = content_hash(&table);
        assert_eq!(h1, h2);
    }

    #[test]
    fn concept_canonical_stability() {
        let concept = fixture_concept();
        let bytes1 = to_canonical_bytes(&concept);
        let bytes2 = to_canonical_bytes(&concept);
        assert_eq!(bytes1, bytes2);
        let h1 = content_hash(&concept);
        let h2 = content_hash(&concept);
        assert_eq!(h1, h2);
    }

    #[test]
    fn terminology_binding_canonical_stability() {
        let binding = fixture_terminology_binding();
        let bytes1 = to_canonical_bytes(&binding);
        let bytes2 = to_canonical_bytes(&binding);
        assert_eq!(bytes1, bytes2);
        let h1 = content_hash(&binding);
        let h2 = content_hash(&binding);
        assert_eq!(h1, h2);
    }

    // -- Cross-type referential integrity --

    #[test]
    fn source_span_references_valid_doc() {
        let doc = fixture_corpus_document();
        let span = fixture_source_span();
        assert_eq!(span.doc_id, doc.doc_id);
    }

    #[test]
    fn extracted_table_references_valid_doc() {
        let doc = fixture_corpus_document();
        let table = fixture_extracted_table();
        assert_eq!(table.doc_id, doc.doc_id);
    }

    #[test]
    fn concept_bindings_match_count() {
        let concept = fixture_concept();
        assert_eq!(concept.terminology_bindings.len(), 1);
        assert_eq!(concept.terminology_bindings[0].system, "MEDIS");
    }

    // -- Table cell ref --

    #[test]
    fn table_cell_ref_roundtrip() {
        let cell = TableCellRef {
            table_id: ExtractedTableId::new("tbl_001"),
            row: 2,
            col: 3,
        };
        let json = serde_json::to_string(&cell).unwrap();
        let rt: TableCellRef = serde_json::from_str(&json).unwrap();
        assert_eq!(cell, rt);
    }

    // -- BBox --

    #[test]
    fn bbox_roundtrip() {
        let bbox = BBox {
            x: 72.0,
            y: 200.5,
            width: 468.0,
            height: 14.3,
        };
        let json = serde_json::to_string(&bbox).unwrap();
        let rt: BBox = serde_json::from_str(&json).unwrap();
        assert_eq!(bbox, rt);
    }

    // -- Extractor vote --

    #[test]
    fn extractor_vote_roundtrip() {
        let vote = ExtractorVote {
            extractor: "yomitoku".into(),
            raw_text: "テスト文字列".into(),
            confidence: 0.85,
        };
        let json = serde_json::to_string(&vote).unwrap();
        let rt: ExtractorVote = serde_json::from_str(&json).unwrap();
        assert_eq!(vote, rt);
    }

    // -- Source span with table cell --

    #[test]
    fn source_span_with_table_cell_roundtrip() {
        let mut span = fixture_source_span();
        span.table_cell = Some(TableCellRef {
            table_id: ExtractedTableId::new("tbl_vitals_001"),
            row: 0,
            col: 1,
        });
        span.bbox = None;
        let json = serde_json::to_string(&span).unwrap();
        let rt: SourceSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(span, rt);
    }

    // -- Distinct fixtures produce distinct hashes --

    #[test]
    fn distinct_types_distinct_hashes() {
        let h_doc = content_hash(&fixture_corpus_document());
        let h_span = content_hash(&fixture_source_span());
        let h_table = content_hash(&fixture_extracted_table());
        let h_concept = content_hash(&fixture_concept());
        let h_binding = content_hash(&fixture_terminology_binding());

        let hashes = [&h_doc, &h_span, &h_table, &h_concept, &h_binding];
        for (i, a) in hashes.iter().enumerate() {
            for b in hashes.iter().skip(i + 1) {
                assert_ne!(a, b, "hash collision between fixture types");
            }
        }
    }
}
