//! Segment processing_stage (SPEC §8.3): wrapped [`SourceDocumentGraph`] → wrapped
//! [`SegmentIr`], `segments.json`'s payload.
//!
//! Rule-based segmentation keyed on the §8.2 test_source structure, walking
//! the graph's spans in reading order:
//!
//! - section heading spans: `CQ<digit>` prefix → `cq`, anything else →
//!   `metadata`; caption spans → `metadata`;
//! - paragraph spans: list-parented → `evidence` (M1 lists are evidence
//!   lists); exception sentence markers (ただし, を除く) → `exception`,
//!   checked before the recommendation markers (the §5 modality
//!   phrases) → `recommendation`;
//! - cell spans group into one segment per table row (the extract `row`
//!   attr): an all-header row → `table_row`; a body row → `definition`
//!   when the table's nearest spanned section ancestor's heading
//!   contains 定義, else `table_row`; the segment's regions are its
//!   cells' regions in canonical set order.
//!
//! Marker matching runs over `search_text` (semantic_ja), so width and
//! CJK-punctuation variants fold before comparison. Segment ids are
//! reading-order counter ids `seg.<k>`; misses never consume one. Every
//! span the rules cannot place — unclassified paragraphs, spans on
//! structural node kinds, cells without a groupable `row` attr, spans no
//! region grounds — yields one `segmentation_boundary_error` residual
//! instead of a segment, so the valid remainder keeps flowing (§4.4).

use std::collections::HashMap;
use std::fmt;

use ckc_core::{
    ArtifactWrapper, EvidenceStatus, CanonError, ClinicalSegment, DiagnosticCode, DiagnosticRecord, Id,
    NodeKind, Origin, Outcome, Producer, SegmentIr, SegmentKind, SourceDocumentGraph, SourceNode,
    SourceTextSpan, StringPolicy, canonical_sort_key, canonicalization_policy_hash, content_hash,
};

use crate::shell::static_id;

/// Segmentation failed mechanically. Rule misses are diagnostics, never
/// errors, so canonical emission while hashing the payload is the sole
/// variant.
#[derive(Debug)]
pub enum SegmentError {
    Canon(CanonError),
}

impl fmt::Display for SegmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SegmentError::Canon(e) => write!(f, "canonical emission: {e}"),
        }
    }
}

impl std::error::Error for SegmentError {}

impl From<CanonError> for SegmentError {
    fn from(e: CanonError) -> Self {
        SegmentError::Canon(e)
    }
}

/// The §5 modality phrases as segmentation markers: presence flags a
/// recommendation sentence; mapping them to direction/strength is the
/// normalize processing_stage's job.
const RECOMMENDATION_MARKERS: [&str; 7] = [
    "推奨する",
    "推奨しない",
    "提案する",
    "提案しない",
    "考慮してもよい",
    "禁忌",
    "投与しないこと",
];

/// Exception sentence markers (§8.2 ただし…を除く), checked before the
/// recommendation markers so a compound sentence segments as its
/// exception clause.
const EXCEPTION_MARKERS: [&str; 2] = ["ただし", "を除く"];

/// A definitions table announces itself through its governing heading.
const DEFINITION_HEADING_MARKER: &str = "定義";

/// Segment `source`'s graph and wrap the result per §4.4 —
/// `schema.segments`, artifact id `<document_id>.segments`,
/// `deterministic_compiler` origin under `mechanical_evidence_status`, the
/// consumed source document graph's content hash as the one input hash, misses in
/// the wrapper diagnostics, payload hashes computed here.
pub fn segment(
    source: &ArtifactWrapper<SourceDocumentGraph>,
    producer: &Producer,
) -> Result<ArtifactWrapper<SegmentIr>, SegmentError> {
    let (segments, diagnostics) = segment_graph(&source.payload);
    let ir = SegmentIr { segments };

    let document_id = &source.payload.document.document_id;
    let artifact_id = Id::new(format!("{document_id}.segments"))
        .expect("a valid document id keeps the Id grammar under a suffix");
    Ok(ArtifactWrapper {
        schema_id: static_id("schema.segments"),
        artifact_id,
        artifact_kind: static_id("segments"),
        producer: producer.clone(),
        input_hashes: vec![source.content_hash.clone()],
        content_hash: content_hash(&ir)?,
        canonicalization_policy_hash: canonicalization_policy_hash(),
        origin: Origin::DeterministicCompiler,
        evidence_status: EvidenceStatus::MechanicalEvidenceStatus,
        external_effects: vec![],
        trace_refs: vec![],
        diagnostics,
        runtime_metadata: vec![],
        payload: ir,
    })
}

/// One table row under construction: the cell spans sharing a (table,
/// `row` attr) key, in reading order.
struct RowBucket<'g> {
    table_id: &'g Id,
    all_header: bool,
    spans: Vec<&'g SourceTextSpan>,
}

/// Classify every span into segments and miss diagnostics, both in
/// reading order.
fn segment_graph(graph: &SourceDocumentGraph) -> (Vec<ClinicalSegment>, Vec<DiagnosticRecord>) {
    let nodes: HashMap<&Id, &SourceNode> = graph.nodes.iter().map(|n| (&n.node_id, n)).collect();
    // A node's heading text is its first span's search text (a section
    // heading spans the section node itself).
    let mut node_text: HashMap<&Id, &str> = HashMap::new();
    for span in &graph.spans {
        node_text.entry(&span.node_id).or_insert(&span.search_text);
    }
    // First region source_linkage each span — extract mints exactly one
    // {node,span} region per span.
    let mut span_region: HashMap<&Id, &Id> = HashMap::new();
    for region in &graph.regions {
        for span_id in &region.span_ids {
            span_region.entry(span_id).or_insert(&region.region_id);
        }
    }
    let mut spans: Vec<&SourceTextSpan> = graph.spans.iter().collect();
    spans.sort_by_key(|s| s.reading_order);

    // Pass 1: bucket groupable cell spans by (table, row attr); pass 2
    // emits each bucket at its first member so row segments hold their
    // reading-order position.
    let mut rows: Vec<RowBucket<'_>> = Vec::new();
    let mut row_index: HashMap<(&Id, &str), usize> = HashMap::new();
    let mut span_bucket: HashMap<&Id, usize> = HashMap::new();
    for span in &spans {
        let Some(node) = nodes.get(&span.node_id).copied() else {
            continue;
        };
        if node.kind != NodeKind::Cell {
            continue;
        }
        let (Some(table_id), Some(row)) = (node.parent_id.as_ref(), attr(node, "row")) else {
            continue;
        };
        let i = *row_index.entry((table_id, row)).or_insert_with(|| {
            rows.push(RowBucket {
                table_id,
                all_header: true,
                spans: Vec::new(),
            });
            rows.len() - 1
        });
        rows[i].all_header &= attr(node, "header") == Some("true");
        rows[i].spans.push(span);
        span_bucket.insert(&span.span_id, i);
    }

    let mut emitted = vec![false; rows.len()];
    let mut segments: Vec<ClinicalSegment> = Vec::new();
    let mut diagnostics: Vec<DiagnosticRecord> = Vec::new();
    for span in &spans {
        let Some(node) = nodes.get(&span.node_id).copied() else {
            miss(
                &mut diagnostics,
                format!("dangling node {}", span.node_id),
                vec![],
            );
            continue;
        };
        let Some(&region_id) = span_region.get(&span.span_id) else {
            miss(
                &mut diagnostics,
                format!("no region grounds span {}", span.span_id),
                vec![],
            );
            continue;
        };
        match node.kind {
            NodeKind::Section => {
                let kind = if is_cq_heading(&span.search_text) {
                    SegmentKind::Cq
                } else {
                    SegmentKind::Metadata
                };
                mint_segment(&mut segments, kind, vec![region_id.clone()]);
            }
            NodeKind::Caption => {
                mint_segment(
                    &mut segments,
                    SegmentKind::Metadata,
                    vec![region_id.clone()],
                );
            }
            NodeKind::Paragraph => {
                let kind = if parent_kind(&nodes, node) == Some(NodeKind::List) {
                    Some(SegmentKind::Evidence)
                } else if contains_any(&span.search_text, &EXCEPTION_MARKERS) {
                    Some(SegmentKind::Exception)
                } else if contains_any(&span.search_text, &RECOMMENDATION_MARKERS) {
                    Some(SegmentKind::Recommendation)
                } else {
                    None
                };
                match kind {
                    Some(kind) => mint_segment(&mut segments, kind, vec![region_id.clone()]),
                    None => miss(
                        &mut diagnostics,
                        format!("unclassified paragraph: {}", span.search_text),
                        vec![region_id.clone()],
                    ),
                }
            }
            NodeKind::Cell => match span_bucket.get(&span.span_id) {
                Some(&i) => {
                    if !emitted[i] {
                        emitted[i] = true;
                        let row = &rows[i];
                        let kind = if row.all_header {
                            SegmentKind::TableRow
                        } else if definition_table(&nodes, &node_text, row.table_id) {
                            SegmentKind::Definition
                        } else {
                            SegmentKind::TableRow
                        };
                        // An ungrounded member misses at its own span
                        // (the region guard above) and contributes no
                        // region here.
                        let region_ids: Vec<Id> = row
                            .spans
                            .iter()
                            .filter_map(|s| span_region.get(&s.span_id).copied().cloned())
                            .collect();
                        mint_segment(&mut segments, kind, region_ids);
                    }
                }
                None => miss(
                    &mut diagnostics,
                    format!("ungroupable cell {}", node.node_id),
                    vec![region_id.clone()],
                ),
            },
            kind => miss(
                &mut diagnostics,
                format!(
                    "no segmentation rule for {} span {}",
                    kind.as_str(),
                    span.span_id
                ),
                vec![region_id.clone()],
            ),
        }
    }
    (segments, diagnostics)
}

/// Mint the next reading-order segment `seg.<k>`; region ids are stored
/// in canonical set order so the stored value round-trips unchanged
/// through a strict read (the `derive_components` precedent).
fn mint_segment(segments: &mut Vec<ClinicalSegment>, kind: SegmentKind, mut region_ids: Vec<Id>) {
    region_ids.sort_by_cached_key(|id| {
        canonical_sort_key(id).expect("Id canonical emission is infallible")
    });
    let segment_id =
        Id::new(format!("seg.{}", segments.len())).expect("counter ids match the Id grammar");
    segments.push(ClinicalSegment {
        segment_id,
        kind,
        region_ids,
    });
}

/// One `segmentation_boundary_error` residual for a span the rules
/// cannot place, grounded in the span's region when one exists.
fn miss(diagnostics: &mut Vec<DiagnosticRecord>, detail: String, region_ids: Vec<Id>) {
    let detail = StringPolicy::DiagnosticText
        .normalize(&detail)
        .expect("diagnostic_text is infallible");
    diagnostics.push(DiagnosticRecord {
        code: DiagnosticCode::SegmentationBoundaryError,
        outcome: Outcome::Residual,
        payload: vec![(static_id("detail"), detail)],
        region_ids,
        artifact_hashes: vec![],
    });
}

/// The raw attr value under `key`, if present.
fn attr<'a>(node: &'a SourceNode, key: &str) -> Option<&'a str> {
    node.attrs
        .iter()
        .find(|(k, _)| k.as_str() == key)
        .map(|(_, v)| v.as_str())
}

fn parent_kind(nodes: &HashMap<&Id, &SourceNode>, node: &SourceNode) -> Option<NodeKind> {
    nodes.get(node.parent_id.as_ref()?).map(|n| n.kind)
}

/// `CQ` then an ASCII digit opens a clinical-question heading; NFKC in
/// search text already folds full-width ＣＱ１ to CQ1.
fn is_cq_heading(text: &str) -> bool {
    text.strip_prefix("CQ")
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))
}

fn contains_any(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|m| text.contains(m))
}

/// Whether the table's nearest spanned section ancestor's heading marks
/// a definitions table; the hop cap guards against parent cycles in
/// graphs no validator has seen.
fn definition_table(
    nodes: &HashMap<&Id, &SourceNode>,
    node_text: &HashMap<&Id, &str>,
    table_id: &Id,
) -> bool {
    let mut current = nodes.get(table_id).and_then(|n| n.parent_id.as_ref());
    for _ in 0..nodes.len() {
        let Some(node) = current.and_then(|id| nodes.get(id)) else {
            return false;
        };
        if node.kind == NodeKind::Section
            && let Some(text) = node_text.get(&node.node_id)
        {
            return text.contains(DEFINITION_HEADING_MARKER);
        }
        current = node.parent_id.as_ref();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::{ExtractConfig, extract};
    use ckc_core::{
        DataClass, Hash, Provenance, SourceDocument, EvidenceRegion, canonical_payload_bytes,
        hash_bytes, read_strict_canonical,
    };

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn producer() -> Producer {
        Producer {
            pipeline_id: id("cand.m1"),
            pipeline_step_id: id("processing_stage.segment"),
            toolchain_manifest_hash: Hash::new(format!("sha256:{}", "0".repeat(64))).unwrap(),
        }
    }

    fn extracted(html: &[u8]) -> ArtifactWrapper<SourceDocumentGraph> {
        let config = ExtractConfig {
            document_id: id("doc.test"),
            source_family: id("synthetic_test_source_html"),
            provenance: Provenance::Synthetic,
            data_class: DataClass::None,
            producer: producer(),
        };
        extract(html, &config).unwrap()
    }

    fn test_source(name: &str) -> Vec<u8> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/test_sources/");
        std::fs::read(format!("{dir}{name}")).unwrap()
    }

    fn shape(wrapper: &ArtifactWrapper<SegmentIr>) -> Vec<(Id, SegmentKind, Vec<Id>)> {
        wrapper
            .payload
            .segments
            .iter()
            .map(|s| (s.segment_id.clone(), s.kind, s.region_ids.clone()))
            .collect()
    }

    fn detail(d: &DiagnosticRecord) -> &str {
        assert_eq!(d.code, DiagnosticCode::SegmentationBoundaryError);
        assert_eq!(d.outcome, Outcome::Residual);
        assert!(d.artifact_hashes.is_empty());
        let [(key, value)] = d.payload.as_slice() else {
            panic!("payload is a single detail entry, got {:?}", d.payload);
        };
        assert_eq!(*key, id("detail"));
        value
    }

    // The committed m1_guideline_a test_source, every span placed and no
    // misses: title/heading metadata, the CQ heading, recommendation and
    // exception paragraphs, the th header row as table_row over the
    // definition body rows (region pairs in canonical set order — note
    // r.10 < r.9 by bytes), and the evidence list items.
    #[test]
    fn test_source_guideline_a_segments() {
        let source = extracted(&test_source("m1_guideline_a.html"));
        let wrapper = segment(&source, &producer()).unwrap();
        assert!(
            wrapper.diagnostics.is_empty(),
            "test_source segments residual-free, got {:?}",
            wrapper.diagnostics
        );
        let want: Vec<(Id, SegmentKind, Vec<Id>)> = [
            ("seg.0", SegmentKind::Metadata, vec!["r.0"]),
            ("seg.1", SegmentKind::Cq, vec!["r.1"]),
            ("seg.2", SegmentKind::Recommendation, vec!["r.2"]),
            ("seg.3", SegmentKind::Exception, vec!["r.3"]),
            ("seg.4", SegmentKind::Metadata, vec!["r.4"]),
            ("seg.5", SegmentKind::TableRow, vec!["r.5", "r.6"]),
            ("seg.6", SegmentKind::Definition, vec!["r.7", "r.8"]),
            ("seg.7", SegmentKind::Definition, vec!["r.10", "r.9"]),
            ("seg.8", SegmentKind::Definition, vec!["r.11", "r.12"]),
            ("seg.9", SegmentKind::Metadata, vec!["r.13"]),
            ("seg.10", SegmentKind::Evidence, vec!["r.14"]),
            ("seg.11", SegmentKind::Evidence, vec!["r.15"]),
        ]
        .into_iter()
        .map(|(s, kind, regions)| (id(s), kind, regions.into_iter().map(id).collect()))
        .collect();
        assert_eq!(shape(&wrapper), want);
    }

    // The two single-recommendation test_sources: 投与しないこと(禁忌) and
    // 禁忌である both flag recommendation sentences under metadata
    // headings.
    #[test]
    fn test_sources_b_and_control_segment_residual_free() {
        for name in ["m1_guideline_b.html", "m1_control.html"] {
            let wrapper = segment(&extracted(&test_source(name)), &producer()).unwrap();
            assert!(
                wrapper.diagnostics.is_empty(),
                "{name} segments residual-free, got {:?}",
                wrapper.diagnostics
            );
            let kinds: Vec<SegmentKind> =
                wrapper.payload.segments.iter().map(|s| s.kind).collect();
            assert_eq!(
                kinds,
                vec![
                    SegmentKind::Metadata,
                    SegmentKind::Metadata,
                    SegmentKind::Recommendation,
                ],
                "{name}"
            );
        }
    }

    // §4.4 wrapper shape: ids, deterministic_compiler origin under
    // mechanical_evidence_status, the consumed source document graph's content hash as
    // the one input hash, empty sets.
    #[test]
    fn wrapper_shape() {
        let source = extracted(
            "<!DOCTYPE html><html><body><p>成人には抗菌薬Aを推奨する。</p></body></html>"
                .as_bytes(),
        );
        let wrapper = segment(&source, &producer()).unwrap();
        assert_eq!(wrapper.schema_id, id("schema.segments"));
        assert_eq!(wrapper.artifact_id, id("doc.test.segments"));
        assert_eq!(wrapper.artifact_kind, id("segments"));
        assert_eq!(wrapper.producer, producer());
        assert_eq!(wrapper.origin, Origin::DeterministicCompiler);
        assert_eq!(wrapper.evidence_status, EvidenceStatus::MechanicalEvidenceStatus);
        assert_eq!(wrapper.input_hashes, vec![source.content_hash.clone()]);
        assert!(wrapper.external_effects.is_empty());
        assert!(wrapper.trace_refs.is_empty());
        assert!(wrapper.runtime_metadata.is_empty());
        wrapper.validate().unwrap();
    }

    // Determinism: identical input gives byte-identical wrappers, and
    // the bytes survive a strict read → re-emit cycle.
    #[test]
    fn double_segment_byte_identical_and_strict_reads() {
        let source = extracted(&test_source("m1_guideline_a.html"));
        let first = canonical_payload_bytes(&segment(&source, &producer()).unwrap()).unwrap();
        let second = canonical_payload_bytes(&segment(&source, &producer()).unwrap()).unwrap();
        assert_eq!(first, second, "double segment is byte-identical");
        let reread: ArtifactWrapper<SegmentIr> = read_strict_canonical(&first).unwrap();
        assert_eq!(
            canonical_payload_bytes(&reread).unwrap(),
            first,
            "strict read re-emits the same bytes"
        );
    }

    // A paragraph matching no marker yields one boundary residual
    // grounded in its region; the valid remainder keeps its segments and
    // misses consume no segment ids.
    #[test]
    fn unclassified_paragraph_is_boundary_residual() {
        let source = extracted(
            concat!(
                "<!DOCTYPE html><html><body>",
                "<h1>経過</h1>",
                "<p>経過観察を継続した。</p>",
                "<p>成人には抗菌薬Aを推奨する。</p>",
                "</body></html>"
            )
            .as_bytes(),
        );
        let wrapper = segment(&source, &producer()).unwrap();
        let [diag] = wrapper.diagnostics.as_slice() else {
            panic!("one miss, got {:?}", wrapper.diagnostics);
        };
        assert_eq!(detail(diag), "unclassified paragraph: 経過観察を継続した.");
        assert_eq!(diag.region_ids, vec![id("r.1")]);
        let want = vec![
            (id("seg.0"), SegmentKind::Metadata, vec![id("r.0")]),
            (id("seg.1"), SegmentKind::Recommendation, vec![id("r.2")]),
        ];
        assert_eq!(shape(&wrapper), want);
    }

    // Outside a 定義 heading every row is table_row (header and body
    // alike) and the caption is metadata.
    #[test]
    fn non_definition_table_rows_are_table_row() {
        let source = extracted(
            concat!(
                "<!DOCTYPE html><html><body>",
                "<h2>結果一覧</h2>",
                "<table><caption>表1</caption>",
                "<tr><th>群</th><th>値</th></tr>",
                "<tr><td>介入</td><td>0.5</td></tr>",
                "</table></body></html>"
            )
            .as_bytes(),
        );
        let wrapper = segment(&source, &producer()).unwrap();
        assert!(wrapper.diagnostics.is_empty());
        let want = vec![
            (id("seg.0"), SegmentKind::Metadata, vec![id("r.0")]),
            (id("seg.1"), SegmentKind::Metadata, vec![id("r.1")]),
            (
                id("seg.2"),
                SegmentKind::TableRow,
                vec![id("r.2"), id("r.3")],
            ),
            (
                id("seg.3"),
                SegmentKind::TableRow,
                vec![id("r.4"), id("r.5")],
            ),
        ];
        assert_eq!(shape(&wrapper), want);
    }

    // Robustness paths extract output cannot reach, on a hand-built
    // graph: a cell span without a row attr and a span no region grounds
    // each miss in reading order with no segments minted.
    #[test]
    fn ungroupable_cell_and_ungrounded_span_are_boundary_residuals() {
        let graph = SourceDocumentGraph {
            document: SourceDocument {
                document_id: id("doc.hand"),
                source_family: id("synthetic_test_source_html"),
                provenance: Provenance::Synthetic,
                raw_hash: hash_bytes(b""),
                content_hash: hash_bytes(b""),
                data_class: DataClass::None,
            },
            nodes: vec![
                SourceNode {
                    node_id: id("n.0"),
                    kind: NodeKind::Document,
                    parent_id: None,
                    attrs: vec![],
                },
                SourceNode {
                    node_id: id("n.1"),
                    kind: NodeKind::Cell,
                    parent_id: Some(id("n.0")),
                    attrs: vec![],
                },
                SourceNode {
                    node_id: id("n.2"),
                    kind: NodeKind::Paragraph,
                    parent_id: Some(id("n.0")),
                    attrs: vec![],
                },
            ],
            spans: vec![
                SourceTextSpan::derive(id("s.0"), id("n.1"), 0, "セル".to_owned(), 0),
                SourceTextSpan::derive(id("s.1"), id("n.2"), 0, "本文。".to_owned(), 1),
            ],
            anchors: vec![],
            regions: vec![EvidenceRegion {
                region_id: id("r.0"),
                node_ids: vec![id("n.1")],
                span_ids: vec![id("s.0")],
                anchor_ids: vec![],
            }],
        };
        let source = ArtifactWrapper {
            schema_id: id("schema.source_document_graph"),
            artifact_id: id("doc.hand.source_document_graph"),
            artifact_kind: id("source_document_graph"),
            producer: producer(),
            input_hashes: vec![],
            content_hash: content_hash(&graph).unwrap(),
            canonicalization_policy_hash: canonicalization_policy_hash(),
            origin: Origin::DeterministicCompiler,
            evidence_status: EvidenceStatus::MechanicalEvidenceStatus,
            external_effects: vec![],
            trace_refs: vec![],
            diagnostics: vec![],
            runtime_metadata: vec![],
            payload: graph,
        };
        let wrapper = segment(&source, &producer()).unwrap();
        assert!(wrapper.payload.segments.is_empty());
        let [cell, ungrounded] = wrapper.diagnostics.as_slice() else {
            panic!("two misses, got {:?}", wrapper.diagnostics);
        };
        assert_eq!(detail(cell), "ungroupable cell n.1");
        assert_eq!(cell.region_ids, vec![id("r.0")]);
        assert_eq!(detail(ungrounded), "no region grounds span s.1");
        assert!(ungrounded.region_ids.is_empty());
    }
}
