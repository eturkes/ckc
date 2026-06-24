//! Extract processing_stage core (SPEC §8.3): test_source HTML → grounded
//! [`SourceDocumentGraph`] in its §4.4 wrapper, `source_document_graph.json`'s payload.
//!
//! `stage-extract.1` lands the flow walker. html5ever parses the bytes
//! (guaranteeing the html/head/body skeleton); the body walk mints
//! counter ids `n.<k>`/`s.<k>`/`r.<k>` in walk order under the document
//! root: h1–h6 drive a section stack, p maps to paragraph, ul/ol map to
//! list with li children as paragraphs. Every nonempty trimmed textual
//! unit gets one span at offset 0 with strictly increasing reading order
//! plus one {node,span} region; whitespace-only units mint nothing;
//! anchors stay empty (§4.5 subspan anchors belong to later processing_stages).
//! Parse errors and unknown flow content become `extraction_uncertain`
//! residuals.
//!
//! `stage-extract.2` lands the table arm. A table node's direct
//! children scan as caption (at most one, minting a textual caption
//! node), colgroup/col (ignored), and thead/tbody/tfoot/tr (html5ever
//! wraps bare tr in tbody); rows flatten in document order. Each
//! nonempty cell parents directly to the table node with attrs `row`
//! and `col` as 0-based decimal strings plus `header` `"true"` on th,
//! absent on td — exactly the `DocIr::from_graph` cell requirements; an
//! empty cell mints no node yet still occupies its column index. Any
//! rowspan or colspan other than `"1"`, nested table, second caption,
//! stray non-whitespace text, or unknown child element rejects the
//! whole table: one `table_structure_uncertain` residual names the
//! table node and every cell is withheld while the table node stays
//! (DocIr then withholds the table from the view).

use std::collections::HashMap;
use std::fmt;

use ckc_core::{
    ArtifactWrapper, CanonError, DataClass, DiagnosticCode, DiagnosticRecord, EvidenceRegion,
    EvidenceStatus, Id, NodeKind, Origin, Outcome, Producer, Provenance, SourceDocument,
    SourceDocumentGraph, SourceLinkageError, SourceNode, SourceTextSpan, StringPolicy,
    canonicalization_policy_hash, content_hash, hash_bytes,
};
use ego_tree::NodeRef;
use scraper::Html;
use scraper::node::Node;

use crate::shell::static_id;

/// Extractor-fixed identity for one document: everything the §4.5
/// [`SourceDocument`] and the §4.4 wrapper need beyond the input bytes.
#[derive(Debug, Clone)]
pub struct ExtractConfig {
    pub document_id: Id,
    /// Open vocabulary, e.g. `synthetic_test_source_html` (§4.5).
    pub source_family: Id,
    pub provenance: Provenance,
    pub data_class: DataClass,
    /// Rides the wrapper verbatim; the runner owns its values.
    pub producer: Producer,
}

/// Extraction failed before an wrapper could form. Parse trouble is
/// never an error — html5ever recovers and the walker emits
/// `extraction_uncertain` residuals — so the variants are the §4.4/§4.5
/// mechanical invariants plus input decoding.
#[derive(Debug)]
pub enum ExtractError {
    /// Input bytes are not UTF-8 (M1 test_sources carry no charset layer).
    Utf8(std::str::Utf8Error),
    /// The built graph violates a §4.5 invariant (extractor bug).
    SourceLinkage(SourceLinkageError),
    /// Canonical emission failed while hashing the payload.
    Canon(CanonError),
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractError::Utf8(e) => write!(f, "input is not UTF-8: {e}"),
            ExtractError::SourceLinkage(e) => write!(f, "source_linkage invariant: {e}"),
            ExtractError::Canon(e) => write!(f, "canonical emission: {e}"),
        }
    }
}

impl std::error::Error for ExtractError {}

impl From<SourceLinkageError> for ExtractError {
    fn from(e: SourceLinkageError) -> Self {
        ExtractError::SourceLinkage(e)
    }
}

impl From<CanonError> for ExtractError {
    fn from(e: CanonError) -> Self {
        ExtractError::Canon(e)
    }
}

/// Parse `html` and build the wrapped source document graph: walk the body,
/// validate the graph (residual-licensed), and wrap it per §4.4 —
/// `schema.source_document_graph`, artifact id `<document_id>.source_document_graph`,
/// `deterministic_compiler` origin under `mechanical_evidence_status`, empty
/// input/effect/trace/runtime sets, payload hashes computed here.
pub fn extract(
    html: &[u8],
    config: &ExtractConfig,
) -> Result<ArtifactWrapper<SourceDocumentGraph>, ExtractError> {
    let text = std::str::from_utf8(html).map_err(ExtractError::Utf8)?;
    let parsed = Html::parse_document(text);

    let mut walker = Walker::default();
    let doc_node = walker.mint_node(NodeKind::Document, None);
    for error in &parsed.errors {
        walker.residual(
            DiagnosticCode::ExtractionUncertain,
            format!("parse error: {error}"),
            &doc_node,
        );
    }
    walker.walk_body(find_body(&parsed), &doc_node);

    let graph = SourceDocumentGraph {
        document: SourceDocument {
            document_id: config.document_id.clone(),
            source_family: config.source_family.clone(),
            provenance: config.provenance,
            // No transport or charset decoding applies to test_source bytes,
            // so acquired and consumed bytes coincide (§4.5).
            raw_hash: hash_bytes(html),
            content_hash: hash_bytes(html),
            data_class: config.data_class,
        },
        nodes: walker.nodes,
        spans: walker.spans,
        anchors: vec![],
        regions: walker.regions,
    };
    graph.validate(&walker.residual_nodes)?;

    let artifact_id = Id::new(format!("{}.source_document_graph", config.document_id))
        .expect("a valid document id keeps the Id grammar under a suffix");
    Ok(ArtifactWrapper {
        schema_id: static_id("schema.source_document_graph"),
        artifact_id,
        artifact_kind: static_id("source_document_graph"),
        producer: config.producer.clone(),
        input_hashes: vec![],
        content_hash: content_hash(&graph)?,
        canonicalization_policy_hash: canonicalization_policy_hash(),
        origin: Origin::DeterministicCompiler,
        evidence_status: EvidenceStatus::MechanicalEvidenceStatus,
        external_effects: vec![],
        trace_refs: vec![],
        diagnostics: walker.diagnostics,
        runtime_metadata: vec![],
        payload: graph,
    })
}

/// The body element node. html5ever's tree builder always synthesizes
/// the html/head/body skeleton, so absence is unreachable.
fn find_body(html: &Html) -> NodeRef<'_, Node> {
    fn is_element(n: &NodeRef<'_, Node>, name: &str) -> bool {
        n.value().as_element().is_some_and(|e| e.name() == name)
    }
    let root = html
        .tree
        .root()
        .children()
        .find(|n| is_element(n, "html"))
        .expect("html5ever guarantees the html element");
    root.children()
        .find(|n| is_element(n, "body"))
        .expect("html5ever guarantees the body element")
}

/// `h1`–`h6` → 1–6; `None` for every other element name.
fn heading_level(name: &str) -> Option<u8> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

/// Concatenated descendant text of `node` in document order; phrasing
/// markup inside a textual unit flattens to its text.
fn collect_text(node: NodeRef<'_, Node>) -> String {
    let mut text = String::new();
    for descendant in node.descendants() {
        if let Node::Text(t) = descendant.value() {
            text.push_str(&t.text);
        }
    }
    text
}

/// Walk state: the four §4.5 pools under construction plus the residual
/// bookkeeping `extract` feeds to [`SourceDocumentGraph::validate`]. Counter ids
/// derive from pool lengths, so mint order is id order.
#[derive(Default)]
struct Walker {
    nodes: Vec<SourceNode>,
    spans: Vec<SourceTextSpan>,
    regions: Vec<EvidenceRegion>,
    diagnostics: Vec<DiagnosticRecord>,
    /// Memoized node-only regions: one region per node grounds every
    /// residual naming that node.
    node_regions: HashMap<Id, Id>,
    /// Nodes named by residual regions, licensing §4.5 coverage.
    residual_nodes: Vec<Id>,
}

impl Walker {
    fn mint_node(&mut self, kind: NodeKind, parent: Option<&Id>) -> Id {
        self.mint_node_attrs(kind, parent, vec![])
    }

    fn mint_node_attrs(
        &mut self,
        kind: NodeKind,
        parent: Option<&Id>,
        attrs: Vec<(Id, String)>,
    ) -> Id {
        let node_id = counter_id("n", self.nodes.len());
        self.nodes.push(SourceNode {
            node_id: node_id.clone(),
            kind,
            parent_id: parent.cloned(),
            attrs,
        });
        node_id
    }

    /// One span over the whole textual unit (offset 0) with its
    /// {node,span} region; reading order is the span counter.
    fn mint_span(&mut self, node_id: &Id, raw_text: String) {
        let k = self.spans.len();
        let span_id = counter_id("s", k);
        self.spans.push(SourceTextSpan::derive(
            span_id.clone(),
            node_id.clone(),
            0,
            raw_text,
            k as u64,
        ));
        let region_id = counter_id("r", self.regions.len());
        self.regions.push(EvidenceRegion {
            region_id,
            node_ids: vec![node_id.clone()],
            span_ids: vec![span_id],
            anchor_ids: vec![],
        });
    }

    /// The memoized node-only region for `node_id`, minting on first use.
    fn node_region(&mut self, node_id: &Id) -> Id {
        if let Some(region_id) = self.node_regions.get(node_id) {
            return region_id.clone();
        }
        let region_id = counter_id("r", self.regions.len());
        self.regions.push(EvidenceRegion {
            region_id: region_id.clone(),
            node_ids: vec![node_id.clone()],
            span_ids: vec![],
            anchor_ids: vec![],
        });
        self.node_regions.insert(node_id.clone(), region_id.clone());
        region_id
    }

    /// One residual under `code` grounded in `node_id`'s memoized
    /// region, licensing that node for §4.5 coverage.
    fn residual(&mut self, code: DiagnosticCode, detail: String, node_id: &Id) {
        let region_id = self.node_region(node_id);
        self.residual_nodes.push(node_id.clone());
        let detail = StringPolicy::DiagnosticText
            .normalize(&detail)
            .expect("diagnostic_text is infallible");
        self.diagnostics.push(DiagnosticRecord {
            code,
            outcome: Outcome::Residual,
            payload: vec![(static_id("detail"), detail)],
            region_ids: vec![region_id],
            artifact_hashes: vec![],
        });
    }

    /// Walk the body's children: headings drive the section stack (pop
    /// depths >= level; the heading text spans the section node itself),
    /// known flow elements mint their nodes, everything else is residual.
    fn walk_body(&mut self, body: NodeRef<'_, Node>, doc: &Id) {
        let mut stack: Vec<(u8, Id)> = Vec::new();
        for child in body.children() {
            match child.value() {
                Node::Element(element) => {
                    let name = element.name();
                    if let Some(level) = heading_level(name) {
                        while stack.last().is_some_and(|&(depth, _)| depth >= level) {
                            stack.pop();
                        }
                        let parent = stack.last().map_or(doc, |(_, id)| id).clone();
                        let section = self.mint_node(NodeKind::Section, Some(&parent));
                        self.span_text(child, &section);
                        stack.push((level, section));
                    } else {
                        let parent = stack.last().map_or(doc, |(_, id)| id).clone();
                        self.flow_element(child, name, &parent);
                    }
                }
                Node::Text(t) => {
                    let parent = stack.last().map_or(doc, |(_, id)| id).clone();
                    self.stray_text(&t.text, &parent);
                }
                _ => {}
            }
        }
    }

    /// One non-heading flow element at structural level. Unknown names
    /// leave one residual and skip the subtree.
    fn flow_element(&mut self, node: NodeRef<'_, Node>, name: &str, parent: &Id) {
        match name {
            "p" => self.textual_node(node, NodeKind::Paragraph, parent),
            "ul" | "ol" => self.list(node, parent),
            "table" => self.table(node, parent),
            _ => self.residual(
                DiagnosticCode::ExtractionUncertain,
                format!("unknown flow element: {name}"),
                parent,
            ),
        }
    }

    /// A textual-kind node: minted only when its trimmed text is
    /// nonempty, so every textual node carries its span by construction.
    fn textual_node(&mut self, node: NodeRef<'_, Node>, kind: NodeKind, parent: &Id) {
        let text = collect_text(node);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        let node_id = self.mint_node(kind, Some(parent));
        self.mint_span(&node_id, trimmed.to_owned());
    }

    /// Span `node`'s trimmed text onto an already-minted structural node
    /// (a section holding its heading text); whitespace-only mints
    /// nothing and structural kinds carry no coverage obligation.
    fn span_text(&mut self, node: NodeRef<'_, Node>, node_id: &Id) {
        let text = collect_text(node);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.mint_span(node_id, trimmed.to_owned());
        }
    }

    /// ul/ol: a list node whose li children are paragraphs; any other
    /// non-whitespace content inside the list is residual.
    fn list(&mut self, node: NodeRef<'_, Node>, parent: &Id) {
        let list_id = self.mint_node(NodeKind::List, Some(parent));
        for child in node.children() {
            match child.value() {
                Node::Element(element) if element.name() == "li" => {
                    self.textual_node(child, NodeKind::Paragraph, &list_id);
                }
                Node::Element(element) => {
                    self.residual(
                        DiagnosticCode::ExtractionUncertain,
                        format!("unknown flow element: {}", element.name()),
                        &list_id,
                    );
                }
                Node::Text(t) => self.stray_text(&t.text, &list_id),
                _ => {}
            }
        }
    }

    /// A table: the node mints unconditionally, then the subtree scan
    /// decides between minting the validated plan and one
    /// `table_structure_uncertain` residual naming the table node with
    /// every cell withheld.
    fn table(&mut self, node: NodeRef<'_, Node>, parent: &Id) {
        let table_id = self.mint_node(NodeKind::Table, Some(parent));
        match scan_table(node) {
            Ok(items) => self.mint_table_items(&table_id, items),
            Err(detail) => {
                self.residual(DiagnosticCode::TableStructureUncertain, detail, &table_id);
            }
        }
    }

    /// Mint an accepted table plan in document order: captions as
    /// textual caption nodes, cells parented directly to the table node
    /// with the `DocIr::from_graph` attrs (`col`/`row` 0-based decimal,
    /// `header` `"true"` on th only); an empty cell mints no node yet
    /// still occupies its column index.
    fn mint_table_items(&mut self, table_id: &Id, items: Vec<TableItem>) {
        let mut row = 0usize;
        for item in items {
            match item {
                TableItem::Caption(text) => {
                    let caption_id = self.mint_node(NodeKind::Caption, Some(table_id));
                    self.mint_span(&caption_id, text);
                }
                TableItem::Row(cells) => {
                    for (col, cell) in cells.into_iter().enumerate() {
                        let Some((header, text)) = cell else { continue };
                        let mut attrs = vec![(static_id("col"), col.to_string())];
                        if header {
                            attrs.push((static_id("header"), "true".to_owned()));
                        }
                        attrs.push((static_id("row"), row.to_string()));
                        let cell_id = self.mint_node_attrs(NodeKind::Cell, Some(table_id), attrs);
                        self.mint_span(&cell_id, text);
                    }
                    row += 1;
                }
            }
        }
    }

    /// Non-whitespace text outside any textual unit: one residual
    /// grounded in the enclosing node; whitespace-only mints nothing.
    fn stray_text(&mut self, text: &str, parent: &Id) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.residual(
                DiagnosticCode::ExtractionUncertain,
                format!("stray text: {trimmed}"),
                parent,
            );
        }
    }
}

/// `<prefix>.<k>`: the walk-order counter ids of §8.3 extract.
fn counter_id(prefix: &str, k: usize) -> Id {
    Id::new(format!("{prefix}.{k}")).expect("counter ids match the Id grammar")
}

/// One document-order unit of a validated table plan; minting happens
/// only after the whole scan accepts, so a rejection withholds every
/// cell without disturbing counter ids.
enum TableItem {
    /// Nonempty trimmed caption text.
    Caption(String),
    /// One row, indexed by column: `None` is an empty cell (occupies
    /// the index, mints nothing), `Some((header, text))` a th/td cell.
    Row(Vec<Option<(bool, String)>>),
}

/// Scan a table element's direct children into the validated plan;
/// `Err` carries the `table_structure_uncertain` detail.
fn scan_table(table: NodeRef<'_, Node>) -> Result<Vec<TableItem>, String> {
    let mut items = Vec::new();
    let mut caption_seen = false;
    for child in table.children() {
        match child.value() {
            Node::Element(element) => match element.name() {
                "caption" => {
                    if caption_seen {
                        return Err("second caption".to_owned());
                    }
                    caption_seen = true;
                    reject_nested_table(child)?;
                    let text = collect_text(child);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        items.push(TableItem::Caption(trimmed.to_owned()));
                    }
                }
                "colgroup" | "col" => {}
                "thead" | "tbody" | "tfoot" => scan_row_section(child, &mut items)?,
                "tr" => items.push(TableItem::Row(scan_row(child)?)),
                name => return Err(format!("unknown table child: {name}")),
            },
            Node::Text(t) => reject_stray_text(&t.text)?,
            _ => {}
        }
    }
    Ok(items)
}

/// thead/tbody/tfoot: rows only, flattened into `items` in document
/// order.
fn scan_row_section(section: NodeRef<'_, Node>, items: &mut Vec<TableItem>) -> Result<(), String> {
    for child in section.children() {
        match child.value() {
            Node::Element(element) => match element.name() {
                "tr" => items.push(TableItem::Row(scan_row(child)?)),
                name => return Err(format!("unknown table child: {name}")),
            },
            Node::Text(t) => reject_stray_text(&t.text)?,
            _ => {}
        }
    }
    Ok(())
}

/// One tr: th/td children in column order. Any rowspan or colspan
/// other than `"1"` rejects — the cell grid must stay literal for the
/// `row`/`col` attrs to mean what `DocIr::from_graph` reads.
fn scan_row(tr: NodeRef<'_, Node>) -> Result<Vec<Option<(bool, String)>>, String> {
    let mut cells = Vec::new();
    for child in tr.children() {
        match child.value() {
            Node::Element(element) => match element.name() {
                name @ ("th" | "td") => {
                    for key in ["rowspan", "colspan"] {
                        if let Some(value) = element.attr(key)
                            && value != "1"
                        {
                            return Err(format!("{key} {value} on {name}"));
                        }
                    }
                    reject_nested_table(child)?;
                    let text = collect_text(child);
                    let trimmed = text.trim();
                    cells.push((!trimmed.is_empty()).then(|| (name == "th", trimmed.to_owned())));
                }
                name => return Err(format!("unknown table child: {name}")),
            },
            Node::Text(t) => reject_stray_text(&t.text)?,
            _ => {}
        }
    }
    Ok(cells)
}

/// A table nested under a caption or cell defeats the literal grid.
fn reject_nested_table(node: NodeRef<'_, Node>) -> Result<(), String> {
    let nested = node
        .descendants()
        .any(|d| d.value().as_element().is_some_and(|e| e.name() == "table"));
    if nested {
        return Err("nested table".to_owned());
    }
    Ok(())
}

/// Non-whitespace text between table structure elements rejects;
/// whitespace is layout.
fn reject_stray_text(text: &str) -> Result<(), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Ok(())
    } else {
        Err(format!("stray text in table: {trimmed}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{Hash, canonical_payload_bytes, read_strict_canonical};

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn config() -> ExtractConfig {
        ExtractConfig {
            document_id: id("doc.test"),
            source_family: id("synthetic_test_source_html"),
            provenance: Provenance::Synthetic,
            data_class: DataClass::None,
            producer: Producer {
                pipeline_id: id("pipe.test"),
                pipeline_step_id: id("processing_stage.test.extract"),
                toolchain_manifest_hash: Hash::new(format!("sha256:{}", "0".repeat(64))).unwrap(),
            },
        }
    }

    /// Extract `html`, returning the payload graph and the wrapper
    /// diagnostics.
    fn graph(html: &str) -> (SourceDocumentGraph, Vec<DiagnosticRecord>) {
        let wrapper = extract(html.as_bytes(), &config()).unwrap();
        (wrapper.payload, wrapper.diagnostics)
    }

    fn node_shape(g: &SourceDocumentGraph) -> Vec<(Id, NodeKind, Option<Id>)> {
        g.nodes
            .iter()
            .map(|n| (n.node_id.clone(), n.kind, n.parent_id.clone()))
            .collect()
    }

    fn span_shape(g: &SourceDocumentGraph) -> Vec<(Id, Id, String, u64)> {
        g.spans
            .iter()
            .map(|s| {
                (
                    s.span_id.clone(),
                    s.node_id.clone(),
                    s.raw_text.clone(),
                    s.reading_order,
                )
            })
            .collect()
    }

    fn region_shape(g: &SourceDocumentGraph) -> Vec<(Id, Vec<Id>, Vec<Id>)> {
        g.regions
            .iter()
            .map(|r| (r.region_id.clone(), r.node_ids.clone(), r.span_ids.clone()))
            .collect()
    }

    fn detail_of(d: &DiagnosticRecord, code: DiagnosticCode) -> &str {
        assert_eq!(d.code, code);
        assert_eq!(d.outcome, Outcome::Residual);
        assert!(d.artifact_hashes.is_empty());
        let [(key, value)] = d.payload.as_slice() else {
            panic!("payload is a single detail entry, got {:?}", d.payload);
        };
        assert_eq!(*key, id("detail"));
        value
    }

    fn detail(d: &DiagnosticRecord) -> &str {
        detail_of(d, DiagnosticCode::ExtractionUncertain)
    }

    // Sections stack by heading level (h1 pops the h1+h2 pair), p and
    // ul/li mint under the open section, every nonempty unit gets one
    // offset-0 span with strictly increasing reading order and one
    // {node,span} region.
    #[test]
    fn walk_shape_sections_paragraphs_lists() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            "<h1>ＣＱ１：高血圧の治療</h1>",
            "<p>成人には降圧薬を推奨する。</p>",
            "<h2>詳細</h2>",
            "<p>小児には注意を要する。</p>",
            "<h1>第二章</h1>",
            "<ul><li>項目一</li><li>項目二</li></ul>",
            "</body></html>"
        ));
        assert!(
            diags.is_empty(),
            "clean test_source HTML extracts residual-free"
        );
        assert_eq!(
            node_shape(&g),
            vec![
                (id("n.0"), NodeKind::Document, None),
                (id("n.1"), NodeKind::Section, Some(id("n.0"))),
                (id("n.2"), NodeKind::Paragraph, Some(id("n.1"))),
                (id("n.3"), NodeKind::Section, Some(id("n.1"))),
                (id("n.4"), NodeKind::Paragraph, Some(id("n.3"))),
                (id("n.5"), NodeKind::Section, Some(id("n.0"))),
                (id("n.6"), NodeKind::List, Some(id("n.5"))),
                (id("n.7"), NodeKind::Paragraph, Some(id("n.6"))),
                (id("n.8"), NodeKind::Paragraph, Some(id("n.6"))),
            ]
        );
        assert_eq!(
            span_shape(&g),
            vec![
                (id("s.0"), id("n.1"), "ＣＱ１：高血圧の治療".to_owned(), 0),
                (
                    id("s.1"),
                    id("n.2"),
                    "成人には降圧薬を推奨する。".to_owned(),
                    1
                ),
                (id("s.2"), id("n.3"), "詳細".to_owned(), 2),
                (id("s.3"), id("n.4"), "小児には注意を要する。".to_owned(), 3),
                (id("s.4"), id("n.5"), "第二章".to_owned(), 4),
                (id("s.5"), id("n.7"), "項目一".to_owned(), 5),
                (id("s.6"), id("n.8"), "項目二".to_owned(), 6),
            ]
        );
        let want: Vec<(Id, Vec<Id>, Vec<Id>)> = [
            ("r.0", "n.1", "s.0"),
            ("r.1", "n.2", "s.1"),
            ("r.2", "n.3", "s.2"),
            ("r.3", "n.4", "s.3"),
            ("r.4", "n.5", "s.4"),
            ("r.5", "n.7", "s.5"),
            ("r.6", "n.8", "s.6"),
        ]
        .iter()
        .map(|(r, n, s)| (id(r), vec![id(n)], vec![id(s)]))
        .collect();
        assert_eq!(region_shape(&g), want);
        assert!(g.spans.iter().all(|s| s.start == 0));
        assert!(g.anchors.is_empty());
    }

    // Whitespace-only units mint nothing: an empty paragraph leaves no
    // node, an empty heading still opens its (structural, spanless)
    // section, whitespace text mints no residual.
    #[test]
    fn whitespace_only_units_mint_nothing() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            "<h2>   </h2><p>  </p><p>本文。</p><ul> </ul>  ",
            "</body></html>"
        ));
        assert!(diags.is_empty());
        assert_eq!(
            node_shape(&g),
            vec![
                (id("n.0"), NodeKind::Document, None),
                (id("n.1"), NodeKind::Section, Some(id("n.0"))),
                (id("n.2"), NodeKind::Paragraph, Some(id("n.1"))),
                (id("n.3"), NodeKind::List, Some(id("n.1"))),
            ]
        );
        assert_eq!(
            span_shape(&g),
            vec![(id("s.0"), id("n.2"), "本文。".to_owned(), 0)]
        );
        assert_eq!(
            region_shape(&g),
            vec![(id("r.0"), vec![id("n.2")], vec![id("s.0")])]
        );
    }

    // One residual per Html::errors entry ("Duplicate attribute" pinned
    // from observed html5ever output), all grounded in one memoized
    // whole-document region minted before any walk region.
    #[test]
    fn parse_errors_share_the_memoized_document_region() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            r#"<p id="a" id="a">甲</p><p id="b" id="b">乙</p>"#,
            "</body></html>"
        ));
        assert_eq!(diags.len(), 2);
        for d in &diags {
            assert_eq!(detail(d), "parse error: Duplicate attribute");
            assert_eq!(d.region_ids, vec![id("r.0")]);
        }
        assert_eq!(
            region_shape(&g),
            vec![
                (id("r.0"), vec![id("n.0")], vec![]),
                (id("r.1"), vec![id("n.1")], vec![id("s.0")]),
                (id("r.2"), vec![id("n.2")], vec![id("s.1")]),
            ]
        );
    }

    // Unknown flow elements and stray text leave one residual each,
    // grounded in the parent node's memoized region; their subtrees
    // mint nothing.
    #[test]
    fn unknown_flow_and_stray_text_residuals() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            "<h1>見出し</h1>",
            "はぐれた本文",
            "<blockquote>引用</blockquote>",
            "<div><p>内部</p></div>",
            "</body></html>"
        ));
        let details: Vec<&str> = diags.iter().map(detail).collect();
        assert_eq!(
            details,
            vec![
                "stray text: はぐれた本文",
                "unknown flow element: blockquote",
                "unknown flow element: div",
            ]
        );
        for d in &diags {
            assert_eq!(d.region_ids, vec![id("r.1")], "memoized parent region");
        }
        // the skipped subtrees mint no nodes
        assert_eq!(
            node_shape(&g),
            vec![
                (id("n.0"), NodeKind::Document, None),
                (id("n.1"), NodeKind::Section, Some(id("n.0"))),
            ]
        );
        assert_eq!(
            region_shape(&g),
            vec![
                (id("r.0"), vec![id("n.1")], vec![id("s.0")]),
                (id("r.1"), vec![id("n.1")], vec![]),
            ]
        );
    }

    // §4.4 wrapper shape: ids, deterministic_compiler origin under
    // mechanical_evidence_status, empty sets, both derived hashes valid, and
    // the §4.5 document identity hashing the input bytes raw.
    #[test]
    fn wrapper_shape() {
        let html = "<!DOCTYPE html><html><body><p>本文。</p></body></html>";
        let wrapper = extract(html.as_bytes(), &config()).unwrap();
        assert_eq!(wrapper.schema_id, id("schema.source_document_graph"));
        assert_eq!(wrapper.artifact_id, id("doc.test.source_document_graph"));
        assert_eq!(wrapper.artifact_kind, id("source_document_graph"));
        assert_eq!(wrapper.producer, config().producer);
        assert_eq!(wrapper.origin, Origin::DeterministicCompiler);
        assert_eq!(
            wrapper.evidence_status,
            EvidenceStatus::MechanicalEvidenceStatus
        );
        assert!(wrapper.input_hashes.is_empty());
        assert!(wrapper.external_effects.is_empty());
        assert!(wrapper.trace_refs.is_empty());
        assert!(wrapper.runtime_metadata.is_empty());
        wrapper.validate().unwrap();
        let document = &wrapper.payload.document;
        assert_eq!(document.document_id, id("doc.test"));
        assert_eq!(document.source_family, id("synthetic_test_source_html"));
        assert_eq!(document.provenance, Provenance::Synthetic);
        assert_eq!(document.data_class, DataClass::None);
        assert_eq!(document.raw_hash, hash_bytes(html.as_bytes()));
        assert_eq!(document.content_hash, document.raw_hash);
    }

    // §4.5 determinism: identical bytes and config give byte-identical
    // wrappers, and the bytes survive a strict read → re-emit cycle.
    #[test]
    fn double_extract_byte_identical_and_strict_reads() {
        let html = concat!(
            "<!DOCTYPE html><html><body>",
            "<h1>ＣＱ１：高血圧の治療</h1>",
            "<p>成人には降圧薬を推奨する。</p>",
            "<table><tr><td>表</td></tr></table>",
            "<ul><li>項目一</li><li>項目二</li></ul>",
            "</body></html>"
        );
        let first = canonical_payload_bytes(&extract(html.as_bytes(), &config()).unwrap()).unwrap();
        let second =
            canonical_payload_bytes(&extract(html.as_bytes(), &config()).unwrap()).unwrap();
        assert_eq!(first, second, "double extract is byte-identical");
        let reread: ArtifactWrapper<SourceDocumentGraph> = read_strict_canonical(&first).unwrap();
        assert_eq!(
            canonical_payload_bytes(&reread).unwrap(),
            first,
            "strict read re-emits the same bytes"
        );
    }

    #[test]
    fn non_utf8_input_is_utf8_error() {
        let err = extract(&[0xff, 0xfe, b'<', b'p', b'>'], &config()).unwrap_err();
        assert!(matches!(err, ExtractError::Utf8(_)), "got {err}");
    }

    fn test_source(name: &str) -> Vec<u8> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/test_sources/");
        std::fs::read(format!("{dir}{name}")).unwrap()
    }

    /// The cell-attr vocabulary `DocIr::from_graph` reads: `col`/`row`
    /// 0-based decimal, `header` `"true"` on th only, stored sorted.
    fn cell_attrs(row: u64, col: u64, header: bool) -> Vec<(Id, String)> {
        let mut attrs = vec![(id("col"), col.to_string())];
        if header {
            attrs.push((id("header"), "true".to_owned()));
        }
        attrs.push((id("row"), row.to_string()));
        attrs
    }

    /// Every region is the {node,span} pair of the same-index span —
    /// the walker's only region shape outside residual source_linkage.
    fn assert_span_regions(g: &SourceDocumentGraph) {
        assert_eq!(g.regions.len(), g.spans.len());
        for (region, span) in g.regions.iter().zip(&g.spans) {
            assert_eq!(region.node_ids, vec![span.node_id.clone()]);
            assert_eq!(region.span_ids, vec![span.span_id.clone()]);
            assert!(region.anchor_ids.is_empty());
        }
    }

    // The committed m1_guideline_a test_source, full shape pinned from
    // observed output: section tree, recommendation and exception
    // paragraphs, the 4x2 definitions table with th header row, and
    // the evidence list; DocIr::from_graph accepts the result.
    #[test]
    fn test_source_guideline_a_full_shape_and_doc_ir() {
        let wrapper = extract(&test_source("m1_guideline_a.html"), &config()).unwrap();
        let (g, diags) = (wrapper.payload, wrapper.diagnostics);
        assert!(diags.is_empty(), "test_source extracts residual-free");

        let n = |k: usize| id(&format!("n.{k}"));
        let mut want_nodes = vec![
            (n(0), NodeKind::Document, None),
            (n(1), NodeKind::Section, Some(n(0))),
            (n(2), NodeKind::Section, Some(n(1))),
            (n(3), NodeKind::Paragraph, Some(n(2))),
            (n(4), NodeKind::Paragraph, Some(n(2))),
            (n(5), NodeKind::Section, Some(n(2))),
            (n(6), NodeKind::Table, Some(n(5))),
        ];
        want_nodes.extend((7..15).map(|k| (n(k), NodeKind::Cell, Some(n(6)))));
        want_nodes.extend([
            (n(15), NodeKind::Section, Some(n(2))),
            (n(16), NodeKind::List, Some(n(15))),
            (n(17), NodeKind::Paragraph, Some(n(16))),
            (n(18), NodeKind::Paragraph, Some(n(16))),
        ]);
        assert_eq!(node_shape(&g), want_nodes);

        let cells: Vec<(Id, Vec<(Id, String)>)> = g
            .nodes
            .iter()
            .filter(|node| node.kind == NodeKind::Cell)
            .map(|node| (node.node_id.clone(), node.attrs.clone()))
            .collect();
        let want_cells: Vec<(Id, Vec<(Id, String)>)> = [
            (7, 0, 0, true),
            (8, 0, 1, true),
            (9, 1, 0, false),
            (10, 1, 1, false),
            (11, 2, 0, false),
            (12, 2, 1, false),
            (13, 3, 0, false),
            (14, 3, 1, false),
        ]
        .map(|(k, row, col, header)| (n(k), cell_attrs(row, col, header)))
        .into();
        assert_eq!(cells, want_cells);
        assert!(
            g.nodes
                .iter()
                .filter(|node| node.kind != NodeKind::Cell)
                .all(|node| node.attrs.is_empty())
        );

        let texts = [
            (1, "敗血症診療ガイドライン(合成)"),
            (2, "CQ1:成人の敗血症患者に対して抗菌薬Aを投与すべきか"),
            (
                3,
                "成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する(強い推奨)。",
            ),
            (4, "ただし、重度腎機能障害のある患者を除く。"),
            (5, "用語の定義"),
            (7, "用語"),
            (8, "定義"),
            (9, "成人"),
            (10, "18歳以上の患者"),
            (11, "小児"),
            (12, "18歳未満の患者"),
            (13, "重度腎機能障害"),
            (14, "高度の腎機能低下を認める状態(合成定義)"),
            (15, "エビデンス"),
            (17, "ランダム化比較試験2件(エビデンスの確実性:中)"),
            (18, "観察研究1件(エビデンスの確実性:低)"),
        ];
        let want_spans: Vec<(Id, Id, String, u64)> = texts
            .iter()
            .enumerate()
            .map(|(k, (node, text))| {
                (
                    id(&format!("s.{k}")),
                    n(*node),
                    (*text).to_owned(),
                    k as u64,
                )
            })
            .collect();
        assert_eq!(span_shape(&g), want_spans);
        assert_span_regions(&g);

        let doc = ckc_core::DocIr::from_graph(&g, diags).unwrap();
        assert!(doc.diagnostics.is_empty());
        assert_eq!(doc.blocks.len(), 16);
        let [table] = doc.tables.as_slice() else {
            panic!("one table view, got {:?}", doc.tables);
        };
        assert_eq!(table.table_node_id, n(6));
        let want: Vec<ckc_core::TableCell> = [
            (7, 0, 0, ckc_core::CellRole::Header),
            (8, 0, 1, ckc_core::CellRole::Header),
            (9, 1, 0, ckc_core::CellRole::Body),
            (10, 1, 1, ckc_core::CellRole::Body),
            (11, 2, 0, ckc_core::CellRole::Body),
            (12, 2, 1, ckc_core::CellRole::Body),
            (13, 3, 0, ckc_core::CellRole::Body),
            (14, 3, 1, ckc_core::CellRole::Body),
        ]
        .map(|(k, row, col, role)| ckc_core::TableCell {
            node_id: n(k),
            row,
            col,
            role,
        })
        .into();
        assert_eq!(table.cells, want);
    }

    #[test]
    fn committed_test_sources_extract_residual_free() {
        for name in [
            "m1_guideline_a.html",
            "m1_guideline_b.html",
            "m1_control.html",
        ] {
            let wrapper = extract(&test_source(name), &config()).unwrap();
            assert!(
                wrapper.diagnostics.is_empty(),
                "{name} extracts residual-free, got {:?}",
                wrapper.diagnostics
            );
        }
    }

    // Caption mints a textual caption node under the table, an empty
    // cell occupies its column index minting nothing, and an explicit
    // colspan="1" is accepted.
    #[test]
    fn table_caption_and_empty_cell() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            "<table><caption>表1:定義</caption>",
            r#"<tr><th colspan="1">語</th><td></td><td>値</td></tr>"#,
            "</table></body></html>"
        ));
        assert!(diags.is_empty());
        assert_eq!(
            node_shape(&g),
            vec![
                (id("n.0"), NodeKind::Document, None),
                (id("n.1"), NodeKind::Table, Some(id("n.0"))),
                (id("n.2"), NodeKind::Caption, Some(id("n.1"))),
                (id("n.3"), NodeKind::Cell, Some(id("n.1"))),
                (id("n.4"), NodeKind::Cell, Some(id("n.1"))),
            ]
        );
        assert_eq!(g.nodes[3].attrs, cell_attrs(0, 0, true));
        assert_eq!(
            g.nodes[4].attrs,
            cell_attrs(0, 2, false),
            "empty td holds col 1"
        );
        assert_eq!(
            span_shape(&g),
            vec![
                (id("s.0"), id("n.2"), "表1:定義".to_owned(), 0),
                (id("s.1"), id("n.3"), "語".to_owned(), 1),
                (id("s.2"), id("n.4"), "値".to_owned(), 2),
            ]
        );
        assert_span_regions(&g);

        let doc = ckc_core::DocIr::from_graph(&g, diags).unwrap();
        let kinds: Vec<NodeKind> = doc.blocks.iter().map(|b| b.kind).collect();
        assert_eq!(
            kinds,
            vec![NodeKind::Caption, NodeKind::Cell, NodeKind::Cell]
        );
        let [table] = doc.tables.as_slice() else {
            panic!("one table view, got {:?}", doc.tables);
        };
        let positions: Vec<(u64, u64)> = table.cells.iter().map(|c| (c.row, c.col)).collect();
        assert_eq!(positions, vec![(0, 0), (0, 2)]);
    }

    // Each rejection arm: one table_structure_uncertain residual whose
    // region names the table node, every cell withheld while the table
    // node stays, and DocIr::from_graph drops the table from the view.
    #[test]
    fn rejected_tables_withhold_cells_and_doc_ir_drops_the_table() {
        let cases = [
            (
                r#"<table><tr><td rowspan="2">甲</td></tr></table>"#,
                "rowspan 2 on td",
            ),
            (
                "<table><tr><td><table><tr><td>乙</td></tr></table></td></tr></table>",
                "nested table",
            ),
            (
                "<table><caption>一</caption><caption>二</caption><tr><td>丙</td></tr></table>",
                "second caption",
            ),
            (
                "<table><style>x</style><tr><td>丁</td></tr></table>",
                "unknown table child: style",
            ),
        ];
        for (html, want_detail) in cases {
            let (g, diags) = graph(&format!("<!DOCTYPE html><html><body>{html}</body></html>"));
            let [diag] = diags.as_slice() else {
                panic!("one residual for {want_detail}, got {diags:?}");
            };
            assert_eq!(
                detail_of(diag, DiagnosticCode::TableStructureUncertain),
                want_detail
            );
            assert_eq!(diag.region_ids, vec![id("r.0")]);
            assert_eq!(
                node_shape(&g),
                vec![
                    (id("n.0"), NodeKind::Document, None),
                    (id("n.1"), NodeKind::Table, Some(id("n.0"))),
                ],
                "table node stays, every cell withheld: {want_detail}"
            );
            assert!(g.spans.is_empty());
            assert_eq!(region_shape(&g), vec![(id("r.0"), vec![id("n.1")], vec![])]);
            let doc = ckc_core::DocIr::from_graph(&g, diags).unwrap();
            assert!(
                doc.tables.is_empty(),
                "DocIr drops the table: {want_detail}"
            );
            assert!(doc.blocks.is_empty());
        }
    }
}
