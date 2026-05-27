use serde::{Deserialize, Serialize};

use crate::enums::{Language, LicenseStatus, SourceType};
use crate::id::{ContentHash, CqId, DocId, ExtractedTableId, ManifestId, SpanId};

/// Page-coordinate bounding box for a source span.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Reference to a specific cell within an extracted table.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableCellRef {
    pub table_id: ExtractedTableId,
    pub row: u32,
    pub col: u32,
}

/// A single extractor's output and confidence for a text unit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtractorVote {
    pub extractor: String,
    pub text: String,
    pub confidence: f64,
}

/// SPEC §10: registered source with permission/version metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorpusDocument {
    pub doc_id: DocId,
    pub title_ja: String,
    pub title_en: Option<String>,
    pub source_type: SourceType,
    pub publisher: String,
    pub society: String,
    pub edition: String,
    pub publication_date: Option<String>,
    pub access_date: Option<String>,
    pub license_status: LicenseStatus,
    pub content_hash: ContentHash,
    pub extraction_manifest_id: ManifestId,
    pub supersedes: Option<DocId>,
    pub superseded_by: Option<DocId>,
}

/// SPEC §10: addressable text unit from a source document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub span_id: SpanId,
    pub doc_id: DocId,
    pub section_path: Vec<String>,
    pub cq_id: Option<CqId>,
    pub page: Option<u32>,
    pub bbox: Option<BoundingBox>,
    pub table_cell: Option<TableCellRef>,
    pub raw_text: String,
    pub nfkc_text: String,
    pub search_text: String,
    pub display_text: String,
    pub language: Language,
    pub previous_span_id: Option<SpanId>,
    pub next_span_id: Option<SpanId>,
    pub extractor_votes: Vec<ExtractorVote>,
    pub confidence: f64,
}

/// SPEC §10: extracted table with cell spans and structural metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtractedTable {
    pub table_id: ExtractedTableId,
    pub doc_id: DocId,
    pub caption_span_id: Option<SpanId>,
    pub cell_span_ids: Vec<SpanId>,
    pub row_headers: Vec<SpanId>,
    pub column_headers: Vec<SpanId>,
    pub reading_order: Vec<SpanId>,
    pub extraction_votes: Vec<ExtractorVote>,
    pub normalized_table_hash: ContentHash,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::content_hash_of;

    #[test]
    fn corpus_document_serde_roundtrip() {
        let doc = CorpusDocument {
            doc_id: DocId::new("sepsis-guideline-2024"),
            title_ja: "日本版敗血症診療ガイドライン2024".into(),
            title_en: Some(
                "Japanese Clinical Practice Guidelines for Sepsis 2024".into(),
            ),
            source_type: SourceType::Guideline,
            publisher: "日本集中治療医学会".into(),
            society: "JSICM".into(),
            edition: "2024".into(),
            publication_date: Some("2024-03-01".into()),
            access_date: Some("2025-01-15".into()),
            license_status: LicenseStatus::Licensed,
            content_hash: ContentHash::from_bytes(b"sepsis-guideline-content"),
            extraction_manifest_id: ManifestId::new("manifest-sepsis-001"),
            supersedes: Some(DocId::new("sepsis-guideline-2020")),
            superseded_by: None,
        };
        let json = serde_json::to_string(&doc).unwrap();
        let back: CorpusDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(back, doc);
    }

    #[test]
    fn source_span_serde_roundtrip() {
        let span = SourceSpan {
            span_id: SpanId::new("span-001"),
            doc_id: DocId::new("sepsis-guideline-2024"),
            section_path: vec!["CQ1".into(), "推奨".into()],
            cq_id: Some(CqId::new("cq-001")),
            page: Some(42),
            bbox: Some(BoundingBox {
                x: 72.0,
                y: 100.0,
                width: 468.0,
                height: 14.0,
            }),
            table_cell: None,
            raw_text: "敗血症にはβラクタム系抗菌薬を推奨する".into(),
            nfkc_text: "敗血症にはβラクタム系抗菌薬を推奨する".into(),
            search_text: "敗血症にはβラクタム系抗菌薬を推奨する".into(),
            display_text: "敗血症にはβラクタム系抗菌薬を推奨する".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: Some(SpanId::new("span-002")),
            extractor_votes: vec![ExtractorVote {
                extractor: "pymupdf".into(),
                text: "敗血症にはβラクタム系抗菌薬を推奨する".into(),
                confidence: 0.99,
            }],
            confidence: 0.99,
        };
        let json = serde_json::to_string(&span).unwrap();
        let back: SourceSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(back, span);
    }

    #[test]
    fn extracted_table_serde_roundtrip() {
        let table = ExtractedTable {
            table_id: ExtractedTableId::new("table-vs-001"),
            doc_id: DocId::new("sepsis-guideline-2024"),
            caption_span_id: Some(SpanId::new("span-100")),
            cell_span_ids: vec![SpanId::new("span-101"), SpanId::new("span-102")],
            row_headers: vec![SpanId::new("span-101")],
            column_headers: vec![SpanId::new("span-102")],
            reading_order: vec![SpanId::new("span-101"), SpanId::new("span-102")],
            extraction_votes: vec![],
            normalized_table_hash: ContentHash::from_bytes(b"table-content"),
        };
        let json = serde_json::to_string(&table).unwrap();
        let back: ExtractedTable = serde_json::from_str(&json).unwrap();
        assert_eq!(back, table);
    }

    #[test]
    fn corpus_document_canonical_hash_deterministic() {
        let doc = CorpusDocument {
            doc_id: DocId::new("test"),
            title_ja: "テスト".into(),
            title_en: None,
            source_type: SourceType::Textbook,
            publisher: "出版社".into(),
            society: "学会".into(),
            edition: "1".into(),
            publication_date: None,
            access_date: None,
            license_status: LicenseStatus::Open,
            content_hash: ContentHash::from_bytes(b""),
            extraction_manifest_id: ManifestId::new("m-1"),
            supersedes: None,
            superseded_by: None,
        };
        let h1 = content_hash_of(&doc).unwrap();
        let h2 = content_hash_of(&doc).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn bounding_box_serde() {
        let bbox = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 595.0,
            height: 842.0,
        };
        let json = serde_json::to_string(&bbox).unwrap();
        let back: BoundingBox = serde_json::from_str(&json).unwrap();
        assert_eq!(back, bbox);
    }

    #[test]
    fn table_cell_ref_serde() {
        let cell = TableCellRef {
            table_id: ExtractedTableId::new("table-001"),
            row: 2,
            col: 3,
        };
        let json = serde_json::to_string(&cell).unwrap();
        let back: TableCellRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cell);
    }
}
