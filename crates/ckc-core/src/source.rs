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
#[derive(
    Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
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
